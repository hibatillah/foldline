//! The HID++ conversation with a Logitech receiver: which device slots answer, their name, kind and battery. Read-only
//! queries; the transport owns the Windows handles.

use super::{
    classify::{DongleKind, kind_from_hidpp10_type, kind_from_hidpp20_type},
    hidpp::{
        Answer, Battery, ERROR_INVALID_SUB_ID, ERROR_RESOURCE, FEATURE_BATTERY_STATUS, FEATURE_BATTERY_VOLTAGE,
        FEATURE_DEVICE_NAME, FEATURE_UNIFIED_BATTERY, RECEIVER_INDEX, RECEIVER_INFO_NAME, RECEIVER_INFO_PAIRING,
        REGISTER_BATTERY_CHARGE, REGISTER_BATTERY_STATUS, REGISTER_RECEIVER_INFO, Request, assemble_name,
        decode_battery_status, decode_battery_voltage, decode_receiver_name, decode_register_battery_charge,
        decode_register_battery_status, decode_unified_battery,
    },
};

/// Unifying receivers hold up to six devices; Nano and Lightspeed receivers use slot 1.
const DEVICE_SLOTS: std::ops::RangeInclusive<u8> = 1..=6;
const DEVICE_NAME_GET_COUNT: u8 = 0;
const DEVICE_NAME_GET_NAME: u8 = 1;
const DEVICE_NAME_GET_TYPE: u8 = 2;
const UNIFIED_BATTERY_GET_STATUS: u8 = 1;
/// A name longer than this is a garbled reply, not a name.
const LONGEST_NAME: usize = 64;

/// Sends one request and returns the matching answer, or `None` when nothing matched in time.
pub(super) trait Transport {
    fn exchange(&mut self, request: &Request) -> Option<Answer>;
}

/// One device slot that answered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SlotDevice {
    pub slot: u8,
    pub name: Option<String>,
    pub kind: Option<DongleKind>,
    pub battery: Option<Battery>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Protocol {
    Hidpp10,
    Hidpp20,
}

/// Every paired device slot: one that answers is read in full, one the receiver cannot reach (asleep or off) keeps the
/// name and kind the receiver remembers and no battery. Empty and silent slots are left out.
pub(super) fn read_slots(transport: &mut impl Transport, trace: &mut impl FnMut(String)) -> Vec<SlotDevice> {
    let mut devices = Vec::new();
    for slot in DEVICE_SLOTS {
        let protocol = match transport.exchange(&Request::ping(slot)) {
            Some(Answer::Parameters(parameters)) => {
                trace(format!(
                    "slot {slot}: HID++ {}.{}",
                    parameters.first().unwrap_or(&0),
                    parameters.get(1).unwrap_or(&0)
                ));
                Protocol::Hidpp20
            }
            Some(Answer::Error(ERROR_INVALID_SUB_ID)) => {
                trace(format!("slot {slot}: HID++ 1.0"));
                Protocol::Hidpp10
            }
            Some(Answer::Error(ERROR_RESOURCE)) => {
                let (name, kind) = receiver_details(transport, slot);
                let device = SlotDevice { slot, name, kind, battery: None };
                trace(format!("slot {slot}: paired, out of reach: {device:?}"));
                devices.push(device);
                continue;
            }
            Some(Answer::Error(code)) => {
                trace(format!("slot {slot}: no device (error 0x{code:02X})"));
                continue;
            }
            None => {
                trace(format!("slot {slot}: no answer"));
                continue;
            }
        };
        let device = match protocol {
            Protocol::Hidpp20 => read_hidpp20(transport, slot),
            Protocol::Hidpp10 => read_hidpp10(transport, slot),
        };
        trace(format!("slot {slot}: {device:?}"));
        devices.push(device);
    }
    devices
}

fn read_hidpp20(transport: &mut impl Transport, slot: u8) -> SlotDevice {
    let name_index = feature_index(transport, slot, FEATURE_DEVICE_NAME);
    let name = name_index.and_then(|index| device_name(transport, slot, index));
    let kind = name_index
        .and_then(|index| parameters(transport, &Request::feature(slot, index, DEVICE_NAME_GET_TYPE, &[])))
        .and_then(|reply| reply.first().copied())
        .map(kind_from_hidpp20_type);
    let battery = hidpp20_battery(transport, slot);
    if name.is_some() && kind.is_some() {
        return SlotDevice { slot, name, kind, battery };
    }
    let (receiver_name, receiver_kind) = receiver_details(transport, slot);
    SlotDevice { slot, name: name.or(receiver_name), kind: kind.or(receiver_kind), battery }
}

