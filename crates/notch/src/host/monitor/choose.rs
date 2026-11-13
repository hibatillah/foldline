//! Pure rules: which monitor the notch sits on and where its window goes (no Windows calls).

use crate::host::notch::{WINDOW_HEIGHT, WINDOW_WIDTH};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::host) struct MonitorId(pub isize);

/// A monitor's rectangle in virtual-desktop pixels; left and top can be negative.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(in crate::host) struct ScreenRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl ScreenRect {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(in crate::host) struct MonitorInfo {
    pub id: MonitorId,
    /// The stored identifier of a chosen screen (`store::screens`); empty when none could be read.
    pub device_id: String,
    pub rect: ScreenRect,
    pub scale: f32,
    pub is_primary: bool,
}

/// The `notch.monitor` setting.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(in crate::host) enum MonitorMode {
    #[default]
    Main,
    Focused,
    Chosen,
}

impl MonitorMode {
    pub fn parse(text: Option<&str>) -> MonitorMode {
        match text.map(str::trim) {
            Some("focused") => MonitorMode::Focused,
            Some("chosen") => MonitorMode::Chosen,
            _ => MonitorMode::Main,
        }
    }
}

/// The monitor the notch should be on, or `None` when no monitor is listed.
/// `chosen_device_id` is `notch.monitor_id`; a `chosen` mode without a listed id sits on the primary monitor.
pub(in crate::host) fn target_monitor(
    mode: MonitorMode,
    monitors: &[MonitorInfo],
    foreground: Option<MonitorId>,
    current: Option<MonitorId>,
    chosen_device_id: Option<&str>,
) -> Option<MonitorId> {
    let is_listed = |id: &MonitorId| monitors.iter().any(|monitor| monitor.id == *id);
    let primary = || monitors.iter().find(|monitor| monitor.is_primary).or(monitors.first()).map(|monitor| monitor.id);
    match mode {
        MonitorMode::Main => primary(),
        MonitorMode::Focused => foreground.filter(is_listed).or(current.filter(is_listed)).or_else(primary),
        MonitorMode::Chosen => chosen_device_id
            .filter(|device_id| !device_id.is_empty())
            .and_then(|device_id| monitors.iter().find(|monitor| monitor.device_id == device_id))
            .map(|monitor| monitor.id)
            .or_else(primary),
    }
}

/// Where the window goes: virtual-desktop position and device-pixel size at the monitor's scale.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(in crate::host) struct Placement {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
    pub scale: f32,
}

