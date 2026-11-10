//! Opens the device-independent unit space around every page and activity hook and converts pointer points into it.

use crate::{
    activity::Activity,
    gfx::{Canvas, Point, Rect},
    page::{Context, Frame, Page, PointerEvent},
};

/// A frame whose bounds (device pixels at `scale`) start at zero and are measured in units.
pub(super) fn unit_frame(frame: &Frame, scale: f32) -> Frame {
    Frame { bounds: unit_area(frame.bounds, scale), ..*frame }
}

/// An area (device pixels) as a rect starting at zero, measured in units.
pub(super) fn unit_area(area: Rect, scale: f32) -> Rect {
    Rect::new(0.0, 0.0, area.width() / scale, area.height() / scale)
}

/// A device-pixel point as a point in the units that start at `origin`.
pub(super) fn unit_point(point: Point, origin: Point, scale: f32) -> Point {
    Point::new((point.x - origin.x) / scale, (point.y - origin.y) / scale)
}

/// The event with every position converted by `unit_point`; events without a position pass through.
pub(super) fn unit_event(event: PointerEvent, origin: Point, scale: f32) -> PointerEvent {
    let convert = |point| unit_point(point, origin, scale);
    match event {
        PointerEvent::Moved(point) => PointerEvent::Moved(convert(point)),
        PointerEvent::Pressed(point) => PointerEvent::Pressed(convert(point)),
        PointerEvent::Released(point) => PointerEvent::Released(convert(point)),
        PointerEvent::SecondaryReleased(point) => PointerEvent::SecondaryReleased(convert(point)),
        PointerEvent::Left | PointerEvent::CaptureLost | PointerEvent::Scrolled { .. } => event,
    }
}

fn top_left(area: Rect) -> Point {
    Point::new(area.left, area.top)
}

/// The canvas passed to these is outside the unit space, so its scale is the display scale.
pub(super) fn draw_page_resting(page: &dyn Page, canvas: &Canvas, frame: &Frame, opacity: f32) {
    let unit = unit_frame(frame, canvas.scale());
    canvas.in_units(top_left(frame.bounds), |canvas| page.draw_resting(canvas, &unit, opacity));
}

pub(super) fn draw_page_expanded(page: &dyn Page, canvas: &Canvas, frame: &Frame, opacity: f32) {
    let unit = unit_frame(frame, canvas.scale());
    canvas.in_units(top_left(frame.bounds), |canvas| page.draw_expanded(canvas, &unit, opacity));
}

pub(super) fn draw_page_presence(page: &dyn Page, canvas: &Canvas, area: Rect, opacity: f32) {
    let unit = unit_area(area, canvas.scale());
    canvas.in_units(top_left(area), |canvas| page.draw_presence(canvas, unit, opacity));
}

pub(super) fn draw_page_short_presence(page: &dyn Page, canvas: &Canvas, area: Rect, opacity: f32) {
    let unit = unit_area(area, canvas.scale());
    canvas.in_units(top_left(area), |canvas| page.draw_short_presence(canvas, unit, opacity));
}

pub(super) fn draw_page_header_status(page: &dyn Page, canvas: &Canvas, area: Rect, opacity: f32) {
    let unit = unit_area(area, canvas.scale());
    canvas.in_units(top_left(area), |canvas| page.draw_header_status(canvas, unit, opacity));
}

pub(super) fn draw_activity(activity: &dyn Activity, canvas: &Canvas, area: Rect, opacity: f32) {
    let unit = unit_area(area, canvas.scale());
    canvas.in_units(top_left(area), |canvas| activity.draw(canvas, unit, opacity));
}

pub(super) fn draw_activity_header_status(activity: &dyn Activity, canvas: &Canvas, area: Rect, opacity: f32) {
    let unit = unit_area(area, canvas.scale());
    canvas.in_units(top_left(area), |canvas| activity.draw_header_status(canvas, unit, opacity));
}

/// `frame` and `event` are in device pixels at `scale`.
pub(super) fn page_pointer(page: &mut dyn Page, event: PointerEvent, frame: &Frame, scale: f32, context: &mut Context) {
    let event = unit_event(event, top_left(frame.bounds), scale);
    page.on_pointer(event, &unit_frame(frame, scale), context);
}

