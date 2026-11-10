//! The devices the page lists and the rules that pick, order and word them. No Windows calls.

use crate::i18n::tr;
use crate::{platform::bluetooth_devices::PairedDevice, ui::Icon};

/// At or under this a battery is low (red); the same 20 % the battery activity warns at.
pub(super) const LOW_PERCENT: u8 = 20;
/// The most tiles the row holds; with more devices the five lowest batteries show.
pub(super) const MAX_TILES: usize = 5;

const MAJOR_CLASS_AUDIO_VIDEO: u32 = 4;
const MAJOR_CLASS_PERIPHERAL: u32 = 5;
const APPEARANCE_CATEGORY_HID: u16 = 15;
/// Peripheral minor class bits: 0x10 keyboard, 0x20 pointing device; the low four bits name the other kinds.
const PERIPHERAL_KEYBOARD: u32 = 0x10;
const PERIPHERAL_POINTER: u32 = 0x20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Mouse,
    Keyboard,
    Headphones,
    Gamepad,
    Pen,
    Other,
}

impl Kind {
    pub(super) fn icon(self) -> Icon {
        match self {
            Kind::Mouse => Icon::Mouse,
            Kind::Keyboard => Icon::Keyboard,
            Kind::Headphones => Icon::Headphones,
            Kind::Gamepad => Icon::Gamepad,
            Kind::Pen => Icon::Pen,
            Kind::Other => Icon::Bluetooth,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Device {
    pub name: String,
    pub kind: Kind,
    pub is_connected: bool,
    /// `None` when the device does not report a battery to Windows.
    pub battery_percent: Option<u8>,
    /// Windows gives no charging state for Bluetooth devices, so the reader leaves this false.
    pub is_charging: bool,
}

/// What the last read found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Listing {
    BluetoothOff,
    /// Empty when no wireless device is connected.
    Devices(Vec<Device>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Tone {
    Idle,
    Low,
    Charging,
}

/// What the row draws: the devices in order.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Shown {
    pub devices: Vec<Device>,
}

/// What a paint borrows: the row's devices with their words written once, when the listing changes.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Prepared {
    pub shown: Shown,
    /// One per shown device, in the same order.
    pub percent_texts: Vec<String>,
}

impl Prepared {
    pub(super) fn new(devices: &[Device]) -> Self {
        let shown = shown(devices);
        let percent_texts = shown.devices.iter().map(Device::percent_text).collect();
        Self { shown, percent_texts }
    }
}

impl Device {
    /// Charging wins over low, as on the battery ring.
    pub(super) fn tone(&self) -> Tone {
        match self.battery_percent {
            Some(_) if self.is_charging => Tone::Charging,
            Some(percent) if percent <= LOW_PERCENT => Tone::Low,
            _ => Tone::Idle,
        }
    }

    /// The share of the ring that is filled; `None` draws a full gray ring (on, no battery reported) or a dashed one (off).
    pub(super) fn ring_fraction(&self) -> Option<f32> {
        self.battery_percent.filter(|_| self.is_connected).map(|percent| f32::from(percent.min(100)) / 100.0)
    }

    pub(super) fn percent_text(&self) -> String {
        match (self.is_connected, self.battery_percent) {
            (false, _) => tr!("devices.off").into_owned(),
            (true, Some(percent)) => format!("{}%", percent.min(100)),
            (true, None) => tr!("devices.on").into_owned(),
        }
    }

    /// Lowest battery first, then devices with no reading, then the ones that are off.
    fn attention_rank(&self) -> (u8, u8) {
        match (self.is_connected, self.battery_percent) {
            (true, Some(percent)) => (0, percent),
            (true, None) => (1, 0),
            (false, _) => (2, 0),
        }
    }
}

/// Up to `MAX_TILES` devices show; with more, the lowest batteries are kept and Windows' order is restored.
pub(super) fn shown(devices: &[Device]) -> Shown {
    if devices.len() <= MAX_TILES {
        return Shown { devices: devices.to_vec() };
    }
    let mut ranked: Vec<(usize, &Device)> = devices.iter().enumerate().collect();
    ranked.sort_by_key(|(_, device)| device.attention_rank());
    ranked.truncate(MAX_TILES);
    ranked.sort_by_key(|(index, _)| *index);
    Shown { devices: ranked.into_iter().map(|(_, device)| device.clone()).collect() }
}

pub(super) fn kind_from_class(class_of_device: u32) -> Kind {
    let major = (class_of_device >> 8) & 0x1F;
    let minor = (class_of_device >> 2) & 0x3F;
    match major {
        MAJOR_CLASS_AUDIO_VIDEO => Kind::Headphones,
        MAJOR_CLASS_PERIPHERAL if minor & PERIPHERAL_KEYBOARD != 0 => Kind::Keyboard,
        MAJOR_CLASS_PERIPHERAL if minor & PERIPHERAL_POINTER != 0 => Kind::Mouse,
        MAJOR_CLASS_PERIPHERAL => match minor & 0x0F {
            1 | 2 => Kind::Gamepad,
            5 => Kind::Pen,
            _ => Kind::Other,
        },
        _ => Kind::Other,
    }
}

pub(super) fn kind_from_appearance(appearance: u16) -> Kind {
    if appearance >> 6 != APPEARANCE_CATEGORY_HID {
        return Kind::Other;
    }
    match appearance & 0x3F {
        1 => Kind::Keyboard,
        2 => Kind::Mouse,
        3 | 4 => Kind::Gamepad,
        5 => Kind::Pen,
        _ => Kind::Other,
    }
}

/// Windows sometimes answers "Bluetooth 08:24:10:15:10:d5" for a device that is off; that is no name to show.
fn is_address_name(name: &str) -> bool {
    name.strip_prefix("Bluetooth ").is_some_and(|address| {
        address.len() == 17
            && address.split(':').count() == 6
            && address.chars().all(|c| c == ':' || c.is_ascii_hexdigit())
    })
}

fn is_peripheral(paired: &PairedDevice) -> bool {
    let by_class = paired
        .class_of_device
        .is_some_and(|class| matches!((class >> 8) & 0x1F, MAJOR_CLASS_AUDIO_VIDEO | MAJOR_CLASS_PERIPHERAL));
    let by_appearance = paired.appearance.is_some_and(|appearance| appearance >> 6 == APPEARANCE_CATEGORY_HID);
    by_class || by_appearance
}

/// Combines the paired devices with the batteries Windows reports.
///
/// A dual-mode device appears twice and counts once. A peripheral or headset is listed; anything else only while
/// connected with a battery. `merge::combine` drops devices that are off when none is connected.
pub(super) fn assemble(paired: &[PairedDevice], batteries: &[(String, u8)]) -> Vec<Device> {
    let mut devices: Vec<Device> = Vec::new();
    for entry in paired {
        let battery_percent = batteries
            .iter()
            .find(|(node_name, _)| node_name.starts_with(&entry.name))
            .map(|&(_, percent)| percent)
            .filter(|_| entry.is_connected);
        if (!entry.is_connected && is_address_name(&entry.name)) || (!is_peripheral(entry) && battery_percent.is_none())
        {
            continue;
        }
        let kind = entry
            .class_of_device
            .map(kind_from_class)
            .into_iter()
            .chain(entry.appearance.map(kind_from_appearance))
            .find(|kind| *kind != Kind::Other)
            .unwrap_or(Kind::Other);
        let device = Device {
            name: entry.name.clone(),
            kind,
            is_connected: entry.is_connected,
            battery_percent,
            is_charging: false,
        };
        match devices.iter_mut().find(|known| known.name == device.name) {
            Some(known) => {
                known.is_connected |= device.is_connected;
                known.battery_percent = known.battery_percent.or(device.battery_percent);
                if known.kind == Kind::Other {
                    known.kind = device.kind;
                }
            }
            None => devices.push(device),
        }
    }
    devices
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOUSE_CLASS: u32 = 0x580;
    const KEYBOARD_CLASS: u32 = 0x540;
    const HEADSET_CLASS: u32 = 0x418;
    const GAMEPAD_CLASS: u32 = 0x508;
    const PHONE_CLASS: u32 = 0x20C;

    fn device(name: &str, is_connected: bool, battery_percent: Option<u8>) -> Device {
        Device { name: name.to_owned(), kind: Kind::Mouse, is_connected, battery_percent, is_charging: false }
    }

    fn paired(name: &str, is_connected: bool, class_of_device: Option<u32>, appearance: Option<u16>) -> PairedDevice {
        PairedDevice { name: name.to_owned(), is_connected, class_of_device, appearance }
    }

    #[test]
    fn a_battery_at_or_under_the_low_level_is_red_and_above_it_is_idle() {
        assert_eq!(device("A", true, Some(12)).tone(), Tone::Low);
        assert_eq!(device("A", true, Some(LOW_PERCENT)).tone(), Tone::Low);
        assert_eq!(device("A", true, Some(LOW_PERCENT + 1)).tone(), Tone::Idle);
        assert_eq!(device("A", true, None).tone(), Tone::Idle);
    }

    #[test]
    fn charging_wins_over_low() {
        let mut charging = device("A", true, Some(12));
        charging.is_charging = true;
        assert_eq!(charging.tone(), Tone::Charging);
    }

    #[test]
    fn a_missing_battery_says_on_and_a_device_that_is_off_says_off() {
        assert_eq!(device("A", true, Some(82)).percent_text(), "82%");
        assert_eq!(device("A", true, None).percent_text(), "On");
        assert_eq!(device("A", false, Some(82)).percent_text(), "Off");
    }

    #[test]
    fn only_a_connected_device_with_a_battery_gets_a_filled_ring() {
        assert_eq!(device("A", true, Some(50)).ring_fraction(), Some(0.5));
        assert_eq!(device("A", true, None).ring_fraction(), None);
        assert_eq!(device("A", false, Some(50)).ring_fraction(), None);
        assert_eq!(device("A", true, Some(250)).ring_fraction(), Some(1.0));
    }

    #[test]
    fn up_to_five_devices_keep_their_order_and_show_everything() {
        let devices: Vec<Device> = (0..5).map(|index| device(&format!("D{index}"), true, Some(90 - index))).collect();
        assert_eq!(shown(&devices).devices, devices);
    }

    #[test]
    fn with_more_than_five_the_five_lowest_batteries_show_in_windows_order() {
        let devices: Vec<Device> = [80, 15, 60, 99, 5, 70, 40]
            .iter()
            .enumerate()
            .map(|(index, &level)| device(&format!("D{index}"), true, Some(level)))
            .collect();
        let result = shown(&devices);
        let levels: Vec<u8> = result.devices.iter().filter_map(|device| device.battery_percent).collect();
        assert_eq!(levels, vec![15, 60, 5, 70, 40]);
    }

    #[test]
    fn when_trimming_devices_with_no_reading_come_after_batteries_and_off_devices_last() {
        let mut devices: Vec<Device> =
            (0..4).map(|index| device(&format!("D{index}"), true, Some(50 + index))).collect();
        devices.push(device("Off", false, None));
        devices.push(device("Unknown", true, None));
        devices.push(device("Full", true, Some(100)));
        let names: Vec<String> = shown(&devices).devices.into_iter().map(|device| device.name).collect();
        assert_eq!(names, vec!["D0", "D1", "D2", "D3", "Full"], "the off device and the unknown one are dropped");
    }

    #[test]
    fn classes_map_to_the_kind_of_device() {
        assert_eq!(kind_from_class(MOUSE_CLASS), Kind::Mouse);
        assert_eq!(kind_from_class(KEYBOARD_CLASS), Kind::Keyboard);
        assert_eq!(kind_from_class(HEADSET_CLASS), Kind::Headphones);
        assert_eq!(kind_from_class(GAMEPAD_CLASS), Kind::Gamepad);
        assert_eq!(kind_from_class(PHONE_CLASS), Kind::Other);
    }

    #[test]
    fn appearances_map_to_the_kind_of_device() {
        let hid = |subcategory: u16| (APPEARANCE_CATEGORY_HID << 6) | subcategory;
        assert_eq!(kind_from_appearance(hid(1)), Kind::Keyboard);
        assert_eq!(kind_from_appearance(hid(2)), Kind::Mouse);
        assert_eq!(kind_from_appearance(hid(4)), Kind::Gamepad);
        assert_eq!(kind_from_appearance(hid(5)), Kind::Pen);
        assert_eq!(kind_from_appearance(0x0040), Kind::Other);
    }

    #[test]
    fn a_device_found_over_both_radios_is_listed_once_with_its_battery() {
        let list =
            [paired("WH-1000XM5", true, Some(HEADSET_CLASS), None), paired("WH-1000XM5", true, None, Some(15 << 6))];
        let devices = assemble(&list, &[("WH-1000XM5".to_owned(), 45)]);
        assert_eq!(devices.len(), 1);
        assert_eq!((devices[0].kind, devices[0].battery_percent), (Kind::Headphones, Some(45)));
    }

    #[test]
    fn a_phone_is_listed_only_while_it_is_connected_with_a_battery() {
        let list = [paired("Phone", true, Some(PHONE_CLASS), None), paired("Mouse", true, Some(MOUSE_CLASS), None)];
        assert_eq!(assemble(&list, &[]).len(), 1);
        assert_eq!(assemble(&list, &[("Phone".to_owned(), 70)]).len(), 2);
    }

    #[test]
    fn a_device_that_is_off_and_known_only_by_its_address_is_left_out() {
        let list = [
            paired("Bluetooth 08:24:10:15:10:d5", false, Some(HEADSET_CLASS), None),
            paired("Keyboard", true, Some(KEYBOARD_CLASS), None),
        ];
        assert_eq!(assemble(&list, &[]).len(), 1);
        assert!(is_address_name("Bluetooth 08:24:10:15:10:d5"));
        assert!(!is_address_name("Bluetooth Speaker"));
    }

    #[test]
    fn a_device_that_is_off_keeps_no_battery_from_a_stale_reading() {
        let off = paired("Old mouse", false, Some(MOUSE_CLASS), None);
        let list = [off, paired("Keyboard", true, Some(KEYBOARD_CLASS), None)];
        let devices = assemble(&list, &[("Old mouse".to_owned(), 30)]);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].battery_percent, None, "a stale battery of a device that is off is dropped");
    }

    #[test]
    fn a_disconnected_device_reads_mati_in_indonesian() {
        store::language::with_override(store::language::Language::Indonesian, || {
            let device = device("D", false, None);
            assert_eq!(device.percent_text(), "Mati");
        });
    }
}
