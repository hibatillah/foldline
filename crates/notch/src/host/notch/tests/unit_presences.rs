//! Banners, badges and header marks of activities and pages are drawn and hear the pointer in units of their own area.

use std::cell::{Cell, RefCell};

use super::*;
use crate::{
    gfx::{Size, test_support::render},
    host::notch::units,
    page::{Frame, Mode, PageSizes, PointerEvent, Priority},
};

/// The area and the canvas scale one draw call received.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Drawn {
    area: Rect,
    canvas_scale: f32,
}

impl Drawn {
    fn of(canvas: &Canvas, area: Rect) -> Self {
        Self { area, canvas_scale: canvas.scale() }
    }
}

#[derive(Default)]
struct Heard {
    drawn: Cell<Option<Drawn>>,
    short_drawn: Cell<Option<Drawn>>,
    header_drawn: Cell<Option<Drawn>>,
    pointer: RefCell<Vec<(PointerEvent, Rect)>>,
}

/// An activity that shows a banner and a header mark and records what the host hands it.
struct BannerRecorder {
    heard: Rc<Heard>,
}

impl Activity for BannerRecorder {
    fn id(&self) -> &'static str {
        "banner_recorder"
    }

    fn presence(&self) -> Option<Presence> {
        Some(Presence::Banner { width: 200.0, height: 60.0 })
    }

    fn draw(&self, canvas: &Canvas, area: Rect, _opacity: f32) {
        self.heard.drawn.set(Some(Drawn::of(canvas, area)));
    }

    fn header_status_width(&self) -> Option<f32> {
        Some(16.0)
    }

    fn draw_header_status(&self, canvas: &Canvas, area: Rect, _opacity: f32) {
        self.heard.header_drawn.set(Some(Drawn::of(canvas, area)));
    }

    fn on_pointer(&mut self, event: PointerEvent, area: Rect, _context: &mut Context) {
        self.heard.pointer.borrow_mut().push((event, area));
    }
}

/// A page whose pill presence a test picks and that records its presence calls.
struct PresenceRecorder {
    presence: Presence,
    heard: Rc<Heard>,
}

impl Page for PresenceRecorder {
    fn id(&self) -> &'static str {
        "presence_recorder"
    }

    fn priority(&self, mode: Mode) -> Option<Priority> {
        matches!(mode, Mode::Expanded).then_some(Priority::BACKGROUND)
    }

    fn sizes(&self) -> PageSizes {
        PageSizes { resting_width: 0.0, expanded: Size::new(200.0, 86.0) }
    }

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn presence(&self) -> Option<Presence> {
        Some(self.presence)
    }

    fn draw_presence(&self, canvas: &Canvas, area: Rect, _opacity: f32) {
        self.heard.drawn.set(Some(Drawn::of(canvas, area)));
    }

    fn short_presence_width(&self) -> Option<f32> {
        Some(20.0)
    }

    fn draw_short_presence(&self, canvas: &Canvas, area: Rect, _opacity: f32) {
        self.heard.short_drawn.set(Some(Drawn::of(canvas, area)));
    }

    fn draw_header_status(&self, canvas: &Canvas, area: Rect, _opacity: f32) {
        self.heard.header_drawn.set(Some(Drawn::of(canvas, area)));
    }

    fn on_presence_pointer(&mut self, event: PointerEvent, area: Rect, _context: &mut Context) {
        self.heard.pointer.borrow_mut().push((event, area));
    }
}

/// A collapsed notch whose only activity is a started `BannerRecorder`.
fn banner_activity_rig() -> (Notch, Rc<Heard>) {
    let heard = Rc::new(Heard::default());
    let activity = BannerRecorder { heard: heard.clone() };
    let mut notch = configured_rig(SampleOptions::default(), Settings::at(None), vec![Box::new(activity)]);
    for slot in &mut notch.activities {
        slot.is_started = true;
    }
    settle(&mut notch);
    (notch, heard)
}

/// A collapsed notch whose only page is a `PresenceRecorder` showing `presence`.
fn presence_page_rig(presence: Presence) -> (Notch, Rc<Heard>) {
    let heard = Rc::new(Heard::default());
    let page = PresenceRecorder { presence, heard: heard.clone() };
    let mut notch = custom_rig(vec![Box::new(page)], Settings::at(None), Vec::new());
    settle(&mut notch);
    (notch, heard)
}

fn banner_area(notch: &Notch) -> Rect {
    notch.banner_area(&notch.frame()).expect("the banner shows").1
}

