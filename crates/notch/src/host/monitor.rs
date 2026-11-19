//! Which monitor the notch sits on and where its window goes there.

mod choose;

use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT},
    Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
    },
    UI::{
        HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
        WindowsAndMessaging::{GetForegroundWindow, GetSystemMetrics, IsIconic, IsWindowVisible, SM_CXSCREEN},
    },
};

use super::{display, screen_mode, settings::Settings};
use crate::log;
pub(super) use choose::{MonitorId, MonitorInfo, MonitorMode, Placement, ScreenRect, placement, target_monitor};

const SETTING_KEY: &str = "notch.monitor";
const CHOSEN_ID_KEY: &str = "notch.monitor_id";

/// The `notch.monitor` word; a missing key keeps `current`, an unknown word reads as `main`.
pub(super) fn read_mode(settings: &Settings, current: MonitorMode) -> MonitorMode {
    settings.get_string(SETTING_KEY).map_or(current, |word| MonitorMode::parse(Some(word)))
}

/// The `notch.monitor_id` value (the chosen screen's identifier); a missing key keeps `current`, an empty one clears it.
pub(super) fn read_chosen_id(settings: &Settings, current: Option<&str>) -> Option<String> {
    match settings.get_string(CHOSEN_ID_KEY) {
        Some(id) => Some(id.trim().to_owned()).filter(|id| !id.is_empty()),
        None => current.map(str::to_owned),
    }
}

const MONITORINFOF_PRIMARY: u32 = 1;

fn monitor_id(monitor: HMONITOR) -> MonitorId {
    MonitorId(monitor.0 as isize)
}

fn monitor_info(monitor: HMONITOR) -> Option<MonitorInfo> {
    let mut info = MONITORINFO { cbSize: size_of::<MONITORINFO>() as u32, ..Default::default() };
    if !unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() {
        log::debug("could not read a monitor's rectangle");
        return None;
    }
    let (mut dpi_x, mut dpi_y) = (0, 0);
    let scale = match unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) } {
        Ok(()) if dpi_x > 0 => dpi_x as f32 / 96.0,
        Ok(()) => display::system_scale(),
        Err(error) => {
            log::debug(&format!("could not read a monitor's scale: {error}"));
            display::system_scale()
        }
    };
    let rect = choose::ScreenRect {
        left: info.rcMonitor.left,
        top: info.rcMonitor.top,
        right: info.rcMonitor.right,
        bottom: info.rcMonitor.bottom,
    };
    // One implementation of the identifier, shared with the settings window.
    let device_id = store::screens::identify(monitor.0 as isize).map(|identity| identity.id).unwrap_or_default();
    let is_primary = info.dwFlags & MONITORINFOF_PRIMARY != 0;
    Some(MonitorInfo { id: monitor_id(monitor), device_id, rect, scale, is_primary })
}

unsafe extern "system" fn collect_monitor(
    monitor: HMONITOR,
    _context: HDC,
    _rect: *mut RECT,
    list: LPARAM,
) -> windows::core::BOOL {
    // SAFETY: `list` is the `Vec` that `list()` passed and keeps alive for the whole enumeration.
    let monitors = unsafe { &mut *(list.0 as *mut Vec<MonitorInfo>) };
    monitors.extend(monitor_info(monitor));
    true.into()
}

/// Every attached monitor with its rectangle in virtual-desktop pixels and its effective scale.
pub(super) fn list() -> Vec<MonitorInfo> {
    let mut monitors: Vec<MonitorInfo> = Vec::new();
    let list = LPARAM(&mut monitors as *mut Vec<MonitorInfo> as isize);
    if !unsafe { EnumDisplayMonitors(None, None, Some(collect_monitor), list) }.as_bool() {
        log::debug("could not list the monitors");
    }
    monitors
}

/// The monitor that holds most of the window, or the nearest one.
pub(super) fn of_window(window: HWND) -> Option<MonitorId> {
    let monitor = unsafe { MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST) };
    (!monitor.0.is_null()).then(|| monitor_id(monitor))
}

