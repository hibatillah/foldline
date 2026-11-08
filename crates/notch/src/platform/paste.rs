//! Sends Ctrl+V to the window that was in front when the notch opened.

use windows::Win32::{
    Foundation::HWND,
    System::Threading::GetCurrentProcessId,
    UI::{
        Input::KeyboardAndMouse::{
            INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY,
            VK_CONTROL, VK_V,
        },
        WindowsAndMessaging::{
            GetClassNameW, GetForegroundWindow, GetWindowThreadProcessId, IsWindow, SetForegroundWindow,
        },
    },
};

/// Windows that show the desktop or the taskbar take no paste.
const SHELL_CLASSES: [&str; 4] = ["Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd"];

/// The window a paste would go to: the one in front, unless it belongs to the notch or is the desktop or taskbar.
/// The notch never takes focus on a click, so this is the window the user was working in.
pub fn paste_target() -> Option<isize> {
    let window = unsafe { GetForegroundWindow() };
    if window.0.is_null() || !unsafe { IsWindow(Some(window)) }.as_bool() {
        return None;
    }
    let mut process = 0u32;
    unsafe { GetWindowThreadProcessId(window, Some(&mut process)) };
    if process == unsafe { GetCurrentProcessId() } {
        return None;
    }
    let mut class = [0u16; 64];
    let length = unsafe { GetClassNameW(window, &mut class) }.max(0) as usize;
    let class = String::from_utf16_lossy(&class[..length.min(class.len())]);
    (!SHELL_CLASSES.contains(&class.as_str())).then_some(window.0 as isize)
}

/// Gives `target` the focus if it lost it, then types Ctrl+V into it. Returns false when the input was not accepted.
pub fn send_paste(target: isize) -> bool {
    let window = HWND(target as *mut _);
    unsafe {
        if GetForegroundWindow() != window {
            let _ = SetForegroundWindow(window);
        }
        let inputs = [key(VK_CONTROL, false), key(VK_V, false), key(VK_V, true), key(VK_CONTROL, true)];
        SendInput(&inputs, size_of::<INPUT>() as i32) == inputs.len() as u32
    }
}

fn key(code: VIRTUAL_KEY, is_release: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: code,
                wScan: 0,
                dwFlags: if is_release { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