/// Unified battery first (newer devices), then battery status, then voltage.
fn hidpp20_battery(transport: &mut impl Transport, slot: u8) -> Option<Battery> {
    if let Some(index) = feature_index(transport, slot, FEATURE_UNIFIED_BATTERY) {
        return parameters(transport, &Request::feature(slot, index, UNIFIED_BATTERY_GET_STATUS, &[]))
            .and_then(|reply| decode_unified_battery(&reply));
    }
    if let Some(index) = feature_index(transport, slot, FEATURE_BATTERY_STATUS) {
        return parameters(transport, &Request::feature(slot, index, 0, &[]))
            .and_then(|reply| decode_battery_status(&reply));
    }
    let index = feature_index(transport, slot, FEATURE_BATTERY_VOLTAGE)?;
    parameters(transport, &Request::feature(slot, index, 0, &[])).and_then(|reply| decode_battery_voltage(&reply))
}

fn device_name(transport: &mut impl Transport, slot: u8, name_index: u8) -> Option<String> {
    let length =
        usize::from(*parameters(transport, &Request::feature(slot, name_index, DEVICE_NAME_GET_COUNT, &[]))?.first()?);
    if length == 0 || length > LONGEST_NAME {
        return None;
    }
    let mut chunks: Vec<Vec<u8>> = Vec::new();
    let mut received = 0;
    while received < length {
        let offset = u8::try_from(received).ok()?;
        let chunk = parameters(transport, &Request::feature(slot, name_index, DEVICE_NAME_GET_NAME, &[offset]))?;
        if chunk.is_empty() {
            break;
        }
        received += chunk.len();
        chunks.push(chunk);
    }
    assemble_name(&chunks, length)
}

/// Battery charge (percent) first, then the coarser battery status register; the name and kind come from the receiver.
fn read_hidpp10(transport: &mut impl Transport, slot: u8) -> SlotDevice {
    let battery = parameters(transport, &Request::read_register(slot, REGISTER_BATTERY_CHARGE, [0; 3]))
        .and_then(|reply| decode_register_battery_charge(&reply))
        .or_else(|| {
            parameters(transport, &Request::read_register(slot, REGISTER_BATTERY_STATUS, [0; 3]))
                .and_then(|reply| decode_register_battery_status(&reply))
        });
    let (name, kind) = receiver_details(transport, slot);
    SlotDevice { slot, name, kind, battery }
}

/// The name and kind the receiver keeps for a paired slot (register 0x2B5), answered even while the device sleeps.
fn receiver_details(transport: &mut impl Transport, slot: u8) -> (Option<String>, Option<DongleKind>) {
    let receiver_info = |sub_register: u8| {
        Request::read_register(RECEIVER_INDEX, REGISTER_RECEIVER_INFO, [sub_register + slot - 1, 0, 0])
    };
    let name = parameters(transport, &receiver_info(RECEIVER_INFO_NAME)).and_then(|reply| decode_receiver_name(&reply));
    // The pairing reply's eighth byte, counted from the echoed sub-register.
    let kind = parameters(transport, &receiver_info(RECEIVER_INFO_PAIRING))
        .and_then(|reply| reply.get(7).copied())
        .map(kind_from_hidpp10_type);
    (name, kind)
}

fn feature_index(transport: &mut impl Transport, slot: u8, feature_id: u16) -> Option<u8> {
    let reply = parameters(transport, &Request::feature_index(slot, feature_id))?;
    reply.first().copied().filter(|&index| index != 0)
}