pub(in crate::host) fn placement(monitor: &MonitorInfo, top_offset: i32) -> Placement {
    let width = (WINDOW_WIDTH * monitor.scale).round() as i32;
    let height = (WINDOW_HEIGHT * monitor.scale).round() as i32;
    let center_x = monitor.rect.left + monitor.rect.width() / 2;
    Placement { left: center_x - width / 2, top: monitor.rect.top + top_offset, width, height, scale: monitor.scale }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitor(id: isize, left: i32, top: i32, scale: f32, is_primary: bool) -> MonitorInfo {
        let rect = ScreenRect { left, top, right: left + 1920, bottom: top + 1080 };
        MonitorInfo { id: MonitorId(id), device_id: format!("device-{id}"), rect, scale, is_primary }
    }

    fn two_monitors() -> [MonitorInfo; 2] {
        [monitor(1, 0, 0, 1.25, true), monitor(2, 1920, 0, 1.0, false)]
    }

    #[test]
    fn main_always_picks_the_primary_monitor() {
        let monitors = [monitor(2, 1920, 0, 1.0, false), monitor(1, 0, 0, 1.25, true)];
        let target = target_monitor(MonitorMode::Main, &monitors, Some(MonitorId(2)), Some(MonitorId(2)), None);
        assert_eq!(target, Some(MonitorId(1)));
    }

    #[test]
    fn focused_picks_the_foreground_windows_monitor() {
        let target =
            target_monitor(MonitorMode::Focused, &two_monitors(), Some(MonitorId(2)), Some(MonitorId(1)), None);
        assert_eq!(target, Some(MonitorId(2)));
    }

    #[test]
    fn a_missing_foreground_keeps_the_current_monitor() {
        let target = target_monitor(MonitorMode::Focused, &two_monitors(), None, Some(MonitorId(2)), None);
        assert_eq!(target, Some(MonitorId(2)));
    }

    #[test]
    fn a_foreground_on_an_unknown_monitor_keeps_the_current_one() {
        let target =
            target_monitor(MonitorMode::Focused, &two_monitors(), Some(MonitorId(9)), Some(MonitorId(2)), None);
        assert_eq!(target, Some(MonitorId(2)));
    }

    #[test]
    fn a_missing_current_monitor_falls_back_to_the_primary() {
        let target = target_monitor(MonitorMode::Focused, &two_monitors(), None, None, None);
        assert_eq!(target, Some(MonitorId(1)));
    }

    #[test]
    fn the_target_is_the_primary_after_the_current_monitor_is_unplugged() {
        let remaining = [monitor(1, 0, 0, 1.25, true)];
        let target = target_monitor(MonitorMode::Focused, &remaining, None, Some(MonitorId(2)), None);
        assert_eq!(target, Some(MonitorId(1)));
    }

    #[test]
    fn an_empty_monitor_list_gives_no_target() {
        let one = Some(MonitorId(1));
        assert_eq!(target_monitor(MonitorMode::Focused, &[], one, one, None), None);
        assert_eq!(target_monitor(MonitorMode::Main, &[], None, None, None), None);
    }

    #[test]
    fn a_monitor_left_of_the_primary_gives_a_negative_left_and_a_centered_window() {
        let left_monitor = monitor(2, -1920, 0, 1.0, false);
        let placed = placement(&left_monitor, 0);
        assert_eq!(placed.left, -960 - placed.width / 2);
        assert!(placed.left < 0);
        assert_eq!(placed.left + placed.width / 2, left_monitor.rect.left + left_monitor.rect.width() / 2);
    }

    #[test]
    fn the_placement_size_follows_scale_1_0_1_25_and_1_5() {
        for scale in [1.0_f32, 1.25, 1.5] {
            let placed = placement(&monitor(1, 0, 0, scale, true), 0);
            assert_eq!(placed.width, (WINDOW_WIDTH * scale).round() as i32);
            assert_eq!(placed.height, (WINDOW_HEIGHT * scale).round() as i32);
            assert_eq!(placed.scale, scale);
        }
    }

    #[test]
    fn the_top_offset_is_measured_from_the_monitors_top_edge() {
        let below = monitor(2, 0, 1080, 1.0, false);
        assert_eq!(placement(&below, 12).top, 1080 + 12);
    }

    #[test]
    fn chosen_picks_the_monitor_with_the_stored_id() {
        let target = target_monitor(MonitorMode::Chosen, &two_monitors(), None, None, Some("device-2"));
        assert_eq!(target, Some(MonitorId(2)));
    }

    #[test]
    fn chosen_with_an_unknown_id_falls_back_to_the_primary() {
        let target = target_monitor(MonitorMode::Chosen, &two_monitors(), None, Some(MonitorId(2)), Some("device-9"));
        assert_eq!(target, Some(MonitorId(1)));
    }

    #[test]
    fn chosen_without_an_id_reads_as_main() {
        assert_eq!(
            target_monitor(MonitorMode::Chosen, &two_monitors(), None, Some(MonitorId(2)), None),
            Some(MonitorId(1))
        );
        assert_eq!(target_monitor(MonitorMode::Chosen, &two_monitors(), None, None, Some("")), Some(MonitorId(1)));
    }

    #[test]
    fn a_chosen_monitor_that_returns_is_picked_again() {
        let remaining = [monitor(1, 0, 0, 1.25, true)];
        assert_eq!(target_monitor(MonitorMode::Chosen, &remaining, None, None, Some("device-2")), Some(MonitorId(1)));
        // Windows hands the returning screen a new handle; the stored id still finds it.
        let returned =
            [monitor(1, 0, 0, 1.25, true), MonitorInfo { id: MonitorId(7), ..monitor(2, 1920, 0, 1.0, false) }];
        assert_eq!(
            target_monitor(MonitorMode::Chosen, &returned, None, Some(MonitorId(1)), Some("device-2")),
            Some(MonitorId(7))
        );
    }

    #[test]
    fn the_word_chosen_reads_as_chosen() {
        assert_eq!(MonitorMode::parse(Some("chosen")), MonitorMode::Chosen);
    }

    #[test]
    fn an_unknown_setting_word_reads_as_main() {
        assert_eq!(MonitorMode::parse(Some("everywhere")), MonitorMode::Main);
        assert_eq!(MonitorMode::parse(None), MonitorMode::Main);
        assert_eq!(MonitorMode::parse(Some(" focused ")), MonitorMode::Focused);
    }
}
