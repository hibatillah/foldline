use std::time::Instant;

use super::{Notch, Source, badge_pointer::BadgeDelivery, units};
use windows::Win32::{
    Foundation::{POINT, RECT},
    UI::WindowsAndMessaging::GetCursorPos,
};

use crate::host::{chrome, motion::WHEEL_NOTCH};
use crate::{gfx::Point, log, page::PointerEvent};

impl Notch {
    pub fn pointer_moved(&mut self, window_x: i32, window_y: i32) {
        let point = self.to_surface_space(window_x, window_y);
        if !self.hover.is_hovering {
            log::debug(&format!("enter move window=({window_x},{window_y}) surface=({:.0},{:.0})", point.x, point.y));
        }
        // The shadow remains during closing but must never count as hovering the pill.
        if !self.hover.is_dragging_files && !self.pointer_is_over_window() {
            if self.hover.is_hovering {
                self.pointer_left();
                self.flush();
            }
            return;
        }
        self.cancel_close();
        self.set_hovering(true);
        self.route_pointer(PointerEvent::Moved(point));
        self.flush();
    }

    pub fn pointer_pressed(&mut self, window_x: i32, window_y: i32) {
        let point = self.to_surface_space(window_x, window_y);
        if self.press_header(point) {
            self.flush();
            return;
        }
        self.route_pointer(PointerEvent::Pressed(point));
        self.flush();
    }

    pub(super) fn drop_page(&self) -> Option<usize> {
        (0..self.pages.len()).find(|&index| self.page_list.is_enabled(index) && self.pages[index].accepts_drops())
    }

    pub(super) fn show_drop_page(&mut self, index: usize) {
        if self.hover.is_hovering {
            self.go_to_page(index);
        }
    }

    pub fn drag_entered(&mut self) {
        let Some(index) = self.drop_page() else { return };
        self.hover.is_dragging_files = true;
        self.set_hovering(true);
        self.hiding.is_revealed = true;
        self.begin_opening();
        self.with_page(index, |page, context| page.on_file_drag_changed(true, context));
        self.show_drop_page(index);
        self.start_animation();
        self.flush();
    }

    pub fn drag_left(&mut self) {
        self.hover.is_dragging_files = false;
        if let Some(index) = self.drop_page() {
            self.with_page(index, |page, context| page.on_file_drag_changed(false, context));
        }
        if !self.pointer_is_over_window() {
            self.set_hovering(false);
        }
        self.flush();
    }

    pub fn files_dropped(&mut self, paths: Vec<std::path::PathBuf>) {
        self.hover.is_dragging_files = false;
        let Some(index) = self.drop_page() else { return };
        self.with_page(index, |page, context| {
            page.on_file_drag_changed(false, context);
            page.on_drop(paths, context);
        });
        self.show_drop_page(index);
        if !self.pointer_is_over_window() {
            self.set_hovering(false);
        }
        self.flush();
    }

    /// A right-click reaches only a page that uses it itself (a file on the shelf).
    pub fn right_clicked(&mut self, window_x: i32, window_y: i32) {
        let point = self.to_surface_space(window_x, window_y);
        let frame = self.frame();
        if !frame.is_open() || !frame.bounds.contains(point) {
            return;
        }
        let in_content = chrome::content_rect(frame.bounds, self.scale).contains(point);
        if in_content && self.page_uses_right_click(point) {
            self.route_pointer(PointerEvent::SecondaryReleased(point));
        }
        self.flush();
    }

    pub fn scrolled(&mut self, amount: i32, is_horizontal: bool, is_touchpad: bool) {
        self.scrolled_at(Instant::now(), amount, is_horizontal, is_touchpad);
    }

    pub(super) fn scrolled_at(&mut self, now: Instant, amount: i32, is_horizontal: bool, is_touchpad: bool) {
        if !self.hover.is_hovering {
            return;
        }
        if !is_horizontal {
            let (cursor, bounds) = self.pointer_and_window_bounds();
            let point = self.to_surface_space(cursor.x - bounds.left, cursor.y - bounds.top);
            if self.page_claims_scroll(point) {
                let notches = -(amount as f32) / WHEEL_NOTCH as f32;
                self.route_pointer(PointerEvent::Scrolled { notches });
                self.flush();
                return;
            }
        }
        let toward_next = if is_horizontal { amount } else { -amount };
        let is_smooth =
            is_touchpad || is_horizontal || toward_next.abs() < WHEEL_NOTCH || self.scroll_gate.is_in_gesture(now);
        if let Some(step) = self.scroll_gate.feed(now, toward_next, is_smooth) {
            self.navigate(step);
        }
        self.flush();
    }

