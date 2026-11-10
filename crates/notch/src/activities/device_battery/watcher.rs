//! A WinRT `DeviceWatcher` over the Bluetooth battery nodes; its events carry property changes.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use windows::{
    Devices::Enumeration::{DeviceInformation, DeviceInformationUpdate, DeviceWatcher},
    Foundation::{IReference, TypedEventHandler},
    core::{HSTRING, IInspectable, Interface, Ref, Result},
};
use windows_collections::{IIterable, IMapView};

pub(super) use crate::platform::bluetooth_devices::BATTERY_PROPERTY;
use crate::platform::bluetooth_devices::{BATTERY_NODE_KIND, BATTERY_NODES_FILTER};

/// `System.Devices.Connected`: whether the device behind the node is connected right now.
pub(super) const CONNECTED_PROPERTY: &str = "{83DA6326-97A6-4088-9453-A1923F573B29} 15";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Value {
    Byte(u8),
    Bool(bool),
    /// A type this activity does not read; the runtime class name is kept for the probe.
    Other(String),
}

/// A property of a node by key; a property set to nothing is left out.
pub(super) type Values = Vec<(String, Value)>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum WatchEvent {
    Added { id: String, name: String, values: Values, is_initial: bool },
    Updated { id: String, values: Values },
    Removed { id: String },
}

fn decode(value: &IInspectable) -> Option<Value> {
    if let Ok(byte) = value.cast::<IReference<u8>>().and_then(|reference| reference.Value()) {
        return Some(Value::Byte(byte));
    }
    if let Ok(flag) = value.cast::<IReference<bool>>().and_then(|reference| reference.Value()) {
        return Some(Value::Bool(flag));
    }
    value.GetRuntimeClassName().ok().map(|name| Value::Other(name.to_string()))
}

fn read_values(properties: &IMapView<HSTRING, IInspectable>, keys: &[&str]) -> Values {
    keys.iter()
        .filter_map(|key| {
            let value = properties.Lookup(&HSTRING::from(*key)).ok()?;
            Some((key.to_string(), decode(&value)?))
        })
        .collect()
}

/// Starts watching; the returned watcher must stay alive for its events to arrive.
pub(super) fn start(
    extra_keys: &'static [&'static str],
    on_event: impl Fn(WatchEvent) + Send + Sync + 'static,
) -> Result<DeviceWatcher> {
    let keys: Vec<HSTRING> = extra_keys.iter().map(|key| HSTRING::from(*key)).collect();
    let watcher = DeviceInformation::CreateWatcherWithKindAqsFilterAndAdditionalProperties(
        &HSTRING::from(BATTERY_NODES_FILTER),
        &IIterable::<HSTRING>::from(keys),
        BATTERY_NODE_KIND,
    )?;
    let on_event = Arc::new(on_event);
    let is_initial = Arc::new(AtomicBool::new(true));

    let sink = on_event.clone();
    let initial = is_initial.clone();
    watcher.Added(&TypedEventHandler::new(move |_: Ref<DeviceWatcher>, information: Ref<DeviceInformation>| {
        if let Some(information) = information.as_ref() {
            sink(WatchEvent::Added {
                id: information.Id()?.to_string(),
                name: information.Name()?.to_string(),
                values: read_values(&information.Properties()?, extra_keys),
                is_initial: initial.load(Ordering::SeqCst),
            });
        }
        Ok(())
    }))?;

    let sink = on_event.clone();
    watcher.Updated(&TypedEventHandler::new(move |_: Ref<DeviceWatcher>, update: Ref<DeviceInformationUpdate>| {
        if let Some(update) = update.as_ref() {
            sink(WatchEvent::Updated {
                id: update.Id()?.to_string(),
                values: read_values(&update.Properties()?, extra_keys),
            });
        }
        Ok(())
    }))?;

    let sink = on_event;
    watcher.Removed(&TypedEventHandler::new(move |_: Ref<DeviceWatcher>, update: Ref<DeviceInformationUpdate>| {
        if let Some(update) = update.as_ref() {
            sink(WatchEvent::Removed { id: update.Id()?.to_string() });
        }
        Ok(())
    }))?;

    watcher.EnumerationCompleted(&TypedEventHandler::new(move |_: Ref<DeviceWatcher>, _: Ref<IInspectable>| {
        is_initial.store(false, Ordering::SeqCst);
        Ok(())
    }))?;
    watcher.Start()?;
    Ok(watcher)
}

/// The two properties the activity reads.
pub(super) const KEYS: &[&str] = &[BATTERY_PROPERTY, CONNECTED_PROPERTY];

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::AtomicUsize,
        thread,
        time::{Duration, Instant},
    };

    use super::*;

    /// Prints what Windows pushes for 60 s: `cargo test device_battery_watcher_probe -- --ignored --nocapture`.
    /// Needs a connected Bluetooth device that reports a battery; connect or disconnect one while it runs.
    #[test]
    #[ignore = "needs a connected Bluetooth device and takes 60 seconds"]
    fn device_battery_watcher_probe() {
        const PROBE_KEYS: &[&str] = &[
            BATTERY_PROPERTY,
            CONNECTED_PROPERTY,
            "{540B947E-8B40-45BC-A8A2-6A0B894CBDA2} 11",
            "{83DA6326-97A6-4088-9453-A1923F573B29} 3",
        ];
        let started_at = Instant::now();
        let update_count = Arc::new(AtomicUsize::new(0));
        println!("battery_levels now: {:?}", crate::platform::bluetooth_devices::battery_levels());
        println!("paired now: {:?}", crate::platform::bluetooth_devices::paired_devices());
        let counter = update_count.clone();
        let _watcher = start(PROBE_KEYS, move |event| {
            if matches!(event, WatchEvent::Updated { .. }) {
                counter.fetch_add(1, Ordering::SeqCst);
            }
            println!("[{:>5.1}s] {event:?}", started_at.elapsed().as_secs_f32());
        })
        .expect("the watcher starts");
        thread::sleep(Duration::from_secs(60));
        println!("Updated events in 60 s: {}", update_count.load(Ordering::SeqCst));
    }
}