pub(super) fn page_presence_pointer(
    page: &mut dyn Page,
    event: PointerEvent,
    area: Rect,
    scale: f32,
    context: &mut Context,
) {
    page.on_presence_pointer(unit_event(event, top_left(area), scale), unit_area(area, scale), context);
}

pub(super) fn activity_pointer(
    activity: &mut dyn Activity,
    event: PointerEvent,
    area: Rect,
    scale: f32,
    context: &mut Context,
) {
    activity.on_pointer(unit_event(event, top_left(area), scale), unit_area(area, scale), context);
}

pub(super) fn page_uses_right_click(page: &dyn Page, point: Point, frame: &Frame, scale: f32) -> bool {
    page.uses_right_click(unit_point(point, top_left(frame.bounds), scale), &unit_frame(frame, scale))
}

pub(super) fn page_claims_scroll(page: &dyn Page, point: Point, frame: &Frame, scale: f32) -> bool {
    page.claims_scroll(unit_point(point, top_left(frame.bounds), scale), &unit_frame(frame, scale))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::Size;

    #[test]
    fn a_point_on_the_last_pixel_of_a_unit_rect_is_still_inside_it() {
        let (scale, origin) = (1.25, Point::new(100.5, 40.0));
        let button = Rect::new(0.0, 0.0, 72.0, 26.0);
        let last_pixel = Point::new(origin.x + 72.0 * scale - 0.5, origin.y + 26.0 * scale - 0.5);
        assert!(button.contains(unit_point(last_pixel, origin, scale)));
    }

    #[test]
    fn a_point_one_pixel_past_a_unit_rect_is_outside_it() {
        let (scale, origin) = (1.25, Point::new(100.5, 40.0));
        let button = Rect::new(0.0, 0.0, 72.0, 26.0);
        let past = Point::new(origin.x + 72.0 * scale + 0.5, origin.y + 10.0);
        assert!(!button.contains(unit_point(past, origin, scale)));
    }

    #[test]
    fn every_pointer_event_with_a_position_is_converted_and_the_others_pass_through() {
        let (scale, origin) = (1.5, Point::new(30.0, 12.0));
        let device = Point::new(60.0, 42.0);
        let unit = Point::new(20.0, 20.0);
        assert_eq!(unit_event(PointerEvent::Moved(device), origin, scale), PointerEvent::Moved(unit));
        assert_eq!(unit_event(PointerEvent::Pressed(device), origin, scale), PointerEvent::Pressed(unit));
        assert_eq!(unit_event(PointerEvent::Released(device), origin, scale), PointerEvent::Released(unit));
        assert_eq!(
            unit_event(PointerEvent::SecondaryReleased(device), origin, scale),
            PointerEvent::SecondaryReleased(unit)
        );
        assert_eq!(unit_event(PointerEvent::Left, origin, scale), PointerEvent::Left);
        assert_eq!(unit_event(PointerEvent::CaptureLost, origin, scale), PointerEvent::CaptureLost);
        let scrolled = PointerEvent::Scrolled { notches: -2.0 };
        assert_eq!(unit_event(scrolled, origin, scale), scrolled);
    }

    #[test]
    fn a_unit_frame_keeps_expansion_and_content_size_and_starts_at_zero() {
        let frame = Frame {
            bounds: Rect::new(100.0, 50.0, 350.0, 175.0),
            expansion: 0.9,
            expanded: Size::new(200.0, 100.0),
            lead_gap: 7.0,
        };
        let unit = unit_frame(&frame, 1.25);
        assert_eq!(unit.bounds, Rect::new(0.0, 0.0, 200.0, 100.0));
        assert_eq!((unit.expansion, unit.expanded, unit.lead_gap), (0.9, Size::new(200.0, 100.0), 7.0));
    }

    #[test]
    fn a_unit_area_starts_at_zero_and_is_measured_in_units() {
        let area = Rect::new(40.0, 10.0, 130.0, 42.5);
        assert_eq!(unit_area(area, 1.25), Rect::new(0.0, 0.0, 72.0, 26.0));
    }
}
