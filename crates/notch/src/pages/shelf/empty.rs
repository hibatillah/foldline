//! The empty shelf: a badge with a download mark, a title and a line, as one row in the middle of the dashed outline.

use super::GRAY;
use crate::{
    gfx::{Align, Canvas, Color, FontStyle, Point, Rect, UI_FONT, Weight},
    ui::{Stroke, draw_strokes},
};

const BADGE: f32 = 36.0;
const GAP: f32 = 16.0;
const TITLE_HEIGHT: f32 = 18.0;
const DETAIL_HEIGHT: f32 = 16.0;
const LINE_GAP: f32 = 3.0;
const ICON_SIZE: f32 = 20.0;

/// The design asks for 600; only Regular and Medium are bundled.
const TITLE: FontStyle = FontStyle::new(UI_FONT, 15.0, Weight::Medium, Align::Leading);
const DETAIL: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Leading);

const BADGE_FILL: Color = Color::rgb(0.09, 0.09, 0.11);
const BADGE_BORDER: Color = Color::rgb(0.149, 0.149, 0.176);
const DOWNLOAD_ARROW: &str = "M12 4v10M8 10l4 4 4-4";
const DOWNLOAD_TRAY: &str = "M5 15v3a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-3";

pub(super) const TITLE_TEXT: &str = "Drop files here";
pub(super) const DETAIL_TEXT: &str = "They are held for a moment, then you drag them out";

/// The room the row needs: the badge, the gap and the wider line of text, in device-independent pixels.
pub(super) fn row_width(title_width: f32, detail_width: f32) -> f32 {
    BADGE + GAP + title_width.max(detail_width)
}

pub(super) fn draw_empty_row(canvas: &Canvas, bounds: Rect, scale: f32, opacity: f32) {
    let text_width = canvas.measure(&TITLE, TITLE_TEXT).max(canvas.measure(&DETAIL, DETAIL_TEXT));
    let left = bounds.center_x() - row_width(text_width, 0.0) * scale / 2.0;
    let center = Point::new(left + BADGE * scale / 2.0, bounds.center_y());
    let radius = BADGE * scale / 2.0;
    canvas.fill_circle(center, radius, BADGE_BORDER.faded(opacity));
    canvas.fill_circle(center, radius - scale, BADGE_FILL.faded(opacity));
    let layers = [Stroke::new(DOWNLOAD_ARROW, GRAY), Stroke::new(DOWNLOAD_TRAY, GRAY)];
    draw_strokes(canvas, &layers, center, ICON_SIZE * scale, 2.0, opacity);
    let text_left = left + (BADGE + GAP) * scale;
    let top = bounds.center_y() - (TITLE_HEIGHT + LINE_GAP + DETAIL_HEIGHT) * scale / 2.0;
    let title = Rect::new(text_left, top, text_left + text_width * scale + 2.0, top + TITLE_HEIGHT * scale);
    canvas.text(&TITLE, TITLE_TEXT, title, Color::WHITE.faded(opacity));
    let detail_top = title.bottom + LINE_GAP * scale;
    let detail = Rect { top: detail_top, bottom: detail_top + DETAIL_HEIGHT * scale, ..title };
    canvas.text(&DETAIL, DETAIL_TEXT, detail, GRAY.faded(opacity));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::Fonts;

    #[test]
    fn the_row_is_the_badge_the_gap_and_the_wider_line() {
        assert_eq!(row_width(100.0, 250.0), 36.0 + 16.0 + 250.0);
        assert_eq!(row_width(250.0, 100.0), 36.0 + 16.0 + 250.0);
    }

    #[test]
    fn the_row_fits_inside_the_shelf_page() {
        let fonts = Fonts::new(1.0).unwrap();
        let width = row_width(fonts.measure(&TITLE, TITLE_TEXT), fonts.measure(&DETAIL, DETAIL_TEXT));
        assert!(width < super::super::EXPANDED_WIDTH - 2.0 * super::super::ZONE_PADDING);
    }
}
