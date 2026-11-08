//! The drawn document for a file that has no picture of its own: a white sheet with a mark for its type.

use std::path::Path;

use super::items::{extension, tile_color, tile_label};
use crate::gfx::{Align, Canvas, Color, FontStyle, Point, Rect, UI_FONT, Weight};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DocumentKind {
    Pdf,
    Picture,
    Design,
    Text,
    Archive,
    Sheet,
    /// Any other type: a sheet with its extension on a tag.
    Other,
}

pub(super) fn document_kind(path: &Path) -> DocumentKind {
    match extension(path).as_str() {
        "pdf" => DocumentKind::Pdf,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "heic" | "avif" | "tif" | "tiff" | "svg" => {
            DocumentKind::Picture
        }
        "fig" | "sketch" | "psd" | "ai" => DocumentKind::Design,
        "txt" | "md" | "log" | "text" | "" => DocumentKind::Text,
        "zip" | "rar" | "7z" | "tar" | "gz" => DocumentKind::Archive,
        "xls" | "xlsx" | "csv" | "ods" => DocumentKind::Sheet,
        _ => DocumentKind::Other,
    }
}

const SHEET_LEFT: f32 = 5.0;
const SHEET_TOP: f32 = 1.0;
const SHEET_WIDTH: f32 = 30.0;
const SHEET_HEIGHT: f32 = 38.0;
const SHEET_RADIUS: f32 = 3.0;
const TAG_HEIGHT: f32 = 11.0;
const TAG_PADDING: f32 = 3.0;
const TAG_MAX_WIDTH: f32 = 26.0;
const LINE_COUNT: usize = 4;
const LINE_GAP: f32 = 3.0;
const LINE_INSET: f32 = 5.0;
const ZIPPER_STRIPE: f32 = 2.0;

/// The design asks for 600; only Regular and Medium are bundled.
const TAG: FontStyle = FontStyle::new(UI_FONT, 8.0, Weight::Medium, Align::Center);
const BADGE: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Medium, Align::Center);

const SHEET: Color = Color::WHITE;
const PDF_RED: Color = Color::rgb(0.878, 0.192, 0.192);
const PICTURE_BLUE: Color = Color::rgb(0.204, 0.584, 0.902);
const DESIGN_PURPLE: Color = Color::rgb(0.439, 0.282, 0.91);
const DESIGN_LILAC: Color = Color::rgb(0.694, 0.592, 0.988);
const LINE_GRAY: Color = Color::rgb(0.769, 0.769, 0.8);
const FOLDER_TAB: Color = Color::rgb(0.949, 0.694, 0.204);
const FOLDER_BODY: Color = Color::rgb(0.973, 0.741, 0.231);
const ZIPPER_DARK: Color = Color::rgb(0.361, 0.255, 0.024);
const STRIP_GREEN: Color = Color::rgb(0.247, 0.686, 0.322);
const EXCEL_GREEN: Color = Color::rgb(0.063, 0.486, 0.255);

/// Draws the file's document into its 40 by 40 box.
pub(super) fn draw_document(canvas: &Canvas, path: &Path, area: Rect, opacity: f32) {
    let box_ = Box40 { canvas, area, opacity };
    match document_kind(path) {
        DocumentKind::Pdf => {
            box_.sheet();
            box_.tag("PDF", PDF_RED);
        }
        DocumentKind::Picture => {
            box_.sheet();
            box_.fill(12.0, 13.0, 16.0, 14.0, 3.0, PICTURE_BLUE);
        }
        DocumentKind::Design => {
            box_.sheet();
            box_.fill(10.0, 10.0, 20.0, 20.0, 3.0, DESIGN_PURPLE);
            box_.fill(11.5, 11.5, 17.0, 17.0, 1.5, SHEET);
            box_.circle(13.0, 13.0, 8.0, DESIGN_PURPLE);
            box_.fill(19.0, 21.0, 8.0, 6.0, 2.0, DESIGN_LILAC);
        }
        DocumentKind::Text => {
            box_.sheet();
            let block = LINE_COUNT as f32 + LINE_GAP * (LINE_COUNT - 1) as f32;
            for line in 0..LINE_COUNT {
                let top = SHEET_TOP + (SHEET_HEIGHT - block) / 2.0 + line as f32 * (1.0 + LINE_GAP);
                box_.fill(SHEET_LEFT + LINE_INSET, top, SHEET_WIDTH - 2.0 * LINE_INSET, 1.0, 0.5, LINE_GRAY);
            }
        }
        DocumentKind::Archive => box_.folder(),
        DocumentKind::Sheet => box_.spreadsheet(),
        DocumentKind::Other => {
            box_.sheet();
            box_.tag(&tile_label(path, false), tile_color(path));
        }
    }
}

