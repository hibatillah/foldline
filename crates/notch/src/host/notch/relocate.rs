//! Moves the notch to the monitor `notch.monitor` picks, at that monitor's scale.

use windows::Win32::{
    Foundation::POINT,
    UI::WindowsAndMessaging::{SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos},
};

use super::{Notch, monitor_state::MonitorState};
use crate::{
    gfx::Fonts,
    host::{
        drop_target,
        monitor::{MonitorId, MonitorMode, Placement, placement, target_monitor},
        screen_mode::ScreenMode,
    },
    log,
    page::PointerEvent,
};

/// What a move asks for, given whether the notch is held open and whether its current monitor still exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MoveDecision {
    Nothing,
    MoveNow,
    WaitForClose,
}

pub(super) fn decide_move(
    target: MonitorId,
    target_placement: Placement,
    state: &MonitorState,
    current_exists: bool,
    is_held_open: bool,
) -> MoveDecision {
    let is_same_monitor = state.current == Some(target);
    if is_same_monitor && state.placed == Some(target_placement) {
        MoveDecision::Nothing
    } else if !is_same_monitor && current_exists && is_held_open {
        MoveDecision::WaitForClose
    } else {
        MoveDecision::MoveNow
    }
}

impl Notch {
    /// Puts the notch where `notch.monitor` says; returns whether the screen mode was judged again.
    pub fn relocate_if_needed(&mut self) -> bool {
        let source = self.monitor.source;
        let monitors = (source.list)();
        let foreground = (source.foreground)();
        let Some(target) = target_monitor(
            self.monitor.mode,
            &monitors,
            foreground,
            self.monitor.current,
            self.monitor.chosen_id.as_deref(),
        ) else {
            return false;
        };
        let Some(target_info) = monitors.iter().find(|monitor| monitor.id == target) else { return false };
        let target_placement = placement(target_info, self.monitor.top_offset);
        let current_exists = MonitorState::is_listed(&monitors, self.monitor.current);
        let is_held_open = self.open.is_open || self.open.is_pinned;
        match decide_move(target, target_placement, &self.monitor, current_exists, is_held_open) {
            MoveDecision::Nothing => {
                self.monitor.waiting = None;
                false
            }
            MoveDecision::WaitForClose => {
                self.monitor.waiting = Some((target, target_placement));
                false
            }
            MoveDecision::MoveNow => self.apply_placement(target, target_placement),
        }
    }

    /// A foreground change: only `focused` follows it, so `main` asks Windows for nothing more.
    /// Known limit: a window dragged to another monitor without a foreground change does not move the notch.
    pub fn foreground_changed(&mut self, mode: ScreenMode) {
        let was_judged_again = self.monitor.mode == MonitorMode::Focused && self.relocate_if_needed();
        if !was_judged_again {
            self.set_screen_mode(mode);
        }
    }

    /// Applies a move that waited, once the notch is closed and not pinned; the monitors are read again.
    pub(super) fn release_waiting_move(&mut self) {
        if self.open.is_open || self.open.is_pinned || self.monitor.waiting.take().is_none() {
            return;
        }
        self.relocate_if_needed();
    }

    pub(super) fn set_monitor_choice(&mut self, mode: MonitorMode, chosen_id: Option<String>) {
        if self.monitor.mode != mode || self.monitor.chosen_id != chosen_id {
            self.monitor.mode = mode;
            self.monitor.chosen_id = chosen_id;
            self.relocate_if_needed();
        }
    }

    /// Moves the window in one message, so no half-moved frame is drawn; returns whether the screen mode was judged.
    pub(super) fn apply_placement(&mut self, monitor: MonitorId, target: Placement) -> bool {
        if target.scale != self.scale && !self.rescale(target) {
            return false;
        }
        self.screen_position = POINT { x: target.left, y: target.top };
        let (origin_x, origin_y) = self.region_origin.get();
        // The size and the stacking stay: the next paint sizes the window to the pill.
        let flags = SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE;
        let moved =
            unsafe { SetWindowPos(self.window, None, target.left + origin_x, target.top + origin_y, 0, 0, flags) };
        if let Err(error) = moved {
            log::debug(&format!("could not move the notch window: {error}"));
        }
        drop_target::set_placement(target.scale, target.left + target.width / 2, target.top);
        let has_monitor_changed = self.monitor.current != Some(monitor);
        self.monitor.current = Some(monitor);
        self.monitor.placed = Some(target);
        self.monitor.waiting = None;
        log::debug(&format!("notch placed at ({}, {}) scale {}", target.left, target.top, target.scale));
        if has_monitor_changed {
            let mode = (self.monitor.source.judge)(Some(monitor));
            self.set_screen_mode(mode);
        }
        self.pending.repaint = true;
        self.flush();
        has_monitor_changed
    }

    /// Builds what holds device pixels for the new scale; a failure leaves the old scale whole.
    fn rescale(&mut self, target: Placement) -> bool {
        let fonts = match Fonts::new(target.scale) {
            Ok(fonts) => fonts,
            Err(error) => {
                log::debug(&format!("could not build fonts for scale {}: {error}", target.scale));
                return false;
            }
        };
        if let Err(error) = self.surface.resize(target.width, target.height) {
            log::debug(&format!("could not resize the surface: {error}"));
            return false;
        }
        // Hover and badge areas are device rects of the old scale; the next pointer move finds them again.
        self.route_pointer(PointerEvent::CaptureLost);
        let ratio = target.scale / self.scale;
        let (origin_x, origin_y) = self.region_origin.get();
        self.region_origin.set(((origin_x as f32 * ratio).round() as i32, (origin_y as f32 * ratio).round() as i32));
        self.scale = target.scale;
        self.fonts = fonts;
        self.rebuild_cached_text();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placed_at(left: i32, scale: f32) -> Placement {
        Placement { left, top: 0, width: 100, height: 40, scale }
    }

    fn state_on(current: Option<MonitorId>, placed: Option<Placement>) -> MonitorState {
        MonitorState::new(MonitorMode::Focused, None, current, placed, 0)
    }

    #[test]
    fn the_same_monitor_and_placement_need_no_move() {
        let state = state_on(Some(MonitorId(1)), Some(placed_at(0, 1.0)));
        assert_eq!(decide_move(MonitorId(1), placed_at(0, 1.0), &state, true, true), MoveDecision::Nothing);
    }

    #[test]
    fn another_monitor_waits_while_the_notch_is_held_open() {
        let state = state_on(Some(MonitorId(1)), Some(placed_at(0, 1.0)));
        assert_eq!(decide_move(MonitorId(2), placed_at(1920, 1.0), &state, true, true), MoveDecision::WaitForClose);
    }

    #[test]
    fn another_monitor_is_moved_to_at_once_when_the_current_one_is_gone() {
        let state = state_on(Some(MonitorId(1)), Some(placed_at(0, 1.0)));
        assert_eq!(decide_move(MonitorId(2), placed_at(1920, 1.0), &state, false, true), MoveDecision::MoveNow);
    }

    #[test]
    fn a_new_placement_on_the_same_monitor_applies_even_while_open() {
        let state = state_on(Some(MonitorId(1)), Some(placed_at(0, 1.0)));
        assert_eq!(decide_move(MonitorId(1), placed_at(0, 1.5), &state, true, true), MoveDecision::MoveNow);
    }
}
