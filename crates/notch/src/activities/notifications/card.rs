//! The notification card: its size, its words and its drawing.

use super::items::{Item, app_color, app_letter, body_text, heading, open_label};
use super::reply::ReplyState;
use crate::i18n::tr;
use crate::ui::palette::{CARD_APP_NAME_TEXT, CARD_BODY_TEXT};
use crate::ui::type_styles::{MEDIUM_14, REGULAR_13, REGULAR_14};
use crate::{
    gfx::{Align, Canvas, Color, FontStyle, Fonts, ImageData, Rect, UI_FONT, Weight, clamp_lines},
    page::PointerEvent,
    ui::{BANNER_BUTTONS_HEIGHT, BannerButton, BannerButtons, BannerLabels},
};

pub const WIDTH: f32 = 340.0;
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

const TILE_LETTER: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Medium, Align::Center);

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
        let app = heading.app.map(|app| fit(&REGULAR_14, &app, width * APP_NAME_SHARE));
        let app_room = app.as_deref().map_or(0.0, |app| fonts.measure(&REGULAR_14, app) + TITLE_GAP);
        let lead = fit(&MEDIUM_14, &heading.lead, width - app_room);
        let lead_width = fonts.measure(&MEDIUM_14, &lead);
        let body =
            clamp_lines(&body_text(item, shows_text), BODY_LINES, width, |text| fonts.measure(&REGULAR_13, text));
        Self { lead, lead_width, app, body }
    }
}

/// Where and how the card is drawn: its area in units, the opacity, and whether the buttons show.
#[derive(Clone, Copy)]
pub struct DrawLook {
    pub area: Rect,
    pub opacity: f32,
    pub shows_buttons: bool,
}

/// What the card shows for one notification, and the state of its buttons.
pub struct Card {
    pub words: Words,
    buttons: BannerButtons,
}

impl Card {
    pub fn new(item: &Item, shows_text: bool, fonts: &Fonts) -> Self {
        let (dismiss, reply, open) = (tr!("banner.dismiss"), tr!("banner.reply"), open_label(item));
        let labels = BannerLabels { dismiss: &dismiss, reply: &reply, open: &open };
        Self { words: Words::new(item, shows_text, fonts), buttons: BannerButtons::new(fonts, labels) }
    }

    /// Adds the Reply button once the toast is known to offer one.
    pub fn offer_reply(&mut self, fonts: &Fonts) {
        let (dismiss, reply, open) = (tr!("banner.dismiss"), tr!("banner.reply"), tr!("banner.open"));
        self.buttons = BannerButtons::with_reply(fonts, BannerLabels { dismiss: &dismiss, reply: &reply, open: &open });
    }

    pub fn height(&self, shows_buttons: bool) -> f32 {
        card_height(self.words.body.len(), shows_buttons)
    }

    /// The right edge of the button row and the top of the gap above it, in units.
    fn buttons_origin(&self, area: Rect) -> (f32, f32) {
        let text_bottom = TOP_PADDING + HEADING_HEIGHT + HEADING_GAP + self.words.body.len() as f32 * BODY_LINE_HEIGHT;
        let top = area.top + text_bottom.max(TOP_PADDING + TILE_SIZE);
        (area.right - PADDING, top)
    }

    /// Returns the clicked button and whether the buttons need a repaint.
    pub fn on_pointer(&mut self, event: PointerEvent, area: Rect) -> (Option<BannerButton>, bool) {
        let (right, top) = self.buttons_origin(area);
        self.buttons.on_pointer(event, right, top, 1.0)
    }

    pub fn draw(&self, canvas: &Canvas, item: &Item, look: DrawLook, reply: &ReplyState) {
        let DrawLook { area, opacity, shows_buttons } = look;
        let tile = Rect::new(
            area.left + PADDING,
            area.top + TOP_PADDING,
            area.left + PADDING + TILE_SIZE,
            area.top + TOP_PADDING + TILE_SIZE,
        );
        draw_tile(canvas, item, tile, opacity);
        let left = tile.right + TILE_GAP;
        let right = area.right - PADDING;
        let heading = Rect::new(left, tile.top, right, tile.top + HEADING_HEIGHT);
        canvas.text(&MEDIUM_14, &self.words.lead, heading, Color::WHITE.faded(opacity));
        if let Some(app) = &self.words.app {
            let app_left = left + self.words.lead_width + TITLE_GAP;
            canvas.text(&REGULAR_14, app, Rect { left: app_left, ..heading }, CARD_APP_NAME_TEXT.faded(opacity));
        }
        for (index, line) in self.words.body.iter().enumerate() {
            let top = heading.bottom + HEADING_GAP + index as f32 * BODY_LINE_HEIGHT;
            canvas.text(
                &REGULAR_13,
                line,
                Rect::new(left, top, right, top + BODY_LINE_HEIGHT),
                CARD_BODY_TEXT.faded(opacity),
            );
        }
        if shows_buttons {
            let (right, top) = self.buttons_origin(area);
            if reply.is_typing() {
                reply.draw(canvas, tile.left, right, top, opacity);
            } else {
                self.buttons.draw(canvas, right, top, 1.0, opacity);
            }
        }
    }
}

