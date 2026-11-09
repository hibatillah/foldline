//! Keeping the notch window topmost and handing the keyboard to it and back.

use windows::Win32::{
    Foundation::HWND,
    UI::{
        Input::KeyboardAndMouse::SetFocus,
        WindowsAndMessaging::{
            GWL_EXSTYLE, GetForegroundWindow, GetWindowLongW, HWND_TOPMOST, IsWindow, SWP_NOACTIVATE, SWP_NOMOVE,
            SWP_NOOWNERZORDER, SWP_NOSIZE, SetForegroundWindow, SetWindowPos, WS_EX_TOPMOST,
        },
    },
};

use crate::log;

fn move_to_top_of_topmost_band(window: HWND) -> windows::core::Result<()> {
    let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER;
    unsafe { SetWindowPos(window, Some(HWND_TOPMOST), 0, 0, 0, 0, flags) }
}

/// Moves the window to the top of the topmost band; another topmost window can sit above it with the flag still set.
pub(in crate::host) fn raise_topmost(window: HWND) {
    if let Err(error) = move_to_top_of_topmost_band(window) {
        log::debug(&format!("could not raise topmost: {error}"));
    }
}

pub(in crate::host) fn restore_topmost(window: HWND) {
    if unsafe { GetWindowLongW(window, GWL_EXSTYLE) } as u32 & WS_EX_TOPMOST.0 != 0 {
        return;
    }
    match move_to_top_of_topmost_band(window) {
        Ok(()) => log::debug("restored topmost"),
        Err(error) => log::debug(&format!("could not restore topmost: {error}")),
    }
}

/// Brings the notch to the front so it receives typing; returns the window that had the keyboard.
pub(in crate::host) fn take_keyboard(window: HWND) -> Option<HWND> {
    let previous = unsafe { GetForegroundWindow() };
    if unsafe { SetForegroundWindow(window) }.as_bool() {
        if let Err(error) = unsafe { SetFocus(Some(window)) } {
            log::debug(&format!("could not focus the notch: {error}"));
        }
    } else {
        log::debug("could not bring the notch to the front for typing");
    }
    (!previous.is_invalid() && previous != window).then_some(previous)
}

/// Hands the keyboard back to the window that had it, when that window still exists.
pub(in crate::host) fn give_back_keyboard(previous: Option<HWND>) {
    let Some(previous) = previous else { return };
    if unsafe { IsWindow(Some(previous)) }.as_bool() && !unsafe { SetForegroundWindow(previous) }.as_bool() {
        log::debug("could not give the keyboard back");
    }
}
