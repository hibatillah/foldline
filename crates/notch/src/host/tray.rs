use windows::{
    Win32::{
        Foundation::{HWND, POINT},
        System::{
            LibraryLoader::{GetProcAddress, LoadLibraryW},
            Time::{GetTimeZoneInformation, TIME_ZONE_INFORMATION},
        },
        UI::WindowsAndMessaging::HMENU,
        UI::{
            HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi},
            Shell::{NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW},
            WindowsAndMessaging::{
                AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, MF_GRAYED, MF_POPUP, MF_SEPARATOR, MF_STRING,
                SM_CXSMICON, SetForegroundWindow, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu, WM_APP,
            },
        },
    },
    core::{HSTRING, PCSTR, PCWSTR, Result, w},
};

use store::{
    pause::{Pause, PauseChoice, clock_text},
    snooze::{Snooze, SnoozeChoice},
};

use super::icon::Icon;
use crate::i18n::tr;

pub(super) const WM_TRAY: u32 = WM_APP + 2;
const TRAY_ICON_ID: u32 = 1;
const TOOLTIP: &str = "Foldline";
const MENU_COMMAND_QUIT: usize = 1;
const MENU_COMMAND_OPEN_SETTINGS: usize = 7;
/// The four pause lengths take the commands from here, in the order of `PauseChoice::ALL`.
const MENU_COMMAND_PAUSE_FIRST: usize = 20;
const MENU_COMMAND_RESUME: usize = 24;
/// The four snooze lengths take the commands from here, in the order of `SnoozeChoice::ALL`.
const MENU_COMMAND_SNOOZE_FIRST: usize = 30;
const MENU_COMMAND_TURN_OFF_SNOOZE: usize = 34;
/// What `GetTimeZoneInformation` returns while daylight saving time applies.
const TIME_ZONE_ID_DAYLIGHT: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MenuChoice {
    OpenSettings,
    Quit,
    Pause(PauseChoice),
    Resume,
    Snooze(SnoozeChoice),
    TurnOffSnooze,
}

#[derive(Clone, Debug, Default)]
pub(super) struct MenuState {
    /// `None` when not paused; else the greyed line, "Paused until 14:30" or "Paused".
    pub(super) paused_line: Option<String>,
    /// `None` when not snoozed; else the greyed line, "Snoozed until 14:30" or "Snoozed".
    pub(super) snoozed_line: Option<String>,
}

/// The choice a menu command stands for; `None` when the menu was dismissed or the command is unknown.
fn choice_for_command(command: usize) -> Option<MenuChoice> {
    match command {
        MENU_COMMAND_OPEN_SETTINGS => Some(MenuChoice::OpenSettings),
        MENU_COMMAND_QUIT => Some(MenuChoice::Quit),
        MENU_COMMAND_RESUME => Some(MenuChoice::Resume),
        MENU_COMMAND_TURN_OFF_SNOOZE => Some(MenuChoice::TurnOffSnooze),
        MENU_COMMAND_SNOOZE_FIRST.. => command
            .checked_sub(MENU_COMMAND_SNOOZE_FIRST)
            .and_then(|position| SnoozeChoice::ALL.get(position))
            .map(|&choice| MenuChoice::Snooze(choice)),
        _ => command
            .checked_sub(MENU_COMMAND_PAUSE_FIRST)
            .and_then(|position| PauseChoice::ALL.get(position))
            .map(|&choice| MenuChoice::Pause(choice)),
    }
}

/// Seconds east of UTC from Windows' bias, which counts minutes west of UTC.
fn utc_offset_seconds(bias: i32, daylight_bias: i32, is_daylight_time: bool) -> i32 {
    -(bias + if is_daylight_time { daylight_bias } else { 0 }) * 60
}

fn local_utc_offset_seconds() -> i32 {
    let mut zone = TIME_ZONE_INFORMATION::default();
    let zone_id = unsafe { GetTimeZoneInformation(&mut zone) };
    utc_offset_seconds(zone.Bias, zone.DaylightBias, zone_id == TIME_ZONE_ID_DAYLIGHT)
}

/// The greyed menu line for a pause; `None` when the notch is not paused.
pub(super) fn paused_line(pause: Pause) -> Option<String> {
    match pause {
        Pause::Off => None,
        Pause::Manual => Some(tr!("tray.paused").into_owned()),
        Pause::Until(end_unix) => {
            Some(tr!("tray.paused_until", time = clock_text(end_unix, local_utc_offset_seconds())).into_owned())
        }
    }
}

/// The greyed menu line for a snooze; `None` when not snoozed.
pub(super) fn snoozed_line(snooze: Snooze) -> Option<String> {
    match snooze {
        Snooze::Off => None,
        Snooze::Manual => Some(tr!("tray.snoozed").into_owned()),
        Snooze::Until(end_unix) => {
            Some(tr!("tray.snoozed_until", time = clock_text(end_unix, local_utc_offset_seconds())).into_owned())
        }
    }
}

fn snooze_choice_text(choice: SnoozeChoice) -> HSTRING {
    let text = match choice {
        SnoozeChoice::TenMinutes => tr!("tray.pause_10"),
        SnoozeChoice::ThirtyMinutes => tr!("tray.pause_30"),
        SnoozeChoice::OneHour => tr!("tray.pause_60"),
        SnoozeChoice::UntilTurnedOff => tr!("tray.snooze_manual"),
    };
    HSTRING::from(text.as_ref())
}

