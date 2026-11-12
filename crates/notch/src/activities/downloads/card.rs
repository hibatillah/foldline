//! The download card: its size, its words and its drawing; the start and the finish card share one layout.

use std::path::{Path, PathBuf};

use crate::{
    gfx::{Canvas, Color, Fonts, ImageBrush, ImageData, ImageSlot, Point, Rect, clamp_lines},
    i18n::tr,
    page::PointerEvent,
    ui::{
        BANNER_BUTTONS_HEIGHT, BannerButton, BannerButtons, BannerLabels, Icon,
        palette::CARD_BODY_TEXT,
        type_styles::{MEDIUM_14, REGULAR_13},
    },
};

pub const WIDTH: f32 = 340.0;
const PADDING: f32 = 20.0;
const TOP_PADDING: f32 = 12.0;
const BOTTOM_PADDING: f32 = 16.0;
const TILE_SIZE: f32 = 30.0;
const TILE_RADIUS: f32 = 8.0;
const TILE_GAP: f32 = 12.0;
const TITLE_HEIGHT: f32 = 18.0;
const TITLE_GAP: f32 = 2.0;
const DETAIL_HEIGHT: f32 = 17.0;
const MIN_HEIGHT: f32 = TOP_PADDING + TILE_SIZE + BOTTOM_PADDING;
const TILE_FILL: Color = Color::gray(0.16);

/// What the card is made for.
pub enum Shown {
    Started { title: String },
    Finished { title: String, detail: String, path: PathBuf, icon: Option<ImageData> },
}

struct Finish {
    path: PathBuf,
    buttons: BannerButtons,
    icon_source: Option<ImageData>,
    icon: ImageSlot,
}

pub struct Card {
    title: String,
    detail: String,
    finish: Option<Finish>,
}

fn text_width() -> f32 {
    WIDTH - 2.0 * PADDING - TILE_SIZE - TILE_GAP
}

