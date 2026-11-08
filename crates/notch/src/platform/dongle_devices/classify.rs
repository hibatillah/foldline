//! Which device-tree nodes are a wireless receiver's devices, what kind they are and what to call them. No Windows calls.

/// What a receiver's device is, read from its HID collection or from what the device says over HID++.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DongleKind {
    Mouse,
    Keyboard,
    Gamepad,
    Other,
}

/// What one HID collection of a receiver is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CollectionRole {
    Device(DongleKind),
    /// Logitech's HID++ short report collection (7 bytes, report id 0x10).
    HidppShort,
    /// Logitech's HID++ long report collection (20 bytes, report id 0x11).
    HidppLong,
    /// Consumer keys, system controls and other collections that are not a device of their own.
    Ignored,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UsbId {
    pub vendor_id: u16,
    pub product_id: u16,
}

pub(super) const LOGITECH_VENDOR_ID: u16 = 0x046D;

const GENERIC_DESKTOP_PAGE: u16 = 0x01;
const USAGE_MOUSE: u16 = 0x02;
const USAGE_JOYSTICK: u16 = 0x04;
const USAGE_GAMEPAD: u16 = 0x05;
const USAGE_KEYBOARD: u16 = 0x06;
const LOGITECH_VENDOR_PAGE: u16 = 0xFF00;
const HIDPP_SHORT_USAGE: u16 = 0x0001;
const HIDPP_LONG_USAGE: u16 = 0x0002;

/// Words a receiver's USB description carries; compared without case.
const RECEIVER_WORDS: [&str; 7] = ["receiver", "dongle", "2.4g", "wireless", "unifying", "lightspeed", "nano"];

/// Logitech receivers whose description may not say so (Solaar's receiver list): Unifying C52B and C532, Nano C52F,
/// C531 and C534, Lightspeed C539, C53A and C547.
const KNOWN_RECEIVERS: [UsbId; 8] = [
    UsbId { vendor_id: LOGITECH_VENDOR_ID, product_id: 0xC52B },
    UsbId { vendor_id: LOGITECH_VENDOR_ID, product_id: 0xC52F },
    UsbId { vendor_id: LOGITECH_VENDOR_ID, product_id: 0xC531 },
    UsbId { vendor_id: LOGITECH_VENDOR_ID, product_id: 0xC532 },
    UsbId { vendor_id: LOGITECH_VENDOR_ID, product_id: 0xC534 },
    UsbId { vendor_id: LOGITECH_VENDOR_ID, product_id: 0xC539 },
    UsbId { vendor_id: LOGITECH_VENDOR_ID, product_id: 0xC53A },
    UsbId { vendor_id: LOGITECH_VENDOR_ID, product_id: 0xC547 },
];

/// Brands that sell 2.4 GHz receivers, by USB vendor id.
const VENDORS: [(u16, &str); 10] = [
    (LOGITECH_VENDOR_ID, "Logitech"),
    (0x045E, "Microsoft"),
    (0x1532, "Razer"),
    (0x1038, "SteelSeries"),
    (0x1B1C, "Corsair"),
    (0x0B05, "ASUS"),
    (0x413C, "Dell"),
    (0x03F0, "HP"),
    (0x17EF, "Lenovo"),
    (0x258A, "SINO WEALTH"),
];

/// Reads `VID_xxxx` and `PID_xxxx` from an instance id such as `USB\VID_046D&PID_C52F\5&...`.
pub(super) fn parse_usb_id(instance_id: &str) -> Option<UsbId> {
    let upper = instance_id.to_ascii_uppercase();
    let hex_after = |marker: &str| {
        let start = upper.find(marker)? + marker.len();
        u16::from_str_radix(upper.get(start..start + 4)?, 16).ok()
    };
    Some(UsbId { vendor_id: hex_after("VID_")?, product_id: hex_after("PID_")? })
}

/// The index of the USB device a HID node hangs from, walking `chain` from the node up to the root; `None` when the
/// node sits on another bus (I2C and ACPI for a built-in touchpad, Bluetooth) before any USB device.
pub(super) fn usb_device_in_chain(chain: &[&str]) -> Option<usize> {
    for (index, instance_id) in chain.iter().enumerate() {
        let upper = instance_id.to_ascii_uppercase();
        if upper.starts_with("HID\\") {
            continue;
        }
        if upper.starts_with("USB\\") {
            if upper.contains("&MI_") {
                continue;
            }
            return Some(index);
        }
        return None;
    }
    None
}