/// Adds the "Snooze notifications" submenu, or the greyed status line and Turn off snooze while snoozed.
fn append_snooze_items(menu: HMENU, state: &MenuState) {
    if let Some(snoozed_line) = &state.snoozed_line {
        let (line_text, turn_off_text) =
            (HSTRING::from(snoozed_line.as_str()), HSTRING::from(tr!("tray.snooze_off").as_ref()));
        let _ = unsafe { AppendMenuW(menu, MF_STRING | MF_GRAYED, 0, &line_text) };
        let _ = unsafe { AppendMenuW(menu, MF_STRING, MENU_COMMAND_TURN_OFF_SNOOZE, &turn_off_text) };
        return;
    }
    let Ok(submenu) = (unsafe { CreatePopupMenu() }) else { return };
    for (position, choice) in SnoozeChoice::ALL.into_iter().enumerate() {
        let command = MENU_COMMAND_SNOOZE_FIRST + position;
        let _ = unsafe { AppendMenuW(submenu, MF_STRING, command, &snooze_choice_text(choice)) };
    }
    let snooze_text = HSTRING::from(tr!("tray.snooze").as_ref());
    // The parent menu owns the submenu and destroys it with itself.
    let _ = unsafe { AppendMenuW(menu, MF_POPUP, submenu.0 as usize, &snooze_text) };
}

fn pause_choice_text(choice: PauseChoice) -> HSTRING {
    let text = match choice {
        PauseChoice::TenMinutes => tr!("tray.pause_10"),
        PauseChoice::ThirtyMinutes => tr!("tray.pause_30"),
        PauseChoice::OneHour => tr!("tray.pause_60"),
        PauseChoice::UntilResumed => tr!("tray.pause_manual"),
    };
    HSTRING::from(text.as_ref())
}

/// Adds the "Pause Notch" submenu, or the greyed status line and Resume Notch while paused.
fn append_pause_items(menu: HMENU, state: &MenuState) {
    if let Some(paused_line) = &state.paused_line {
        let (line_text, resume_text) =
            (HSTRING::from(paused_line.as_str()), HSTRING::from(tr!("tray.resume").as_ref()));
        let _ = unsafe { AppendMenuW(menu, MF_STRING | MF_GRAYED, 0, &line_text) };
        let _ = unsafe { AppendMenuW(menu, MF_STRING, MENU_COMMAND_RESUME, &resume_text) };
        return;
    }
    let Ok(submenu) = (unsafe { CreatePopupMenu() }) else { return };
    for (position, choice) in PauseChoice::ALL.into_iter().enumerate() {
        let _ =
            unsafe { AppendMenuW(submenu, MF_STRING, MENU_COMMAND_PAUSE_FIRST + position, &pause_choice_text(choice)) };
    }
    let pause_text = HSTRING::from(tr!("tray.pause").as_ref());
    // The parent menu owns the submenu and destroys it with itself.
    let _ = unsafe { AppendMenuW(menu, MF_POPUP, submenu.0 as usize, &pause_text) };
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

pub(super) fn show_context_menu(window: HWND, state: &MenuState) -> Option<MenuChoice> {
    // Each time, so a change of theme while the notch runs shows in the next menu.
    follow_system_theme();
    let Ok(menu) = (unsafe { CreatePopupMenu() }) else { return None };
    // Read each time the menu opens, so a language change shows in the next menu.
    let (settings_text, quit_text) =
        (HSTRING::from(tr!("tray.settings").as_ref()), HSTRING::from(tr!("tray.quit").as_ref()));
    append_snooze_items(menu, state);
    append_pause_items(menu, state);
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
        assert_eq!(choice_for_command(MENU_COMMAND_QUIT), Some(MenuChoice::Quit));
    }

    #[test]
    fn each_pause_length_and_resume_has_its_own_command() {
        assert_eq!(choice_for_command(20), Some(MenuChoice::Pause(PauseChoice::TenMinutes)));
        assert_eq!(choice_for_command(21), Some(MenuChoice::Pause(PauseChoice::ThirtyMinutes)));
        assert_eq!(choice_for_command(22), Some(MenuChoice::Pause(PauseChoice::OneHour)));
        assert_eq!(choice_for_command(23), Some(MenuChoice::Pause(PauseChoice::UntilResumed)));
        assert_eq!(choice_for_command(24), Some(MenuChoice::Resume));
    }

    #[test]
    fn each_snooze_length_and_turning_it_off_has_its_own_command() {
        assert_eq!(choice_for_command(30), Some(MenuChoice::Snooze(SnoozeChoice::TenMinutes)));
        assert_eq!(choice_for_command(31), Some(MenuChoice::Snooze(SnoozeChoice::ThirtyMinutes)));
        assert_eq!(choice_for_command(32), Some(MenuChoice::Snooze(SnoozeChoice::OneHour)));
        assert_eq!(choice_for_command(33), Some(MenuChoice::Snooze(SnoozeChoice::UntilTurnedOff)));
        assert_eq!(choice_for_command(34), Some(MenuChoice::TurnOffSnooze));
    }

    #[test]
    fn only_an_active_snooze_has_a_menu_line() {
        assert_eq!(snoozed_line(Snooze::Off), None);
        assert!(snoozed_line(Snooze::Manual).is_some());
        assert!(snoozed_line(Snooze::Until(1_600)).is_some());
    }

    #[test]
    fn the_utc_offset_follows_the_bias_and_daylight_time() {
        assert_eq!(utc_offset_seconds(-420, 0, false), 7 * 3600);
        assert_eq!(utc_offset_seconds(480, -60, false), -8 * 3600);
        assert_eq!(utc_offset_seconds(480, -60, true), -7 * 3600);
    }

    #[test]
    fn a_dismissed_menu_and_retired_commands_choose_nothing() {
        for command in [0, 2, 3, 4, 5, 6, 25, 29, 35, 50, 100, 1000] {
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
