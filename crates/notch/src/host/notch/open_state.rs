use super::{CLOSE_TIMER_ID, EXPAND_TIMER_ID, HOVER_WATCH_TIMER_ID, Notch, REVEAL_TIMER_ID};
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

use crate::{log, page::PointerEvent};

/// Whether the notch is open, and the pointer facts that decide it.
#[derive(Default)]
pub(super) struct OpenState {
    pub(super) is_open: bool,
    pub(super) is_close_pending: bool,
    pub(super) is_dragging_files: bool,
    /// The user pinned the notch open; the pointer leaving no longer closes it (`pin.rs`).
    pub(super) is_pinned: bool,
}

const HOVER_WATCH_INTERVAL_MILLISECONDS: u32 = 120;
const CLOSE_DELAY_MILLISECONDS: u32 = 200;
const POINTER_OPEN_DELAY_MILLISECONDS: u32 = 100;

impl Notch {
    pub(super) fn close_if_pointer_is_away(&mut self) {
        if self.open.is_open && !self.open.is_pinned && !self.pointer_is_over_window() {
            self.set_open(false);
        }
    }

    pub(super) fn pointer_left(&mut self) {
        if self.open.is_close_pending {
            return;
        }
        self.route_pointer(PointerEvent::Left);
        if self.open.is_pinned {
            return;
        }
        self.open.is_close_pending = true;
        unsafe { SetTimer(Some(self.window), CLOSE_TIMER_ID, CLOSE_DELAY_MILLISECONDS, None) };
    }

    pub(super) fn cancel_close(&mut self) {
        if self.open.is_close_pending {
            self.open.is_close_pending = false;
            let _ = unsafe { KillTimer(Some(self.window), CLOSE_TIMER_ID) };
        }
    }

    pub(super) fn begin_opening(&mut self) {
        self.springs.expansion.target = 1.0;
    }

    pub(super) fn begin_closing(&mut self) {
        self.springs.expansion.target = 0.0;
    }

    pub(super) fn set_open(&mut self, is_open: bool) {
        if is_open && !self.is_suppressed() {
            crate::host::window::restore_topmost(self.window);
        }
        if self.open.is_open == is_open {
            return;
        }
        self.open.is_open = is_open;
        if !is_open {
            self.cancel_close();
        }
        log::debug(&format!("open={is_open}"));
        crate::host::drop_target::set_open(is_open);
        let waits_to_reveal = is_open && self.is_hidden();
        if is_open && self.open.is_pinned {
            self.begin_opening();
        } else if is_open && !self.opens_on_pointer() {
            log::debug("pointer opening is off over this window");
        } else if waits_to_reveal {
            unsafe { SetTimer(Some(self.window), REVEAL_TIMER_ID, self.reveal_delay(), None) };
        } else if is_open {
            unsafe { SetTimer(Some(self.window), EXPAND_TIMER_ID, POINTER_OPEN_DELAY_MILLISECONDS, None) };
        } else {
            self.begin_closing();
        }
        if !is_open {
            self.hiding.is_revealed = false;
            let _ = unsafe { KillTimer(Some(self.window), REVEAL_TIMER_ID) };
            let _ = unsafe { KillTimer(Some(self.window), EXPAND_TIMER_ID) };
        }
        self.start_animation();
        for index in 0..self.pages.len() {
            self.with_page(index, |page, context| page.on_open_changed(is_open, context));
        }
        for index in 0..self.activities.len() {
            if self.activities[index].is_started {
                self.with_activity(index, |activity, context| activity.on_open_changed(is_open, context));
            }
        }
        if !is_open {
            self.route_pointer(PointerEvent::Left);
        }
        if is_open && !self.open.is_pinned {
            self.arm_hover_watch();
        } else {
            let _ = unsafe { KillTimer(Some(self.window), HOVER_WATCH_TIMER_ID) };
        }
        if !is_open {
            self.release_waiting_move();
        }
    }

    pub(super) fn arm_hover_watch(&self) {
        unsafe { SetTimer(Some(self.window), HOVER_WATCH_TIMER_ID, HOVER_WATCH_INTERVAL_MILLISECONDS, None) };
    }
}
