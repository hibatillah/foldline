//! The notification card: its size, its words and its drawing.

use super::items::{Item, app_color, app_letter, body_text, heading, open_label};
use crate::{
    gfx::{Align, Canvas, Color, FontStyle, Fonts, ImageData, Rect, UI_FONT, Weight, clamp_lines},
    page::PointerEvent,
    ui::{BANNER_BUTTONS_HEIGHT, BannerButton, BannerButtons},
};

pub const WIDTH: f32 = 400.0;
const PADDING: f32 = 20.0;
const TOP_PADDING: f32 = 12.0;
const BOTTOM_PADDING: f32 = 16.0;
const TILE_SIZE: f32 = 30.0;
const TILE_RADIUS: f32 = 8.0;
const TILE_GAP: f32 = 12.0;
const HEADING_HEIGHT: f32 = 18.0;
const HEADING_GAP: f32 = 2.0;
const BODY_LINE_HEIGHT: f32 = 17.0;
pub const BODY_LINES: usize = 2;
/// Between the sender and the app name.
const TITLE_GAP: f32 = 8.0;
/// The app name never takes more than this share of the heading, so the sender stays readable.
const APP_NAME_SHARE: f32 = 0.5;
const MIN_HEIGHT: f32 = TOP_PADDING + TILE_SIZE + BOTTOM_PADDING;

const LEAD: FontStyle = FontStyle::new(UI_FONT, 14.0, Weight::Medium, Align::Leading);
const APP_NAME: FontStyle = FontStyle::new(UI_FONT, 14.0, Weight::Regular, Align::Leading);
const BODY: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Regular, Align::Leading);
const TILE_LETTER: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Medium, Align::Center);

const APP_NAME_COLOR: Color = Color::rgb(0.557, 0.557, 0.6);
const BODY_COLOR: Color = Color::rgb(0.678, 0.678, 0.722);

/// The card's height in DIP for `body_lines` lines of text, with or without the buttons.
pub fn card_height(body_lines: usize, shows_buttons: bool) -> f32 {
    let buttons = if shows_buttons { BANNER_BUTTONS_HEIGHT } else { 0.0 };
    let text = TOP_PADDING + HEADING_HEIGHT + HEADING_GAP + body_lines as f32 * BODY_LINE_HEIGHT;
    (text + buttons + BOTTOM_PADDING).max(MIN_HEIGHT + buttons)
}

fn text_width() -> f32 {
    WIDTH - 2.0 * PADDING - TILE_SIZE - TILE_GAP
}

/// The words cut to fit, measured once when the notification arrives; widths are in DIP.
pub struct Words {
    pub lead: String,
    lead_width: f32,
    pub app: Option<String>,
    pub body: Vec<String>,
}

impl Words {
    pub fn new(item: &Item, shows_text: bool, fonts: &Fonts) -> Self {
        let heading = heading(item);
        let width = text_width();
        let fit = |style: &FontStyle, text: &str, room: f32| {
            clamp_lines(text, 1, room, |text| fonts.measure(style, text)).pop().unwrap_or_default()
        };
        let app = heading.app.map(|app| fit(&APP_NAME, &app, width * APP_NAME_SHARE));
        let app_room = app.as_deref().map_or(0.0, |app| fonts.measure(&APP_NAME, app) + TITLE_GAP);
        let lead = fit(&LEAD, &heading.lead, width - app_room);
        let lead_width = fonts.measure(&LEAD, &lead);
        let body = clamp_lines(&body_text(item, shows_text), BODY_LINES, width, |text| fonts.measure(&BODY, text));
        Self { lead, lead_width, app, body }
    }
}

/// What the card shows for one notification, and the state of its buttons.
pub struct Card {
    pub words: Words,
    buttons: BannerButtons,
}

impl Card {
    pub fn new(item: &Item, shows_text: bool, fonts: &Fonts) -> Self {
        Self { words: Words::new(item, shows_text, fonts), buttons: BannerButtons::new(fonts, &open_label(item)) }
    }

    pub fn height(&self, shows_buttons: bool) -> f32 {
        card_height(self.words.body.len(), shows_buttons)
    }

    /// The right edge of the button row and the top of the gap above it, in device pixels.
    fn buttons_origin(&self, area: Rect, scale: f32) -> (f32, f32) {
        let text_bottom = TOP_PADDING + HEADING_HEIGHT + HEADING_GAP + self.words.body.len() as f32 * BODY_LINE_HEIGHT;
        let top = area.top + text_bottom.max(TOP_PADDING + TILE_SIZE) * scale;
        (area.right - PADDING * scale, top)
    }

    /// Returns the clicked button and whether the buttons need a repaint.
    pub fn on_pointer(&mut self, event: PointerEvent, area: Rect, scale: f32) -> (Option<BannerButton>, bool) {
        let (right, top) = self.buttons_origin(area, scale);
        self.buttons.on_pointer(event, right, top, scale)
    }

