//! What Windows says about another program's window: its class and the executable behind it.

use windows::{
    Win32::{
        Foundation::{CloseHandle, HWND},
        System::Threading::{
            OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
        },
        UI::WindowsAndMessaging::GetClassNameW,
    },
    core::PWSTR,
};

/// The window's class name, cut at 64 characters; empty when it cannot be read.
pub fn class_name(window: HWND) -> String {
    let mut buffer = [0u16; 64];
    let length = unsafe { GetClassNameW(window, &mut buffer) }.max(0) as usize;
    String::from_utf16_lossy(&buffer[..length.min(buffer.len())])
}

/// The full path of a running process's executable, or `None` when the process cannot be opened or read.
pub fn process_image_path(process_id: u32) -> Option<String> {
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) }.ok()?;
    let mut buffer = [0u16; 1024];
    let mut length = buffer.len() as u32;
    let result =
        unsafe { QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, PWSTR(buffer.as_mut_ptr()), &mut length) };
    if let Err(error) = unsafe { CloseHandle(process) } {
        crate::log::debug(&format!("could not close a process handle: {error}"));
    }
    result.ok()?;
    Some(String::from_utf16_lossy(&buffer[..length as usize]))
}
