//! Which screen mode the facts about the window in front give (no Windows calls).

use super::ScreenMode;

/// Which screenshot tool, if any, owns the window in front.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CaptureHost {
    None,
    /// The process hosts only the screenshot overlay, so any window of it counts.
    OverlayOnly,
    /// The process also has a regular window, so only a window covering the monitor counts as its overlay.
    AppWithOverlay,
}

/// Win+Shift+S and the Print Screen key on Windows 11 open the overlay in `ScreenClippingHost.exe`.
const OVERLAY_ONLY_PROCESSES: [&str; 1] = ["ScreenClippingHost.exe"];
/// Newer Snipping Tool builds draw the overlay in the app itself; Xbox Game Bar's capture widget sits on its overlay.
const APP_WITH_OVERLAY_PROCESSES: [&str; 2] = ["SnippingTool.exe", "GameBar.exe"];

pub(super) fn capture_host(process_file_name: &str) -> CaptureHost {
    let matches = |names: &[&str]| names.iter().any(|name| name.eq_ignore_ascii_case(process_file_name));
    if matches(&OVERLAY_ONLY_PROCESSES) {
        CaptureHost::OverlayOnly
    } else if matches(&APP_WITH_OVERLAY_PROCESSES) {
        CaptureHost::AppWithOverlay
    } else {
        CaptureHost::None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::host) struct Bounds {
    pub(in crate::host) left: i32,
    pub(in crate::host) top: i32,
    pub(in crate::host) right: i32,
    pub(in crate::host) bottom: i32,
}

impl Bounds {
    fn covers(&self, other: &Bounds) -> bool {
        self.left <= other.left && self.top <= other.top && self.right >= other.right && self.bottom >= other.bottom
    }
}

pub(super) struct Facts {
    pub(super) window: Bounds,
    pub(super) monitor: Bounds,
    pub(super) is_zoomed: bool,
    pub(super) has_caption: bool,
    pub(super) capture_host: CaptureHost,
    /// The desktop, the taskbar, the notch itself, a window that is hidden or minimized, or one on another monitor.
    pub(super) is_ignored: bool,
}

/// A maximized window with an auto-hidden taskbar can cover the monitor without being full screen.
pub(super) fn classify(facts: &Facts) -> ScreenMode {
    // The overlay can span every monitor, so it is checked before the primary-monitor rule in `is_ignored`.
    match facts.capture_host {
        CaptureHost::OverlayOnly => return ScreenMode::Capture,
        CaptureHost::AppWithOverlay if facts.window.covers(&facts.monitor) => return ScreenMode::Capture,
        CaptureHost::AppWithOverlay | CaptureHost::None => {}
    }
    if facts.is_ignored {
        return ScreenMode::Normal;
    }
    if facts.window.covers(&facts.monitor) && !(facts.is_zoomed && facts.has_caption) {
        ScreenMode::Fullscreen
    } else if facts.is_zoomed {
        ScreenMode::Maximized
    } else {
        ScreenMode::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MONITOR: Bounds = Bounds { left: 0, top: 0, right: 1920, bottom: 1080 };

    fn facts(window: Bounds, is_zoomed: bool, has_caption: bool) -> Facts {
        Facts { window, monitor: MONITOR, is_zoomed, has_caption, capture_host: CaptureHost::None, is_ignored: false }
    }

    #[test]
    fn the_screenshot_tools_are_recognized_by_their_process_name_in_any_case() {
        assert_eq!(capture_host("ScreenClippingHost.exe"), CaptureHost::OverlayOnly);
        assert_eq!(capture_host("screenclippinghost.EXE"), CaptureHost::OverlayOnly);
        assert_eq!(capture_host("SnippingTool.exe"), CaptureHost::AppWithOverlay);
        assert_eq!(capture_host("GameBar.exe"), CaptureHost::AppWithOverlay);
        assert_eq!(capture_host("explorer.exe"), CaptureHost::None);
        assert_eq!(capture_host(""), CaptureHost::None);
    }

    #[test]
    fn any_window_of_the_win_shift_s_overlay_process_is_a_capture_even_on_another_monitor() {
        let mut overlay = facts(Bounds { left: 1920, top: 0, right: 3840, bottom: 1080 }, false, false);
        overlay.capture_host = CaptureHost::OverlayOnly;
        overlay.is_ignored = true;
        assert_eq!(classify(&overlay), ScreenMode::Capture);
    }

    #[test]
    fn the_snipping_tool_overlay_over_the_whole_monitor_is_a_capture() {
        let mut overlay = facts(Bounds { left: 0, top: 0, right: 3840, bottom: 1080 }, false, false);
        overlay.capture_host = CaptureHost::AppWithOverlay;
        assert_eq!(classify(&overlay), ScreenMode::Capture);
    }

    #[test]
    fn the_snipping_tool_app_in_a_regular_or_maximized_window_is_not_a_capture() {
        let mut regular = facts(Bounds { left: 100, top: 100, right: 900, bottom: 700 }, false, true);
        regular.capture_host = CaptureHost::AppWithOverlay;
        assert_eq!(classify(&regular), ScreenMode::Normal);
        let mut maximized = facts(Bounds { left: -8, top: -8, right: 1928, bottom: 1040 }, true, true);
        maximized.capture_host = CaptureHost::AppWithOverlay;
        assert_eq!(classify(&maximized), ScreenMode::Maximized);
    }

    #[test]
    fn a_window_over_the_whole_monitor_with_no_title_bar_is_full_screen() {
        assert_eq!(classify(&facts(MONITOR, false, false)), ScreenMode::Fullscreen);
        let outside = Bounds { left: -1, top: -1, right: 1921, bottom: 1081 };
        assert_eq!(classify(&facts(outside, false, false)), ScreenMode::Fullscreen);
    }

    #[test]
    fn a_maximized_window_stops_short_of_the_taskbar() {
        let above_taskbar = Bounds { left: -8, top: -8, right: 1928, bottom: 1040 };
        assert_eq!(classify(&facts(above_taskbar, true, true)), ScreenMode::Maximized);
    }

    #[test]
    fn a_maximized_window_over_a_hidden_taskbar_is_still_maximized() {
        let outside = Bounds { left: -8, top: -8, right: 1928, bottom: 1088 };
        assert_eq!(classify(&facts(outside, true, true)), ScreenMode::Maximized);
    }

    #[test]
    fn a_window_that_does_not_fill_the_screen_is_normal() {
        let window = Bounds { left: 100, top: 100, right: 900, bottom: 700 };
        assert_eq!(classify(&facts(window, false, true)), ScreenMode::Normal);
    }

    #[test]
    fn the_desktop_and_other_ignored_windows_are_never_full_screen() {
        let mut desktop = facts(MONITOR, false, false);
        desktop.is_ignored = true;
        assert_eq!(classify(&desktop), ScreenMode::Normal);
    }
}
