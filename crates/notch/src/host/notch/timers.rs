use super::{
    ANIMATION_TIMER_ID, CLOSE_TIMER_ID, EXPAND_TIMER_ID, HOVER_WATCH_TIMER_ID, Notch, RAISE_TIMER_ID, REVEAL_TIMER_ID,
    STAGE_TIMER_ID, SWIPE_END_TIMER_ID,
};
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

use crate::host::clock;
use crate::{
    activity::ACTIVITY_TIMER_BASE,
    log,
    page::{split_owned_timer_key, split_timer_key},
};

const EXPAND_DELAY_MILLISECONDS: u32 = 450;
const CLOCK_TIMER_ID: usize = 6;

impl Notch {
    pub(super) fn arm_clock_timer(&self) {
        unsafe { SetTimer(Some(self.window), CLOCK_TIMER_ID, clock::milliseconds_until_next_minute(), None) };
    }

    pub fn timer_fired(&mut self, timer_key: usize) {
        match timer_key {
            ANIMATION_TIMER_ID => self.advance_animation(),
            CLOCK_TIMER_ID => {
                self.arm_clock_timer();
                if self.resting_index.is_none() || self.springs.expansion.position > 0.001 {
                    self.paint();
                }
            }
            SWIPE_END_TIMER_ID => {
                if self.swipe.as_ref().is_some_and(|swipe| !swipe.quiet_remaining().is_zero()) {
                    // A WM_TIMER may already be queued when fresh scroll input resets its deadline.
                    self.arm_swipe_end();
                } else {
                    self.end_swipe();
                    self.flush();
                }
            }
            REVEAL_TIMER_ID => {
                unsafe {
                    let _ = KillTimer(Some(self.window), REVEAL_TIMER_ID);
                    SetTimer(Some(self.window), EXPAND_TIMER_ID, EXPAND_DELAY_MILLISECONDS, None);
                }
                if self.hover.is_hovering {
                    self.hiding.is_revealed = true;
                    self.flush();
                }
            }
            EXPAND_TIMER_ID => {
                unsafe {
                    let _ = KillTimer(Some(self.window), EXPAND_TIMER_ID);
                }
                if self.hover.is_hovering && self.shown_banner.is_none() {
                    self.begin_opening();
                    self.start_animation();
                    self.flush();
                }
            }
            STAGE_TIMER_ID => {
                unsafe {
                    let _ = KillTimer(Some(self.window), STAGE_TIMER_ID);
                }
                self.springs.expansion.target = if self.hover.is_hovering { 1.0 } else { 0.0 };
                self.start_animation();
            }
            HOVER_WATCH_TIMER_ID => {
                if !self.hover.is_captured
                    && !self.hover.is_dragging_files
                    && !self.hover.is_close_pending
                    && !self.pointer_is_over_window()
                {
                    let (cursor, bounds) = self.pointer_and_window_bounds();
                    log::debug(&format!(
                        "hover watch: pointer left the window; cursor=({},{}) window=({},{})-({},{}) expansion={:.2}",
                        cursor.x,
                        cursor.y,
                        bounds.left,
                        bounds.top,
                        bounds.right,
                        bounds.bottom,
                        self.springs.expansion.position
                    ));
                    self.pointer_left();
                    self.flush();
                }
            }
            RAISE_TIMER_ID => {
                unsafe {
                    let _ = KillTimer(Some(self.window), RAISE_TIMER_ID);
                }
                if !self.is_suppressed() {
                    crate::host::window::raise_topmost(self.window);
                }
            }
            CLOSE_TIMER_ID => {
                unsafe {
                    let _ = KillTimer(Some(self.window), CLOSE_TIMER_ID);
                }
                self.hover.is_close_pending = false;
                // The pointer may have come back without moving, which sends no message; a page that is typing keeps
                // the notch open until it gives the keyboard back.
                if !self.hover.is_captured
                    && !self.hover.is_dragging_files
                    && self.keyboard.owner.is_none()
                    && !self.pointer_is_over_window()
                {
                    self.set_hovering(false);
                    self.flush();
                }
            }
            key if key >= ACTIVITY_TIMER_BASE => {
                let Some((index, timer)) = split_owned_timer_key(ACTIVITY_TIMER_BASE, usize::MAX, key) else { return };
                if index < self.activities.len() {
                    self.with_activity(index, |activity, context| activity.on_timer(timer, context));
                    self.flush();
                }
            }
            key => {
                let Some((index, timer)) = split_timer_key(key) else { return };
                if index < self.pages.len() {
                    self.with_page(index, |page, context| page.on_timer(timer, context));
                    self.flush();
                }
            }
        }
    }
}
