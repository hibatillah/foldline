//! What a disk row shows beside its name: the kind of drive, its size and how full it is. Only a drive letter such
//! as `C:` can be read; any other name has no figures.

use windows::{
    Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetDriveTypeW},
    core::PCWSTR,
};

use crate::i18n::tr;

// `GetDriveTypeW` results; the constants live in a feature the window does not enable.
const DRIVE_REMOVABLE: u32 = 2;
const DRIVE_FIXED: u32 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriveKind {
    Local,
    Removable,
    Other,
}

impl DriveKind {
    pub fn label(self) -> &'static str {
        match self {
            DriveKind::Local => tr!("stats.disk_local"),
            DriveKind::Removable => tr!("stats.disk_removable"),
            DriveKind::Other => tr!("stats.disk_other"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiskInfo {
    pub kind: DriveKind,
    pub total_bytes: u64,
    pub used_percent: u32,
}

/// The root path of a drive written as a letter and a colon (`C:` gives `C:\`).
fn root_path(name: &str) -> Option<Vec<u16>> {
    let mut chars = name.trim().chars();
    let letter = chars.next().filter(char::is_ascii_alphabetic)?;
    (chars.as_str() == ":").then(|| format!("{}:\\", letter.to_ascii_uppercase()).encode_utf16().chain([0]).collect())
}

pub fn read(name: &str) -> Option<DiskInfo> {
    let root = root_path(name)?;
    let root = PCWSTR(root.as_ptr());
    let (mut total_bytes, mut free_bytes) = (0u64, 0u64);
    // SAFETY: `root` is a NUL-terminated string that outlives both calls, and the two outputs are live locals.
    let read = unsafe { GetDiskFreeSpaceExW(root, None, Some(&mut total_bytes), Some(&mut free_bytes)) };
    read.ok()?;
    // SAFETY: the same string.
    let kind = match unsafe { GetDriveTypeW(root) } {
        DRIVE_FIXED => DriveKind::Local,
        DRIVE_REMOVABLE => DriveKind::Removable,
        _ => DriveKind::Other,
    };
    Some(DiskInfo { kind, total_bytes, used_percent: used_percent(total_bytes, free_bytes) })
}

/// How much of the disk is taken, rounded to a whole percent; an empty disk reads 0.
pub fn used_percent(total_bytes: u64, free_bytes: u64) -> u32 {
    let used = u128::from(total_bytes.saturating_sub(free_bytes));
    ((used * 100 + u128::from(total_bytes) / 2).checked_div(u128::from(total_bytes)).unwrap_or(0)) as u32
}

/// The size the way a disk is sold: "512 GB", "1 TB", "1.5 TB".
pub fn size_text(bytes: u64) -> String {
    const GIGABYTE: u64 = 1_000_000_000;
    const TERABYTE: u64 = 1_000 * GIGABYTE;
    if bytes >= TERABYTE {
        let tenths = (bytes + TERABYTE / 20) / (TERABYTE / 10);
        if tenths.is_multiple_of(10) {
            format!("{} TB", tenths / 10)
        } else {
            format!("{}.{} TB", tenths / 10, tenths % 10)
        }
    } else {
        format!("{} GB", (bytes + GIGABYTE / 2) / GIGABYTE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_drive_letter_has_a_root_path() {
        assert_eq!(root_path("c:"), Some("C:\\\0".encode_utf16().collect()));
        assert_eq!(root_path("C:\\"), None);
        assert_eq!(root_path("Data"), None);
        assert_eq!(root_path(""), None);
    }

    #[test]
    fn the_used_share_is_rounded_to_a_whole_percent() {
        assert_eq!(used_percent(512, 26), 95);
        assert_eq!(used_percent(1000, 1000), 0);
        assert_eq!(used_percent(0, 0), 0);
    }

    #[test]
    fn a_size_reads_in_gigabytes_below_a_terabyte_and_terabytes_above() {
        assert_eq!(size_text(512_110_190_592), "512 GB");
        assert_eq!(size_text(1_000_204_886_016), "1 TB");
        assert_eq!(size_text(1_500_000_000_000), "1.5 TB");
    }

    #[test]
    fn the_system_drive_can_be_read() {
        let info = read("C:").expect("C: exists");
        assert!(info.total_bytes > 0 && info.used_percent <= 100);
    }
}
