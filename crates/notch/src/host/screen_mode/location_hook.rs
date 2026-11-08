//! Follows the foreground window's moves and resizes, pausing while its caret or the pointer floods the hook.

use std::sync::atomic::{AtomicIsize, AtomicU32, AtomicUsize, Ordering};

use windows::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
        WindowsAndMessaging::{
            EVENT_OBJECT_LOCATIONCHANGE, GetForegroundWindow, GetWindowThreadProcessId, KillTimer, OBJID_WINDOW,
            SetTimer, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
        },
    },
};

/// Moves of other parts closer together than this are a flood: the pointer or a caret moving over the window.
const FLOOD_GAP_MILLISECONDS: u32 = 100;
/// The hook stays off this long during a flood; the mode is read again when it comes back.
const FLOOD_PAUSE_MILLISECONDS: u32 = 250;

static LOCATION_HOOK: AtomicIsize = AtomicIsize::new(0);
/// Event time of the last move that was not the foreground window itself.
static LAST_PART_MOVE_TIME: AtomicU32 = AtomicU32::new(0);
static RESUME_TIMER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Reaction {
    ReadMode,
    Ignore,
    Pause,
}

/// Event times are `GetTickCount` milliseconds, so the gap is taken with wrapping arithmetic.
fn reaction(is_foreground_window: bool, previous_part_move_time: u32, event_time: u32) -> Reaction {
    if is_foreground_window {
        Reaction::ReadMode
    } else if event_time.wrapping_sub(previous_part_move_time) < FLOOD_GAP_MILLISECONDS {
        Reaction::Pause
    } else {
        Reaction::Ignore
    }
}

/// Hooks only the foreground window's thread: watching every window would wake the notch on every mouse move.
pub(super) fn follow_foreground_thread() {
    unhook();
    let window = unsafe { GetForegroundWindow() };
    if window.0.is_null() {
        return;
    }
    let thread = unsafe { GetWindowThreadProcessId(window, None) };
    let hook = unsafe {
        SetWinEventHook(
            EVENT_OBJECT_LOCATIONCHANGE,
            EVENT_OBJECT_LOCATIONCHANGE,
            None,
            Some(super::on_event),
            0,
            thread,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        )
    };
    LOCATION_HOOK.store(hook.0 as isize, Ordering::Relaxed);
}

/// Unhooks and returns true when a hook was set.
fn unhook() -> bool {
    let previous = LOCATION_HOOK.swap(0, Ordering::Relaxed);
    if previous != 0 && !unsafe { UnhookWinEvent(HWINEVENTHOOK(previous as *mut _)) }.as_bool() {
        crate::log::debug("could not unhook the foreground location hook");
    }
    previous != 0
}

/// The caret and the pointer report moves on the window's thread too, hundreds a second while the mouse moves.
pub(super) fn location_changed(window: HWND, object: i32, event_time: u32) {
    let is_foreground_window = object == OBJID_WINDOW.0 && window == unsafe { GetForegroundWindow() };
    let previous_part_move_time =
        if is_foreground_window { 0 } else { LAST_PART_MOVE_TIME.swap(event_time, Ordering::Relaxed) };
    match reaction(is_foreground_window, previous_part_move_time, event_time) {
        Reaction::ReadMode => super::refresh(false),
        Reaction::Ignore => {}
        Reaction::Pause => pause(),
    }
}

fn pause() {
    // Events queued before the unhook still arrive; the hook is already off then.
    if !unhook() {
        return;
    }
    let pending = RESUME_TIMER.swap(0, Ordering::Relaxed);
    if pending != 0 {
        kill_timer(pending);
    }
    let timer = unsafe { SetTimer(None, 0, FLOOD_PAUSE_MILLISECONDS, Some(resume)) };
    RESUME_TIMER.store(timer, Ordering::Relaxed);
}

unsafe extern "system" fn resume(_window: HWND, _message: u32, timer: usize, _time: u32) {
    kill_timer(timer);
    let _ = RESUME_TIMER.compare_exchange(timer, 0, Ordering::Relaxed, Ordering::Relaxed);
    follow_foreground_thread();
    // The window may have been maximized or restored while the hook was off.
    super::refresh(false);
}

fn kill_timer(timer: usize) {
    if let Err(error) = unsafe { KillTimer(None, timer) } {
        crate::log::debug(&format!("could not stop the location hook pause: {error}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_move_of_the_foreground_window_reads_the_mode_even_in_a_flood_of_other_moves() {
        assert_eq!(reaction(true, 1_000, 1_001), Reaction::ReadMode);
    }

    #[test]
    fn a_lone_caret_move_is_ignored_without_pausing_the_hook() {
        assert_eq!(reaction(false, 1_000, 1_000 + FLOOD_GAP_MILLISECONDS), Reaction::Ignore);
    }

    #[test]
    fn pointer_moves_that_follow_each_other_closely_pause_the_hook() {
        assert_eq!(reaction(false, 1_000, 1_016), Reaction::Pause);
    }

    #[test]
    fn the_first_part_move_after_start_is_not_taken_for_a_flood() {
        assert_eq!(reaction(false, 0, 5_000_000), Reaction::Ignore);
    }

    #[test]
    fn moves_close_together_across_the_tick_count_wrap_still_pause_the_hook() {
        assert_eq!(reaction(false, u32::MAX - 10, 20), Reaction::Pause);
    }
}
