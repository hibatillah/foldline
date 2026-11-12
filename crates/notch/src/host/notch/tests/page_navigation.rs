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
    assert!(notch.frame_timer.is_running || notch.pending.animate || notch.track.target > 0.0);
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

/// The first page after the open one whose content height differs from it, reached with that many wheel notches.
fn steps_to_a_page_of_another_height(notch: &Notch) -> usize {
    let open_height = notch.pages[notch.expanded_index.unwrap()].sizes().expanded.height;
    let position = notch.navigable.iter().position(|&index| Some(index) == notch.expanded_index).unwrap();
    (1..notch.navigable.len())
        .find(|step| {
            let index = notch.navigable[(position + step) % notch.navigable.len()];
            layout::content_height(notch.pages[index].sizes().expanded.height) != layout::content_height(open_height)
        })
        .expect("the rig has pages of two heights")
}

#[test]
fn a_page_switch_always_ends_on_the_new_page_at_rest_and_stops_the_frame_timer() {
    let mut notch = media_rig();
    let start = Instant::now();
    notch.scrolled_at(start, -30, true, true);
    notch.scrolled_at(at(start, 16), -100, true, true);
    let arriving = notch.expanded_index;
    assert!(notch.page_swap.is_running(), "a swipe past the threshold starts a swap");
    let mut frames = 0;
    while notch.frame_timer.is_running && frames < 600 {
        notch.advance_by(1.0 / 120.0);
        frames += 1;
    }
    assert!(!notch.frame_timer.is_running, "the frame timer still runs");
    assert!(!notch.page_swap.is_running());
    let [leaving, open] = notch.swap_page_frames(&notch.frame());
    assert!(leaving.is_none());
    assert_eq!(open.map(|(index, _, opacity)| (Some(index), opacity)), Some((arriving, 1.0)));
}

#[test]
fn a_small_swipe_below_the_threshold_leaves_the_open_page_untouched() {
    let mut notch = media_rig();
    let open = notch.expanded_index;
    notch.scrolled_at(Instant::now(), -60, true, true);
    assert_eq!(notch.expanded_index, open);
    assert!(!notch.page_swap.is_running());
}

#[test]
fn the_arriving_page_moves_only_sideways_while_the_pill_changes_height() {
    let mut notch = media_rig();
    notch.navigate(steps_to_a_page_of_another_height(&notch) as i32);
    notch.flush();
    let target = notch.expanded_index.unwrap();
    assert!(notch.page_swap.is_running());
    let height_before = notch.frame().bounds.height();
    let mut tops = Vec::new();
    let mut lefts = Vec::new();
    while notch.page_swap.is_running() {
        notch.advance_by(1.0 / 120.0);
        let viewport = notch.frame();
        let [_, arriving] = notch.swap_page_frames(&viewport);
        let (_, frame, _) = arriving.expect("the open page is drawn");
        tops.push(frame.bounds.top);
        lefts.push(frame.bounds.left - notch.natural_page_frame(target, &viewport, 0.0).bounds.left);
        let natural = layout::content_height(notch.pages[target].sizes().expanded.height);
        assert!((frame.bounds.height() / SCALE - natural).abs() < 0.01, "the page keeps its own height");
    }
    assert_ne!(notch.frame().bounds.height(), height_before, "the pill changed height");
    assert!(tops.windows(2).all(|pair| pair[0] == pair[1]), "the page never moves up or down");
    assert!(lefts.windows(2).all(|pair| pair[1] <= pair[0] + 0.01), "the next page only moves west, toward its place");
}

#[test]
fn the_open_page_is_clipped_below_the_header() {
    let notch = media_rig();
    let viewport = notch.frame();
    let [_, open] = notch.swap_page_frames(&viewport);
    let (_, frame, _) = open.unwrap();
    let clip = chrome::page_clip_rect(viewport.bounds, SCALE);
    assert_eq!(clip.top, frame.bounds.top);
    assert!(clip.top >= chrome::header_rect(viewport.bounds, SCALE).bottom);
}

#[test]
fn a_page_change_while_the_notch_is_closed_swaps_nothing() {
    let mut notch = media_rig();
    close(&mut notch);
    let next = notch.navigable[1];
    notch.select_page(next);
    notch.refresh_layout();
    assert_eq!(notch.expanded_index, Some(next));
    assert!(!notch.page_swap.is_running());
}

#[test]
fn a_page_that_asks_to_open_from_its_banner_opens_the_notch_on_itself() {
    let mut notch = rig(SampleOptions { media: Some(true), timer: true, ..SampleOptions::default() });
    settle(&mut notch);
    let timer = notch.pages.iter().position(|page| page.id() == "timer").expect("the timer page is in the rig");
    notch.set_open(true);
    notch.with_page(timer, |_, context| context.request_open());
    notch.flush();
    assert_eq!(notch.springs.expansion.target, 1.0);
    assert_eq!(notch.expanded_index, Some(timer));
}

#[test]
fn an_activity_cannot_open_the_notch_on_a_page() {
    let mut notch = marked_rig(SampleOptions::default(), vec![Mark::showing(None).0]);
    notch.set_open(true);
    notch.with_activity(0, |_, context| context.request_open());
    notch.flush();
    assert_eq!(notch.springs.expansion.target, 0.0);
}
