//! Fixed devices for the snapshots and the tests, one per state of the design.

use super::{
    DevicesPage,
    model::{Device, Kind, Listing},
};

pub(super) const NORMAL: u8 = 0;
pub(super) const ATTENTION: u8 = 1;
pub(super) const UNKNOWN: u8 = 2;
pub(super) const MANY: u8 = 3;
pub(super) const EMPTY: u8 = 4;
pub(super) const BLUETOOTH_OFF: u8 = 5;

fn device(name: &str, kind: Kind, battery_percent: Option<u8>) -> Device {
    Device { name: name.to_owned(), kind, is_connected: true, battery_percent, is_charging: false }
}

pub(super) fn five() -> Vec<Device> {
    vec![
        device("MX Master 3", Kind::Mouse, Some(82)),
        device("Keychron K3", Kind::Keyboard, Some(64)),
        device("WH-1000XM5", Kind::Headphones, Some(45)),
        device("Controller", Kind::Gamepad, Some(91)),
        device("Pen", Kind::Pen, Some(100)),
    ]
}

pub(super) fn listing(scenario: u8) -> Listing {
    let devices = match scenario {
        ATTENTION => vec![
            device("MX Master 3", Kind::Mouse, Some(12)),
            Device { is_charging: true, ..device("Keychron K3", Kind::Keyboard, Some(64)) },
            Device { is_connected: false, ..device("WH-1000XM5", Kind::Headphones, Some(45)) },
            device("Controller", Kind::Gamepad, Some(91)),
        ],
        UNKNOWN => vec![device("2.4 GHz mouse", Kind::Mouse, None), device("Keychron K3", Kind::Keyboard, Some(64))],
        MANY => {
            let mut devices =
                vec![device("MX Master 3", Kind::Mouse, Some(12)), device("Keychron K3", Kind::Keyboard, Some(64))];
            devices.extend(five().into_iter().skip(2));
            devices.push(device("Earbuds", Kind::Headphones, Some(77)));
            devices.push(device("Trackpad", Kind::Mouse, Some(88)));
            devices
        }
        EMPTY => Vec::new(),
        BLUETOOTH_OFF => return Listing::BluetoothOff,
        _ => five(),
    };
    Listing::Devices(devices)
}

impl DevicesPage {
    pub fn sample(scenario: u8) -> Self {
        let mut page = Self::new();
        page.set_listing(listing(scenario));
        page
    }
}