/// Whether a USB device is a wireless receiver, by the words in its description or the known receiver table.
pub(super) fn is_receiver(description: &str, usb_id: UsbId) -> bool {
    let lower = description.to_lowercase();
    RECEIVER_WORDS.iter().any(|word| lower.contains(word)) || KNOWN_RECEIVERS.contains(&usb_id)
}

pub(super) fn collection_role(vendor_id: u16, usage_page: u16, usage: u16) -> CollectionRole {
    match (usage_page, usage) {
        (GENERIC_DESKTOP_PAGE, USAGE_MOUSE) => CollectionRole::Device(DongleKind::Mouse),
        (GENERIC_DESKTOP_PAGE, USAGE_KEYBOARD) => CollectionRole::Device(DongleKind::Keyboard),
        (GENERIC_DESKTOP_PAGE, USAGE_JOYSTICK | USAGE_GAMEPAD) => CollectionRole::Device(DongleKind::Gamepad),
        (LOGITECH_VENDOR_PAGE, HIDPP_SHORT_USAGE) if vendor_id == LOGITECH_VENDOR_ID => CollectionRole::HidppShort,
        (LOGITECH_VENDOR_PAGE, HIDPP_LONG_USAGE) if vendor_id == LOGITECH_VENDOR_ID => CollectionRole::HidppLong,
        _ => CollectionRole::Ignored,
    }
}

pub(super) fn vendor_name(vendor_id: u16) -> Option<&'static str> {
    VENDORS.iter().find(|(id, _)| *id == vendor_id).map(|(_, name)| *name)
}

/// "Logitech mouse", or "Wireless mouse" for a brand the table does not know; Windows only says "HID-compliant mouse".
pub(super) fn friendly_name(vendor_id: u16, kind: DongleKind) -> String {
    let noun = match kind {
        DongleKind::Mouse => "mouse",
        DongleKind::Keyboard => "keyboard",
        DongleKind::Gamepad => "controller",
        DongleKind::Other => "device",
    };
    format!("{} {noun}", vendor_name(vendor_id).unwrap_or("Wireless"))
}

/// The kind of a HID++ 2.0 device type (feature 0x0005, GetDeviceType).
pub(super) fn kind_from_hidpp20_type(device_type: u8) -> DongleKind {
    match device_type {
        0x00 | 0x02 => DongleKind::Keyboard,
        0x03..=0x05 => DongleKind::Mouse,
        0x0A..=0x0C => DongleKind::Gamepad,
        _ => DongleKind::Other,
    }
}

