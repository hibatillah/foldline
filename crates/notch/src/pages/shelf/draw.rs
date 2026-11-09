use super::{
    ACCENT_FILL_ALPHA, BORDER, BORDER_WIDTH, DASH, DASH_GAP, FULL_BORDER, FULL_FILL, FULL_FILL_ALPHA, NAME_COLOR,
    NAME_LINE_HEIGHT, NAME_TOP, ShelfPage, Target, ZONE_RADIUS, fitted, preview_rect, remove_center, shown_count,
    slot_rect,
};
use super::{document::draw_document, empty::draw_empty_row};
use crate::ui::palette::{ACCENT_BLUE, ACCENT_BLUE_LIGHT, SOFT_RED, TEXT_MUTED};
use crate::ui::palette::{LIGHT_GRAY, REMOVE_BUTTON_FILL};
use crate::ui::type_styles::{REGULAR_10_CENTER, REGULAR_12_CENTER};
use crate::{
    gfx::{Canvas, Color, Point, Rect},
    page::Frame,
    ui::{Icon, Stroke, draw_dashed_outline, draw_strokes},
};
use std::path::Path;

pub(super) const PREVIEW_RADIUS: f32 = 6.0;
pub(super) const WASH_REACH: f32 = 3.0;
pub(super) const REMOVE_RADIUS: f32 = 8.0;
pub(super) const REMOVE_RING: f32 = 2.0;
const REMOVE_CROSS_SIZE: f32 = 8.0;
const REMOVE_CROSS_WIDTH: f32 = 3.5;
const REMOVE_CROSS: &str = "M6 6l12 12M18 6 6 18";
const REMOVE_FILL_PRESSED: Color = Color::rgb(0.3, 0.3, 0.34);
pub(super) const ARRIVAL_SCALE: f32 = 0.7;
pub(super) const ARRIVAL_FALL: f32 = 12.0;
pub(super) const WASH: Color = Color::rgb(1.0, 1.0, 1.0);

impl ShelfPage {
    pub(super) fn draw_preview(&self, canvas: &Canvas, path: &Path, area: Rect, opacity: f32) {
        let scale = canvas.scale();
        if let Some(preview) = self.previews.borrow_mut().get_mut(path) {
            if preview.brush.is_none()
                && let Some(image) = preview.waiting.take()
            {
                preview.brush = canvas.upload(&image);
            }
            if let Some(brush) = &preview.brush {
                canvas.image(brush, fitted(preview.size, area), PREVIEW_RADIUS * scale, opacity);
                return;
            }
        }
        if path.is_dir() {
            canvas.fill_rounded_rect(area, PREVIEW_RADIUS * scale, Color::gray(0.16).faded(opacity));
            Icon::Folder.draw(canvas, Point::new(area.center_x(), area.center_y()), 22.0 * scale, TEXT_MUTED, opacity);
        } else {
            draw_document(canvas, path, area, opacity);
        }
    }

    pub(super) fn draw_remove_button(&self, canvas: &Canvas, preview: Rect, is_pressed_target: bool, opacity: f32) {
        let scale = canvas.scale();
        let center = remove_center(preview, scale);
        let fill = if is_pressed_target { REMOVE_FILL_PRESSED } else { REMOVE_BUTTON_FILL };
        canvas.fill_circle(center, (REMOVE_RADIUS + REMOVE_RING) * scale, Color::BLACK.faded(opacity));
        canvas.fill_circle(center, REMOVE_RADIUS * scale, fill.faded(opacity));
        let cross = [Stroke::new(REMOVE_CROSS, LIGHT_GRAY)];
        draw_strokes(canvas, &cross, center, REMOVE_CROSS_SIZE * scale, REMOVE_CROSS_WIDTH, opacity);
    }