    pub fn pointer_released(&mut self, window_x: i32, window_y: i32) {
        let point = self.to_surface_space(window_x, window_y);
        self.route_pointer(PointerEvent::Released(point));
        self.flush();
    }

    /// Handles `WM_MOUSELEAVE`; returns false when the leave was not real and tracking must be re-armed.
    pub fn pointer_left_window(&mut self) -> bool {
        if self.hover.is_dragging_files {
            return true;
        }
        // A resize under a stationary pointer can report a leave that did not happen; only believe a real one.
        let over_window = self.pointer_is_over_window();
        log::debug(&format!("mouse leave message, pointer still over window: {over_window}"));
        if over_window {
            return false;
        }
        self.pointer_left();
        self.flush();
        true
    }

    pub fn capture_lost(&mut self) {
        self.route_pointer(PointerEvent::CaptureLost);
        self.flush();
    }

    pub(super) fn route_pointer(&mut self, event: PointerEvent) {
        let viewport = self.frame();
        // A collapsed notch shows no page, so its input belongs to the banner on it, a page badge under it, or nothing.
        if self.springs.expansion.target == 0.0 && !viewport.is_open() {
            if let Some((source, area)) = self.banner_area(&viewport) {
                self.leave_badge();
                let scale = self.scale;
                match source {
                    Source::Activity(index) => self.with_activity(index, |activity, context| {
                        units::activity_pointer(activity, event, area, scale, context)
                    }),
                    Source::Page(index) => self.with_page(index, |page, context| {
                        units::page_presence_pointer(page, event, area, scale, context)
                    }),
                }
            } else {
                // Badges draw only while no message takes over the pill and the pill is not hidden.
                let shows_badges = self.springs.activity_blend.target == 0.0 && !self.is_hidden();
                let badges = if shows_badges { self.page_badge_areas(&viewport) } else { Vec::new() };
                for delivery in self.badge_pointer.route(event, &badges).into_iter().flatten() {
                    self.deliver_to_badge(delivery);
                }
            }
            return;
        }
        self.leave_badge();
        // During a swap only the arriving page takes input, at the place it is moving to.
        let Some(index) = self.expanded_index else { return };
        let frame = self.natural_page_frame(index, &viewport, 0.0);
        self.with_page(index, |page, context| units::page_pointer(page, event, &frame, context));
    }

    fn leave_badge(&mut self) {
        if let Some(delivery) = self.badge_pointer.leave() {
            self.deliver_to_badge(delivery);
        }
    }

    fn deliver_to_badge(&mut self, delivery: BadgeDelivery) {
        let BadgeDelivery { page_index, event, area } = delivery;
        let scale = self.scale;
        self.with_page(page_index, |page, context| units::page_presence_pointer(page, event, area, scale, context));
    }

    pub(super) fn pointer_is_over_window(&self) -> bool {
        let (cursor, bounds) = self.pointer_and_window_bounds();
        cursor.x >= bounds.left && cursor.x < bounds.right && cursor.y >= bounds.top && cursor.y < bounds.bottom
    }

    pub(super) fn pointer_and_window_bounds(&self) -> (POINT, RECT) {
        let mut cursor = POINT::default();
        let _ = unsafe { GetCursorPos(&mut cursor) };
        let region = self.pill_region(&self.frame(), false);
        let bounds = RECT {
            left: self.screen_position.x + region.left,
            top: self.screen_position.y + region.top,
            right: self.screen_position.x + region.right,
            bottom: self.screen_position.y + region.bottom,
        };
        (cursor, bounds)
    }

    /// Mouse coordinates arrive relative to the window, which only covers the pill; pages use surface space.
    pub(super) fn to_surface_space(&self, window_x: i32, window_y: i32) -> Point {
        let (origin_x, origin_y) = self.region_origin.get();
        Point::new((window_x + origin_x) as f32, (window_y + origin_y) as f32)
    }
}
