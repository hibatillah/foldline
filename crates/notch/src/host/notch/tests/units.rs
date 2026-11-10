//! A page that answers `uses_units` draws and hears the pointer in device-independent units.

use std::cell::{Cell, RefCell};

use super::*;
use crate::{
    gfx::Size,
    page::{Frame, Mode, Page, PageSizes, PointerEvent, Priority},
};

#[derive(Default)]
struct Heard {
    drawn_frame: Cell<Option<Frame>>,
    canvas_scale: Cell<Option<f32>>,
    pointer: RefCell<Vec<PointerEvent>>,
    scroll_point: Cell<Option<Point>>,
    right_click_point: Cell<Option<Point>>,
}

/// A page that records what the host hands it, in units or in device pixels as `in_units` says.
struct Recorder {
    in_units: bool,
    heard: Rc<Heard>,
}

impl Page for Recorder {
    fn id(&self) -> &'static str {
        "recorder"
    }

    fn priority(&self, mode: Mode) -> Option<Priority> {
        matches!(mode, Mode::Expanded).then_some(Priority::BACKGROUND)
    }

    fn sizes(&self) -> PageSizes {
        PageSizes { resting_width: 0.0, expanded: Size::new(200.0, 86.0) }
    }

    fn uses_units(&self) -> bool {
        self.in_units
    }

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, _opacity: f32) {
        self.heard.drawn_frame.set(Some(*frame));
        self.heard.canvas_scale.set(Some(canvas.scale()));
    }

    fn on_pointer(&mut self, event: PointerEvent, _frame: &Frame, _context: &mut Context) {
        self.heard.pointer.borrow_mut().push(event);
    }

    fn claims_scroll(&self, point: Point, _frame: &Frame) -> bool {
        self.heard.scroll_point.set(Some(point));
        true
    }

    fn uses_right_click(&self, point: Point, _frame: &Frame) -> bool {
        self.heard.right_click_point.set(Some(point));
        true
    }
}

fn opened_recorder(in_units: bool) -> (Notch, Rc<Heard>) {
    let heard = Rc::new(Heard::default());
    let mut notch =
        custom_rig(vec![Box::new(Recorder { in_units, heard: heard.clone() })], Settings::at(None), Vec::new());
    open(&mut notch, "recorder");
    (notch, heard)
}

fn page_center(notch: &Notch) -> (Frame, Point) {
    let frame = notch.natural_page_frame(0, &notch.frame(), 0.0);
    (frame, Point::new(frame.bounds.center_x(), frame.bounds.center_y()))
}

#[test]
fn a_page_that_uses_units_draws_with_a_unit_frame_and_a_canvas_at_scale_one() {
    let (notch, heard) = opened_recorder(true);
    notch.paint();
    let frame = heard.drawn_frame.get().expect("the page drew");
    assert_eq!((frame.scale, heard.canvas_scale.get()), (1.0, Some(1.0)));
    assert_eq!(frame.bounds, Rect::new(0.0, 0.0, 200.0, 86.0));
}

#[test]
fn a_page_that_does_not_use_units_still_draws_in_device_pixels() {
    let (notch, heard) = opened_recorder(false);
    notch.paint();
    let frame = heard.drawn_frame.get().expect("the page drew");
    assert_eq!((frame.scale, heard.canvas_scale.get()), (SCALE, Some(SCALE)));
    assert_eq!(frame.bounds.width(), 200.0 * SCALE);
}

#[test]
fn a_page_that_uses_units_hears_the_pointer_in_units_from_its_top_left() {
    let (mut notch, heard) = opened_recorder(true);
    let (frame, center) = page_center(&notch);
    notch.route_pointer(PointerEvent::Moved(center));
    let expected = Point::new(frame.bounds.width() / SCALE / 2.0, frame.bounds.height() / SCALE / 2.0);
    assert_eq!(*heard.pointer.borrow(), [PointerEvent::Moved(expected)]);
}

#[test]
fn a_page_that_does_not_use_units_hears_the_pointer_in_device_pixels() {
    let (mut notch, heard) = opened_recorder(false);
    let (_, center) = page_center(&notch);
    notch.route_pointer(PointerEvent::Moved(center));
    assert_eq!(*heard.pointer.borrow(), [PointerEvent::Moved(center)]);
}

#[test]
fn scroll_and_right_click_questions_reach_a_unit_page_in_units() {
    let (notch, heard) = opened_recorder(true);
    let (frame, center) = page_center(&notch);
    let expected = Point::new(frame.bounds.width() / SCALE / 2.0, frame.bounds.height() / SCALE / 2.0);
    assert!(notch.page_claims_scroll(center));
    assert!(notch.page_uses_right_click(center));
    assert_eq!((heard.scroll_point.get(), heard.right_click_point.get()), (Some(expected), Some(expected)));
}
