//! The pin that keeps the notch open until the user unpins it.

use super::{HOVER_WATCH_TIMER_ID, Notch, RAISE_TIMER_ID, hiding::RAISE_DELAY_MILLISECONDS};
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

use crate::{host::screen_mode::ScreenMode, log};

/// What an open notch does once it is unpinned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AfterUnpin {
    /// It was not open (a screenshot overlay or a hiding word had closed it): nothing to do.
    Nothing,
    /// The pointer is on it: it stays open and the hover watch looks for the pointer leaving.
    WatchPointer,
    /// The pointer is away: the usual close delay starts at once.
    StartClosing,
}

pub(super) fn after_unpin(is_open: bool, is_pointer_over: bool) -> AfterUnpin {
    match (is_open, is_pointer_over) {
        (false, _) => AfterUnpin::Nothing,
        (true, true) => AfterUnpin::WatchPointer,
        (true, false) => AfterUnpin::StartClosing,
    }
}

/// Whether pinning must raise the notch above the window in front, which may sit above it in the topmost band.
pub(super) fn raises_on_pin(mode: ScreenMode) -> bool {
    matches!(mode, ScreenMode::Maximized | ScreenMode::Fullscreen)
}

impl Notch {
    pub(super) fn is_pinned(&self) -> bool {
        self.open.is_pinned
    }

    pub(super) fn toggle_pin(&mut self) {
        self.set_pinned(!self.open.is_pinned);
    }

    pub(super) fn set_pinned(&mut self, is_pinned: bool) {
        if self.open.is_pinned == is_pinned {
            return;
        }
        let was_suppressed = self.is_suppressed();
        self.open.is_pinned = is_pinned;
        log::debug(&format!("pinned={is_pinned}"));
        // Showing a suppressed notch reopens it pinned (`apply_suppression`); unpinning over `hidden` closes it.
        self.apply_suppression(was_suppressed);
        if is_pinned {
            self.hold_open();
        } else {
            // A trial that ended while pinned open locks its pages now, not at the next open.
            self.reevaluate_access_if_due(store::pause::now_unix());
            self.release_pin();
        }
        // The header pin changes color even when nothing else moves.
        self.pending.repaint = true;
        self.flush();
    }

    fn hold_open(&mut self) {
        self.cancel_close();
        let _ = unsafe { KillTimer(Some(self.window), HOVER_WATCH_TIMER_ID) };
        if self.is_suppressed() {
            return;
        }
        self.set_open(true);
        self.begin_opening();
        self.start_animation();
        if raises_on_pin(self.hiding.screen_mode) {
            crate::host::window::raise_topmost(self.window);
            unsafe { SetTimer(Some(self.window), RAISE_TIMER_ID, RAISE_DELAY_MILLISECONDS, None) };
        }
    }

    fn release_pin(&mut self) {
        match after_unpin(self.open.is_open, self.pointer_is_over_window()) {
            AfterUnpin::Nothing => self.release_waiting_move(),
            AfterUnpin::WatchPointer => self.arm_hover_watch(),
            AfterUnpin::StartClosing => self.pointer_left(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpinning_a_closed_notch_does_nothing_wherever_the_pointer_is() {
        assert_eq!(after_unpin(false, true), AfterUnpin::Nothing);
        assert_eq!(after_unpin(false, false), AfterUnpin::Nothing);
    }

    #[test]
    fn unpinning_with_the_pointer_on_the_notch_watches_for_it_to_leave() {
        assert_eq!(after_unpin(true, true), AfterUnpin::WatchPointer);
    }

    #[test]
    fn unpinning_with_the_pointer_away_starts_the_close_delay() {
        assert_eq!(after_unpin(true, false), AfterUnpin::StartClosing);
    }

    #[test]
    fn pinning_raises_the_notch_only_over_a_maximized_or_full_screen_window() {
        assert!(raises_on_pin(ScreenMode::Maximized));
        assert!(raises_on_pin(ScreenMode::Fullscreen));
        assert!(!raises_on_pin(ScreenMode::Normal));
        assert!(!raises_on_pin(ScreenMode::Capture));
    }
}
