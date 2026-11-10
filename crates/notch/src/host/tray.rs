use windows::{
    Win32::{
        Foundation::{HWND, POINT},
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
        UI::{
            HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi},
            Shell::{NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW},
            WindowsAndMessaging::{
                AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, MF_CHECKED, MF_SEPARATOR, MF_STRING,
                SM_CXSMICON, SetForegroundWindow, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu, WM_APP,
            },
        },
    },
    core::{HSTRING, PCSTR, PCWSTR, Result, w},
};

use super::icon::Icon;
use crate::i18n::tr;

pub(super) const WM_TRAY: u32 = WM_APP + 2;
const TRAY_ICON_ID: u32 = 1;
const TOOLTIP: &str = "Notch";
const MENU_COMMAND_QUIT: usize = 1;
const MENU_COMMAND_START_WITH_WINDOWS: usize = 3;
const MENU_COMMAND_OPEN_SETTINGS: usize = 7;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MenuChoice {
    OpenSettings,
    ToggleStartWithWindows,
    Quit,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct MenuState {
    pub(super) starts_with_windows: bool,
}

/// The choice a menu command stands for; `None` when the menu was dismissed or the command is unknown.
fn choice_for_command(command: usize) -> Option<MenuChoice> {
    match command {
        MENU_COMMAND_OPEN_SETTINGS => Some(MenuChoice::OpenSettings),
        MENU_COMMAND_START_WITH_WINDOWS => Some(MenuChoice::ToggleStartWithWindows),
        MENU_COMMAND_QUIT => Some(MenuChoice::Quit),
        _ => None,
    }
}

fn notify_data(window: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: window,
        uID: TRAY_ICON_ID,
        ..Default::default()
    }
}

/// Adds the notch's icon to the notification area; also called again if Explorer restarts.
pub(super) fn add(window: HWND) -> Result<()> {
    let dpi = unsafe { GetDpiForWindow(window) };
    let size = unsafe { GetSystemMetricsForDpi(SM_CXSMICON, dpi) };
    let icon = Icon::load(size)?;
    let mut data = notify_data(window);
    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    data.uCallbackMessage = WM_TRAY;
    data.hIcon = icon.handle();
    for (slot, unit) in data.szTip.iter_mut().zip(TOOLTIP.encode_utf16()) {
        *slot = unit;
    }
    // The shell copies the icon, so this temporary size-specific handle can be released afterwards.
    unsafe { Shell_NotifyIconW(NIM_ADD, &data).ok() }
}

/// Removes the icon; without this a stale icon stays in the tray until the pointer hovers it.
pub(super) fn remove(window: HWND) {
    let data = notify_data(window);
    let _ = unsafe { Shell_NotifyIconW(NIM_DELETE, &data) };
}

/// Undocumented uxtheme ordinals; resolve them at runtime so unsupported Windows versions still work.
const SET_PREFERRED_APP_MODE: usize = 135;
const FLUSH_MENU_THEMES: usize = 136;
const APP_MODE_ALLOW_DARK: i32 = 1;

/// Missing theme entry points leave the default light menu in place.
fn follow_system_theme() {
    type Entry = unsafe extern "system" fn() -> isize;
    unsafe {
        let Ok(module) = LoadLibraryW(w!("uxtheme.dll")) else { return };
        if let Some(entry) = GetProcAddress(module, PCSTR(SET_PREFERRED_APP_MODE as *const u8)) {
            // SAFETY: this entry point takes the app mode and returns the previous one.
            let set_mode = std::mem::transmute::<Entry, extern "system" fn(i32) -> i32>(entry);
            set_mode(APP_MODE_ALLOW_DARK);
        }
        if let Some(entry) = GetProcAddress(module, PCSTR(FLUSH_MENU_THEMES as *const u8)) {
            // SAFETY: this entry point takes nothing and returns nothing.
            let flush = std::mem::transmute::<Entry, extern "system" fn()>(entry);
            flush();
        }
    }
}

pub(super) fn show_context_menu(window: HWND, state: MenuState) -> Option<MenuChoice> {
    // Each time, so a change of theme while the notch runs shows in the next menu.
    follow_system_theme();
    let Ok(menu) = (unsafe { CreatePopupMenu() }) else { return None };
    let startup_flags = if state.starts_with_windows { MF_STRING | MF_CHECKED } else { MF_STRING };
    // Read each time the menu opens, so a language change shows in the next menu.
    let (start_text, settings_text, quit_text) = (
        HSTRING::from(tr!("tray.start_with_windows").as_ref()),
        HSTRING::from(tr!("tray.settings").as_ref()),
        HSTRING::from(tr!("tray.quit").as_ref()),
    );
    let _ = unsafe { AppendMenuW(menu, startup_flags, MENU_COMMAND_START_WITH_WINDOWS, &start_text) };
    let _ = unsafe { AppendMenuW(menu, MF_STRING, MENU_COMMAND_OPEN_SETTINGS, &settings_text) };
    let _ = unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null()) };
    let _ = unsafe { AppendMenuW(menu, MF_STRING, MENU_COMMAND_QUIT, &quit_text) };
    let mut cursor = POINT::default();
    let _ = unsafe { GetCursorPos(&mut cursor) };
    // A popup menu needs a foreground owner to dismiss correctly when clicking elsewhere.
    let _ = unsafe { SetForegroundWindow(window) };
    let flags = TPM_RIGHTBUTTON | TPM_RETURNCMD;
    let command = unsafe { TrackPopupMenu(menu, flags, cursor.x, cursor.y, None, window, None) };
    let _ = unsafe { DestroyMenu(menu) };
    choice_for_command(command.0 as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_item_has_its_own_command() {
        assert_eq!(choice_for_command(MENU_COMMAND_OPEN_SETTINGS), Some(MenuChoice::OpenSettings));
        assert_eq!(choice_for_command(MENU_COMMAND_START_WITH_WINDOWS), Some(MenuChoice::ToggleStartWithWindows));
        assert_eq!(choice_for_command(MENU_COMMAND_QUIT), Some(MenuChoice::Quit));
    }

    #[test]
    fn a_dismissed_menu_and_retired_commands_choose_nothing() {
        for command in [0, 2, 4, 5, 6, 50, 100, 1000] {
            assert_eq!(choice_for_command(command), None, "command {command}");
        }
    }

    #[test]
    #[ignore = "depends on the Windows version; run on purpose"]
    fn this_windows_has_the_entry_points_that_make_menus_follow_the_theme() {
        let module = unsafe { LoadLibraryW(w!("uxtheme.dll")) }.expect("uxtheme.dll");
        for ordinal in [SET_PREFERRED_APP_MODE, FLUSH_MENU_THEMES] {
            assert!(unsafe { GetProcAddress(module, PCSTR(ordinal as *const u8)) }.is_some(), "ordinal {ordinal}");
        }
    }
}