    pub(super) fn draw_slot(&self, canvas: &Canvas, position: usize, bounds: Rect, opacity: f32) {
        let scale = canvas.scale();
        let column = slot_rect(position, self.shelf.len(), bounds, scale);
        let arrival = self.arrivals.get(&self.shelf.items()[position]).map(|spring| spring.position);
        let Some(progress) = arrival else { return self.draw_column(canvas, position, column, opacity) };
        let shown = progress.max(0.0);
        let rise = (1.0 - shown) * ARRIVAL_FALL * scale;
        let column = Rect::new(column.left, column.top - rise, column.right, column.bottom - rise);
        let factor = ARRIVAL_SCALE + (1.0 - ARRIVAL_SCALE) * shown;
        let center = Point::new(column.center_x(), column.center_y());
        let faded = opacity * progress.clamp(0.0, 1.0);
        canvas.scaled_about(center, factor, || self.draw_column(canvas, position, column, faded));
    }

    pub(super) fn draw_column(&self, canvas: &Canvas, position: usize, column: Rect, opacity: f32) {
        let scale = canvas.scale();
        let is_hovered = matches!(self.hovered, Some(Target::Slot(at) | Target::Remove(at)) if at == position);
        if is_hovered {
            let wash = Rect::new(
                column.left,
                column.top - WASH_REACH * scale,
                column.right,
                column.bottom + WASH_REACH * scale,
            );
            canvas.fill_rounded_rect(wash, 8.0 * scale, WASH.with_alpha(0.07 * opacity));
        }
        let preview = preview_rect(column, scale);
        let name_top = preview.bottom + NAME_TOP * scale;
        let name_color = if is_hovered { Color::WHITE } else { NAME_COLOR };
        let path = &self.shelf.items()[position];
        let faded = if self.dragging.as_ref() == Some(path) { opacity * 0.35 } else { opacity };
        self.draw_preview(canvas, path, preview, faded);
        for (line, text) in self.names.get(position).into_iter().flatten().enumerate() {
            let top = name_top + line as f32 * NAME_LINE_HEIGHT * scale;
            let rect = Rect::new(column.left, top, column.right, top + NAME_LINE_HEIGHT * scale);
            canvas.text(&REGULAR_10_CENTER, text, rect, name_color.faded(faded));
        }
        if is_hovered {
            self.draw_remove_button(canvas, preview, self.hovered == Some(Target::Remove(position)), opacity);
        }
    }
}

impl ShelfPage {
    pub(super) fn draw_zone(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let (bounds, scale) = (frame.bounds, frame.scale);
        let is_refusing = self.is_drag_over && self.shelf.is_full();
        if self.is_drag_over {
            let (fill, alpha) =
                if is_refusing { (FULL_FILL, FULL_FILL_ALPHA) } else { (ACCENT_BLUE, ACCENT_FILL_ALPHA) };
            canvas.fill_rounded_rect(bounds, ZONE_RADIUS * scale, fill.with_alpha(alpha * opacity));
        }
        let border = match (self.is_drag_over, is_refusing) {
            (false, _) => BORDER,
            (true, false) => ACCENT_BLUE,
            (true, true) => FULL_BORDER,
        };
        draw_dashed_outline(
            canvas,
            bounds,
            ZONE_RADIUS * scale,
            BORDER_WIDTH * scale,
            DASH * scale,
            DASH_GAP * scale,
            border.faded(opacity),
        );
        let message_area =
            Rect::new(bounds.left, bounds.center_y() - 8.0 * scale, bounds.right, bounds.center_y() + 8.0 * scale);
        if is_refusing {
            canvas.text(
                &REGULAR_12_CENTER,
                "The shelf is full. Remove a file to add more.",
                message_area,
                SOFT_RED.faded(opacity),
            );
            return;
        }
        if self.shelf.len() == 0 {
            if self.is_drag_over {
                canvas.text(&REGULAR_12_CENTER, "Release to hold them", message_area, ACCENT_BLUE_LIGHT.faded(opacity));
            } else {
                draw_empty_row(canvas, bounds, scale, opacity);
            }
            return;
        }
        for position in 0..shown_count(self.shelf.len()) {
            self.draw_slot(canvas, position, bounds, opacity);
        }
    }
}
