use super::{Notch, SWIPE_END_TIMER_ID};
use windows::Win32::{
    Foundation::{POINT, RECT},
    UI::{
        Input::KeyboardAndMouse::ReleaseCapture,
        WindowsAndMessaging::{GetCursorPos, KillTimer, SetTimer},
    },
};

use crate::host::{chrome, navigation};
use crate::{gfx::Point, log, page::PointerEvent};

impl Notch {
    pub fn pointer_moved(&mut self, window_x: i32, window_y: i32) {
        let point = self.to_surface_space(window_x, window_y);
        if !self.hover.is_hovering {
            log::debug(&format!("enter move window=({window_x},{window_y}) surface=({:.0},{:.0})", point.x, point.y));
        }
        // The shadow remains during closing but must never count as hovering the pill.
        if !self.hover.is_captured && !self.hover.is_dragging_files && !self.pointer_is_over_window() {
            if self.hover.is_hovering {
                self.pointer_left();
                self.flush();
            }
            return;
        }
        self.cancel_close();
        self.set_hovering(true);
        let frame = self.frame();
        let is_over_settings = self.editor.is_none()
            && frame.is_open()
            && chrome::settings_button(frame.bounds, self.scale).contains(point);
        if is_over_settings != self.hover.is_settings_hovered {
            self.hover.is_settings_hovered = is_over_settings;
            self.pending.repaint = true;
        }
        if let Some(menu) = self.menu.as_mut() {
            if menu.moved(point, self.scale) {
                self.pending.repaint = true;
            }
        } else if let Some(editor) = self.editor.as_mut() {
            if editor.moved(point) {
                self.pending.repaint = true;
            }
        } else {
            self.route_pointer(PointerEvent::Moved(point));
        }
        self.flush();
    }

    pub fn pointer_pressed(&mut self, window_x: i32, window_y: i32) {
        let point = self.to_surface_space(window_x, window_y);
        if let Some(menu) = &self.menu {
            match menu.action_at(point, self.scale) {
                Some(action) => self.run_menu_action(action),
                None if menu.contains(point) => {}
                None => self.close_menu(),
            }
            self.flush();
            return;
        }
        if self.editor.is_some() {
            self.press_editor(point);
            self.flush();
            return;
        }
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
        if self.hover.is_hovering && self.editor.is_none() {
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

    /// A right-click anywhere on the open notch opens the menu, except where the page uses it itself (a file on the shelf).
    pub fn right_clicked(&mut self, window_x: i32, window_y: i32) {
        let point = self.to_surface_space(window_x, window_y);
        let frame = self.frame();
        if self.menu.is_some() {
            self.close_menu();
            self.flush();
            return;
        }
        if self.editor.is_some() || !frame.is_open() || !frame.bounds.contains(point) {
            return;
        }
        let in_content = crate::host::chrome::content_rect(frame.bounds, self.scale).contains(point);
        if in_content && self.page_uses_right_click(point) {
            self.route_pointer(PointerEvent::SecondaryReleased(point));
        } else {
            self.open_menu(point);
        }
        self.flush();
    }

    pub fn scrolled(&mut self, amount: i32, is_horizontal: bool, is_touchpad: bool) {
        if !self.hover.is_hovering || self.editor.is_some() {
            return;
        }
        if self.menu.is_some() {
            self.close_menu();
            self.flush();
            return;
        }
        let toward_next = if is_horizontal { amount } else { -amount };
        match navigation::classify_scroll(toward_next, is_touchpad || is_horizontal || self.swipe.is_some()) {
            navigation::Scroll::Pages(steps) => {
                self.end_swipe();
                self.navigate(steps.clamp(-(navigation::MAX_WHEEL_TRAVEL as i32), navigation::MAX_WHEEL_TRAVEL as i32));
            }
            navigation::Scroll::Drag(pages) => self.drag_pages(pages),
        }
        self.flush();
    }

    pub(super) fn drag_pages(&mut self, pages: f32) {
        if self.navigable.len() < 2 {
            return;
        }
        self.wrap_track();
        self.holds_page_position = true;
        let position = self.swipe.get_or_insert_with(|| navigation::Swipe::new(self.track.position)).drag_by(pages);
        // Smooth discrete wheel-message deltas without restarting the visible position or velocity.
        self.track.set_response(navigation::PAGE_DRAG_RESPONSE.0, navigation::PAGE_DRAG_RESPONSE.1);
        self.track.target = position;
        self.arm_swipe_end();
        self.start_animation();
    }

    pub(super) fn arm_swipe_end(&self) {
        if let Some(swipe) = &self.swipe {
            let remaining_milliseconds = swipe.quiet_remaining().as_millis().max(1) as u32;
            unsafe {
                SetTimer(Some(self.window), SWIPE_END_TIMER_ID, remaining_milliseconds, None);
            }
        }
    }

    pub(super) fn end_swipe(&mut self) {
        unsafe {
            let _ = KillTimer(Some(self.window), SWIPE_END_TIMER_ID);
        }
        if let Some(swipe) = self.swipe.take() {
            self.holds_page_position = false;
            self.track.set_response(navigation::TOUCHPAD_SNAP_RESPONSE.0, navigation::TOUCHPAD_SNAP_RESPONSE.1);
            self.track.target = swipe.destination();
            if !self.navigable.is_empty() {
                self.select_page(self.page_at(self.track.target));
            }
            self.start_animation();
        }
    }

    pub fn pointer_released(&mut self, window_x: i32, window_y: i32) {
        let point = self.to_surface_space(window_x, window_y);
        let was_captured = self.hover.is_captured;
        if self.editor.is_some() {
            self.release_editor(point);
            if was_captured {
                self.hover.is_captured = false;
                unsafe {
                    let _ = ReleaseCapture();
                }
            }
        } else {
            self.route_pointer(PointerEvent::Released(point));
        }
        self.flush();
        // A drag can end outside the pill, where no leave message follows.
        if was_captured && !self.frame().bounds.contains(point) {
            self.pointer_left();
            self.flush();
        }
    }

    /// Handles `WM_MOUSELEAVE`; returns false when the leave was not real and tracking must be re-armed.
    pub fn pointer_left_window(&mut self) -> bool {
        if self.hover.is_captured || self.hover.is_dragging_files {
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
        self.hover.is_captured = false;
        self.route_pointer(PointerEvent::CaptureLost);
        self.flush();
    }

    pub(super) fn route_pointer(&mut self, event: PointerEvent) {
        if self.editor.is_some() {
            return;
        }
        let viewport = self.frame();
        if let Some(pages) = self.visible_page_frames(&viewport) {
            for (index, frame, _) in pages {
                self.with_page(index, |page, context| page.on_pointer(event, &frame, context));
            }
            return;
        }
        let Some(index) = self.expanded_index else { return };
        let index = if self.navigable.len() < 2 { index } else { self.page_at(self.track.position) };
        let frame = self.natural_page_frame(index, &viewport, 0.0);
        self.with_page(index, |page, context| page.on_pointer(event, &frame, context));
    }

    pub(super) fn pointer_is_over_window(&self) -> bool {
        let (cursor, bounds) = self.pointer_and_window_bounds();
        cursor.x >= bounds.left && cursor.x < bounds.right && cursor.y >= bounds.top && cursor.y < bounds.bottom
    }

    pub(super) fn pointer_and_window_bounds(&self) -> (POINT, RECT) {
        let mut cursor = POINT::default();
        unsafe {
            let _ = GetCursorPos(&mut cursor);
        }
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
