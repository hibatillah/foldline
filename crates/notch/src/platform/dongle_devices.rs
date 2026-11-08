//! Wireless mice, keyboards and controllers that reach Windows through a USB receiver (2.4 GHz), not Bluetooth, and
//! their battery when a Logitech receiver's device answers HID++. A read blocks for up to 300 ms per silent request,
//! so call it from a worker thread, never from the UI thread.

mod channel;
mod classify;
mod hidpp;
mod logitech;
mod tree;

use windows::Win32::Devices::Properties::{
    DEVPKEY_Device_BusReportedDeviceDesc, DEVPKEY_Device_DeviceDesc, DEVPKEY_Device_InLocalMachineContainer,
};

pub use classify::DongleKind;
use classify::{CollectionRole, LOGITECH_VENDOR_ID, UsbId};

/// One device connected through a receiver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DongleDevice {
    pub name: String,
    pub kind: DongleKind,
    /// `None` when the device does not answer a battery query.
    pub battery_percent: Option<u8>,
    /// `None` when the device does not say.
    pub charging: Option<bool>,
}

/// A receiver plugged into this machine and the collections Windows made of it.
#[derive(Debug)]
struct Receiver {
    usb_instance_id: String,
    usb_id: UsbId,
    /// Each kind once, in the order Windows lists the collections.
    kinds: Vec<DongleKind>,
    hidpp_short: Option<(String, channel::CollectionInfo)>,
    hidpp_long: Option<(String, channel::CollectionInfo)>,
}

/// Every device behind a wireless receiver; built-in, wired and Bluetooth devices are left out.
pub fn connected_dongle_devices() -> Vec<DongleDevice> {
    connected_dongle_devices_traced(&mut |message| crate::log::debug(&format!("dongle devices: {message}")))
}

fn connected_dongle_devices_traced(trace: &mut impl FnMut(String)) -> Vec<DongleDevice> {
    receivers(trace).iter().flat_map(|receiver| devices_of(receiver, trace)).collect()
}

fn receivers(trace: &mut impl FnMut(String)) -> Vec<Receiver> {
    let mut receivers: Vec<Receiver> = Vec::new();
    for path in tree::hid_interface_paths() {
        let chain = tree::ancestry_of_interface(&path);
        let instance_ids: Vec<&str> = chain.iter().map(|node| node.instance_id.as_str()).collect();
        let Some(usb_device) = classify::usb_device_in_chain(&instance_ids).map(|index| &chain[index]) else {
            continue;
        };
        let Some(usb_id) = classify::parse_usb_id(&usb_device.instance_id) else { continue };
        // A device inside the laptop (the ITE keyboard controller) shares the computer's own container.
        if tree::flag_property(usb_device.handle, &DEVPKEY_Device_InLocalMachineContainer) == Some(true) {
            continue;
        }
        let description = tree::string_property(usb_device.handle, &DEVPKEY_Device_BusReportedDeviceDesc)
            .or_else(|| tree::string_property(usb_device.handle, &DEVPKEY_Device_DeviceDesc))
            .unwrap_or_default();
        if !classify::is_receiver(&description, usb_id) {
            continue;
        }
        let info = match channel::collection_info(&path) {
            Ok(info) => info,
            Err(error) => {
                trace(format!("could not open {path}: {error}"));
                continue;
            }
        };
        let role = classify::collection_role(info.vendor_id, info.usage_page, info.usage);
        trace(format!(
            "{} \"{description}\" usage {:04X}:{:04X} -> {role:?}",
            instance_ids[0], info.usage_page, info.usage
        ));
        let index = match receivers.iter().position(|receiver| receiver.usb_instance_id == usb_device.instance_id) {
            Some(index) => index,
            None => {
                receivers.push(Receiver {
                    usb_instance_id: usb_device.instance_id.clone(),
                    usb_id,
                    kinds: Vec::new(),
                    hidpp_short: None,
                    hidpp_long: None,
                });
                receivers.len() - 1
            }
        };
        let receiver = &mut receivers[index];
        match role {
            CollectionRole::Device(kind) if !receiver.kinds.contains(&kind) => receiver.kinds.push(kind),
            CollectionRole::HidppShort => receiver.hidpp_short = Some((path, info)),
            CollectionRole::HidppLong => receiver.hidpp_long = Some((path, info)),
            CollectionRole::Device(_) | CollectionRole::Ignored => {}
        }
    }
    receivers
}