/// Draws in the units of the design (a 40 by 40 box), scaled to the screen.
struct Box40<'a> {
    canvas: &'a Canvas<'a>,
    area: Rect,
    opacity: f32,
}

impl Box40<'_> {
    fn rect(&self, left: f32, top: f32, width: f32, height: f32) -> Rect {
        let unit = self.area.width() / 40.0;
        let (x, y) = (self.area.left + left * unit, self.area.top + top * unit);
        Rect::new(x, y, x + width * unit, y + height * unit)
    }

    fn fill(&self, left: f32, top: f32, width: f32, height: f32, radius: f32, color: Color) {
        let unit = self.area.width() / 40.0;
        self.canvas.fill_rounded_rect(self.rect(left, top, width, height), radius * unit, color.faded(self.opacity));
    }

    fn circle(&self, left: f32, top: f32, diameter: f32, color: Color) {
        let rect = self.rect(left, top, diameter, diameter);
        let center = Point::new(rect.center_x(), rect.center_y());
        self.canvas.fill_circle(center, rect.width() / 2.0, color.faded(self.opacity));
    }

    fn sheet(&self) {
        self.fill(SHEET_LEFT, SHEET_TOP, SHEET_WIDTH, SHEET_HEIGHT, SHEET_RADIUS, SHEET);
    }

    /// A colored tag with a short label, centered on the sheet.
    fn tag(&self, label: &str, color: Color) {
        if label.is_empty() {
            return;
        }
        let width = (self.canvas.measure(&TAG, label) + 2.0 * TAG_PADDING).min(TAG_MAX_WIDTH);
        let left = SHEET_LEFT + (SHEET_WIDTH - width) / 2.0;
        let top = SHEET_TOP + (SHEET_HEIGHT - TAG_HEIGHT) / 2.0;
        self.fill(left, top, width, TAG_HEIGHT, 0.0, color);
        self.canvas.text(&TAG, label, self.rect(left, top, width, TAG_HEIGHT), Color::WHITE.faded(self.opacity));
    }

    fn folder(&self) {
        self.fill(1.0, 4.0, 16.0, 8.0, 3.0, FOLDER_TAB);
        self.fill(1.0, 8.0, 38.0, 28.0, 3.0, FOLDER_BODY);
        let stripes = (28.0 / ZIPPER_STRIPE) as usize;
        for stripe in 0..stripes {
            let color = if stripe % 2 == 0 { ZIPPER_DARK } else { Color::WHITE };
            self.fill(17.0, 8.0 + stripe as f32 * ZIPPER_STRIPE, 6.0, ZIPPER_STRIPE, 0.0, color);
        }
        self.fill(16.0, 21.0, 8.0, 6.0, 2.0, ZIPPER_DARK);
    }

    fn spreadsheet(&self) {
        self.fill(9.0, 5.0, 30.0, 28.0, SHEET_RADIUS, SHEET);
        self.fill(17.0, 3.0, 14.0, 22.0, 0.0, STRIP_GREEN);
        self.fill(1.0, 19.0, 18.0, 18.0, 3.0, EXCEL_GREEN);
        self.canvas.text(&BADGE, "X", self.rect(1.0, 19.0, 18.0, 18.0), Color::WHITE.faded(self.opacity));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind(name: &str) -> DocumentKind {
        document_kind(Path::new(name))
    }

    #[test]
    fn each_file_type_gets_its_own_document() {
        assert_eq!(kind("brief.PDF"), DocumentKind::Pdf);
        assert_eq!(kind("hero.png"), DocumentKind::Picture);
        assert_eq!(kind("mock.fig"), DocumentKind::Design);
        assert_eq!(kind("notes.txt"), DocumentKind::Text);
        assert_eq!(kind("backup.zip"), DocumentKind::Archive);
        assert_eq!(kind("budget.xlsx"), DocumentKind::Sheet);
    }

    #[test]
    fn an_unknown_type_is_a_tagged_sheet_and_no_extension_is_a_plain_text_sheet() {
        assert_eq!(kind("report.docx"), DocumentKind::Other);
        assert_eq!(kind("a.unknown"), DocumentKind::Other);
        assert_eq!(kind("README"), DocumentKind::Text);
    }

    #[test]
    fn the_design_boxes_fit_inside_the_forty_unit_square() {
        const {
            assert!(SHEET_LEFT + SHEET_WIDTH <= 40.0 && SHEET_TOP + SHEET_HEIGHT <= 40.0);
            assert!(TAG_MAX_WIDTH <= SHEET_WIDTH);
            assert!(LINE_COUNT as f32 + LINE_GAP * (LINE_COUNT - 1) as f32 <= SHEET_HEIGHT);
        }
    }
}