/// The foreground window's monitor; `None` for no window, a hidden or minimized one, the desktop, a taskbar or the notch.
pub(super) fn foreground() -> Option<MonitorId> {
    let window = unsafe { GetForegroundWindow() };
    if window.0.is_null()
        || !unsafe { IsWindowVisible(window) }.as_bool()
        || unsafe { IsIconic(window) }.as_bool()
        || !screen_mode::is_user_window(window)
    {
        return None;
    }
    of_window(window)
}

/// Where the window starts: on the monitor `mode` picks, else centered on the primary screen at the system scale.
pub(super) fn start_placement(mode: MonitorMode, chosen_id: Option<&str>) -> (Option<MonitorId>, Placement) {
    let monitors = list();
    let start = target_monitor(mode, &monitors, foreground(), None, chosen_id)
        .and_then(|id| monitors.iter().find(|monitor| monitor.id == id))
        .map(|monitor| (Some(monitor.id), placement(monitor)));
    start.unwrap_or_else(|| {
        let scale = display::system_scale();
        let screen = ScreenRect { left: 0, top: 0, right: unsafe { GetSystemMetrics(SM_CXSCREEN) }, bottom: 0 };
        let primary = MonitorInfo { id: MonitorId(0), device_id: String::new(), rect: screen, scale, is_primary: true };
        (None, placement(&primary))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::{CreateWindowExW, DestroyWindow, WINDOW_EX_STYLE, WS_POPUP};
    use windows::core::w;

    #[test]
    fn the_list_has_exactly_one_primary_monitor() {
        let primaries = list().iter().filter(|monitor| monitor.is_primary).count();
        assert_eq!(primaries, 1);
    }

    #[test]
    fn every_monitor_has_a_positive_size_and_a_scale_of_at_least_1() {
        let monitors = list();
        assert!(!monitors.is_empty());
        for monitor in monitors {
            assert!(monitor.rect.width() > 0 && monitor.rect.bottom > monitor.rect.top);
            assert!(monitor.scale >= 1.0);
        }
    }

    #[test]
    fn the_notch_window_class_is_not_a_user_window() {
        for class in ["FoldlineWindow", "NotchDropTarget", "Shell_TrayWnd", "Progman"] {
            assert!(!screen_mode::is_user_class(class), "{class}");
        }
        assert!(screen_mode::is_user_class("Chrome_WidgetWin_1"));
    }

    #[test]
    fn a_window_on_the_primary_monitor_reports_the_primary_monitor() {
        let window = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!("monitor test"),
                WS_POPUP,
                0,
                0,
                10,
                10,
                None,
                None,
                None,
                None,
            )
        }
        .expect("a test window");
        let found = of_window(window);
        let _ = unsafe { DestroyWindow(window) };
        let primary = list().into_iter().find(|monitor| monitor.is_primary).map(|monitor| monitor.id);
        assert_eq!(found, primary);
    }

    #[test]
    fn a_missing_monitor_key_keeps_the_current_choice_and_a_written_word_replaces_it() {
        let mut settings = Settings::at(None);
        assert_eq!(read_mode(&settings, MonitorMode::Focused), MonitorMode::Focused);
        settings.set_string(SETTING_KEY, "focused");
        assert_eq!(read_mode(&settings, MonitorMode::Main), MonitorMode::Focused);
        settings.set_string(SETTING_KEY, "elsewhere");
        assert_eq!(read_mode(&settings, MonitorMode::Focused), MonitorMode::Main);
    }

    #[test]
    fn a_missing_monitor_id_key_keeps_the_current_id_and_a_written_one_replaces_it() {
        let mut settings = Settings::at(None);
        assert_eq!(read_chosen_id(&settings, Some("old")), Some("old".to_owned()));
        settings.set_string(CHOSEN_ID_KEY, " new ");
        assert_eq!(read_chosen_id(&settings, Some("old")), Some("new".to_owned()));
        settings.set_string(CHOSEN_ID_KEY, "");
        assert_eq!(read_chosen_id(&settings, Some("old")), None);
    }

    #[test]
    fn every_listed_monitor_has_an_identifier() {
        assert!(list().iter().all(|monitor| !monitor.device_id.is_empty()));
    }
}
