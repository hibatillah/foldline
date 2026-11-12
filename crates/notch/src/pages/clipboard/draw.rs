use std::time::Instant;

use super::{
    ClipboardPage,
    item::{Content, Item, Kind, StoredImage, age_text, details, parse_color, row_label},
    layout::{Layout, Target},
    preview::preview_lines,
    state::{Bar, Confirm, Preview},
    style::*,
};
use crate::i18n::tr;
use crate::{
    gfx::{Canvas, Color, FontStyle, ImageData, Point, Rect},
    ui::{Icon, Stroke, draw_strokes},
};

const ROW_RADIUS: f32 = 8.0;
const ROW_PADDING: f32 = 8.0;
const ROW_ICON_SIZE: f32 = 14.0;
const PIN_ICON_SIZE: f32 = 13.0;
const ROW_BADGE_SIZE: f32 = 15.0;
const ROW_GAP: f32 = 7.0;
const PANEL_PADDING_X: f32 = 10.0;
const PANEL_PADDING_Y: f32 = 8.0;
const PREVIEW_LINES: usize = 7;
const PREVIEW_LINE_HEIGHT: f32 = 17.0;
const SWATCH_WIDTH: f32 = 96.0;
const SWATCH_HEIGHT: f32 = 56.0;
const DETAIL_LINE_HEIGHT: f32 = 12.0;
const BUTTON_ICON_SIZE: f32 = 13.0;
const BUTTON_ICON_GAP: f32 = 5.0;
const EMPTY_BADGE_SIZE: f32 = 36.0;
const EMPTY_GAP: f32 = 16.0;
const BORDER: f32 = 1.0;

/// A border drawn as a slightly larger rounded rectangle under the fill.
fn bordered(canvas: &Canvas, rect: Rect, radius: f32, fill: Color, border: Color, opacity: f32) {
    canvas.fill_rounded_rect(rect, radius, border.faded(opacity));
    let inner = Rect::new(rect.left + BORDER, rect.top + BORDER, rect.right - BORDER, rect.bottom - BORDER);
    canvas.fill_rounded_rect(inner, radius - BORDER, fill.faded(opacity));
}

fn stroke_icon(
    canvas: &Canvas,
    path: &'static str,
    center: Point,
    size: f32,
    line_width: f32,
    color: Color,
    opacity: f32,
) {
    draw_strokes(canvas, &[Stroke::new(path, color)], center, size, line_width, opacity);
}

fn kind_icon(kind: Kind) -> &'static str {
    match kind {
        Kind::Link => LINK_ICON,
        Kind::Code => CODE_ICON,
        Kind::Text | Kind::Color | Kind::Image => TEXT_ICON,
    }
}

/// A small picture stand-in for the list: the design's orange gradient with a dark notch at the top.
fn image_badge(side: u32) -> ImageData {
    let side_f = side as f32;
    let last = (side_f - 1.0).max(1.0);
    let mut bytes = Vec::with_capacity((side * side * 4) as usize);
    for y in 0..side {
        for x in 0..side {
            let is_notch = (y as f32) < side_f * 4.0 / 15.0
                && (x as f32) >= side_f * 4.0 / 15.0
                && (x as f32) < side_f * 11.0 / 15.0;
            let amount = (x + y) as f32 / (2.0 * last);
            let channel = |from: f32, to: f32| if is_notch { 0 } else { (from + (to - from) * amount).round() as u8 };
            bytes.extend_from_slice(&[channel(127.0, 44.0), channel(161.0, 106.0), channel(233.0, 238.0), 255]);
        }
    }
    ImageData::new(side, side, bytes)
}

impl ClipboardPage {
    pub(super) fn draw_page(&self, canvas: &Canvas, bounds: Rect, opacity: f32) {
        let layout = Layout::new(bounds);
        let now = Instant::now();
        if self.history.is_empty() {
            draw_empty(canvas, bounds, opacity);
        } else {
            let selected = self.history.selected_index();
            for (index, item) in self.history.items().iter().enumerate() {
                let is_selected = index == selected;
                let is_pin_hovered = self.hover.is(Target::Pin(index));
                let is_hovered = self.hover.is(Target::Row(index)) || is_pin_hovered;
                let pin = match (self.history.is_pinned(index), is_hovered && self.history.has_pin_button(index)) {
                    (true, _) => Some(PINNED),
                    (false, true) if is_pin_hovered => Some(PIN_OFFERED_HOVER),
                    (false, true) => Some(PIN_OFFERED),
                    (false, false) => None,
                };
                let row = Row {
                    is_selected,
                    is_hovered,
                    confirm: if is_selected { self.confirm } else { None },
                    pin: pin.map(|color| (layout.pin_area(index), color)),
                };
                draw_row(canvas, layout.row(index), item, row, opacity);
            }
            if let Some(item) = self.history.selected() {
                self.draw_preview(canvas, &layout, item, opacity);
                self.draw_actions(canvas, &layout, item, now, opacity);
            }
        }
        if let Some(bar) = self.bar {
            draw_bar(canvas, layout.bar(), bar, opacity);
        }
    }

