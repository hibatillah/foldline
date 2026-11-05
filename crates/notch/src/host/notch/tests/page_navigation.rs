//! Wheel, touchpad and dot navigation between pages.

use super::*;

#[test]
fn touchpad_only_centers_after_release_and_can_be_interrupted_from_the_visible_position() {
    let mut notch = rig(SampleOptions { media: Some((true, false)), timer: true, ..SampleOptions::default() });
    open(&mut notch, "media");
    notch.drag_pages(0.35);
    notch.flush();
    for _ in 0..10 {
        notch.refresh_layout();
    }
    assert!((notch.track.target - 0.35).abs() < 0.001, "input must stay fractional until the quiet period ends");
    notch.track.step(1.0 / 60.0);
    let visible_position = notch.track.position;
    notch.end_swipe();
    assert_eq!(notch.track.target, 0.0);
    assert_eq!(notch.track.position, visible_position, "release must not jump the rendered position");
    notch.drag_pages(0.1);
    assert!((notch.track.target - visible_position - 0.1).abs() < 0.001);
}

#[test]
fn touchpad_progress_interpolates_between_the_pages_real_widths() {
    let mut notch = rig(SampleOptions { media: Some((true, false)), timer: true, ..SampleOptions::default() });
    open(&mut notch, "media");
    let first_width = notch.pages[notch.navigable[0]].sizes().expanded.width;
    let next_width = notch.pages[notch.navigable[1]].sizes().expanded.width;
    assert!(first_width > next_width);
    notch.drag_pages(0.25);
    for progress in [0.25, 0.5, 0.75] {
        notch.track.jump_to(progress);
        notch.refresh_layout();
        let expected = lerp(first_width, next_width, progress) + chrome::WIDTH_EXTRA;
        assert!(
            (notch.springs.expanded_width.target - expected).abs() < 0.01,
            "width must follow swipe progress {progress}"
        );
    }
}

#[test]
fn visible_page_frames_keep_natural_widths_and_the_shorter_page_enters_early() {
    let mut notch = rig(SampleOptions { media: Some((true, false)), timer: true, ..SampleOptions::default() });
    open(&mut notch, "media");
    let first_width = notch.pages[notch.navigable[0]].sizes().expanded.width;
    let next_width = notch.pages[notch.navigable[1]].sizes().expanded.width;
    notch.holds_page_position = true;
    for progress in [0.25, 0.5, 0.75] {
        notch.track.jump_to(progress);
        notch.refresh_layout();
        notch.follow_page_width();
        let viewport = notch.frame();
        let [(first, first_frame, _), (next, next_frame, _)] = notch.visible_page_frames(&viewport).unwrap();
        assert_eq!((first, next), (notch.navigable[0], notch.navigable[1]));
        assert!((first_frame.bounds.width() / SCALE - first_width).abs() < 0.01);
        assert!((next_frame.bounds.width() / SCALE - next_width).abs() < 0.01);
        assert_eq!(next_frame.expanded.width, next_width);
        assert!((next_frame.bounds.left - first_frame.bounds.right - PAGE_GAP * SCALE).abs() < 0.01);
        assert!(next_frame.bounds.left < chrome::content_rect(viewport.bounds, SCALE).right);
        assert!(
            (notch.springs.expanded_width.position - chrome::WIDTH_EXTRA - lerp(first_width, next_width, progress))
                .abs()
                < 0.01
        );
    }
}

#[test]
fn explicit_page_controls_keep_mouse_and_dot_navigation_centered() {
    let mut notch = rig(SampleOptions { media: Some((true, false)), timer: true, ..SampleOptions::default() });
    open(&mut notch, "media");
    let first_page = notch.selected_page.unwrap();
    notch.drag_pages(0.35);
    notch.track.snap_to_target();
    notch.flush();
    notch.go_to_page(first_page);
    notch.flush();
    assert!(!notch.holds_page_position);
    assert!(notch.track.target.abs() < 0.001);
    notch.track.snap_to_target();
    notch.drag_pages(0.35);
    notch.end_swipe();
    notch.track.snap_to_target();
    notch.scrolled(-120, false, false);
    assert!(!notch.holds_page_position);
    assert!((notch.track.target - 1.0).abs() < 0.001);
}

#[test]
fn touchpad_scroll_keeps_large_deltas_in_the_same_gesture() {
    let mut notch = rig(SampleOptions { media: Some((false, false)), timer: true, ..SampleOptions::default() });
    open(&mut notch, "media");
    notch.scrolled(-30, false, false);
    notch.scrolled(-120, false, false);
    assert!(notch.swipe.is_some(), "a large touchpad delta must not turn into a mouse-wheel snap");
    assert!((notch.track.target - 0.375).abs() < 0.001);
}

#[test]
fn a_tagged_touchpad_starts_smoothly_even_with_a_full_wheel_sized_delta() {
    let mut notch = rig(SampleOptions { media: Some((false, false)), timer: true, ..SampleOptions::default() });
    open(&mut notch, "media");
    notch.scrolled(-120, false, true);
    assert!(notch.swipe.is_some());
    assert!((notch.track.target - 0.3).abs() < 0.001);
    assert_eq!(notch.track.position, 0.0);
}

#[test]
fn touchpad_scroll_does_not_change_logical_selection_at_the_midpoint() {
    let mut notch = rig(SampleOptions { media: Some((false, false)), timer: true, ..SampleOptions::default() });
    open(&mut notch, "media");
    let selection = notch.selected_page;
    for delta in [0.45, 0.1, -0.1, 0.1] {
        notch.drag_pages(delta);
        notch.flush();
        assert_eq!(notch.selected_page, selection);
    }
    notch.end_swipe();
    notch.flush();
    assert_ne!(notch.selected_page, selection);
}

#[test]
fn touchpad_scroll_ignores_a_snap_timer_queued_before_the_latest_input() {
    let mut notch = rig(SampleOptions { media: Some((false, false)), timer: true, ..SampleOptions::default() });
    open(&mut notch, "media");
    notch.drag_pages(0.25);
    notch.timer_fired(SWIPE_END_TIMER_ID);
    assert!(notch.swipe.is_some(), "fresh input must win over a previously queued timer");
    assert_eq!(notch.track.target, 0.25);
}

#[test]
fn a_touchpad_drag_follows_smoothly_and_can_continue_across_pages() {
    let mut notch = rig(SampleOptions { media: Some((false, false)), timer: true, ..SampleOptions::default() });
    open(&mut notch, "media");
    let origin = notch.track.position;
    notch.drag_pages(0.25);
    assert_eq!(notch.track.position, origin, "input must not teleport the rendered page");
    assert_eq!(notch.track.target, origin + 0.25);
    notch.track.step(1.0 / 60.0);
    assert!(notch.track.position > origin && notch.track.position < notch.track.target);
    notch.end_swipe();
    assert_eq!(notch.track.target, origin);
    for _ in 0..60 {
        notch.track.step(1.0 / 120.0);
    }
    assert!(notch.track.is_settled());

    for _ in 0..30 {
        notch.drag_pages(0.125);
        notch.track.step(1.0 / 60.0);
    }
    assert_eq!(notch.track.target, origin + 3.75);
    notch.end_swipe();
    assert_eq!(notch.track.target, origin + 4.0);
    for _ in 0..90 {
        notch.track.step(1.0 / 120.0);
    }
    assert!(notch.track.is_settled());
}
