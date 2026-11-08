//! Wheel, touchpad and dot navigation between pages.

use std::time::{Duration, Instant};

use super::*;

fn media_rig() -> Notch {
    let mut notch = rig(SampleOptions { media: Some(true), timer: true, ..SampleOptions::default() });
    open(&mut notch, "media");
    notch
}

fn settle_track(notch: &mut Notch) {
    notch.track.snap_to_target();
    notch.flush();
}

fn at(start: Instant, milliseconds: u64) -> Instant {
    start + Duration::from_millis(milliseconds)
}

#[test]
fn one_wheel_notch_moves_one_page_and_the_next_notch_waits_for_the_cooldown() {
    let mut notch = media_rig();
    let origin = notch.track.target;
    let start = Instant::now();
    notch.scrolled_at(start, -120, false, false);
    assert_eq!(notch.track.target, origin + 1.0);
    notch.scrolled_at(at(start, 120), -120, false, false);
    notch.scrolled_at(at(start, 240), -120, false, false);
    assert_eq!(notch.track.target, origin + 1.0, "notches inside the cooldown are ignored");
    notch.scrolled_at(at(start, 320), -120, false, false);
    assert_eq!(notch.track.target, origin + 2.0);
}

#[test]
fn the_wheel_loops_from_the_last_page_to_the_first_and_back() {
    let mut notch = media_rig();
    let count = notch.navigable.len();
    let first_page = *notch.navigable.first().unwrap();
    let last_page = *notch.navigable.last().unwrap();
    let start = Instant::now();
    notch.scrolled_at(start, 120, false, false);
    assert_eq!(notch.selected_page, Some(last_page), "previous from the first page goes to the last");
    settle_track(&mut notch);
    notch.scrolled_at(at(start, 1000), -120, false, false);
    assert_eq!(notch.selected_page, Some(first_page), "next from the last page goes to the first");
    settle_track(&mut notch);
    for step in 0..count as u64 {
        notch.scrolled_at(at(start, 2000 + 1000 * step), -120, false, false);
        settle_track(&mut notch);
    }
    assert_eq!(notch.selected_page, Some(notch.page_at(notch.track.target)), "a full lap returns to the same page");
    assert_eq!(notch.selected_page, Some(first_page));
}

#[test]
fn a_touchpad_swipe_switches_one_page_per_gesture_even_when_it_keeps_going() {
    let mut notch = media_rig();
    let origin = notch.track.target;
    let start = Instant::now();
    for tick in 0..40 {
        notch.scrolled_at(at(start, tick * 16), -30, false, true);
    }
    assert_eq!(notch.track.target, origin + 1.0);
    for tick in 0..40 {
        notch.scrolled_at(at(start, 2000 + tick * 16), -30, false, true);
    }
    assert_eq!(notch.track.target, origin + 2.0, "a new gesture after a quiet gap moves one more page");
}

#[test]
fn a_large_delta_inside_a_touchpad_gesture_is_not_taken_for_a_wheel_notch() {
    let mut notch = media_rig();
    let origin = notch.track.target;
    let start = Instant::now();
    notch.scrolled_at(start, -30, false, false);
    notch.scrolled_at(at(start, 16), -120, false, false);
    notch.scrolled_at(at(start, 32), -120, false, false);
    notch.scrolled_at(at(start, 48), -120, false, false);
    assert_eq!(notch.track.target, origin + 1.0);
}

#[test]
fn a_horizontal_swipe_goes_the_other_way_for_the_same_sign() {
    let mut notch = media_rig();
    let origin = notch.track.target;
    notch.scrolled_at(Instant::now(), 130, true, true);
    assert_eq!(notch.track.target, origin + 1.0);
}

#[test]
fn a_page_switch_gives_width_and_height_new_targets_for_springs_that_animate_them() {
    let mut notch = media_rig();
    let (width, height) = (notch.springs.expanded_width.position, notch.springs.expanded_height.position);
    let next = notch.navigable[1];
    let next_size = notch.pages[next].sizes().expanded;
    notch.scrolled_at(Instant::now(), -120, false, false);
    assert!(notch.is_animating || notch.pending.animate || notch.track.target > 0.0);
    assert_eq!(notch.springs.expanded_width.position, width, "the size must not snap");
    assert_eq!(notch.springs.expanded_height.position, height, "the size must not snap");
    let next_width = (next_size.width + chrome::WIDTH_EXTRA).max(notch.open_min_width());
    assert!((notch.springs.expanded_width.target - next_width).abs() < 0.01);
}

#[test]
fn explicit_page_controls_stay_unrestricted_and_can_jump_across_the_row() {
    let mut notch = media_rig();
    let first_page = notch.selected_page.unwrap();
    let last_page = *notch.navigable.last().unwrap();
    notch.go_to_page(last_page);
    notch.flush();
    assert_eq!(notch.selected_page, Some(last_page));
    settle_track(&mut notch);
    notch.go_to_page(first_page);
    notch.flush();
    assert_eq!(notch.selected_page, Some(first_page));
}

#[test]
fn page_blend_between_neighbours_keeps_natural_widths_and_the_shorter_page_enters_early() {
    let mut notch = media_rig();
    let first_width = notch.pages[notch.navigable[0]].sizes().expanded.width;
    let next_width = notch.pages[notch.navigable[1]].sizes().expanded.width;
    for progress in [0.25, 0.5, 0.75] {
        notch.track.jump_to(progress);
        notch.refresh_layout();
        let viewport = notch.frame();
        let [(first, first_frame, _), (next, next_frame, _)] = notch.visible_page_frames(&viewport).unwrap();
        assert_eq!((first, next), (notch.navigable[0], notch.navigable[1]));
        assert!((first_frame.bounds.width() / SCALE - first_width).abs() < 0.01);
        assert!((next_frame.bounds.width() / SCALE - next_width).abs() < 0.01);
        assert_eq!(next_frame.expanded.width, next_width);
        assert!((next_frame.bounds.left - first_frame.bounds.right - PAGE_GAP * SCALE).abs() < 0.01);
    }
}

#[test]
fn a_page_that_asks_to_open_from_its_banner_opens_the_notch_on_itself() {
    let mut notch = rig(SampleOptions { media: Some(true), timer: true, ..SampleOptions::default() });
    settle(&mut notch);
    let timer = notch.pages.iter().position(|page| page.id() == "timer").expect("the timer page is in the rig");
    notch.set_hovering(true);
    notch.with_page(timer, |_, context| context.request_open());
    notch.flush();
    assert_eq!(notch.springs.expansion.target, 1.0);
    assert_eq!(notch.expanded_index, Some(timer));
}

#[test]
fn an_activity_cannot_open_the_notch_on_a_page() {
    let mut notch = marked_rig(SampleOptions::default(), vec![Mark::showing(None).0]);
    notch.set_hovering(true);
    notch.with_activity(0, |_, context| context.request_open());
    notch.flush();
    assert_eq!(notch.springs.expansion.target, 0.0);
}
