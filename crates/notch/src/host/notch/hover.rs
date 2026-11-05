use super::{CLOSE_TIMER_ID, EXPAND_TIMER_ID, HOVER_WATCH_TIMER_ID, Notch, REVEAL_TIMER_ID};
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

use crate::{log, page::PointerEvent};

/// Pointer facts that decide whether the notch is open.
#[derive(Default)]
pub(super) struct HoverState {
    pub(super) is_hovering: bool,
    pub(super) is_close_pending: bool,
    pub(super) is_captured: bool,
    pub(super) is_dragging_files: bool,
}

const HOVER_WATCH_INTERVAL_MILLISECONDS: u32 = 120;
const CLOSE_DELAY_MILLISECONDS: u32 = 200;
const HOVER_OPEN_DELAY_MILLISECONDS: u32 = 100;

impl Notch {
    /// The notch lost activation to another window while a page was typing.
    pub fn deactivated(&mut self) {
        let Some(owner) = self.keyboard.owner.take() else { return };
        self.keyboard.previous = None;
        self.keyboard.pending_high_surrogate = None;
        log::debug(&format!("keyboard lost by {owner:?}"));
        self.keyboard_lost_by(owner);
        self.close_if_pointer_is_away();
        self.flush();
    }

    pub(super) fn close_if_pointer_is_away(&mut self) {
        if self.hover.is_hovering && !self.hover.is_captured && !self.pointer_is_over_window() {
            self.set_hovering(false);
        }
    }

    pub(super) fn pointer_left(&mut self) {
        if self.hover.is_close_pending {
            return;
        }
        self.route_pointer(PointerEvent::Left);
        self.hover.is_close_pending = true;
        unsafe {
            SetTimer(Some(self.window), CLOSE_TIMER_ID, CLOSE_DELAY_MILLISECONDS, None);
        }
    }

    pub(super) fn cancel_close(&mut self) {
        if self.hover.is_close_pending {
            self.hover.is_close_pending = false;
            unsafe {
                let _ = KillTimer(Some(self.window), CLOSE_TIMER_ID);
            }
        }
    }

    pub(super) fn begin_opening(&mut self) {
        self.springs.expansion.target = 1.0;
    }

    pub(super) fn begin_closing(&mut self) {
        self.springs.expansion.target = 0.0;
    }

    pub(super) fn set_hovering(&mut self, is_hovering: bool) {
        if is_hovering && !self.is_suppressed() {
            crate::host::window::restore_topmost(self.window);
        }
        if self.hover.is_hovering == is_hovering {
            return;
        }
        self.hover.is_hovering = is_hovering;
        if !is_hovering {
            self.editor = None;
            self.menu = None;
            self.cancel_close();
        }
        log::debug(&format!("hovering={is_hovering}"));
        crate::host::drop_target::set_hovering(is_hovering);
        let waits_to_reveal = is_hovering && self.is_hidden();
        if is_hovering && !self.opens_on_hover() {
            log::debug("hover opening is off over this window");
        } else if waits_to_reveal {
            unsafe {
                SetTimer(Some(self.window), REVEAL_TIMER_ID, self.reveal_delay(), None);
            }
        } else {
            if is_hovering {
                unsafe {
                    SetTimer(Some(self.window), EXPAND_TIMER_ID, HOVER_OPEN_DELAY_MILLISECONDS, None);
                }
            } else {
                self.begin_closing();
            }
        }
        if !is_hovering {
            self.hiding.is_revealed = false;
            unsafe {
                let _ = KillTimer(Some(self.window), REVEAL_TIMER_ID);
                let _ = KillTimer(Some(self.window), EXPAND_TIMER_ID);
            }
        }
        self.start_animation();
        for index in 0..self.pages.len() {
            self.with_page(index, |page, context| page.on_hover_changed(is_hovering, context));
        }
        for index in 0..self.activities.len() {
            if self.activities[index].is_started {
                self.with_activity(index, |activity, context| activity.on_hover_changed(is_hovering, context));
            }
        }
        if !is_hovering {
            self.route_pointer(PointerEvent::Left);
        }
        unsafe {
            if is_hovering {
                SetTimer(Some(self.window), HOVER_WATCH_TIMER_ID, HOVER_WATCH_INTERVAL_MILLISECONDS, None);
            } else {
                let _ = KillTimer(Some(self.window), HOVER_WATCH_TIMER_ID);
            }
        }
    }
}