    fn draw_preview(&self, canvas: &Canvas, layout: &Layout, item: &Item, opacity: f32) {
        let panel = layout.panel();
        canvas.fill_rounded_rect(panel, ROW_RADIUS, PANEL.faded(opacity));
        let inner = Rect::new(
            panel.left + PANEL_PADDING_X,
            panel.top + PANEL_PADDING_Y,
            panel.right - PANEL_PADDING_X,
            panel.bottom - PANEL_PADDING_Y,
        );
        match &item.content {
            Content::Image(image) => self.draw_picture(canvas, inner, item.id, image, opacity),
            Content::Text(text) if item.kind == Kind::Color => {
                let Some(color) = parse_color(text.trim()) else { return };
                let swatch = Rect::new(inner.left, inner.top, inner.left + SWATCH_WIDTH, inner.top + SWATCH_HEIGHT);
                bordered(canvas, swatch, ROW_RADIUS, color, BADGE_BORDER, opacity);
                let label = Rect::new(inner.left, swatch.bottom + 8.0, inner.right, swatch.bottom + 25.0);
                canvas.text(&PREVIEW_CODE, text.trim(), label, TEXT.faded(opacity));
            }
            Content::Text(text) => {
                let style = if item.kind == Kind::Code { PREVIEW_CODE } else { PREVIEW_TEXT };
                let lines = preview_lines(text, PREVIEW_LINES, inner.width(), |line| canvas.measure(&style, line));
                for (index, line) in lines.iter().enumerate() {
                    let top = inner.top + index as f32 * PREVIEW_LINE_HEIGHT;
                    let area = Rect::new(inner.left, top, inner.right, top + PREVIEW_LINE_HEIGHT);
                    canvas.text(&style, line, area, TEXT.faded(opacity));
                }
            }
        }
    }

    /// The picture fits inside the panel without cropping, centered, on whole device pixels.
    fn draw_picture(&self, canvas: &Canvas, inner: Rect, item_id: u64, image: &StoredImage, opacity: f32) {
        let raster_scale = canvas.raster_scale();
        let (width, height) = (image.pixels.width as f32, image.pixels.height as f32);
        let fit = (inner.width() * raster_scale / width).min(inner.height() * raster_scale / height).min(raster_scale);
        let (shown_width, shown_height) = ((width * fit).floor().max(1.0), (height * fit).floor().max(1.0));
        let mut preview = self.preview.borrow_mut();
        if preview.item_id != item_id || preview.width != shown_width as u32 {
            let uploaded = image.pixels.downscaled(shown_width as u32);
            *preview = Preview { item_id, width: shown_width as u32, brush: canvas.upload(&uploaded) };
        }
        let Some(brush) = preview.brush.as_ref() else { return };
        let origin = canvas.origin();
        let device_left = (origin.x + inner.center_x() * raster_scale - shown_width / 2.0).round();
        let device_top = (origin.y + inner.center_y() * raster_scale - shown_height / 2.0).round();
        let left = (device_left - origin.x) / raster_scale;
        let top = (device_top - origin.y) / raster_scale;
        let rect = Rect::new(left, top, left + shown_width / raster_scale, top + shown_height / raster_scale);
        canvas.image(brush, rect, 4.0, opacity);
    }

