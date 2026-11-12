//! The hover wash on the open header's pin and gear: which item it follows, when it clears and what it repaints.

use super::*;
use crate::gfx::Point;
use crate::host::{
    notch::header_row::{self, HeaderTarget},
    pill_row,
};
use crate::page::PointerEvent;
use windows::Win32::UI::WindowsAndMessaging::KillTimer;

/// A notch open on the media page with the timer's header icon, its window far from the real pointer.
fn open_header() -> Notch {
    let mut notch = rig(SampleOptions { timer: true, ..SampleOptions::default() });
    notch.screen_position = POINT { x: -100_000, y: -100_000 };
    open(&mut notch, "media");
    notch
}

/// The device-pixel point at the middle of the header item that starts `left` DIP from the surface origin.
fn at_item(notch: &Notch, left: f32, width: f32) -> Point {
    let header = chrome::header_rect(notch.frame().bounds, notch.scale);
    Point::new((left + width / 2.0) * notch.scale, header.center_y())
}

fn pin_point(notch: &Notch) -> Point {
    at_item(notch, notch.header_layout(notch.frame().bounds).pin_left, pill_row::PIN_SIZE)
}

fn gear_point(notch: &Notch) -> Point {
    at_item(notch, notch.header_layout(notch.frame().bounds).gear_left, pill_row::GEAR_SIZE)
}

fn move_to(notch: &mut Notch, point: Point) {
    notch.route_pointer(PointerEvent::Moved(point));
}

#[test]
fn the_hovered_header_item_follows_the_pointer_from_the_pin_to_the_gear() {
    let mut notch = open_header();
    let (pin, gear) = (pin_point(&notch), gear_point(&notch));
    move_to(&mut notch, pin);
    assert_eq!(notch.header_marks.hovered.get(), Some(HeaderTarget::Pin));
    move_to(&mut notch, gear);
    assert_eq!(notch.header_marks.hovered.get(), Some(HeaderTarget::Settings));
}

#[test]
fn the_header_hover_clears_when_the_pointer_leaves_the_notch() {
    let mut notch = open_header();
    let pin = pin_point(&notch);
    move_to(&mut notch, pin);
    notch.pointer_left();
    assert_eq!(notch.header_marks.hovered.get(), None);
}

#[test]
fn the_header_hover_clears_when_the_notch_closes() {
    let mut notch = open_header();
    let gear = gear_point(&notch);
    move_to(&mut notch, gear);
    notch.set_open(false);
    assert_eq!(notch.header_marks.hovered.get(), None);
}

#[test]
fn a_page_icon_in_the_header_and_the_clock_never_hover() {
    let mut notch = open_header();
    let row = notch.header_layout(notch.frame().bounds);
    let header = chrome::header_rect(notch.frame().bounds, notch.scale);
    let clock = Point::new(row.clock_center * notch.scale, header.center_y());
    let timer = notch.header_targets(&row).into_iter().find(|&(_, _, target)| target == HeaderTarget::Page("timer"));
    let (timer_left, timer_width, _) = timer.expect("the timer icon is in the header");
    let timer_icon = at_item(&notch, timer_left, timer_width);
    for point in [clock, timer_icon] {
        move_to(&mut notch, point);
        assert_eq!(notch.header_marks.hovered.get(), None, "{point:?}");
    }
}

#[test]
fn only_the_pin_and_the_gear_are_hover_targets() {
    let hovers = [HeaderTarget::Pin, HeaderTarget::Settings, HeaderTarget::Page("timer"), HeaderTarget::Nothing]
        .map(|target| header_row::hover_target(Some(target)));
    assert_eq!(hovers, [Some(HeaderTarget::Pin), Some(HeaderTarget::Settings), None, None]);
}

#[test]
fn a_repaint_is_asked_for_only_when_the_hovered_item_changes() {
    let mut notch = open_header();
    let pin = pin_point(&notch);
    let mut repaints = Vec::new();
    for point in [pin, Point::new(pin.x + 1.0, pin.y), gear_point(&notch)] {
        notch.pending.repaint = false;
        notch.follow_header_hover(PointerEvent::Moved(point));
        repaints.push(notch.pending.repaint);
    }
    assert_eq!(repaints, [true, false, true], "onto the pin, within the pin, onto the gear");
}

#[test]
fn hovering_the_pin_and_the_gear_arms_no_timer() {
    let mut notch = open_header();
    let is_armed = |notch: &Notch, timer_key: usize| unsafe { KillTimer(Some(notch.window), timer_key) }.is_ok();
    for timer_key in [ANIMATION_TIMER_ID, HOVER_WATCH_TIMER_ID] {
        is_armed(&notch, timer_key);
    }
    let (pin, gear) = (pin_point(&notch), gear_point(&notch));
    for point in [pin, gear] {
        move_to(&mut notch, point);
        notch.flush();
    }
    assert!(!is_armed(&notch, ANIMATION_TIMER_ID), "no frame timer");
    assert!(!is_armed(&notch, HOVER_WATCH_TIMER_ID), "no hover watch beyond the one opening armed");
}
