//! The pin: what keeps a pinned notch open, what still hides it, and what unpinning does.
//! The pointer's real position cannot be set here, so every rig keeps the window far from it; staying open after an
//! unpin with the pointer on the notch is checked on the pure decision in `host/notch/pin.rs`.

use super::*;
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

/// Whether `timer_key` was armed on the rig's window; disarms it as a side effect.
fn is_armed(notch: &Notch, timer_key: usize) -> bool {
    unsafe { KillTimer(Some(notch.window), timer_key) }.is_ok()
}

/// A rig whose window sits far off every screen, so the pointer is never over it.
fn pointer_away_rig() -> Notch {
    let mut notch = rig(SampleOptions::default());
    notch.screen_position = POINT { x: -100_000, y: -100_000 };
    notch
}

fn pinned_open() -> Notch {
    let mut notch = pointer_away_rig();
    open(&mut notch, "media");
    notch.set_pinned(true);
    notch
}

fn is_fully_open(notch: &Notch) -> bool {
    notch.open.is_open && notch.springs.expansion.target == 1.0
}

#[test]
fn a_pinned_notch_stays_open_when_the_pointer_leaves() {
    let mut notch = pinned_open();
    notch.pointer_left();
    assert!(!is_armed(&notch, CLOSE_TIMER_ID), "no close delay starts");
    notch.close_if_pointer_is_away();
    notch.timer_fired(CLOSE_TIMER_ID);
    assert!(is_fully_open(&notch));
}

#[test]
fn unpinning_with_the_pointer_away_closes_after_the_usual_delay() {
    let mut notch = pinned_open();
    notch.set_pinned(false);
    assert!(notch.open.is_close_pending);
    assert!(is_fully_open(&notch), "the close waits for its delay");
    notch.timer_fired(CLOSE_TIMER_ID);
    assert!(!notch.open.is_open);
    assert_eq!(notch.springs.expansion.target, 0.0);
}

#[test]
fn a_pinned_notch_arms_no_hover_watch_timer() {
    let mut notch = pointer_away_rig();
    notch.set_open(true);
    assert!(is_armed(&notch, HOVER_WATCH_TIMER_ID), "an open notch watches the pointer");
    notch.set_pinned(true);
    assert!(!is_armed(&notch, HOVER_WATCH_TIMER_ID), "pinning stops the watch");

    let mut closed = pointer_away_rig();
    closed.set_pinned(true);
    assert!(is_fully_open(&closed), "pinning a collapsed notch opens it at once");
    assert!(!is_armed(&closed, HOVER_WATCH_TIMER_ID));
    assert!(!is_armed(&closed, EXPAND_TIMER_ID), "no pointer delay applies to a pin");
}

#[test]
fn the_pin_survives_a_locked_session() {
    let mut notch = pinned_open();
    notch.open.is_dragging_files = true;
    notch.reset_session_interaction();
    assert!(notch.is_pinned());
    assert!(is_fully_open(&notch));
    assert!(!notch.open.is_dragging_files, "the drag state is still cleared");
    assert!(notch.drop_target_stays_told_open(), "the drop overlay hears the notch is still open");
}

#[test]
fn a_locked_session_tells_the_drop_overlay_an_unpinned_notch_closed() {
    let mut notch = pointer_away_rig();
    open(&mut notch, "media");
    notch.reset_session_interaction();
    assert!(!notch.drop_target_stays_told_open());
}

#[test]
fn a_pinned_notch_over_a_fullscreen_window_set_to_hidden_stays_visible() {
    let mut notch = pointer_away_rig();
    notch.hiding.fullscreen_showing = Showing::Hidden;
    notch.set_screen_mode(ScreenMode::Fullscreen);
    assert!(notch.is_suppressed());
    notch.set_pinned(true);
    assert!(!notch.is_suppressed(), "pinning shows a suppressed notch at once");
    assert!(is_fully_open(&notch));
    assert!(is_armed(&notch, RAISE_TIMER_ID), "pinning raises the notch above the full-screen window");

    let mut pinned_first = pinned_open();
    pinned_first.hiding.fullscreen_showing = Showing::Hidden;
    pinned_first.set_screen_mode(ScreenMode::Fullscreen);
    assert!(!pinned_first.is_suppressed(), "a full-screen window that comes later does not hide it");
    assert!(is_fully_open(&pinned_first));
}

#[test]
fn unpinning_over_a_fullscreen_window_set_to_hidden_hides_the_notch_again() {
    let mut notch = pinned_open();
    notch.hiding.fullscreen_showing = Showing::Hidden;
    notch.set_screen_mode(ScreenMode::Fullscreen);
    notch.set_pinned(false);
    assert!(notch.is_suppressed());
    assert!(!notch.open.is_open);
}

#[test]
fn pinning_a_notch_hidden_to_the_strip_shows_it_at_once() {
    let mut notch = pointer_away_rig();
    notch.hiding.hides_when_idle = true;
    assert!(notch.is_hidden());
    notch.set_pinned(true);
    assert!(!notch.is_hidden());
    assert!(is_fully_open(&notch));
}

#[test]
fn a_screenshot_overlay_hides_a_pinned_notch_and_it_returns_pinned() {
    let mut notch = pinned_open();
    notch.set_screen_mode(ScreenMode::Capture);
    assert!(notch.is_suppressed());
    assert!(!notch.open.is_open);
    assert!(notch.is_pinned(), "the overlay keeps the pin");
    notch.set_screen_mode(ScreenMode::Normal);
    notch.timer_fired(WINDOW_REVEAL_TIMER_ID);
    assert!(!notch.is_suppressed());
    assert!(is_fully_open(&notch), "the notch opens again, pinned");
}

#[test]
fn a_page_cannot_close_a_pinned_notch() {
    let mut notch = pinned_open();
    notch.pending.close = true;
    notch.flush();
    assert!(is_fully_open(&notch));
}

#[test]
fn a_file_drag_that_ends_away_from_a_pinned_notch_keeps_it_open() {
    let mut notch = pinned_open();
    notch.open.is_dragging_files = true;
    notch.drag_left();
    assert!(is_fully_open(&notch));
}

#[test]
fn a_notch_pinned_on_another_page_stops_the_stats_sampler() {
    let mut notch = pointer_away_rig();
    let stats = notch.pages.iter().position(|page| page.id() == "stats").unwrap();
    notch.page_list.set_enabled(stats, true);
    notch.with_page(stats, |page, context| page.on_enabled_changed(true, context));
    let sample_timer = crate::page::timer_key(stats, crate::page::TimerId(0));
    notch.selected_page = Some(stats);
    notch.set_pinned(true);
    assert!(is_armed(&notch, sample_timer), "the shown stats page samples while pinned");
    // The check above disarmed it; arm it again so the page has a running timer to stop.
    unsafe { SetTimer(Some(notch.window), sample_timer, 1000, None) };
    let media = notch.pages.iter().position(|page| page.id() == "media").unwrap();
    notch.go_to_page(media);
    notch.flush();
    assert!(notch.is_pinned());
    assert!(!is_armed(&notch, sample_timer), "another page shows, so the sampler stops");
}

#[test]
fn toggling_the_pin_twice_leaves_it_unpinned() {
    let mut notch = pinned_open();
    assert!(notch.is_pinned());
    notch.toggle_pin();
    assert!(!notch.is_pinned());
}