    fn draw_actions(&self, canvas: &Canvas, layout: &Layout, item: &Item, now: Instant, opacity: f32) {
        let details_area = layout.details();
        let first =
            Rect { top: details_area.top + 1.0, bottom: details_area.top + (1.0 + DETAIL_LINE_HEIGHT), ..details_area };
        let second = Rect { top: first.bottom, bottom: first.bottom + DETAIL_LINE_HEIGHT, ..details_area };
        canvas.text(&DETAIL, &details(item), first, DETAIL_GRAY.faded(opacity));
        let age = age_text(now.saturating_duration_since(item.copied_at));
        canvas.text(&DETAIL, &age, second, DETAIL_GRAY.faded(opacity));

        let hovered = |target| self.hover.is(target);
        let center = |rect: Rect| Point::new(rect.center_x(), rect.center_y());

        let delete = layout.delete_button();
        let border = if hovered(Target::Delete) { BUTTON_BORDER_HOVER } else { BUTTON_BORDER };
        bordered(canvas, delete, ROW_RADIUS, BUTTON_FILL, border, opacity);
        stroke_icon(canvas, TRASH_ICON, center(delete), BUTTON_ICON_SIZE, 1.8, TRASH, opacity);

        let copy = layout.copy_button();
        if self.confirm == Some(Confirm::Copied) {
            bordered(canvas, copy, ROW_RADIUS, GREEN_FILL, GREEN, opacity);
            stroke_icon(canvas, CHECK_ICON, center(copy), BUTTON_ICON_SIZE, 1.8, GREEN, opacity);
        } else {
            let border = if hovered(Target::Copy) { BUTTON_BORDER_HOVER } else { BUTTON_BORDER };
            bordered(canvas, copy, ROW_RADIUS, BUTTON_FILL, border, opacity);
            stroke_icon(canvas, COPY_ICON, center(copy), BUTTON_ICON_SIZE, 1.8, COPY_GRAY, opacity);
        }

        let paste = layout.paste_button();
        let (fill, border, foreground, icon, label) = if self.confirm == Some(Confirm::Pasted) {
            (GREEN_FILL, GREEN, GREEN, CHECK_ICON, tr!("clipboard.pasted"))
        } else {
            let white = if hovered(Target::Paste) { Color::gray(0.85) } else { Color::WHITE };
            (white, white, Color::BLACK, PASTE_ICON, tr!("clipboard.paste"))
        };
        bordered(canvas, paste, ROW_RADIUS, fill, border, opacity);
        let label_width = canvas.measure(&BUTTON_LABEL, &label);
        let content_width = (BUTTON_ICON_SIZE + BUTTON_ICON_GAP) + label_width;
        let left = paste.center_x() - content_width / 2.0;
        let icon_center = Point::new(left + BUTTON_ICON_SIZE / 2.0, paste.center_y());
        stroke_icon(canvas, icon, icon_center, BUTTON_ICON_SIZE, 1.8, foreground, opacity);
        let label_area = Rect { left: left + (BUTTON_ICON_SIZE + BUTTON_ICON_GAP), ..paste };
        canvas.text(&BUTTON_LABEL, &label, label_area, foreground.faded(opacity));
    }
}

struct Row {
    is_selected: bool,
    is_hovered: bool,
    confirm: Option<Confirm>,
    /// The pin button's area and icon color, when the row shows one.
    pin: Option<(Rect, Color)>,
}

fn draw_row(canvas: &Canvas, rect: Rect, item: &Item, row: Row, opacity: f32) {
    let (fill, border) = match (row.confirm, row.is_selected, row.is_hovered) {
        (Some(_), _, _) => (GREEN_FILL, GREEN),
        (None, true, _) => (SELECTED_FILL, SELECTED_BORDER),
        (None, false, true) => (ROW_HOVER_FILL, ROW_BORDER),
        (None, false, false) => (ROW_FILL, ROW_BORDER),
    };
    bordered(canvas, rect, ROW_RADIUS, fill, border, opacity);
    let leading = rect.left + ROW_PADDING;
    let middle = rect.center_y();
    let text_right = match row.pin {
        Some((area, color)) => {
            Icon::Pin.draw(canvas, Point::new(area.center_x(), area.center_y()), PIN_ICON_SIZE, color, opacity);
            area.left
        }
        None => rect.right - ROW_PADDING,
    };
    if let Some(confirm) = row.confirm {
        let center = Point::new(leading + ROW_ICON_SIZE / 2.0, middle);
        stroke_icon(canvas, CHECK_ICON, center, ROW_ICON_SIZE, 1.8, GREEN, opacity);
        let area = Rect::new(leading + (ROW_ICON_SIZE + ROW_GAP), rect.top, text_right, rect.bottom);
        canvas.text(&ROW_CONFIRM, &confirm.row_text(), area, GREEN.faded(opacity));
        return;
    }
    let (badge_width, style) = match item.kind {
        Kind::Color | Kind::Image => (ROW_BADGE_SIZE, ROW_TEXT),
        Kind::Code => (ROW_ICON_SIZE, ROW_CODE),
        Kind::Text | Kind::Link => (ROW_ICON_SIZE, ROW_TEXT),
    };
    match (&item.content, item.kind) {
        (Content::Text(text), Kind::Color) => {
            if let Some(color) = parse_color(text.trim()) {
                canvas.fill_rounded_rect(badge_rect(leading, middle), 4.0, color.faded(opacity));
            }
        }
        (Content::Image(_), _) => {
            let area = badge_rect(leading, middle);
            let side = (area.width() * canvas.raster_scale()).round() as u32;
            if let Some(brush) = canvas.cached_image(("clipboard-image-badge", side), || image_badge(side)) {
                canvas.image(&brush, area, 4.0, opacity);
            }
        }
        _ => {
            let center = Point::new(leading + ROW_ICON_SIZE / 2.0, middle);
            stroke_icon(canvas, kind_icon(item.kind), center, ROW_ICON_SIZE, 1.8, ICON, opacity);
        }
    }
    let area = Rect::new(leading + (badge_width + ROW_GAP), rect.top, text_right, rect.bottom);
    draw_label(canvas, &style, &row_label(item), area, opacity);
}