impl Card {
    pub fn new(shown: Shown, fonts: &Fonts) -> Self {
        let one_line = |text: &str| {
            clamp_lines(text, 1, text_width(), |line| fonts.measure(&MEDIUM_14, line)).pop().unwrap_or_default()
        };
        match shown {
            Shown::Started { title } => {
                Self { title: one_line(&title), detail: tr!("downloads.started").into_owned(), finish: None }
            }
            Shown::Finished { title, detail, path, icon } => {
                let (show_in_folder, open) = (tr!("downloads.show_in_folder"), tr!("downloads.open"));
                let labels = BannerLabels { dismiss: &show_in_folder, reply: "", open: &open };
                let buttons = BannerButtons::new(fonts, labels);
                let finish = Finish { path, buttons, icon_source: icon, icon: ImageSlot::default() };
                Self { title: one_line(&title), detail, finish: Some(finish) }
            }
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finish.is_some()
    }

    #[cfg(test)]
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn path(&self) -> Option<&Path> {
        self.finish.as_ref().map(|finish| finish.path.as_path())
    }

    /// Uploads the file icon once; dropping the card releases it.
    pub fn upload_icon(&mut self, upload: impl FnOnce(&ImageData) -> Option<ImageBrush>) {
        if let Some(finish) = &mut self.finish {
            finish.icon.sync(finish.icon_source.as_ref(), upload);
        }
    }

    pub fn height(&self, shows_buttons: bool) -> f32 {
        if shows_buttons && self.is_finished() { MIN_HEIGHT + BANNER_BUTTONS_HEIGHT } else { MIN_HEIGHT }
    }

    /// The right edge of the button row and its top, in units.
    fn buttons_origin(area: Rect) -> (f32, f32) {
        (area.right - PADDING, area.top + TOP_PADDING + TILE_SIZE)
    }

    /// Returns the clicked button and whether the buttons need a repaint; the start card has none.
    pub fn on_pointer(&mut self, event: PointerEvent, area: Rect) -> (Option<BannerButton>, bool) {
        let (right, top) = Self::buttons_origin(area);
        match &mut self.finish {
            Some(finish) => finish.buttons.on_pointer(event, right, top, 1.0),
            None => (None, false),
        }
    }

    pub fn draw(&self, canvas: &Canvas, area: Rect, opacity: f32, shows_buttons: bool) {
        let tile = Rect::new(
            area.left + PADDING,
            area.top + TOP_PADDING,
            area.left + PADDING + TILE_SIZE,
            area.top + TOP_PADDING + TILE_SIZE,
        );
        match self.finish.as_ref().and_then(|finish| finish.icon.brush()) {
            Some(brush) => canvas.image(brush, tile, TILE_RADIUS, opacity),
            None => {
                canvas.fill_rounded_rect(tile, TILE_RADIUS, TILE_FILL.faded(opacity));
                draw_icon(canvas, tile, opacity);
            }
        }
        let (left, right) = (tile.right + TILE_GAP, area.right - PADDING);
        let title = Rect::new(left, tile.top, right, tile.top + TITLE_HEIGHT);
        canvas.text(&MEDIUM_14, &self.title, title, Color::WHITE.faded(opacity));
        let detail_top = title.bottom + TITLE_GAP;
        let detail = Rect::new(left, detail_top, right, detail_top + DETAIL_HEIGHT);
        canvas.text(&REGULAR_13, &self.detail, detail, CARD_BODY_TEXT.faded(opacity));
        if let (Some(finish), true) = (&self.finish, shows_buttons) {
            let (right, top) = Self::buttons_origin(area);
            finish.buttons.draw(canvas, right, top, opacity);
        }
    }
}

/// The download mark, white and centered, as in the pill and the header.
pub fn draw_icon(canvas: &Canvas, area: Rect, opacity: f32) {
    let center = Point::new(area.center_x(), area.center_y());
    Icon::Download.draw(canvas, center, Icon::Download.size(), Color::WHITE, opacity);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finished(title: &str, fonts: &Fonts) -> Card {
        let shown = Shown::Finished {
            title: title.to_owned(),
            detail: "Downloaded \u{b7} 17.8 MB".to_owned(),
            path: PathBuf::from("C:/Downloads/file.msi"),
            icon: None,
        };
        Card::new(shown, fonts)
    }

    #[test]
    fn a_long_file_name_is_cut_to_one_line() {
        let fonts = Fonts::new(1.0).unwrap();
        let card = finished(
            &"a-very-long-installer-name-for-the-calendar-app-windows-x64-release-candidate".repeat(3),
            &fonts,
        );
        assert!(card.title().ends_with('\u{2026}'));
        assert!(fonts.measure(&MEDIUM_14, card.title()) <= WIDTH - 2.0 * PADDING - TILE_SIZE - TILE_GAP);
    }

    #[test]
    fn a_short_file_name_is_kept_whole() {
        let fonts = Fonts::new(1.0).unwrap();
        assert_eq!(finished("setup.exe", &fonts).title(), "setup.exe");
    }

    #[test]
    fn the_buttons_add_height_to_the_finish_card_only() {
        let fonts = Fonts::new(1.0).unwrap();
        let finish = finished("a.msi", &fonts);
        assert_eq!(finish.height(false), 58.0);
        assert_eq!(finish.height(true), 58.0 + BANNER_BUTTONS_HEIGHT);
        let start = Card::new(Shown::Started { title: "a.msi".to_owned() }, &fonts);
        assert_eq!(start.height(true), 58.0);
        assert!(!start.is_finished());
    }

    #[test]
    fn the_start_card_ignores_the_pointer() {
        let fonts = Fonts::new(1.0).unwrap();
        let mut start = Card::new(Shown::Started { title: "a.msi".to_owned() }, &fonts);
        let area = Rect::new(0.0, 0.0, WIDTH, 58.0);
        assert_eq!(start.on_pointer(PointerEvent::Moved(Point::new(300.0, 45.0)), area), (None, false));
    }

    #[test]
    fn show_in_folder_is_the_left_button_and_open_the_right_one() {
        let fonts = Fonts::new(1.0).unwrap();
        let mut card = finished("a.msi", &fonts);
        let area = Rect::new(0.0, 0.0, WIDTH, card.height(true));
        let (right, top) = Card::buttons_origin(area);
        let finish = card.finish.as_ref().unwrap();
        let rects: Vec<_> = finish.buttons.rects(right, top, 1.0).collect();
        assert_eq!(
            rects.iter().map(|(button, _)| *button).collect::<Vec<_>>(),
            [BannerButton::Dismiss, BannerButton::Open]
        );
        let open = rects[1].1;
        let point = Point::new(open.center_x(), open.center_y());
        card.on_pointer(PointerEvent::Pressed(point), area);
        assert_eq!(card.on_pointer(PointerEvent::Released(point), area).0, Some(BannerButton::Open));
    }

    #[test]
    fn the_buttons_fit_inside_the_card_in_both_languages() {
        let fonts = Fonts::new(1.0).unwrap();
        for language in [store::language::Language::English, store::language::Language::Indonesian] {
            store::language::with_override(language, || {
                let card = finished("a.msi", &fonts);
                let area = Rect::new(0.0, 0.0, WIDTH, card.height(true));
                let (right, top) = Card::buttons_origin(area);
                let finish = card.finish.as_ref().unwrap();
                let leftmost =
                    finish.buttons.rects(right, top, 1.0).map(|(_, rect)| rect.left).fold(f32::MAX, f32::min);
                assert!(leftmost >= PADDING, "{language:?}: the buttons start at {leftmost}");
            });
        }
    }
}
