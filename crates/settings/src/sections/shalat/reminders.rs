//! The two reminder cards: each previews the collapsed pill with the reminder and has a switch.

use crate::i18n::tr;
use gpui::{AnyElement, App, SharedString, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, Icon, IconName, h_flex, v_flex};
use store::Settings;

use super::{BADGE_KEY, MESSAGE_KEY};
use crate::{
    card, field,
    layout::{self, Item},
    picture::{color, pill_frame, sunset},
};

const TEAL: u32 = 0x63e6be;
const PICTURE_HEIGHT: f32 = 84.0;

struct Reminder {
    key: &'static str,
    title: fn() -> &'static str,
    description: fn() -> &'static str,
}

// Both are on unless the file says `off`, as the notch reads them.
const REMINDERS: [Reminder; 2] = [
    Reminder {
        key: MESSAGE_KEY,
        title: || tr!("shalat.reminder_message"),
        description: || tr!("shalat.reminder_message_note"),
    },
    Reminder {
        key: BADGE_KEY,
        title: || tr!("shalat.reminder_badge"),
        description: || tr!("shalat.reminder_badge_note"),
    },
];

// What the notch writes: the message title and value, and the collapsed mark, which reads "Asr in 24m".
fn message_text() -> String {
    tr!("shalat.alert_title", prayer = tr!("shalat.asr"))
}

fn badge_text() -> String {
    tr!("shalat.badge_text", prayer = tr!("shalat.asr"), minutes = 24)
}

const MESSAGE_TIME: &str = "15:19";
const IDLE_CLOCK: &str = "15:05";

fn moon() -> Icon {
    Icon::new(IconName::Moon).size(px(14.0)).text_color(color(TEAL))
}

fn message_picture() -> AnyElement {
    let content = h_flex()
        .gap(px(8.0))
        .child(moon())
        .child(div().child(message_text()))
        .child(div().text_color(color(TEAL)).child(MESSAGE_TIME));
    preview(pill_frame(content, 40.0, 20.0, 0.0, 14.0))
}

fn badge_picture() -> AnyElement {
    let content = h_flex()
        .gap(px(24.0))
        .child(div().child(IDLE_CLOCK))
        .child(h_flex().gap(px(4.0)).text_color(color(TEAL)).child(moon()).child(div().child(badge_text())));
    preview(pill_frame(content, 36.0, 14.0, 3.0, 12.0))
}

fn preview(pill: impl IntoElement) -> AnyElement {
    div()
        .relative()
        .w_full()
        .h(px(PICTURE_HEIGHT))
        .rounded(px(6.0))
        .overflow_hidden()
        .child(sunset())
        .child(pill)
        .into_any_element()
}

fn reminder_card(reminder: &'static Reminder, picture: AnyElement, cx: &App) -> AnyElement {
    let on = Settings::load().get_bool(reminder.key, true);
    let switch = field::bare_switch(reminder.key, on, move |enabled, _window, cx| {
        field::write(reminder.key, field::on_off(enabled), cx)
    });
    let text = v_flex()
        .flex_1()
        .min_w_0()
        .gap(px(1.0))
        .child(div().text_size(px(13.0)).font_weight(gpui::FontWeight::MEDIUM).child((reminder.title)()))
        .child(
            div()
                .text_size(px(12.0))
                .line_height(px(16.0))
                .text_color(cx.theme().muted_foreground)
                .child((reminder.description)()),
        );
    card::frame(SharedString::from(format!("card-{}", reminder.key)), on, cx)
        .child(picture)
        .child(h_flex().items_center().gap_3().child(text).child(switch))
        .into_any_element()
}

pub fn item() -> Item {
    Item::custom(|_window, cx| {
        let cards = [message_picture(), badge_picture()]
            .into_iter()
            .zip(REMINDERS.iter())
            .map(|(picture, reminder)| reminder_card(reminder, picture, cx))
            .collect();
        v_flex()
            .gap_3()
            .child(layout::block_heading(tr!("shalat.reminders_title"), None, cx))
            .child(card::grid(2, cards))
            .into_any_element()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notch_source(file: &str) -> String {
        std::fs::read_to_string(format!("{}/../notch/src/pages/shalat/{file}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|error| panic!("notch source {file}: {error}"))
    }

    #[test]
    fn the_reminder_keys_are_the_ones_the_notch_reads() {
        let config = notch_source("config.rs");
        for reminder in &REMINDERS {
            assert!(config.contains(&format!("\"{}\"", reminder.key)), "{} missing in config.rs", reminder.key);
        }
    }

    #[test]
    fn the_previews_use_the_words_the_notch_shows() {
        for key in ["shalat.alert_title", "shalat.badge_text"] {
            assert!(crate::i18n::same_in_the_notch(key), "{key} differs between the two programs");
        }
    }

    #[test]
    fn both_reminders_are_on_by_default() {
        let config = notch_source("config.rs");
        assert!(config.contains("shows_message: is_on(MESSAGE_KEY, true)"));
        assert!(config.contains("shows_badge: is_on(BADGE_KEY, true)"));
    }
}