fn badge_rect(left: f32, middle: f32) -> Rect {
    let side = ROW_BADGE_SIZE;
    Rect::new(left, middle - side / 2.0, left + side, middle + side / 2.0)
}

fn draw_label(canvas: &Canvas, style: &FontStyle, text: &str, area: Rect, opacity: f32) {
    canvas.text(style, text, area, TEXT.faded(opacity));
}

fn draw_bar(canvas: &Canvas, rect: Rect, bar: Bar, opacity: f32) {
    canvas.fill_rounded_rect(rect, ROW_RADIUS, BAR.faded(opacity));
    let padding = 10.0;
    let text_area = Rect::new(rect.left + padding, rect.top, rect.right - padding, rect.bottom);
    canvas.text(&BAR_TEXT, &bar.text(), text_area, TEXT.faded(opacity));
    if bar.has_undo() {
        canvas.text(&BAR_ACTION, &tr!("clipboard.undo"), text_area, UNDO_BLUE.faded(opacity));
    }
}

/// The badge and two lines side by side, centered on the page.
fn draw_empty(canvas: &Canvas, bounds: Rect, opacity: f32) {
    let title = tr!("clipboard.empty_title");
    let hint = tr!("clipboard.empty_hint");
    let text_width = canvas.measure(&EMPTY_TITLE, &title).max(canvas.measure(&EMPTY_HINT, &hint)).ceil();
    let total = EMPTY_BADGE_SIZE + EMPTY_GAP + text_width;
    let left = bounds.left + (bounds.width() - total) / 2.0;
    let middle = bounds.top + bounds.height() / 2.0;
    let badge_center = Point::new(left + EMPTY_BADGE_SIZE / 2.0, middle);
    let radius = EMPTY_BADGE_SIZE / 2.0;
    canvas.fill_circle(badge_center, radius, BADGE_BORDER.faded(opacity));
    canvas.fill_circle(badge_center, radius - BORDER, BADGE_FILL.faded(opacity));
    stroke_icon(canvas, PASTE_ICON, badge_center, 20.0, 2.0, HINT, opacity);
    let text_left = left + (EMPTY_BADGE_SIZE + EMPTY_GAP);
    let text_right = text_left + text_width + 2.0;
    let title_area = Rect::new(text_left, middle - 18.5, text_right, middle - 0.5);
    let hint_area = Rect::new(text_left, middle + 2.5, text_right, middle + 18.5);
    canvas.text(&EMPTY_TITLE, &title, title_area, TEXT.faded(opacity));
    canvas.text(&EMPTY_HINT, &hint, hint_area, HINT.faded(opacity));
}

#[cfg(test)]
mod tests {
    use super::{
        super::layout::{BAR_HEIGHT, BUTTON_SIZE, ROW_HEIGHT},
        *,
    };

    #[test]
    fn the_image_badge_is_orange_with_a_dark_notch_at_the_top() {
        let badge = image_badge(15);
        let pixel = |x: usize, y: usize| &badge.bgra_premultiplied[(y * 15 + x) * 4..(y * 15 + x) * 4 + 4];
        assert_eq!(pixel(7, 0), [0, 0, 0, 255]);
        assert_eq!(pixel(0, 0)[3], 255);
        assert!(pixel(14, 14)[2] > pixel(0, 0)[2] - 1, "red stays high toward the corner");
        assert_ne!(pixel(1, 1), pixel(13, 13));
    }

    #[test]
    fn the_rows_and_bar_use_the_designed_sizes() {
        assert_eq!((ROW_HEIGHT, BAR_HEIGHT, BUTTON_SIZE), (23.0, 26.0, 26.0));
    }
}