/// The devices HID++ finds on a Logitech receiver, or one device per kind of collection when it cannot be asked.
fn devices_of(receiver: &Receiver, trace: &mut impl FnMut(String)) -> Vec<DongleDevice> {
    let vendor_id = receiver.usb_id.vendor_id;
    let slots = if vendor_id == LOGITECH_VENDOR_ID { hidpp_slots(receiver, trace) } else { Vec::new() };
    if !slots.is_empty() {
        let fallback_kind = receiver.kinds.first().copied().unwrap_or(DongleKind::Other);
        return slots
            .into_iter()
            .map(|slot| {
                let kind = slot.kind.unwrap_or(fallback_kind);
                DongleDevice {
                    name: slot.name.unwrap_or_else(|| classify::friendly_name(vendor_id, kind)),
                    kind,
                    battery_percent: slot.battery.map(|battery| battery.percent),
                    charging: slot.battery.and_then(|battery| battery.charging),
                }
            })
            .collect();
    }
    receiver
        .kinds
        .iter()
        .map(|&kind| DongleDevice {
            name: classify::friendly_name(vendor_id, kind),
            kind,
            battery_percent: None,
            charging: None,
        })
        .collect()
}

fn hidpp_slots(receiver: &Receiver, trace: &mut impl FnMut(String)) -> Vec<logitech::SlotDevice> {
    let short = receiver.hidpp_short.as_ref().map(|(path, info)| (path.as_str(), *info));
    let long = receiver.hidpp_long.as_ref().map(|(path, info)| (path.as_str(), *info));
    if short.is_none() && long.is_none() {
        trace(format!("{}: no HID++ collection", receiver.usb_instance_id));
        return Vec::new();
    }
    match channel::Channel::open(short, long) {
        Ok(mut channel) => logitech::read_slots(&mut channel, trace),
        Err(error) => {
            trace(format!("could not open the HID++ channel of {}: {error}", receiver.usb_instance_id));
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn combo_receiver(vendor_id: u16) -> Receiver {
        Receiver {
            usb_instance_id: "USB\\VID_1234&PID_0001\\5&1".to_owned(),
            usb_id: UsbId { vendor_id, product_id: 0x0001 },
            kinds: vec![DongleKind::Keyboard, DongleKind::Mouse],
            hidpp_short: None,
            hidpp_long: None,
        }
    }

    #[test]
    fn a_receiver_that_cannot_be_asked_lists_one_device_per_kind_without_a_battery() {
        let devices = devices_of(&combo_receiver(0x1532), &mut |_| {});
        let names: Vec<(&str, Option<u8>)> =
            devices.iter().map(|device| (device.name.as_str(), device.battery_percent)).collect();
        assert_eq!(names, [("Razer keyboard", None), ("Razer mouse", None)]);
    }

    #[test]
    fn a_logitech_receiver_without_hidpp_collections_still_lists_its_devices() {
        let devices = devices_of(&combo_receiver(LOGITECH_VENDOR_ID), &mut |_| {});
        assert_eq!(devices.len(), 2);
    }

    #[test]
    #[ignore = "asks this machine's receivers over HID++; run on purpose with --nocapture"]
    fn reading_this_machine_lists_its_receiver_devices() {
        let started = std::time::Instant::now();
        let devices = connected_dongle_devices_traced(&mut |message| println!("{message}"));
        println!("devices: {devices:#?}");
        println!("took {:?}", started.elapsed());
    }
}
