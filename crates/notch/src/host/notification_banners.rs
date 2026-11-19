//! Keeps Windows' own notification banners away while the notch runs (`ShowBanner=0` for every app), so the notch's
//! banner is the only one. Windows still lists each toast, which is how the notch reads it; the saved values go back
//! when the setting is switched off, when the notch exits, and at the next start after a crash.

mod backup;
mod passes;

use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use windows::{
    Win32::{Foundation::HWND, UI::WindowsAndMessaging::FindWindowExW},
    core::w,
};

use crate::{
    log,
    platform::notification_settings::{SettingsWatch, Signal, Wake},
};

pub(super) const SETTING_KEY: &str = "notch.hide_windows_notifications";

/// On until the user switches it off; the first start with the key unset backs up the values before hiding.
pub(super) fn wants_hidden(settings: &store::Settings) -> bool {
    settings.get_bool(SETTING_KEY, true)
}

static WORKER: Mutex<Option<Worker>> = Mutex::new(None);

/// A pass reads and writes a few registry values per app in milliseconds; a longer wait means the watch is stuck.
const STOP_TIMEOUT: Duration = Duration::from_secs(1);

struct Worker {
    stop: Arc<Signal>,
    finished: Receiver<()>,
    thread: JoinHandle<()>,
}

pub(super) fn is_hidden() -> bool {
    WORKER.lock().is_ok_and(|worker| worker.is_some())
}

/// Off also puts back values a crashed notch left behind, so calling it at start cleans up.
pub(super) fn set_hidden(is_hidden: bool) {
    if is_hidden {
        start();
    } else {
        stop();
        passes::restore_banners();
    }
}

/// At exit: a newer notch that replaced this one owns the saved values and keeps banners off. Values not restored
/// in time stay in the backup for a later restore.
pub(super) fn release_at_exit(own_window: HWND) {
    stop();
    if is_another_notch_running(own_window) {
        log::debug("notification banners: a newer notch runs; leaving the banners to it");
        return;
    }
    passes::restore_banners();
}

/// Windows is signing out or shutting down: every notch puts the values back.
pub(super) fn release_at_session_end() {
    stop();
    passes::restore_banners();
}

fn start() {
    let Ok(mut worker) = WORKER.lock() else { return };
    if worker.is_some() {
        return;
    }
    let Some(stop) = Signal::new().map(Arc::new) else {
        log::debug("could not create the notification banner stop signal");
        return;
    };
    let thread_stop = stop.clone();
    let (finished_sender, finished) = mpsc::channel();
    let spawned = thread::Builder::new().name("notification-banners".into()).spawn(move || {
        watch(&thread_stop);
        let _ = finished_sender.send(());
    });
    match spawned {
        Ok(thread) => *worker = Some(Worker { stop, finished, thread }),
        Err(error) => log::debug(&format!("could not start the notification banner watch: {error}")),
    }
}

/// Bounded, so quitting never hangs on a pass stuck in a registry or file call.
fn stop() {
    let Some(worker) = WORKER.lock().ok().and_then(|mut worker| worker.take()) else { return };
    let stop = worker.stop;
    super::shutdown::stop_worker(
        "notification banner watch",
        || stop.set(),
        &worker.finished,
        worker.thread,
        STOP_TIMEOUT,
    );
}

/// Wakes only when Windows writes a notification setting, e.g. when an app shows its first toast and gets its key.
fn watch(stop: &Signal) {
    let Some(settings) = SettingsWatch::open() else {
        log::debug("notification banners: cannot watch the settings; hiding the current apps once");
        passes::hide_banners();
        return;
    };
    loop {
        if let Err(error) = settings.arm() {
            log::debug(&format!("could not watch the notification settings: {error:?}"));
        }
        // The pass's own writes wake the next round once; that round finds nothing to change.
        passes::hide_banners();
        if settings.wait(stop) == Wake::Stopped {
            return;
        }
    }
}

pub(super) fn is_another_notch_running(own_window: HWND) -> bool {
    let mut after = None;
    while let Ok(window) = unsafe { FindWindowExW(None, after, w!("FoldlineWindow"), None) } {
        if window != own_window {
            return true;
        }
        after = Some(window);
    }
    false
}

#[cfg(test)]
mod default_tests {
    use super::*;

    fn settings_with(name: &str, text: Option<&str>) -> store::Settings {
        let path = std::env::temp_dir().join(format!("notch-{name}-{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&path);
        if let Some(text) = text {
            std::fs::write(&path, text).unwrap();
        }
        let settings = store::Settings::at(Some(path.clone()));
        let _ = std::fs::remove_file(&path);
        settings
    }

    #[test]
    fn an_unset_key_hides_the_popup() {
        assert!(wants_hidden(&settings_with("notification_banners-unset", None)));
    }

    #[test]
    fn an_explicit_off_shows_the_popup() {
        assert!(!wants_hidden(&settings_with(
            "notification_banners-off",
            Some("notch.hide_windows_notifications=off\n")
        )));
    }

    #[test]
    fn an_explicit_false_shows_the_popup() {
        assert!(!wants_hidden(&settings_with(
            "notification_banners-false",
            Some("notch.hide_windows_notifications=false\n")
        )));
    }
}
