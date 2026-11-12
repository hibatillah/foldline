//! Pointer input on the page badges of the collapsed pill; the page owning the badge under the pointer hears it.

use crate::{gfx::Rect, page::PointerEvent};

/// The page badge the pointer is over and the rect it is drawn into, so that page hears `Left` when the pointer goes.
#[derive(Default)]
pub(super) struct BadgePointer {
    over: Option<(usize, Rect)>,
}

/// One pointer event for the page at `page_index`, measured in the badge `area` it draws into.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct BadgeDelivery {
    pub(super) page_index: usize,
    pub(super) event: PointerEvent,
    pub(super) area: Rect,
}

impl BadgePointer {
    /// What the badges hear for `event`, given each shown page badge as `(page_index, area)`: a `Left` for the badge
    /// the pointer moved off, then the event itself for the badge under the pointer.
    pub(super) fn route(&mut self, event: PointerEvent, badges: &[(usize, Rect)]) -> [Option<BadgeDelivery>; 2] {
        let point = match event {
            PointerEvent::Moved(point) | PointerEvent::Pressed(point) | PointerEvent::Released(point) => point,
            PointerEvent::Left | PointerEvent::CaptureLost => return [self.leave_with(event), None],
            PointerEvent::Scrolled { .. } | PointerEvent::SecondaryReleased(_) => return [None, None],
        };
        let target = badges.iter().copied().find(|(_, area)| area.contains(point));
        let left = match (self.over, target) {
            (Some((was, _)), Some((now, _))) if was == now => None,
            _ => self.leave_with(PointerEvent::Left),
        };
        self.over = target;
        let delivered = target.map(|(page_index, area)| BadgeDelivery { page_index, event, area });
        [left, delivered]
    }

    /// Tells the badge the pointer was over that it left, as the notch opens or the badge stops showing.
    pub(super) fn leave(&mut self) -> Option<BadgeDelivery> {
        self.leave_with(PointerEvent::Left)
    }

    #[cfg(test)]
    pub(super) fn is_over_a_badge(&self) -> bool {
        self.over.is_some()
    }

    fn leave_with(&mut self, event: PointerEvent) -> Option<BadgeDelivery> {
        self.over.take().map(|(page_index, area)| BadgeDelivery { page_index, event, area })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::Point;

    const WEATHER: usize = 7;
    const TIMER: usize = 3;

    fn badges() -> [(usize, Rect); 2] {
        [(TIMER, Rect::new(100.0, 0.0, 140.0, 41.0)), (WEATHER, Rect::new(150.0, 0.0, 230.0, 41.0))]
    }

    fn delivered(deliveries: [Option<BadgeDelivery>; 2]) -> Vec<(usize, PointerEvent)> {
        deliveries.into_iter().flatten().map(|delivery| (delivery.page_index, delivery.event)).collect()
    }

    #[test]
    fn a_press_inside_a_badge_reaches_its_page_in_the_badge_area() {
        let mut pointer = BadgePointer::default();
        let press = PointerEvent::Pressed(Point::new(180.0, 20.0));
        let [_, delivery] = pointer.route(press, &badges());
        assert_eq!(delivery, Some(BadgeDelivery { page_index: WEATHER, event: press, area: badges()[1].1 }));
    }

    #[test]
    fn a_press_outside_every_badge_reaches_no_page() {
        let mut pointer = BadgePointer::default();
        let press = PointerEvent::Pressed(Point::new(40.0, 20.0));
        assert_eq!(delivered(pointer.route(press, &badges())), []);
    }

    #[test]
    fn moving_from_one_badge_to_another_tells_the_first_it_was_left() {
        let mut pointer = BadgePointer::default();
        pointer.route(PointerEvent::Moved(Point::new(120.0, 20.0)), &badges());
        let moved = PointerEvent::Moved(Point::new(200.0, 20.0));
        assert_eq!(delivered(pointer.route(moved, &badges())), [(TIMER, PointerEvent::Left), (WEATHER, moved)]);
    }

    #[test]
    fn moving_within_a_badge_sends_no_leave() {
        let mut pointer = BadgePointer::default();
        pointer.route(PointerEvent::Moved(Point::new(160.0, 20.0)), &badges());
        let moved = PointerEvent::Moved(Point::new(210.0, 20.0));
        assert_eq!(delivered(pointer.route(moved, &badges())), [(WEATHER, moved)]);
    }

    #[test]
    fn leaving_the_pill_tells_the_badge_under_the_pointer_once() {
        let mut pointer = BadgePointer::default();
        pointer.route(PointerEvent::Moved(Point::new(200.0, 20.0)), &badges());
        assert_eq!(delivered(pointer.route(PointerEvent::Left, &badges())), [(WEATHER, PointerEvent::Left)]);
        assert_eq!(pointer.leave(), None);
    }

    #[test]
    fn a_scroll_over_a_badge_reaches_no_page() {
        let mut pointer = BadgePointer::default();
        pointer.route(PointerEvent::Moved(Point::new(200.0, 20.0)), &badges());
        assert_eq!(delivered(pointer.route(PointerEvent::Scrolled { notches: 1.0 }, &badges())), []);
    }
}
