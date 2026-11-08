//! Logitech HID++ messages: the read-only requests this module sends and the replies it decodes, as Solaar documents
//! them. No Windows calls.

pub(super) const SHORT_REPORT_ID: u8 = 0x10;
pub(super) const LONG_REPORT_ID: u8 = 0x11;
pub(super) const SHORT_LENGTH: usize = 7;
pub(super) const LONG_LENGTH: usize = 20;

pub(super) const FEATURE_DEVICE_NAME: u16 = 0x0005;
pub(super) const FEATURE_BATTERY_STATUS: u16 = 0x1000;
pub(super) const FEATURE_BATTERY_VOLTAGE: u16 = 0x1001;
pub(super) const FEATURE_UNIFIED_BATTERY: u16 = 0x1004;

pub(super) const REGISTER_BATTERY_STATUS: u16 = 0x07;
pub(super) const REGISTER_BATTERY_CHARGE: u16 = 0x0D;
pub(super) const REGISTER_RECEIVER_INFO: u16 = 0x2B5;
/// Sub-registers of `REGISTER_RECEIVER_INFO`, plus the device index minus one.
pub(super) const RECEIVER_INFO_PAIRING: u8 = 0x20;
pub(super) const RECEIVER_INFO_NAME: u8 = 0x40;

/// HID++ 1.0 error codes that matter here.
pub(super) const ERROR_INVALID_SUB_ID: u8 = 0x01;
/// The receiver has the device paired but cannot reach it (asleep or switched off).
pub(super) const ERROR_RESOURCE: u8 = 0x09;

/// The device index that addresses the receiver itself.
pub(super) const RECEIVER_INDEX: u8 = 0xFF;

/// Marks our requests so replies to other programs (Logitech Options, G Hub) and notifications (software id 0) are not
/// taken for ours.
const SOFTWARE_ID: u8 = 0x0B;
const PING_MARK: u8 = 0x5A;
const ROOT_FEATURE_INDEX: u8 = 0x00;
const HIDPP10_ERROR: u8 = 0x8F;
const HIDPP20_ERROR: u8 = 0xFF;
const REGISTER_READ_SHORT: u8 = 0x81;

/// One request: which device it goes to, the two bytes a reply echoes and its parameters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Request {
    pub device_index: u8,
    /// A feature index or register sub id.
    pub selector: u8,
    /// A function and software id, or a register address.
    pub detail: u8,
    pub parameters: Vec<u8>,
}

/// A reply that matched its request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Answer {
    /// The bytes after the echoed header.
    Parameters(Vec<u8>),
    /// A HID++ 1.0 or 2.0 error code.
    Error(u8),
}

/// A battery reading; `charging` is `None` when the device's state byte is unknown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Battery {
    pub percent: u8,
    pub charging: Option<bool>,
}

impl Request {
    /// IRoot.GetProtocolVersion: a HID++ 2.0 device answers with its version, a HID++ 1.0 one with error 0x01.
    pub(super) fn ping(device_index: u8) -> Self {
        Self::feature(device_index, ROOT_FEATURE_INDEX, 1, &[0, 0, PING_MARK])
    }

    /// IRoot.GetFeature: the reply's first byte is the feature's index on this device, 0 when it has none.
    pub(super) fn feature_index(device_index: u8, feature_id: u16) -> Self {
        Self::feature(device_index, ROOT_FEATURE_INDEX, 0, &feature_id.to_be_bytes())
    }

    pub(super) fn feature(device_index: u8, feature_index: u8, function: u8, parameters: &[u8]) -> Self {
        Self {
            device_index,
            selector: feature_index,
            detail: (function << 4) | SOFTWARE_ID,
            parameters: parameters.to_vec(),
        }
    }

    /// A HID++ 1.0 register read; registers above 0xFF are long registers (sub id 0x83).
    pub(super) fn read_register(device_index: u8, register: u16, parameters: [u8; 3]) -> Self {
        let [high, low] = register.to_be_bytes();
        Self {
            device_index,
            selector: REGISTER_READ_SHORT | (high & 0x02),
            detail: low,
            parameters: parameters.to_vec(),
        }
    }