fn parameters(transport: &mut impl Transport, request: &Request) -> Option<Vec<u8>> {
    match transport.exchange(request)? {
        Answer::Parameters(parameters) => Some(parameters),
        Answer::Error(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Answers the requests in its table; anything else gets no answer.
    struct ScriptedReceiver {
        answers: Vec<(Request, Answer)>,
    }

    impl Transport for ScriptedReceiver {
        fn exchange(&mut self, request: &Request) -> Option<Answer> {
            self.answers.iter().find(|(asked, _)| asked == request).map(|(_, answer)| answer.clone())
        }
    }

    fn no_trace(_: String) {}

    #[test]
    fn a_hidpp20_mouse_in_slot_one_reports_its_unified_battery() {
        let mut receiver = ScriptedReceiver {
            answers: vec![
                (Request::ping(1), Answer::Parameters(vec![4, 5, 0x5A])),
                (Request::feature_index(1, FEATURE_UNIFIED_BATTERY), Answer::Parameters(vec![6])),
                (Request::feature(1, 6, 1, &[]), Answer::Parameters(vec![77, 0x04, 0, 0])),
            ],
        };
        let devices = read_slots(&mut receiver, &mut no_trace);
        assert_eq!(devices[0].battery, Some(Battery { percent: 77, charging: Some(false) }));
    }

    #[test]
    fn a_hidpp20_device_without_unified_battery_falls_back_to_battery_status() {
        let mut receiver = ScriptedReceiver {
            answers: vec![
                (Request::ping(1), Answer::Parameters(vec![2, 0, 0x5A])),
                (Request::feature_index(1, FEATURE_UNIFIED_BATTERY), Answer::Parameters(vec![0])),
                (Request::feature_index(1, FEATURE_BATTERY_STATUS), Answer::Parameters(vec![4])),
                (Request::feature(1, 4, 0, &[]), Answer::Parameters(vec![30, 20, 1])),
            ],
        };
        let devices = read_slots(&mut receiver, &mut no_trace);
        assert_eq!(devices[0].battery, Some(Battery { percent: 30, charging: Some(true) }));
    }

    #[test]
    fn a_hidpp20_device_gives_its_name_and_kind() {
        let mut receiver = ScriptedReceiver {
            answers: vec![
                (Request::ping(1), Answer::Parameters(vec![4, 5, 0x5A])),
                (Request::feature_index(1, FEATURE_DEVICE_NAME), Answer::Parameters(vec![3])),
                (Request::feature(1, 3, 0, &[]), Answer::Parameters(vec![4])),
                (Request::feature(1, 3, 1, &[0]), Answer::Parameters(b"M331\0\0\0\0\0\0\0\0\0\0\0\0".to_vec())),
                (Request::feature(1, 3, 2, &[]), Answer::Parameters(vec![3])),
            ],
        };
        let device = &read_slots(&mut receiver, &mut no_trace)[0];
        assert_eq!((device.name.as_deref(), device.kind), (Some("M331"), Some(DongleKind::Mouse)));
    }

    #[test]
    fn a_hidpp10_device_reads_the_battery_status_register_when_charge_is_missing() {
        let mut receiver = ScriptedReceiver {
            answers: vec![
                (Request::ping(1), Answer::Error(ERROR_INVALID_SUB_ID)),
                (Request::read_register(1, REGISTER_BATTERY_CHARGE, [0; 3]), Answer::Error(0x02)),
                (Request::read_register(1, REGISTER_BATTERY_STATUS, [0; 3]), Answer::Parameters(vec![7, 0, 0])),
            ],
        };
        let devices = read_slots(&mut receiver, &mut no_trace);
        assert_eq!(devices[0].battery, Some(Battery { percent: 90, charging: Some(false) }));
    }

    #[test]
    fn a_hidpp10_device_takes_its_name_from_the_receiver() {
        let mut receiver = ScriptedReceiver {
            answers: vec![
                (Request::ping(2), Answer::Error(ERROR_INVALID_SUB_ID)),
                (
                    Request::read_register(RECEIVER_INDEX, REGISTER_RECEIVER_INFO, [0x41, 0, 0]),
                    Answer::Parameters(vec![0x41, 3, b'K', b'8', b'0']),
                ),
            ],
        };
        let devices = read_slots(&mut receiver, &mut no_trace);
        assert_eq!(devices[0].name.as_deref(), Some("K80"));
    }

    #[test]
    fn empty_slots_are_left_out() {
        let mut receiver = ScriptedReceiver { answers: vec![(Request::ping(2), Answer::Error(0x08))] };
        assert!(read_slots(&mut receiver, &mut no_trace).is_empty());
    }

    #[test]
    fn a_sleeping_mouse_keeps_the_receivers_name_and_kind_without_a_battery() {
        let name_reply = [&[0x40, 14][..], b"B330/M330/M331"].concat();
        let mut receiver = ScriptedReceiver {
            answers: vec![
                (Request::ping(1), Answer::Error(ERROR_RESOURCE)),
                (
                    Request::read_register(RECEIVER_INDEX, REGISTER_RECEIVER_INFO, [0x40, 0, 0]),
                    Answer::Parameters(name_reply),
                ),
                (
                    Request::read_register(RECEIVER_INDEX, REGISTER_RECEIVER_INFO, [0x20, 0, 0]),
                    Answer::Parameters(vec![0x20, 0x07, 0x08, 0x40, 0x57, 0x04, 0x02, 0x02, 0x0C]),
                ),
            ],
        };
        let expected = SlotDevice {
            slot: 1,
            name: Some("B330/M330/M331".to_owned()),
            kind: Some(DongleKind::Mouse),
            battery: None,
        };
        assert_eq!(read_slots(&mut receiver, &mut no_trace), [expected]);
    }

    #[test]
    fn a_device_that_answers_the_ping_but_no_battery_query_has_no_battery() {
        let mut receiver = ScriptedReceiver { answers: vec![(Request::ping(1), Answer::Parameters(vec![4, 2, 0x5A]))] };
        let devices = read_slots(&mut receiver, &mut no_trace);
        assert_eq!((devices.len(), devices[0].battery), (1, None));
    }
}
