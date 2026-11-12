//! Pointer input on a page's badge in the collapsed pill.

use std::cell::RefCell;

use super::*;
use crate::{
    gfx::Size,
    page::{Frame, Mode, Page, PageSizes, PointerEvent, Priority},
};

/// A page that shows a badge while collapsed, keeps every pointer event its badge hears and opens on a press.
struct BadgePage {
    heard: Rc<RefCell<Vec<PointerEvent>>>,
}

impl Page for BadgePage {
    fn id(&self) -> &'static str {
        "badge"
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
        Some(Presence::Badge { width: 40.0 })
    }

    fn on_presence_pointer(&mut self, event: PointerEvent, _area: Rect, context: &mut Context) {
        self.heard.borrow_mut().push(event);
        if matches!(event, PointerEvent::Pressed(_)) {
            context.request_open();
        }
    }
}

/// A collapsed, hovered notch whose last page is a `BadgePage`; returns its index and what its badge hears.
fn badge_rig() -> (Notch, usize, Rc<RefCell<Vec<PointerEvent>>>) {
    let heard = Rc::new(RefCell::new(Vec::new()));
    let mut notch = rig(SampleOptions::default());
    notch.pages.push(Box::new(BadgePage { heard: heard.clone() }));
    notch.page_list = PageList::new(notch.pages.len());
    settle(&mut notch);
    notch.open.is_open = true;
    let index = notch.pages.len() - 1;
    (notch, index, heard)
}

fn badge_area(notch: &Notch, index: usize) -> Rect {
    let areas = notch.page_badge_areas(&notch.frame());
    areas.into_iter().find(|&(page_index, _)| page_index == index).expect("the badge shows").1
}

/// A device point `x` and `y` units right of and below the badge's top-left; its page hears `Point::new(x, y)`.
fn inside(area: Rect, x: f32, y: f32) -> Point {
    Point::new(area.left + x * SCALE, area.top + y * SCALE)
}

#[test]
fn a_press_on_a_collapsed_page_badge_reaches_its_page_and_opens_the_notch_on_it() {
    let (mut notch, index, heard) = badge_rig();
    let area = badge_area(&notch, index);
    notch.route_pointer(PointerEvent::Pressed(inside(area, 6.0, 4.0)));
    notch.flush();
    assert_eq!(*heard.borrow(), [PointerEvent::Pressed(Point::new(6.0, 4.0))]);
    assert_eq!(notch.springs.expansion.target, 1.0);
    assert_eq!(notch.selected_page, Some(index));
}

#[test]
fn a_press_on_the_collapsed_pill_beside_a_page_badge_reaches_no_page_and_keeps_it_closed() {
    let (mut notch, index, heard) = badge_rig();
    let area = badge_area(&notch, index);
    notch.route_pointer(PointerEvent::Pressed(Point::new(notch.frame().bounds.left + 2.0, area.center_y())));
    notch.flush();
    assert!(heard.borrow().is_empty());
    assert_eq!(notch.springs.expansion.target, 0.0);
}

#[test]
fn moving_off_a_collapsed_page_badge_tells_its_page_the_pointer_left() {
    let (mut notch, index, heard) = badge_rig();
    let area = badge_area(&notch, index);
    notch.route_pointer(PointerEvent::Moved(inside(area, 6.0, 4.0)));
    notch.route_pointer(PointerEvent::Moved(Point::new(notch.frame().bounds.left + 2.0, area.center_y())));
    assert_eq!(*heard.borrow(), [PointerEvent::Moved(Point::new(6.0, 4.0)), PointerEvent::Left]);
}
