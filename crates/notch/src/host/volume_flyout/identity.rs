//! Tells the volume flyout apart from the other XAML island windows of Explorer.

use crate::host::screen_mode::Bounds;

/// The class of Explorer's flyout window on Windows 11 (seen on build 26300); other Explorer popups share it.
pub(super) const FLYOUT_CLASS: &str = "XamlExplorerHostIslandWindow";

// WS_EX_NOACTIVATE, WS_EX_TOPMOST and WS_EX_TOOLWINDOW: the task switcher and the full-screen island lack NOACTIVATE.
const FLYOUT_EXTENDED_STYLE: u32 = 0x0800_0000 | 0x0000_0008 | 0x0000_0080;
/// The flyout is centered on its monitor; both edges are whole pixels, so the sums may differ by one.
const CENTER_TOLERANCE_PIXELS: i32 = 4;

pub(super) struct WindowFacts<'a> {
    pub class_name: &'a str,
    pub has_title: bool,
    pub extended_style: u32,
    pub window: Bounds,
    pub monitor: Bounds,
}

/// The flyout has no title, never takes focus, and sits centered in the lower half of its monitor.
pub(super) fn is_volume_flyout(facts: &WindowFacts) -> bool {
    let window_center_doubled = facts.window.left + facts.window.right;
    let monitor_center_doubled = facts.monitor.left + facts.monitor.right;
    let monitor_middle = (facts.monitor.top + facts.monitor.bottom) / 2;
    facts.class_name == FLYOUT_CLASS
        && !facts.has_title
        && facts.extended_style & FLYOUT_EXTENDED_STYLE == FLYOUT_EXTENDED_STYLE
        && (window_center_doubled - monitor_center_doubled).abs() <= CENTER_TOLERANCE_PIXELS
        && facts.window.top >= monitor_middle
}

#[cfg(test)]
mod tests {
    use super::*;

    const MONITOR: Bounds = Bounds { left: 0, top: 0, right: 1920, bottom: 1080 };
    /// Measured on Windows 11 build 26300 at 125 % scale while the flyout showed the volume.
    const SHOWN_FLYOUT: Bounds = Bounds { left: 814, top: 943, right: 1106, bottom: 1020 };

    fn flyout_facts(window: Bounds) -> WindowFacts<'static> {
        WindowFacts {
            class_name: FLYOUT_CLASS,
            has_title: false,
            extended_style: 0x0820_0088,
            window,
            monitor: MONITOR,
        }
    }

    #[test]
    fn the_volume_flyout_explorer_shows_above_the_taskbar_is_recognized() {
        assert!(is_volume_flyout(&flyout_facts(SHOWN_FLYOUT)));
    }

    #[test]
    fn the_flyout_is_recognized_while_hidden_at_its_narrower_size() {
        assert!(is_volume_flyout(&flyout_facts(Bounds { left: 854, top: 944, right: 1065, bottom: 1020 })));
    }

    #[test]
    fn the_flyout_is_recognized_on_a_second_monitor() {
        let mut facts = flyout_facts(Bounds { left: 2734, top: 943, right: 3026, bottom: 1020 });
        facts.monitor = Bounds { left: 1920, top: 0, right: 3840, bottom: 1080 };
        assert!(is_volume_flyout(&facts));
    }

    #[test]
    fn the_task_switcher_island_with_a_title_is_left_alone() {
        let mut facts = flyout_facts(SHOWN_FLYOUT);
        facts.has_title = true;
        assert!(!is_volume_flyout(&facts));
    }

    #[test]
    fn a_full_screen_island_that_can_take_focus_is_left_alone() {
        let mut facts = flyout_facts(MONITOR);
        facts.extended_style = 0x0020_0088;
        assert!(!is_volume_flyout(&facts));
    }

    #[test]
    fn an_island_popup_away_from_the_bottom_center_is_left_alone() {
        assert!(!is_volume_flyout(&flyout_facts(Bounds { left: 300, top: 40, right: 600, bottom: 200 })));
    }

    #[test]
    fn a_centered_island_popup_in_the_upper_half_is_left_alone() {
        assert!(!is_volume_flyout(&flyout_facts(Bounds { left: 814, top: 100, right: 1106, bottom: 177 })));
    }

    #[test]
    fn a_window_of_another_class_is_left_alone() {
        let mut facts = flyout_facts(SHOWN_FLYOUT);
        facts.class_name = "Shell_TrayWnd";
        assert!(!is_volume_flyout(&facts));
    }
}