    pub(super) fn fits_short(&self) -> bool {
        self.parameters.len() <= SHORT_LENGTH - 4
    }

    /// The report bytes, zero padded to the short or long length.
    pub(super) fn encode(&self, short: bool) -> Vec<u8> {
        let (report_id, length) =
            if short && self.fits_short() { (SHORT_REPORT_ID, SHORT_LENGTH) } else { (LONG_REPORT_ID, LONG_LENGTH) };
        let mut bytes = vec![report_id, self.device_index, self.selector, self.detail];
        bytes.extend(self.parameters.iter().take(length - 4));
        bytes.resize(length, 0);
        bytes
    }

    /// The answer `report` carries for this request, or `None` when it is another message.
    pub(super) fn answer(&self, report: &[u8]) -> Option<Answer> {
        let (&report_id, rest) = report.split_first()?;
        if !matches!(report_id, SHORT_REPORT_ID | LONG_REPORT_ID) || rest.len() < 5 || rest[0] != self.device_index {
            return None;
        }
        let echoes = |at: usize| rest[at] == self.selector && rest[at + 1] == self.detail;
        match rest[1] {
            HIDPP10_ERROR | HIDPP20_ERROR if echoes(2) => Some(Answer::Error(rest[4])),
            _ if echoes(1) => Some(Answer::Parameters(rest[3..].to_vec())),
            _ => None,
        }
    }
}

/// Feature 0x1000 GetBatteryLevelStatus: level, next level, status.
pub(super) fn decode_battery_status(parameters: &[u8]) -> Option<Battery> {
    let (&percent, &status) = (parameters.first()?, parameters.get(2)?);
    if percent == 0 {
        return None;
    }
    // 0 discharging, 1 recharging, 2 almost full, 3 full, 4 slow recharge; 5 and 6 are battery faults.
    let charging = match status {
        0 => Some(false),
        1..=4 => Some(true),
        _ => None,
    };
    Some(Battery { percent: percent.min(100), charging })
}

/// Feature 0x1004 GetStatus: state of charge, level flags, charging status.
pub(super) fn decode_unified_battery(parameters: &[u8]) -> Option<Battery> {
    let (&state_of_charge, &level_flags, &status) = (parameters.first()?, parameters.get(1)?, parameters.get(2)?);
    // A device without a gauge reports 0 % and a level; Solaar's approximations stand in for the number.
    let percent = if state_of_charge > 0 {
        state_of_charge.min(100)
    } else {
        match level_flags {
            flags if flags & 0x08 != 0 => 90,
            flags if flags & 0x04 != 0 => 50,
            flags if flags & 0x02 != 0 => 20,
            flags if flags & 0x01 != 0 => 5,
            _ => return None,
        }
    };
    // 0 discharging, 1 charging, 2 slow charging, 3 complete, 4 error.
    let charging = match status {
        0 => Some(false),
        1..=3 => Some(true),
        _ => None,
    };
    Some(Battery { percent, charging })
}

/// Millivolts to percent for one lithium cell (Solaar's table), interpolated between entries.
const VOLTAGE_CURVE: [(u16, u8); 13] = [
    (4186, 100),
    (4067, 90),
    (3989, 80),
    (3922, 70),
    (3859, 60),
    (3811, 50),
    (3778, 40),
    (3751, 30),
    (3717, 20),
    (3671, 10),
    (3646, 5),
    (3579, 2),
    (3500, 0),
];

/// Feature 0x1001 GetBatteryVoltage: millivolts (big endian) and flags; bit 7 is charging.
pub(super) fn decode_battery_voltage(parameters: &[u8]) -> Option<Battery> {
    let millivolts = u16::from_be_bytes([*parameters.first()?, *parameters.get(1)?]);
    let flags = *parameters.get(2)?;
    if millivolts == 0 {
        return None;
    }
    Some(Battery { percent: percent_from_millivolts(millivolts), charging: Some(flags & 0x80 != 0) })
}

