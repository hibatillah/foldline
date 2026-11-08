//! Reads and writes the Windows clipboard for the Clipboard page. Nothing here runs unless that page is on.

use std::{
    sync::{
        OnceLock,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};

use windows::{
    Win32::{
        Foundation::{GlobalFree, HANDLE, HGLOBAL, HWND},
        System::{
            DataExchange::{
                CloseClipboard, EmptyClipboard, GetClipboardData, GetClipboardSequenceNumber,
                IsClipboardFormatAvailable, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
            },
            Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock},
        },
    },
    core::{PCWSTR, w},
};

const FORMAT_UNICODE_TEXT: u32 = 13;
const FORMAT_DIB: u32 = 8;
const FORMAT_DIB_V5: u32 = 17;
const OPEN_ATTEMPTS: u32 = 10;
const OPEN_RETRY_DELAY: Duration = Duration::from_millis(5);

/// The clipboard sequence number right after the notch wrote to it; `0` before the first write.
static OWN_SEQUENCE: AtomicU32 = AtomicU32::new(0);

/// What one copy left on the clipboard. Text wins over a picture when both are there, as in most clipboard managers.
#[derive(Default)]
pub struct Snapshot {
    /// The copy asked not to be recorded (a password manager); nothing else is read then.
    pub is_excluded: bool,
    pub text: Option<String>,
    /// The raw bytes of `CF_DIBV5` or `CF_DIB`, without the file header.
    pub dib: Option<Vec<u8>>,
    /// The picture also came as PNG, which names its format in the details line.
    pub has_png: bool,
    /// The text was longer than the limit and was left out.
    pub is_text_too_long: bool,
    /// The picture was larger than the limit and was left out.
    pub is_image_too_large: bool,
}

/// True when the copy that is on the clipboard now was made by the notch itself, so nothing should react to it.
pub fn is_own_write() -> bool {
    let own = OWN_SEQUENCE.load(Ordering::Relaxed);
    own != 0 && own == unsafe { GetClipboardSequenceNumber() }
}

fn registered(name: PCWSTR, cache: &'static OnceLock<u32>) -> u32 {
    *cache.get_or_init(|| unsafe { RegisterClipboardFormatW(name) })
}

fn is_available(format: u32) -> bool {
    format != 0 && unsafe { IsClipboardFormatAvailable(format) }.is_ok()
}

/// Password managers mark a copy with these to ask clipboard watchers and history to ignore it.
pub fn is_excluded() -> bool {
    static MONITOR: OnceLock<u32> = OnceLock::new();
    static HISTORY: OnceLock<u32> = OnceLock::new();
    if is_available(registered(w!("ExcludeClipboardContentFromMonitorProcessing"), &MONITOR)) {
        return true;
    }
    let history = registered(w!("CanIncludeInClipboardHistory"), &HISTORY);
    is_available(history) && with_clipboard(None, || history_value_is_zero(history)).unwrap_or(false)
}

fn history_value_is_zero(format: u32) -> bool {
    read_bytes(format, 16).is_some_and(|bytes| bytes.len() >= 4 && bytes[..4] == [0, 0, 0, 0])
}

/// Reads what is on the clipboard now. `max_text_bytes` and `max_dib_bytes` bound what is copied out of it.
pub fn read_snapshot(max_text_bytes: usize, max_dib_bytes: usize) -> Snapshot {
    static PNG: OnceLock<u32> = OnceLock::new();
    let mut snapshot = Snapshot { is_excluded: is_excluded(), ..Snapshot::default() };
    if snapshot.is_excluded {
        return snapshot;
    }
    let has_png = is_available(registered(w!("PNG"), &PNG));
    let read = with_clipboard(None, || {
        if is_available(FORMAT_UNICODE_TEXT) {
            // Two more bytes than the text limit make room for the terminating NUL.
            match read_bytes(FORMAT_UNICODE_TEXT, max_text_bytes * 2 + 2) {
                Some(bytes) => snapshot.text = Some(utf16_text(&bytes)),
                None => snapshot.is_text_too_long = true,
            }
        }
        if snapshot.text.is_none() && !snapshot.is_text_too_long {
            for format in [FORMAT_DIB_V5, FORMAT_DIB] {
                if !is_available(format) {
                    continue;
                }
                match read_bytes(format, max_dib_bytes) {
                    Some(bytes) => {
                        snapshot.dib = Some(bytes);
                        snapshot.has_png = has_png;
                    }
                    None => snapshot.is_image_too_large = true,
                }
                break;
            }
        }
    });
    if read.is_none() {
        return Snapshot::default();
    }
    snapshot
}

