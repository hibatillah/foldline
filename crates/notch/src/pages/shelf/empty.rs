//! The empty shelf: a badge with a download mark, a title and a line, as one row in the middle of the dashed outline.

use crate::i18n::tr;
use crate::ui::palette::TEXT_MUTED;
use crate::ui::palette::{SURFACE_BADGE, SURFACE_BORDER};
use crate::ui::type_styles::{MEDIUM_15, REGULAR_12};
use crate::{
    gfx::{Canvas, Color, Point, Rect},
    ui::{Stroke, draw_strokes},
};

const BADGE: f32 = 36.0;
const GAP: f32 = 16.0;
const TITLE_HEIGHT: f32 = 18.0;
const DETAIL_HEIGHT: f32 = 16.0;
const LINE_GAP: f32 = 3.0;
const ICON_SIZE: f32 = 20.0;

const DOWNLOAD_ARROW: &str = "M12 4v10M8 10l4 4 4-4";
const DOWNLOAD_TRAY: &str = "M5 15v3a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-3";

pub(super) fn title_text() -> std::borrow::Cow<'static, str> {
    tr!("shelf.drop_title")
}

pub(super) fn detail_text() -> std::borrow::Cow<'static, str> {
    tr!("shelf.drop_detail")
}

/// The room the row needs: the badge, the gap and the wider line of text, in device-independent pixels.
pub(super) fn row_width(title_width: f32, detail_width: f32) -> f32 {
    BADGE + GAP + title_width.max(detail_width)
}

pub(super) fn draw_empty_row(canvas: &Canvas, bounds: Rect, opacity: f32) {
    let text_width = canvas.measure(&MEDIUM_15, &title_text()).max(canvas.measure(&REGULAR_12, &detail_text()));
    let left = bounds.center_x() - row_width(text_width, 0.0) / 2.0;
    let center = Point::new(left + BADGE / 2.0, bounds.center_y());
    let radius = BADGE / 2.0;
    canvas.fill_circle(center, radius, SURFACE_BORDER.faded(opacity));
    canvas.fill_circle(center, radius - 1.0, SURFACE_BADGE.faded(opacity));
    let layers = [Stroke::new(DOWNLOAD_ARROW, TEXT_MUTED), Stroke::new(DOWNLOAD_TRAY, TEXT_MUTED)];
    draw_strokes(canvas, &layers, center, ICON_SIZE, 2.0, opacity);
    let text_left = left + BADGE + GAP;
    let top = bounds.center_y() - (TITLE_HEIGHT + LINE_GAP + DETAIL_HEIGHT) / 2.0;
    let title = Rect::new(text_left, top, text_left + text_width + 2.0, top + TITLE_HEIGHT);
    canvas.text(&MEDIUM_15, &title_text(), title, Color::WHITE.faded(opacity));
    let detail_top = title.bottom + LINE_GAP;
    let detail = Rect { top: detail_top, bottom: detail_top + DETAIL_HEIGHT, ..title };
    canvas.text(&REGULAR_12, &detail_text(), detail, TEXT_MUTED.faded(opacity));
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
        let width = row_width(fonts.measure(&MEDIUM_15, &title_text()), fonts.measure(&REGULAR_12, &detail_text()));
        assert!(width < super::super::EXPANDED_WIDTH - 2.0 * super::super::ZONE_PADDING);
    }
}
