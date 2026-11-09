//! Finds the icon of an app from its Windows app identifier.

use windows::{
    Win32::{
        Foundation::{CloseHandle, SIZE},
        Graphics::Gdi::HBITMAP,
        System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
        },
        UI::Shell::{IShellItem, IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_ICONONLY},
    },
    core::{HSTRING, Interface},
};

use super::{ComApartment, bitmap, window_info};
use crate::gfx::ImageData;

const REQUESTED_PIXELS: i32 = 256;
/// Retain roughly twice the drawn size for smooth downscaling.
const STORED_PIXELS: u32 = 64;

/// Loads the icon of an app, from its Windows app identifier.
/// Packaged apps resolve through the Apps folder; classic apps (such as `Spotify.exe`) through their running process.
pub fn load(app_user_model_id: &str) -> Option<ImageData> {
    if app_user_model_id.is_empty() {
        return None;
    }
    // The shell image factory needs COM initialised on the calling thread.
    let _apartment = ComApartment::enter();
    let bitmap = image_for(&format!("shell:AppsFolder\\{app_user_model_id}"))
        .or_else(|| executable_path(app_user_model_id).and_then(|path| image_for(&path)));
    bitmap.and_then(bitmap::take_pixels).map(|icon| icon.downscaled(STORED_PIXELS))
}

fn image_for(parsing_name: &str) -> Option<HBITMAP> {
    let item: IShellItem = unsafe { SHCreateItemFromParsingName(&HSTRING::from(parsing_name), None) }.ok()?;
    let factory: IShellItemImageFactory = item.cast().ok()?;
    unsafe { factory.GetImage(SIZE { cx: REQUESTED_PIXELS, cy: REQUESTED_PIXELS }, SIIGBF_ICONONLY) }.ok()
}

fn executable_path(file_name: &str) -> Option<String> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }.ok()?;
    let mut entry = PROCESSENTRY32W { dwSize: size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
    let mut path = None;
    let mut more = unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok();
    while more && path.is_none() {
        let length = entry.szExeFile.iter().position(|&unit| unit == 0).unwrap_or(entry.szExeFile.len());
        if String::from_utf16_lossy(&entry.szExeFile[..length]).eq_ignore_ascii_case(file_name) {
            path = window_info::process_image_path(entry.th32ProcessID);
        }
        more = unsafe { Process32NextW(snapshot, &mut entry) }.is_ok();
    }
    let _ = unsafe { CloseHandle(snapshot) };
    path
}
