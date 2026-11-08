//! What Windows knows about paired Bluetooth devices: connection state, class, battery and the radio.
//! Every call blocks on Windows, so call them from a worker thread, never from the UI thread.

use windows::{
    Devices::{
        Bluetooth::{BluetoothAdapter, BluetoothConnectionStatus, BluetoothDevice, BluetoothLEDevice},
        Enumeration::DeviceInformation,
        Radios::RadioState,
    },
    Foundation::IReference,
    core::{HSTRING, Interface},
};

/// The battery level (percent) Windows keeps on a device's PnP node; only some devices report it.
pub(crate) const BATTERY_PROPERTY: &str = "{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2";
pub(crate) const BATTERY_NODES_FILTER: &str = "System.Devices.DeviceInstanceId:~<\"BTHENUM\" OR System.Devices.DeviceInstanceId:~<\"BTHHFENUM\" OR System.Devices.DeviceInstanceId:~<\"BTHLE\"";

/// One paired device as Windows reports it; the class and the appearance are the raw Bluetooth values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairedDevice {
    pub name: String,
    pub is_connected: bool,
    /// The 24-bit class of device of a classic device.
    pub class_of_device: Option<u32>,
    /// The 16-bit GAP appearance of a low-energy device.
    pub appearance: Option<u16>,
}

/// The battery of every device that has a battery node, by the node's name.
pub fn battery_levels() -> Vec<(String, u8)> {
    let properties = windows_collections::IIterable::<HSTRING>::from(vec![HSTRING::from(BATTERY_PROPERTY)]);
    let Some(nodes) = DeviceInformation::FindAllAsyncAqsFilterAndAdditionalProperties(
        &HSTRING::from(BATTERY_NODES_FILTER),
        &properties,
    )
    .ok()
    .and_then(|operation| operation.join().ok()) else {
        return Vec::new();
    };
    nodes
        .into_iter()
        .filter_map(|node| {
            let name = node.Name().ok()?.to_string();
            let value = node.Properties().ok()?.Lookup(&HSTRING::from(BATTERY_PROPERTY)).ok()?;
            let level = value.cast::<IReference<u8>>().ok()?.Value().ok()?;
            Some((name, level))
        })
        .collect()
}

/// The battery of the device whose name starts with `name`.
pub fn read_battery(name: &str) -> Option<u8> {
    battery_levels().into_iter().find(|(node_name, _)| node_name.starts_with(name)).map(|(_, level)| level)
}

/// Every paired classic and low-energy device; an unreadable device is skipped.
pub fn paired_devices() -> Vec<PairedDevice> {
    let mut devices = Vec::new();
    if let Ok(selector) = BluetoothDevice::GetDeviceSelectorFromPairingState(true)
        && let Some(found) = find_all(&selector)
    {
        for information in &found {
            let Some(device) = information
                .Id()
                .ok()
                .and_then(|id| BluetoothDevice::FromIdAsync(&id).ok())
                .and_then(|operation| operation.join().ok())
            else {
                continue;
            };
            let Ok(name) = device.Name() else { continue };
            devices.push(PairedDevice {
                name: name.to_string(),
                is_connected: device
                    .ConnectionStatus()
                    .is_ok_and(|status| status == BluetoothConnectionStatus::Connected),
                class_of_device: device.ClassOfDevice().and_then(|class| class.RawValue()).ok(),
                appearance: None,
            });
        }
    }
    if let Ok(selector) = BluetoothLEDevice::GetDeviceSelectorFromPairingState(true)
        && let Some(found) = find_all(&selector)
    {
        for information in &found {
            let Some(device) = information
                .Id()
                .ok()
                .and_then(|id| BluetoothLEDevice::FromIdAsync(&id).ok())
                .and_then(|operation| operation.join().ok())
            else {
                continue;
            };
            let Ok(name) = device.Name() else { continue };
            devices.push(PairedDevice {
                name: name.to_string(),
                is_connected: device
                    .ConnectionStatus()
                    .is_ok_and(|status| status == BluetoothConnectionStatus::Connected),
                class_of_device: None,
                appearance: device.Appearance().and_then(|appearance| appearance.RawValue()).ok(),
            });
        }
    }
    devices
}

/// Whether the Bluetooth radio is on; `None` when there is no adapter or Windows does not answer.
pub fn is_radio_on() -> Option<bool> {
    let adapter = BluetoothAdapter::GetDefaultAsync().ok()?.join().ok()?;
    let radio = adapter.GetRadioAsync().ok()?.join().ok()?;
    Some(radio.State().ok()? == RadioState::On)
}

fn find_all(selector: &HSTRING) -> Option<windows::Devices::Enumeration::DeviceInformationCollection> {
    DeviceInformation::FindAllAsyncAqsFilter(selector).ok()?.join().ok()
}