fn unit_size_of(area: Rect) -> Rect {
    Rect::new(0.0, 0.0, area.width() / SCALE, area.height() / SCALE)
}

/// A device point `x` and `y` units right of and below the area's top-left.
fn inside(area: Rect, x: f32, y: f32) -> Point {
    Point::new(area.left + x * SCALE, area.top + y * SCALE)
}

#[test]
fn an_activity_banner_hears_the_pointer_in_units_from_the_banner_top_left() {
    let (mut notch, heard) = banner_activity_rig();
    let area = banner_area(&notch);
    assert_ne!(area.left, 0.0, "the banner sits away from the window edge, so its own origin matters");
    notch.route_pointer(PointerEvent::Pressed(inside(area, 12.0, 8.0)));
    let expected = (PointerEvent::Pressed(Point::new(12.0, 8.0)), unit_size_of(area));
    assert_eq!(*heard.pointer.borrow(), [expected]);
}

#[test]
fn an_activity_banner_draws_into_a_unit_area_on_a_canvas_at_scale_one() {
    let (notch, heard) = banner_activity_rig();
    let area = banner_area(&notch);
    notch.paint();
    let drawn = heard.drawn.get().expect("the banner drew");
    assert_eq!(drawn, Drawn { area: unit_size_of(area), canvas_scale: 1.0 });
    assert_eq!(drawn.area.width(), 200.0);
}

#[test]
fn an_activity_header_mark_draws_into_a_unit_area_on_a_canvas_at_scale_one() {
    let (notch, heard) = banner_activity_rig();
    let device_area = Rect::new(40.0, 4.0, 40.0 + 16.0 * SCALE, 4.0 + 32.0 * SCALE);
    let activity = notch.activities[0].activity.as_ref();
    render(SCALE, |canvas| units::draw_activity_header_status(activity, canvas, device_area, 1.0));
    let expected = Drawn { area: Rect::new(0.0, 0.0, 16.0, 32.0), canvas_scale: 1.0 };
    assert_eq!(heard.header_drawn.get(), Some(expected));
}

#[test]
fn a_page_banner_hears_the_pointer_and_draws_in_units_of_the_banner() {
    let (mut notch, heard) = presence_page_rig(Presence::Banner { width: 220.0, height: 50.0 });
    let area = banner_area(&notch);
    notch.route_pointer(PointerEvent::Moved(inside(area, 30.0, 10.0)));
    let expected = (PointerEvent::Moved(Point::new(30.0, 10.0)), unit_size_of(area));
    assert_eq!(*heard.pointer.borrow(), [expected]);
    notch.paint();
    assert_eq!(heard.drawn.get(), Some(Drawn { area: unit_size_of(area), canvas_scale: 1.0 }));
}

#[test]
fn a_page_badge_hears_the_pointer_in_units_from_the_badge_top_left() {
    let (mut notch, heard) = presence_page_rig(Presence::Badge { width: 40.0 });
    notch.open.is_open = true;
    let (_, area) = notch.page_badge_areas(&notch.frame()).into_iter().next().expect("the badge shows");
    notch.route_pointer(PointerEvent::Moved(inside(area, 6.0, 4.0)));
    let heard_events = heard.pointer.borrow();
    let (event, unit_area) = heard_events.first().copied().expect("the badge heard the move");
    assert_eq!((event, unit_area), (PointerEvent::Moved(Point::new(6.0, 4.0)), unit_size_of(area)));
}

#[test]
fn a_page_short_badge_and_header_mark_draw_into_unit_areas_on_a_canvas_at_scale_one() {
    let (notch, heard) = presence_page_rig(Presence::Badge { width: 40.0 });
    let page = notch.pages[0].as_ref();
    let short_area = Rect::new(60.0, 6.0, 60.0 + 20.0 * SCALE, 6.0 + 24.0 * SCALE);
    let header_area = Rect::new(90.0, 2.0, 90.0 + 14.0 * SCALE, 2.0 + 32.0 * SCALE);
    render(SCALE, |canvas| {
        units::draw_page_short_presence(page, canvas, short_area, 1.0);
        units::draw_page_header_status(page, canvas, header_area, 1.0);
    });
    let unit = |width, height| Some(Drawn { area: Rect::new(0.0, 0.0, width, height), canvas_scale: 1.0 });
    assert_eq!(heard.short_drawn.get(), unit(20.0, 24.0));
    assert_eq!(heard.header_drawn.get(), unit(14.0, 32.0));
}
