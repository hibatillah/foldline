//! Joins the Bluetooth devices with the ones behind a 2.4 GHz receiver and picks what the page says. No Windows calls.

use super::model::{Device, Kind, Listing};
use crate::platform::dongle_devices::{DongleDevice, DongleKind};

pub(super) fn kind_from_dongle(kind: DongleKind) -> Kind {
    match kind {
        DongleKind::Mouse => Kind::Mouse,
        DongleKind::Keyboard => Kind::Keyboard,
        DongleKind::Gamepad => Kind::Gamepad,
        DongleKind::Other => Kind::Other,
    }
}

/// A receiver lists only devices that are present, so it is connected; no charging answer means no bolt.
pub(super) fn from_dongle(device: &DongleDevice) -> Device {
    Device {
        name: device.name.clone(),
        kind: kind_from_dongle(device.kind),
        is_connected: true,
        battery_percent: device.battery_percent,
        is_charging: device.charging == Some(true),
    }
}

/// Bluetooth devices first, then receiver devices, as Windows lists them; a name and kind already listed counts once.
///
/// A device that is off stays only beside a connected one, so an old pairing never hides the "no wireless device"
/// message. "Bluetooth is off" shows only when the radio is off and no receiver device is there to list.
pub(super) fn combine(is_radio_on: Option<bool>, bluetooth: Vec<Device>, dongle: &[DongleDevice]) -> Listing {
    let mut devices = bluetooth;
    for device in dongle.iter().map(from_dongle) {
        if !devices.iter().any(|known| known.name == device.name && known.kind == device.kind) {
            devices.push(device);
        }
    }
    if !devices.iter().any(|device| device.is_connected) {
        return if is_radio_on == Some(false) { Listing::BluetoothOff } else { Listing::Devices(Vec::new()) };
    }
    Listing::Devices(devices)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dongle(name: &str, kind: DongleKind, battery_percent: Option<u8>, charging: Option<bool>) -> DongleDevice {
        DongleDevice { name: name.to_owned(), kind, battery_percent, charging }
    }

    fn bluetooth(name: &str, kind: Kind, is_connected: bool) -> Device {
        Device { name: name.to_owned(), kind, is_connected, battery_percent: Some(50), is_charging: false }
    }

    #[test]
    fn every_receiver_kind_maps_to_a_page_kind() {
        assert_eq!(kind_from_dongle(DongleKind::Mouse), Kind::Mouse);
        assert_eq!(kind_from_dongle(DongleKind::Keyboard), Kind::Keyboard);
        assert_eq!(kind_from_dongle(DongleKind::Gamepad), Kind::Gamepad);
        assert_eq!(kind_from_dongle(DongleKind::Other), Kind::Other);
    }

    #[test]
    fn a_receiver_device_is_connected_and_carries_its_battery_and_charging_state() {
        let charging = from_dongle(&dongle("M", DongleKind::Mouse, Some(40), Some(true)));
        assert!(charging.is_connected && charging.is_charging);
        assert_eq!(charging.battery_percent, Some(40));
        let unknown = from_dongle(&dongle("M", DongleKind::Mouse, Some(40), None));
        assert!(!unknown.is_charging, "no answer means no bolt");
        assert!(!from_dongle(&dongle("M", DongleKind::Mouse, Some(40), Some(false))).is_charging);
    }

    #[test]
    fn a_receiver_device_with_no_battery_says_on_on_a_full_ring() {
        let device = from_dongle(&dongle("B330/M330/M331", DongleKind::Mouse, None, None));
        assert_eq!(device.percent_text(), "On");
        assert_eq!(device.ring_fraction(), None);
    }

    #[test]
    fn bluetooth_devices_come_first_and_receiver_devices_after_them() {
        let listing = combine(
            Some(true),
            vec![bluetooth("Keyboard", Kind::Keyboard, true)],
            &[dongle("Mouse", DongleKind::Mouse, Some(70), None)],
        );
        let Listing::Devices(devices) = listing else { panic!("devices") };
        let names: Vec<&str> = devices.iter().map(|device| device.name.as_str()).collect();
        assert_eq!(names, ["Keyboard", "Mouse"]);
    }

    #[test]
    fn a_device_in_both_lists_with_the_same_name_and_kind_counts_once() {
        let listing = combine(
            Some(true),
            vec![bluetooth("MX Master 3", Kind::Mouse, true)],
            &[
                dongle("MX Master 3", DongleKind::Mouse, Some(70), None),
                dongle("MX Master 3", DongleKind::Keyboard, Some(70), None),
            ],
        );
        let Listing::Devices(devices) = listing else { panic!("devices") };
        assert_eq!(devices.len(), 2, "the same name with another kind is another device");
    }

    #[test]
    fn bluetooth_off_shows_only_when_no_receiver_device_is_there() {
        assert_eq!(combine(Some(false), Vec::new(), &[]), Listing::BluetoothOff);
        let listing = combine(Some(false), Vec::new(), &[dongle("Mouse", DongleKind::Mouse, None, None)]);
        let Listing::Devices(devices) = listing else { panic!("devices listed with Bluetooth off") };
        assert_eq!(devices.len(), 1);
    }

    #[test]
    fn the_empty_message_needs_both_lists_empty_and_the_radio_not_off() {
        assert_eq!(combine(Some(true), Vec::new(), &[]), Listing::Devices(Vec::new()));
        assert_eq!(combine(None, Vec::new(), &[]), Listing::Devices(Vec::new()));
    }

    #[test]
    fn with_no_bluetooth_adapter_the_receiver_devices_are_still_listed() {
        let listing = combine(None, Vec::new(), &[dongle("Mouse", DongleKind::Mouse, Some(10), None)]);
        assert!(matches!(listing, Listing::Devices(devices) if devices.len() == 1));
    }

    #[test]
    fn a_paired_device_that_is_off_shows_beside_a_receiver_device_but_never_alone() {
        let off = bluetooth("Old headset", Kind::Headphones, false);
        assert_eq!(combine(Some(true), vec![off.clone()], &[]), Listing::Devices(Vec::new()));
        let listing = combine(Some(true), vec![off], &[dongle("Mouse", DongleKind::Mouse, Some(60), None)]);
        assert!(matches!(listing, Listing::Devices(devices) if devices.len() == 2));
    }
}