    pub fn draw(&self, canvas: &Canvas, item: &Item, area: Rect, scale: f32, opacity: f32, shows_buttons: bool) {
        let tile = Rect::new(
            area.left + PADDING * scale,
            area.top + TOP_PADDING * scale,
            area.left + (PADDING + TILE_SIZE) * scale,
            area.top + (TOP_PADDING + TILE_SIZE) * scale,
        );
        draw_tile(canvas, item, tile, scale, opacity);
        let left = tile.right + TILE_GAP * scale;
        let right = area.right - PADDING * scale;
        let heading = Rect::new(left, tile.top, right, tile.top + HEADING_HEIGHT * scale);
        canvas.text(&LEAD, &self.words.lead, heading, Color::WHITE.faded(opacity));
        if let Some(app) = &self.words.app {
            let app_left = left + (self.words.lead_width + TITLE_GAP) * scale;
            canvas.text(&APP_NAME, app, Rect { left: app_left, ..heading }, APP_NAME_COLOR.faded(opacity));
        }
        for (index, line) in self.words.body.iter().enumerate() {
            let top = heading.bottom + (HEADING_GAP + index as f32 * BODY_LINE_HEIGHT) * scale;
            canvas.text(
                &BODY,
                line,
                Rect::new(left, top, right, top + BODY_LINE_HEIGHT * scale),
                BODY_COLOR.faded(opacity),
            );
        }
        if shows_buttons {
            let (right, top) = self.buttons_origin(area, scale);
            self.buttons.draw(canvas, right, top, scale, opacity);
        }
    }
}

fn draw_tile(canvas: &Canvas, item: &Item, tile: Rect, scale: f32, opacity: f32) {
    let logo = item.icon.as_deref().and_then(|icon| {
        let key = ("notification-logo", item.app_id.as_str());
        canvas.cached_image(key, || ImageData::new(icon.width, icon.height, icon.bgra_premultiplied.clone()))
    });
    match logo {
        Some(brush) => canvas.image(&brush, tile, TILE_RADIUS * scale, opacity),
        None => {
            canvas.fill_rounded_rect(tile, TILE_RADIUS * scale, app_color(&item.app).faded(opacity));
            canvas.text(&TILE_LETTER, &app_letter(&item.app), tile, Color::WHITE.faded(opacity));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;
    use crate::gfx::Point;

    fn item(title: &str, body: &str) -> Item {
        Item {
            id: 1,
            app: "Discord".to_owned(),
            app_id: String::new(),
            icon: None,
            title: title.to_owned(),
            body: body.to_owned(),
            read_at: Instant::now(),
        }
    }

    #[test]
    fn the_card_grows_by_one_body_line_each_and_by_the_buttons_when_pointed_at() {
        assert_eq!(card_height(2, false) - card_height(1, false), BODY_LINE_HEIGHT);
        assert_eq!(card_height(2, false), 82.0);
        assert_eq!(card_height(2, true) - card_height(2, false), BANNER_BUTTONS_HEIGHT);
    }

    #[test]
    fn the_card_is_taller_than_the_idle_pill() {
        assert!(card_height(0, false) > crate::page::RESTING_HEIGHT * 1.25);
    }

    #[test]
    fn a_long_message_stays_at_two_lines_and_a_long_sender_is_cut() {
        let fonts = Fonts::new(1.0).unwrap();
        let long = item(
            "A very long conversation name that goes on and on and on and on",
            &"Deploy is green. Can you check the hover state on the 1440p monitor before we tag it? ".repeat(3),
        );
        let words = Words::new(&long, true, &fonts);
        assert!(words.lead.ends_with('\u{2026}'));
        assert_eq!(words.app.as_deref(), Some("Discord"));
        assert_eq!(words.body.len(), BODY_LINES);
        assert!(words.body[BODY_LINES - 1].ends_with('\u{2026}'));
    }

    #[test]
    fn private_mode_shows_only_the_private_words() {
        let fonts = Fonts::new(1.0).unwrap();
        let words = Words::new(&item("Invoice", "secret"), false, &fonts);
        assert_eq!(words.body, vec![super::super::items::PRIVATE_TEXT.to_owned()]);
    }

    #[test]
    fn the_buttons_sit_below_the_text_inside_the_card() {
        let fonts = Fonts::new(1.0).unwrap();
        let mut card = Card::new(&item("Rafi", "Deploy is green."), true, &fonts);
        let area = Rect::new(0.0, 0.0, WIDTH, card.height(true));
        let (right, top) = card.buttons_origin(area, 1.0);
        assert_eq!(right, WIDTH - PADDING);
        assert!(top + BANNER_BUTTONS_HEIGHT <= area.bottom - BOTTOM_PADDING + 0.01);
        let open = Point::new(right - 10.0, area.bottom - BOTTOM_PADDING - 10.0);
        card.on_pointer(PointerEvent::Pressed(open), area, 1.0);
        assert_eq!(card.on_pointer(PointerEvent::Released(open), area, 1.0).0, Some(BannerButton::Open));
    }
}
