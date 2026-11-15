//! The two banner cards under "Show notification text": the banner with the message, and the same banner reading
//! "New message"; a click writes `notifications.show_text`.

use gpui::{AnyElement, App, SharedString, div, prelude::*, px};
use gpui_component::v_flex;
use store::Settings;

use crate::{card, field, i18n::tr, illustration, layout::Item, picture::color};

const SHOW_TEXT_KEY: &str = "notifications.show_text";
const PICTURE_HEIGHT: f32 = 70.0;
const BANNER_HEIGHT: f32 = 40.0;
const APP_ICON: f32 = 18.0;
/// The sample app's icon color: an indigo square, no real app's logo.
const APP_COLOR: u32 = 0x5865f2;
const TEXT_SIZE: f32 = 11.0;
const LINE_HEIGHT: f32 = 14.0;
const TEXT_WIDTH: f32 = 190.0;

/// The banner as the notch draws it: the app icon, the sender and the line under it.
fn banner(line: &'static str) -> AnyElement {
    let text = v_flex()
        .max_w(px(TEXT_WIDTH))
        .text_size(px(TEXT_SIZE))
        .line_height(px(LINE_HEIGHT))
        .child(
            gpui_component::h_flex()
                .gap(px(3.0))
                .child(div().font_weight(gpui::FontWeight::SEMIBOLD).child(tr!("general.notification_sample_name")))
                .child(div().child(tr!("general.notification_sample_app"))),
        )
        .child(div().text_color(color(0xaaaaaa)).truncate().child(line));
    let content = gpui_component::h_flex()
        .gap(px(10.0))
        .items_center()
        .child(div().flex_none().size(px(APP_ICON)).rounded(px(5.0)).bg(color(APP_COLOR)))
        .child(text);
    illustration::scene(PICTURE_HEIGHT)
        .child(illustration::notch(content, BANNER_HEIGHT, 12.0, 14.0))
        .into_any_element()
}

fn option_card(shows_text: bool, selected: bool, cx: &App) -> AnyElement {
    let (line, title) = if shows_text {
        (tr!("general.notification_sample_message"), tr!("general.notification_show_message"))
    } else {
        (tr!("general.notification_sample_hidden"), tr!("general.notification_hide_message"))
    };
    let word = field::on_off(shows_text);
    card::frame(SharedString::from(format!("{SHOW_TEXT_KEY}-{word}")), selected, cx)
        .cursor_pointer()
        .on_click(move |_event, _window, cx| {
            if Settings::load().get_bool(SHOW_TEXT_KEY, true) != shows_text {
                field::write(SHOW_TEXT_KEY, word, cx);
            }
        })
        .child(banner(line))
        .child(card::header(title, None, selected, cx))
        .into_any_element()
}

/// Both cards in one row of the Notifications block; the selected one follows the switch above it.
pub fn item() -> Item {
    Item::picture_row(|_window, cx| {
        let shows_text = Settings::load().get_bool(SHOW_TEXT_KEY, true);
        let cards = [true, false].into_iter().map(|card_shows| option_card(card_shows, card_shows == shows_text, cx));
        card::grid(2, cards.collect()).into_any_element()
    })
}