fn percent_from_millivolts(millivolts: u16) -> u8 {
    let (highest_millivolts, _) = VOLTAGE_CURVE[0];
    if millivolts >= highest_millivolts {
        return 100;
    }
    for pair in VOLTAGE_CURVE.windows(2) {
        let ((upper_millivolts, upper_percent), (lower_millivolts, lower_percent)) = (pair[0], pair[1]);
        if millivolts >= lower_millivolts {
            let span = f32::from(upper_millivolts - lower_millivolts);
            let share = f32::from(millivolts - lower_millivolts) / span;
            return (f32::from(lower_percent) + share * f32::from(upper_percent - lower_percent)).round() as u8;
        }
    }
    0
}

/// HID++ 1.0 register 0x07: a level of 7, 5, 3 or 1 and a charging byte.
pub(super) fn decode_register_battery_status(parameters: &[u8]) -> Option<Battery> {
    let (&level, &charging_byte) = (parameters.first()?, parameters.get(1)?);
    let percent = match level {
        7 => 90,
        5 => 50,
        3 => 20,
        1 => 5,
        _ => return None,
    };
    let charging = match charging_byte {
        0x00 => Some(false),
        byte if byte & 0x21 == 0x21 || byte & 0x22 == 0x22 => Some(true),
        _ => None,
    };
    Some(Battery { percent, charging })
}

/// HID++ 1.0 register 0x0D: percent, then a status nibble in the third byte.
pub(super) fn decode_register_battery_charge(parameters: &[u8]) -> Option<Battery> {
    let (&percent, &status) = (parameters.first()?, parameters.get(2)?);
    if percent == 0 || percent > 100 {
        return None;
    }
    let charging = match status & 0xF0 {
        0x30 => Some(false),
        0x50 | 0x90 => Some(true),
        _ => None,
    };
    Some(Battery { percent, charging })
}

/// A receiver's device name (register 0x2B5, sub-register 0x4N): the echoed sub-register, a length, the name.
pub(super) fn decode_receiver_name(parameters: &[u8]) -> Option<String> {
    let length = usize::from(*parameters.get(1)?);
    let name = parameters.get(2..2 + length)?;
    clean_name(name)
}

/// Joins the chunks of feature 0x0005 GetDeviceName up to `length` bytes.
pub(super) fn assemble_name(chunks: &[Vec<u8>], length: usize) -> Option<String> {
    let bytes: Vec<u8> = chunks.iter().flatten().copied().take(length).collect();
    clean_name(&bytes)
}

