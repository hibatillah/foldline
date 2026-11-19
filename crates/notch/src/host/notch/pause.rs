//! Pausing the notch: it disappears completely and Windows' own popups come back until the pause ends.

use store::pause::{PAUSED_UNTIL_KEY, Pause};
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

use super::{
    CLOSE_TIMER_ID, EXPAND_TIMER_ID, HOVER_WATCH_TIMER_ID, Notch, PAUSE_TIMER_ID, RAISE_TIMER_ID, REVEAL_TIMER_ID,
    timers::CLOCK_TIMER_ID,
};
use crate::host::{notification_banners, volume_flyout};

/// The largest interval a Windows timer accepts; a longer pause re-arms for the rest when the timer fires early.
pub(super) const LONGEST_TIMER_MILLISECONDS: u32 = i32::MAX as u32;

pub(super) struct PauseState {
    pause: Pause,
    /// The popups' state when the pause began, used on resume when their settings keys are unreadable.
    flyout_was_hidden: bool,
    banners_were_hidden: bool,
}

impl Default for PauseState {
    fn default() -> Self {
        Self { pause: Pause::Off, flyout_was_hidden: false, banners_were_hidden: false }
    }
}

impl Notch {
    pub(crate) fn is_paused(&self) -> bool {
        self.pause.pause.is_paused()
    }

    pub(crate) fn pause(&self) -> Pause {
        self.pause.pause
    }

    /// Writes the key and applies it; the tray calls this.
    pub fn set_pause(&mut self, pause: Pause, now_unix: i64) {
        self.settings.set_string(PAUSED_UNTIL_KEY, &pause.as_text());
        self.apply_stored_pause(now_unix);
    }

    /// Applies the key as read from settings; a stale or unknown value is cleared and counts as not paused.
    pub(crate) fn apply_stored_pause(&mut self, now_unix: i64) {
        let stored = self.settings.get_string(PAUSED_UNTIL_KEY).unwrap_or("").to_owned();
        let pause = Pause::parse(Some(&stored), now_unix);
        if !stored.is_empty() && pause == Pause::Off {
            self.settings.set_string(PAUSED_UNTIL_KEY, "");
        }
        self.apply_pause(pause, now_unix);
    }

    /// A timer that fires a little early re-arms for the rest.
    pub(super) fn pause_timer_fired(&mut self, now_unix: i64) {
        self.apply_stored_pause(now_unix);
    }

    fn apply_pause(&mut self, pause: Pause, now_unix: i64) {
        let was_paused = self.is_paused();
        let was_suppressed = self.is_suppressed();
        self.pause.pause = pause;
        self.arm_pause_timer(now_unix);
        if pause.is_paused() && !was_paused {
            self.give_popups_back();
        } else if !pause.is_paused() && was_paused {
            self.hide_popups_again();
        }
        // Suppression closes an open notch (a pin stays set), hides the window and tells the drop target.
        self.apply_suppression(was_suppressed);
        if pause.is_paused() && !was_paused {
            self.go_quiet();
        } else if !pause.is_paused() && was_paused {
            self.arm_clock_timer();
            // Changes during the pause were not animated, so the springs jump to where they are going.
            self.refresh_layout();
            for spring in self.layout_springs() {
                spring.snap_to_target();
            }
            self.sync_page_hotkeys();
            self.pending.repaint = true;
            self.flush();
            self.paint();
        }
    }

    fn arm_pause_timer(&self, now_unix: i64) {
        if let Some(milliseconds) = self.pause.pause.remaining_milliseconds(now_unix) {
            unsafe { SetTimer(Some(self.window), PAUSE_TIMER_ID, milliseconds.min(LONGEST_TIMER_MILLISECONDS), None) };
        } else {
            let _ = unsafe { KillTimer(Some(self.window), PAUSE_TIMER_ID) };
        }
    }

    fn give_popups_back(&mut self) {
        self.pause.flyout_was_hidden = volume_flyout::is_hidden();
        self.pause.banners_were_hidden = notification_banners::is_hidden();
        volume_flyout::set_hidden(false);
        if self.pause.banners_were_hidden {
            notification_banners::set_hidden(false);
        }
    }

    fn hide_popups_again(&mut self) {
        let wants_flyout_hidden = self.settings.get_bool(volume_flyout::SETTING_KEY, self.pause.flyout_was_hidden);
        volume_flyout::set_hidden(wants_flyout_hidden);
        let wants_banners_hidden =
            self.settings.get_bool(notification_banners::SETTING_KEY, self.pause.banners_were_hidden);
        // Unchanged stays put: switching off restores, which a resume with banners already off must not repeat.
        if wants_banners_hidden != notification_banners::is_hidden() {
            notification_banners::set_hidden(wants_banners_hidden);
        }
    }

    /// Nothing is drawn while paused, so the animation and the hover watch have no output to follow.
    fn go_quiet(&mut self) {
        // A mode that waited to show more applies now, while hidden, so no reveal timer outlives the pause.
        self.finish_window_reveal();
        for timer_id in
            [HOVER_WATCH_TIMER_ID, REVEAL_TIMER_ID, EXPAND_TIMER_ID, CLOSE_TIMER_ID, RAISE_TIMER_ID, CLOCK_TIMER_ID]
        {
            let _ = unsafe { KillTimer(Some(self.window), timer_id) };
        }
        self.open.is_close_pending = false;
        if self.frame_timer.is_running {
            self.stop_animation();
        }
        self.springs.expansion.jump_to(0.0);
        self.sync_page_hotkeys();
    }
}
