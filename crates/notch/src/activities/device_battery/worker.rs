use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use super::{
    tracker::{Change, Tracker, Warning},
    watcher::{self, BATTERY_PROPERTY, CONNECTED_PROPERTY, Value, Values, WatchEvent},
};
use crate::{log, page::Notifier, platform::bluetooth_devices};

/// Events this soon after the start still describe the state the notch found, not something that happened.
const SETTLE_TIME: Duration = Duration::from_secs(10);

pub(super) struct Shared {
    pub warnings: Mutex<VecDeque<Warning>>,
    pub notifier: Notifier,
}

pub(super) fn change_from(values: &Values, is_initial: bool) -> Change {
    let mut change = Change { is_initial, ..Change::default() };
    for (key, value) in values {
        match (key.as_str(), value) {
            (BATTERY_PROPERTY, Value::Byte(percent)) => change.percent = Some(*percent),
            (CONNECTED_PROPERTY, Value::Bool(is_connected)) => change.is_connected = Some(*is_connected),
            _ => {}
        }
    }
    change
}

/// Windows can keep a stale level on a node whose device is off, so ask the device itself before announcing.
fn is_still_connected(name: &str) -> bool {
    bluetooth_devices::paired_devices()
        .iter()
        .find(|device| name.starts_with(device.name.as_str()))
        .is_none_or(|device| device.is_connected)
}

fn announce(shared: &Arc<Shared>, warning: Warning) {
    let shared = shared.clone();
    thread::spawn(move || {
        if !is_still_connected(&warning.name) {
            log::debug(&format!("device battery: {} is low but not connected", warning.name));
            return;
        }
        log::debug(&format!("device battery: {} at {} %", warning.name, warning.percent));
        if let Ok(mut warnings) = shared.warnings.lock() {
            warnings.push_back(warning);
        }
        shared.notifier.notify();
    });
}

fn handle(shared: &Arc<Shared>, tracker: &Mutex<Tracker>, started_at: Instant, event: WatchEvent) {
    let is_settling = started_at.elapsed() < SETTLE_TIME;
    let Ok(mut tracker) = tracker.lock() else { return };
    let warning = match event {
        WatchEvent::Added { id, name, values, is_initial } => {
            tracker.apply(&id, Some(&name), change_from(&values, is_initial || is_settling))
        }
        WatchEvent::Updated { id, values } => tracker.apply(&id, None, change_from(&values, is_settling)),
        WatchEvent::Removed { id } => {
            tracker.forget(&id);
            None
        }
    };
    drop(tracker);
    if let Some(warning) = warning {
        announce(shared, warning);
    }
}

/// Runs on a thread of its own and never returns: the watcher only delivers while it stays alive.
pub(super) fn run(shared: Arc<Shared>) {
    let tracker = Mutex::new(Tracker::default());
    let started_at = Instant::now();
    let watcher = match watcher::start(watcher::KEYS, move |event| handle(&shared, &tracker, started_at, event)) {
        Ok(watcher) => watcher,
        Err(error) => {
            log::debug(&format!("device battery: cannot watch devices: {error}"));
            return;
        }
    };
    log::debug("device battery: watching battery nodes");
    loop {
        thread::park();
        let _ = &watcher;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_battery_and_connection_values_become_a_change() {
        let values =
            vec![(BATTERY_PROPERTY.to_owned(), Value::Byte(18)), (CONNECTED_PROPERTY.to_owned(), Value::Bool(true))];
        assert_eq!(
            change_from(&values, false),
            Change { percent: Some(18), is_connected: Some(true), is_initial: false }
        );
    }

    #[test]
    fn values_of_other_kinds_are_ignored() {
        let values = vec![
            (BATTERY_PROPERTY.to_owned(), Value::Other("Windows.Foundation.String".to_owned())),
            ("unrelated".to_owned(), Value::Byte(3)),
        ];
        assert_eq!(change_from(&values, true), Change { is_initial: true, ..Change::default() });
    }
}
