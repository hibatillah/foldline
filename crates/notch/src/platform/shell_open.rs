//! Opens a downloaded file, or shows it selected in its folder.

use std::path::Path;

use windows::{
    Win32::UI::{
        Shell::{ILCreateFromPathW, ILFree, SHOpenFolderAndSelectItems, ShellExecuteW},
        WindowsAndMessaging::SW_SHOWNORMAL,
    },
    core::{HSTRING, PCWSTR, w},
};

use super::ComApartment;

/// Opens the file with its default app.
pub fn open_file(path: &Path) {
    let result = unsafe {
        ShellExecuteW(None, w!("open"), &HSTRING::from(path.as_os_str()), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL)
    };
    // ShellExecuteW reports success with a value above 32.
    if result.0 as usize <= 32 {
        crate::log::debug(&format!("could not open the file: shell error {}", result.0 as usize));
    }
}

/// Opens the file's folder in Explorer with the file selected.
pub fn show_in_folder(path: &Path) {
    let _apartment = ComApartment::enter();
    let item_list = unsafe { ILCreateFromPathW(&HSTRING::from(path.as_os_str())) };
    if item_list.is_null() {
        crate::log::debug("could not show the file in its folder: no item id list");
        return;
    }
    if let Err(error) = unsafe { SHOpenFolderAndSelectItems(item_list, None, 0) } {
        crate::log::debug(&format!("could not show the file in its folder: {error}"));
    }
    // SAFETY: `item_list` came from `ILCreateFromPathW` and is freed exactly once, after its last use.
    unsafe { ILFree(Some(item_list)) };
}