fn clean_name(bytes: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(bytes);
    let name = text.trim_matches(char::from(0)).trim();
    (!name.is_empty()).then(|| name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ping_is_a_short_root_request_with_our_software_id() {
        assert_eq!(Request::ping(1).encode(true), vec![0x10, 0x01, 0x00, 0x1B, 0x00, 0x00, 0x5A]);
    }

    #[test]
    fn a_feature_lookup_carries_the_feature_id_big_endian() {
        let bytes = Request::feature_index(2, FEATURE_UNIFIED_BATTERY).encode(true);
        assert_eq!(bytes, vec![0x10, 0x02, 0x00, 0x0B, 0x10, 0x04, 0x00]);
    }

    #[test]
    fn a_long_encoding_pads_to_twenty_bytes() {
        let bytes = Request::feature(1, 0x05, 1, &[0x00]).encode(false);
        assert_eq!((bytes.len(), &bytes[..5]), (LONG_LENGTH, &[0x11, 0x01, 0x05, 0x1B, 0x00][..]));
    }

    #[test]
    fn a_long_register_is_read_with_sub_id_0x83() {
        let bytes = Request::read_register(RECEIVER_INDEX, REGISTER_RECEIVER_INFO, [0x40, 0, 0]).encode(true);
        assert_eq!(bytes, vec![0x10, 0xFF, 0x83, 0xB5, 0x40, 0x00, 0x00]);
    }

    #[test]
    fn a_hidpp20_ping_reply_gives_the_protocol_version() {
        let reply = [0x11, 0x01, 0x00, 0x1B, 0x04, 0x05, 0x5A, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let answer = Request::ping(1).answer(&reply);
        assert!(matches!(answer, Some(Answer::Parameters(parameters)) if parameters[..3] == [0x04, 0x05, 0x5A]));
    }

    #[test]
    fn a_hidpp10_error_reply_to_a_ping_gives_error_0x01() {
        let reply = [0x10, 0x01, 0x8F, 0x00, 0x1B, 0x01, 0x00];
        assert_eq!(Request::ping(1).answer(&reply), Some(Answer::Error(ERROR_INVALID_SUB_ID)));
    }

    #[test]
    fn a_hidpp20_error_reply_gives_its_code() {
        let reply = [0x11, 0x01, 0xFF, 0x03, 0x1B, 0x09, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(Request::feature(1, 3, 1, &[]).answer(&reply), Some(Answer::Error(0x09)));
    }

    #[test]
    fn a_notification_from_another_device_index_is_not_an_answer() {
        let reply = [0x10, 0x02, 0x00, 0x1B, 0x04, 0x05, 0x5A];
        assert_eq!(Request::ping(1).answer(&reply), None);
    }

    #[test]
    fn a_reply_to_another_programs_software_id_is_not_an_answer() {
        let reply = [0x11, 0x01, 0x00, 0x11, 0x04, 0x05, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(Request::ping(1).answer(&reply), None);
    }

    #[test]
    fn battery_status_0x1000_reads_level_and_discharging() {
        assert_eq!(decode_battery_status(&[60, 50, 0x00]), Some(Battery { percent: 60, charging: Some(false) }));
    }

    #[test]
    fn battery_status_0x1000_recharging_is_charging() {
        assert_eq!(decode_battery_status(&[35, 30, 0x01]), Some(Battery { percent: 35, charging: Some(true) }));
    }

    #[test]
    fn battery_status_0x1000_with_level_zero_is_no_reading() {
        assert_eq!(decode_battery_status(&[0, 0, 0x00]), None);
    }

    #[test]
    fn unified_battery_0x1004_reads_the_state_of_charge() {
        assert_eq!(decode_unified_battery(&[82, 0x04, 0x00, 0]), Some(Battery { percent: 82, charging: Some(false) }));
    }

    #[test]
    fn unified_battery_0x1004_without_a_gauge_approximates_from_the_level() {
        assert_eq!(decode_unified_battery(&[0, 0x02, 0x01, 1]), Some(Battery { percent: 20, charging: Some(true) }));
    }

    #[test]
    fn battery_voltage_0x1001_maps_millivolts_to_percent() {
        let reply = 3811u16.to_be_bytes();
        assert_eq!(
            decode_battery_voltage(&[reply[0], reply[1], 0x00]),
            Some(Battery { percent: 50, charging: Some(false) })
        );
    }

    #[test]
    fn battery_voltage_between_entries_is_interpolated() {
        assert_eq!(percent_from_millivolts(3835), 55);
    }

    #[test]
    fn register_0x07_level_five_is_about_half() {
        assert_eq!(decode_register_battery_status(&[5, 0x00, 0]), Some(Battery { percent: 50, charging: Some(false) }));
    }

    #[test]
    fn register_0x07_recharging_byte_is_charging() {
        assert_eq!(decode_register_battery_status(&[3, 0x21, 0]), Some(Battery { percent: 20, charging: Some(true) }));
    }

    #[test]
    fn register_0x07_unknown_level_is_no_reading() {
        assert_eq!(decode_register_battery_status(&[4, 0x00, 0]), None);
    }

    #[test]
    fn register_0x0d_reads_percent_and_discharging() {
        assert_eq!(
            decode_register_battery_charge(&[74, 0, 0x30]),
            Some(Battery { percent: 74, charging: Some(false) })
        );
    }

    #[test]
    fn a_receiver_name_reply_gives_the_name() {
        assert_eq!(decode_receiver_name(&[0x40, 4, b'M', b'3', b'3', b'1', 0, 0]), Some("M331".to_owned()));
    }

    #[test]
    fn device_name_chunks_join_up_to_the_length() {
        let chunks = vec![b"Wireless Mouse M".to_vec(), b"331\0\0\0\0\0\0\0\0\0\0\0\0\0".to_vec()];
        assert_eq!(assemble_name(&chunks, 19), Some("Wireless Mouse M331".to_owned()));
    }
}