/// The kind in a HID++ 1.0 receiver's pairing information (low nibble of its eighth byte).
pub(super) fn kind_from_hidpp10_type(device_type: u8) -> DongleKind {
    match device_type & 0x0F {
        0x01 | 0x03 => DongleKind::Keyboard,
        0x02 | 0x08 | 0x09 => DongleKind::Mouse,
        _ => DongleKind::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const M331_RECEIVER: UsbId = UsbId { vendor_id: LOGITECH_VENDOR_ID, product_id: 0xC52F };

    #[test]
    fn a_usb_instance_id_gives_its_vendor_and_product() {
        assert_eq!(parse_usb_id("USB\\VID_046D&PID_C52F\\5&2B3C&0&3"), Some(M331_RECEIVER));
    }

    #[test]
    fn an_instance_id_without_a_product_id_gives_nothing() {
        assert_eq!(parse_usb_id("ACPI\\FUJ7401\\4&1"), None);
    }

    #[test]
    fn a_receiver_mouse_is_found_through_its_usb_interface_to_the_composite_device() {
        let chain = [
            "HID\\VID_046D&PID_C52F&MI_00\\7&1F0&0&0000",
            "USB\\VID_046D&PID_C52F&MI_00\\6&2A&0&0000",
            "USB\\VID_046D&PID_C52F\\5&2B3C&0&3",
            "USB\\ROOT_HUB30\\4&1",
            "PCI\\VEN_8086&DEV_A0ED\\3&1",
        ];
        assert_eq!(usb_device_in_chain(&chain), Some(2));
    }

    #[test]
    fn a_built_in_i2c_touchpad_has_no_usb_device() {
        let chain = ["HID\\SYNA2BA6&COL01\\5&1", "ACPI\\SYNA2BA6\\4&1", "ACPI\\PNP0C50\\1"];
        assert_eq!(usb_device_in_chain(&chain), None);
    }

    #[test]
    fn a_bluetooth_keyboard_has_no_usb_device() {
        let chain = ["HID\\{00001124-0000-1000-8000-00805F9B34FB}_VID&0002", "BTHENUM\\{00001124}\\8&1"];
        assert_eq!(usb_device_in_chain(&chain), None);
    }

    #[test]
    fn a_usb_receiver_description_is_a_receiver() {
        assert!(is_receiver("USB Receiver", UsbId { vendor_id: 0x1234, product_id: 0x0001 }));
    }

    #[test]
    fn a_dongle_description_is_a_receiver_whatever_its_case() {
        assert!(is_receiver("2.4G Wireless DONGLE", UsbId { vendor_id: 0x1234, product_id: 0x0001 }));
    }

    #[test]
    fn a_known_logitech_receiver_is_a_receiver_with_a_plain_description() {
        assert!(is_receiver("USB Composite Device", M331_RECEIVER));
    }

    #[test]
    fn a_wired_mouse_is_not_a_receiver() {
        assert!(!is_receiver("USB Optical Mouse", UsbId { vendor_id: 0x046D, product_id: 0xC077 }));
    }

    #[test]
    fn the_internal_ite_keyboard_is_not_a_receiver() {
        assert!(!is_receiver("ITE Device(8910)", UsbId { vendor_id: 0x048D, product_id: 0xC993 }));
    }

    #[test]
    fn a_generic_desktop_mouse_collection_is_a_mouse() {
        assert_eq!(collection_role(0x1234, 0x01, 0x02), CollectionRole::Device(DongleKind::Mouse));
    }

    #[test]
    fn a_gamepad_collection_is_a_gamepad() {
        assert_eq!(collection_role(0x1234, 0x01, 0x05), CollectionRole::Device(DongleKind::Gamepad));
    }

    #[test]
    fn a_consumer_control_collection_is_ignored() {
        assert_eq!(collection_role(LOGITECH_VENDOR_ID, 0x0C, 0x01), CollectionRole::Ignored);
    }

    #[test]
    fn logitech_vendor_collections_are_the_hidpp_short_and_long_channels() {
        let roles = [collection_role(LOGITECH_VENDOR_ID, 0xFF00, 1), collection_role(LOGITECH_VENDOR_ID, 0xFF00, 2)];
        assert_eq!(roles, [CollectionRole::HidppShort, CollectionRole::HidppLong]);
    }

    #[test]
    fn another_vendors_ff00_collection_is_not_hidpp() {
        assert_eq!(collection_role(0x1532, 0xFF00, 1), CollectionRole::Ignored);
    }

    #[test]
    fn a_logitech_mouse_is_named_by_brand_and_kind() {
        assert_eq!(friendly_name(LOGITECH_VENDOR_ID, DongleKind::Mouse), "Logitech mouse");
    }

    #[test]
    fn an_unknown_brands_gamepad_is_a_wireless_controller() {
        assert_eq!(friendly_name(0x9999, DongleKind::Gamepad), "Wireless controller");
    }

    #[test]
    fn hidpp20_device_types_map_to_kinds() {
        let kinds = [0x00, 0x03, 0x05, 0x0C, 0x08].map(kind_from_hidpp20_type);
        let expected =
            [DongleKind::Keyboard, DongleKind::Mouse, DongleKind::Mouse, DongleKind::Gamepad, DongleKind::Other];
        assert_eq!(kinds, expected);
    }

    #[test]
    fn hidpp10_pairing_types_map_to_kinds() {
        let kinds = [0x41, 0x02, 0x09, 0x04].map(kind_from_hidpp10_type);
        assert_eq!(kinds, [DongleKind::Keyboard, DongleKind::Mouse, DongleKind::Mouse, DongleKind::Other]);
    }
}
