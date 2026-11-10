//! Windows integration shared by pages and activities.

use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use windows::{
    Win32::{
        Foundation::SIZE,
        System::{
            Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize, IBindCtx, IDataObject},
            Ole::{DROPEFFECT_COPY, DROPEFFECT_NONE, IDropSource, OleInitialize, OleUninitialize},
            Threading::{AttachThreadInput, GetCurrentThreadId},
        },
        UI::{
            Input::KeyboardAndMouse::{GetAsyncKeyState, GetDoubleClickTime, VK_LBUTTON},
            Shell::{
                BHID_DataObject, IShellItem, IShellItemImageFactory, SHCreateItemFromParsingName, SHDoDragDrop,
                SIIGBF_RESIZETOFIT, ShellExecuteW,
            },
            WindowsAndMessaging::SW_SHOWNORMAL,
        },
    },
    core::{HSTRING, PCWSTR, w},
};

pub mod app_icon;
mod bitmap;
pub mod bluetooth_devices;
pub mod clipboard;
pub mod dongle_devices;
pub mod geolocation;
pub mod notification_settings;
mod paste;
mod power;
pub mod toast_activation;
pub(crate) mod window_info;

pub use paste::{paste_target, send_paste};
pub use power::has_system_battery;

use crate::gfx::ImageData;

/// True while a file dragged out of the notch is being carried, so the notch's own drop target ignores it.
static IS_DRAGGING_OUT: AtomicBool = AtomicBool::new(false);

pub fn is_dragging_out() -> bool {
    IS_DRAGGING_OUT.load(Ordering::Relaxed)
}

pub fn open(target: &str) {
    unsafe { ShellExecuteW(None, w!("open"), &HSTRING::from(target), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL) };
}

pub fn double_click_time() -> Duration {
    Duration::from_millis(u64::from(unsafe { GetDoubleClickTime() }))
}

/// Keeps COM initialized on the current thread until dropped.
struct ComApartment(bool);

impl ComApartment {
    fn enter() -> Self {
        Self(unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.is_ok())
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        if self.0 {
            unsafe { CoUninitialize() };
        }
    }
}

/// Premultiplied BGRA; shell previews can be slow, so call off the UI thread.
pub fn file_preview(path: &Path, pixels: i32) -> Option<ImageData> {
    let _apartment = ComApartment::enter();
    let item: IShellItemImageFactory =
        unsafe { SHCreateItemFromParsingName(&HSTRING::from(path.as_os_str()), None::<&IBindCtx>) }.ok()?;
    let bitmap = unsafe { item.GetImage(SIZE { cx: pixels, cy: pixels }, SIIGBF_RESIZETOFIT) }.ok()?;
    bitmap::take_pixels(bitmap)
}

/// Dragging needs its own STA thread. Returns false if it cannot start.
/// finished receives the path and whether the drop accepted a copy.
pub fn drag_file_out(path: PathBuf, finished: impl FnOnce(PathBuf, bool) + Send + 'static) -> bool {
    if !is_left_button_down() {
        return false;
    }
    let notch_thread = unsafe { GetCurrentThreadId() };
    IS_DRAGGING_OUT.store(true, Ordering::Relaxed);
    let spawned = std::thread::Builder::new().name("shelf-drag".into()).spawn(move || {
        let was_taken = drag_out(&path, notch_thread);
        IS_DRAGGING_OUT.store(false, Ordering::Relaxed);
        finished(path, was_taken);
    });
    if spawned.is_err() {
        IS_DRAGGING_OUT.store(false, Ordering::Relaxed);
    }
    spawned.is_ok()
}

fn is_left_button_down() -> bool {
    unsafe { GetAsyncKeyState(i32::from(VK_LBUTTON.0)) < 0 }
}

/// The drag thread must share input to see the button release from the notch window.
struct SharedInput {
    own_thread: u32,
    notch_thread: u32,
    is_attached: bool,
}

impl SharedInput {
    fn attach(notch_thread: u32) -> Self {
        let own_thread = unsafe { GetCurrentThreadId() };
        let is_attached = unsafe { AttachThreadInput(own_thread, notch_thread, true) }.as_bool();
        Self { own_thread, notch_thread, is_attached }
    }
}

impl Drop for SharedInput {
    fn drop(&mut self) {
        if self.is_attached {
            let _ = unsafe { AttachThreadInput(self.own_thread, self.notch_thread, false) };
        }
    }
}

/// Offer only copying so a drop can never move or delete the original file.
fn drag_out(path: &Path, notch_thread: u32) -> bool {
    if unsafe { OleInitialize(None) }.is_err() {
        return false;
    }
    let input = SharedInput::attach(notch_thread);
    let taken = (|| {
        let item: IShellItem =
            unsafe { SHCreateItemFromParsingName(&HSTRING::from(path.as_os_str()), None::<&IBindCtx>) }.ok()?;
        let data: IDataObject = unsafe { item.BindToHandler(None::<&IBindCtx>, &BHID_DataObject) }.ok()?;
        let effect = unsafe { SHDoDragDrop(None, &data, None::<&IDropSource>, DROPEFFECT_COPY) }.ok()?;
        Some(effect != DROPEFFECT_NONE)
    })();
    drop(input);
    unsafe { OleUninitialize() };
    taken.unwrap_or(false)
}

/// The clipboard's text, or `None` when it holds no text or another program has it open.
pub fn clipboard_text() -> Option<String> {
    use windows::Win32::{
        Foundation::{HANDLE, HGLOBAL},
        System::{
            DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard},
            Memory::{GlobalLock, GlobalSize, GlobalUnlock},
        },
    };
    const CF_UNICODETEXT: u32 = 13;
    unsafe { OpenClipboard(None) }.ok()?;
    let text = (|| {
        let HANDLE(handle) = unsafe { GetClipboardData(CF_UNICODETEXT) }.ok()?;
        let memory = HGLOBAL(handle);
        let start = unsafe { GlobalLock(memory) }.cast::<u16>();
        if start.is_null() {
            return None;
        }
        // SAFETY: CF_UNICODETEXT is a null-terminated UTF-16 string that stays locked until GlobalUnlock.
        // Bounded by the block's size, in case the text lacks its terminator.
        let capacity = unsafe { GlobalSize(memory) } / size_of::<u16>();
        let length = (0..capacity).take_while(|&offset| unsafe { *start.add(offset) } != 0).count();
        let text = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(start, length) });
        let _ = unsafe { GlobalUnlock(memory) };
        Some(text)
    })();
    if let Err(error) = unsafe { CloseClipboard() } {
        crate::log::debug(&format!("could not close the clipboard: {error}"));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_file_has_no_preview() {
        assert!(file_preview(Path::new(r"C:\no\such\folder\file.png"), 64).is_none());
    }
}