/// Replaces the clipboard with `text`; `owner` is the notch window.
pub fn write_text(owner: HWND, text: &str) -> bool {
    let mut units: Vec<u16> = text.encode_utf16().collect();
    units.push(0);
    let bytes: Vec<u8> = units.iter().flat_map(|unit| unit.to_le_bytes()).collect();
    write(owner, FORMAT_UNICODE_TEXT, &bytes)
}

/// Replaces the clipboard with a picture given as the bytes of a `CF_DIBV5` structure.
pub fn write_dib_v5(owner: HWND, dib: &[u8]) -> bool {
    write(owner, FORMAT_DIB_V5, dib)
}

fn write(owner: HWND, format: u32, bytes: &[u8]) -> bool {
    let written = with_clipboard(Some(owner), || unsafe {
        if EmptyClipboard().is_err() {
            return false;
        }
        let Ok(memory) = GlobalAlloc(GMEM_MOVEABLE, bytes.len()) else { return false };
        let target = GlobalLock(memory);
        if target.is_null() {
            let _ = GlobalFree(Some(memory));
            return false;
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), target.cast::<u8>(), bytes.len());
        let _ = GlobalUnlock(memory);
        // On success the system owns the memory; on failure it is still ours to free.
        if SetClipboardData(format, Some(HANDLE(memory.0))).is_err() {
            let _ = GlobalFree(Some(memory));
            return false;
        }
        true
    })
    .unwrap_or(false);
    if written {
        OWN_SEQUENCE.store(unsafe { GetClipboardSequenceNumber() }, Ordering::Relaxed);
    }
    written
}

/// Opens the clipboard for the duration of `work`; another program may hold it for a moment, so it retries briefly.
fn with_clipboard<R>(owner: Option<HWND>, work: impl FnOnce() -> R) -> Option<R> {
    for attempt in 0..OPEN_ATTEMPTS {
        if unsafe { OpenClipboard(owner) }.is_ok() {
            let result = work();
            let _ = unsafe { CloseClipboard() };
            return Some(result);
        }
        if attempt + 1 < OPEN_ATTEMPTS {
            std::thread::sleep(OPEN_RETRY_DELAY);
        }
    }
    None
}

/// The bytes of one format, or `None` when it is missing or larger than `limit`. The clipboard must be open.
fn read_bytes(format: u32, limit: usize) -> Option<Vec<u8>> {
    unsafe {
        let handle = GetClipboardData(format).ok()?;
        let memory = HGLOBAL(handle.0);
        let size = GlobalSize(memory);
        if size == 0 || size > limit {
            return None;
        }
        let source = GlobalLock(memory);
        if source.is_null() {
            return None;
        }
        let bytes = std::slice::from_raw_parts(source.cast::<u8>(), size).to_vec();
        let _ = GlobalUnlock(memory);
        Some(bytes)
    }
}

/// UTF-16 up to the first NUL; a lone half of a surrogate pair becomes U+FFFD.
fn utf16_text(bytes: &[u8]) -> String {
    let units: Vec<u16> =
        bytes.as_chunks::<2>().0.iter().map(|pair| u16::from_le_bytes(*pair)).take_while(|&unit| unit != 0).collect();
    String::from_utf16_lossy(&units)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_ends_at_the_first_nul() {
        let bytes: Vec<u8> = "hi\0junk".encode_utf16().flat_map(u16::to_le_bytes).collect();
        assert_eq!(utf16_text(&bytes), "hi");
    }

    #[test]
    fn text_keeps_characters_outside_the_basic_plane() {
        let bytes: Vec<u8> = "a\u{1F600}b".encode_utf16().flat_map(u16::to_le_bytes).collect();
        assert_eq!(utf16_text(&bytes), "a\u{1F600}b");
    }
}
