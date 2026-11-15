use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, RegGetValueW};
use windows::Win32::System::WindowsProgramming::GetComputerNameW;
use windows::core::{PWSTR, w};

use crate::hash;

const MAX_NAME_LENGTH: usize = 64;

/// SHA-256 of `"notch-device-v1:" + MachineGuid` as lowercase hex; debug builds take `NOTCH_DEVICE_HASH_OVERRIDE` instead when valid.
pub fn device_hash() -> Option<String> {
    let overridden = valid_override(std::env::var("NOTCH_DEVICE_HASH_OVERRIDE").ok().as_deref());
    if cfg!(debug_assertions) && overridden.is_some() {
        return overridden;
    }
    hash_of_machine_guid(&machine_guid()?)
}

/// The computer name, cut to the 64 characters the site accepts.
pub fn device_name() -> String {
    let mut buffer = [0u16; 256];
    let mut length = buffer.len() as u32;
    // SAFETY: `buffer` holds `length` UTF-16 units and `length` receives the count written.
    let read = unsafe { GetComputerNameW(Some(PWSTR(buffer.as_mut_ptr())), &mut length) };
    if read.is_err() {
        return String::new();
    }
    String::from_utf16_lossy(&buffer[..length as usize]).chars().take(MAX_NAME_LENGTH).collect()
}

fn hash_of_machine_guid(machine_guid: &str) -> Option<String> {
    Some(hash::to_hex(&hash::sha256(format!("notch-device-v1:{machine_guid}").as_bytes())?))
}

/// Exactly 64 hex characters, lowercased; anything else is ignored.
fn valid_override(text: Option<&str>) -> Option<String> {
    let text = text?;
    (text.len() == 64 && text.bytes().all(|byte| byte.is_ascii_hexdigit())).then(|| text.to_ascii_lowercase())
}

/// Reads the registry view of the process; the notch ships as x64 only, so this is the 64-bit MachineGuid.
fn machine_guid() -> Option<String> {
    let mut buffer = [0u16; 128];
    let mut byte_count = std::mem::size_of_val(&buffer) as u32;
    // SAFETY: `buffer` is `byte_count` bytes long and `byte_count` receives the size written.
    let status = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            w!(r"SOFTWARE\Microsoft\Cryptography"),
            w!("MachineGuid"),
            RRF_RT_REG_SZ,
            None,
            Some(buffer.as_mut_ptr().cast()),
            Some(&mut byte_count),
        )
    };
    if status != ERROR_SUCCESS {
        return None;
    }
    let unit_count = (byte_count as usize / 2).min(buffer.len());
    let text = String::from_utf16_lossy(&buffer[..unit_count]);
    let text = text.trim_end_matches('\0');
    (!text.is_empty()).then(|| text.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_device_hash_is_the_sha256_of_the_prefixed_machine_guid() {
        let expected = hash::to_hex(&hash::sha256(b"notch-device-v1:guid-1").expect("cng works"));
        assert_eq!(hash_of_machine_guid("guid-1"), Some(expected));
        assert_ne!(hash_of_machine_guid("guid-1"), hash_of_machine_guid("guid-2"));
    }

    #[test]
    fn only_64_hex_characters_count_as_an_override() {
        assert_eq!(valid_override(Some(&"AB".repeat(32))), Some("ab".repeat(32)));
        assert_eq!(valid_override(Some(&"a".repeat(63))), None);
        assert_eq!(valid_override(Some(&"g".repeat(64))), None);
        assert_eq!(valid_override(None), None);
    }

    #[test]
    fn this_pc_has_a_machine_guid_and_a_name() {
        assert!(machine_guid().is_some());
        assert!(!device_name().is_empty());
    }
}