fn draw_tile(canvas: &Canvas, item: &Item, tile: Rect, opacity: f32) {
    let logo = item.icon.as_deref().and_then(|icon| {
        let key = ("notification-logo", item.app_id.as_str());
        canvas.cached_image(key, || ImageData::new(icon.width, icon.height, icon.bgra_premultiplied.clone()))
    });
    match logo {
        Some(brush) => canvas.image(&brush, tile, TILE_RADIUS, opacity),
        None => {
            canvas.fill_rounded_rect(tile, TILE_RADIUS, app_color(&item.app).faded(opacity));
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
        assert_eq!(words.body, vec!["New message".to_owned()]);
    }

    #[test]
    fn the_buttons_sit_below_the_text_inside_the_card() {
        let fonts = Fonts::new(1.0).unwrap();
        let mut card = Card::new(&item("Rafi", "Deploy is green."), true, &fonts);
        let area = Rect::new(0.0, 0.0, WIDTH, card.height(true));
        let (right, top) = card.buttons_origin(area);
        assert_eq!(right, WIDTH - PADDING);
        assert!(top + BANNER_BUTTONS_HEIGHT <= area.bottom - BOTTOM_PADDING + 0.01);
        let open = Point::new(right - 10.0, area.bottom - BOTTOM_PADDING - 10.0);
        card.on_pointer(PointerEvent::Pressed(open), area);
        assert_eq!(card.on_pointer(PointerEvent::Released(open), area).0, Some(BannerButton::Open));
    }

    #[test]
    fn a_long_app_name_keeps_both_buttons_inside_the_card_padding() {
        let fonts = Fonts::new(1.0).unwrap();
        let mut long = item("Rafi", "Deploy is green.");
        long.app = "A very long application name from a store package".to_owned();
        let card = Card::new(&long, true, &fonts);
        let area = Rect::new(0.0, 0.0, WIDTH, card.height(true));
        let (right, top) = card.buttons_origin(area);
        let dismiss = card.buttons.rects(right, top, 1.0).next().unwrap().1;
        assert!(dismiss.left >= PADDING, "Dismiss starts at {}", dismiss.left);
    }

    #[test]
    fn the_message_keeps_room_for_about_thirty_five_characters_a_line() {
        let fonts = Fonts::new(1.0).unwrap();
        let words = Words::new(&item("Rafi", "Can you check the hover state on the"), true, &fonts);
        assert_eq!(words.body.len(), 1);
    }

    #[test]
    fn a_card_that_can_be_answered_reads_tutup_balas_buka_in_indonesian() {
        let fonts = Fonts::new(1.0).unwrap();
        store::language::with_override(store::language::Language::Indonesian, || {
            let mut card = Card::new(&item("Rafi", "hi"), true, &fonts);
            card.offer_reply(&fonts);
            let buttons = &card.buttons;
            assert_eq!(
                (buttons.dismiss_label(), buttons.reply_label(), buttons.open_label()),
                ("Tutup", "Balas", "Buka")
            );
        });
    }

    #[test]
    fn the_three_buttons_fit_the_notification_card_in_both_languages() {
        let fonts = Fonts::new(1.0).unwrap();
        for language in [store::language::Language::English, store::language::Language::Indonesian] {
            store::language::with_override(language, || {
                let mut card = Card::new(&item("Rafi", "hi"), true, &fonts);
                card.offer_reply(&fonts);
                // A notification card is 340 DIP wide with 20 DIP of padding on each side.
                let rects: Vec<_> = card.buttons.rects(300.0, 0.0, 1.0).collect();
                let leftmost = rects.iter().map(|(_, rect)| rect.left).fold(f32::MAX, f32::min);
                assert!(leftmost >= 0.0, "{language:?}: the buttons need {} px of 300", 300.0 - leftmost);
            });
        }
    }
}
