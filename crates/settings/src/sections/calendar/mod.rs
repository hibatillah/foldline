//! The Calendar page: how the address reaches the notch, the address and refresh rows, the steps to find the address
//! and the month or week view.

mod flow;
mod steps;

use std::rc::Rc;

use gpui::{AnyElement, App, SharedString, prelude::*, px};
use gpui_component::{h_flex, v_flex};
use store::Settings;

use super::about::widgets::{Handler, pill};
use crate::{
    card, chips,
    field::{self, Kind, stored_whole_number},
    i18n::tr,
    icons,
    info_box::{info_box, warning_box},
    layout::{Group, Item, Page, Render},
    stepper::{self, REGULAR},
};

const VIEW_KEY: &str = "calendar.view";
const LINK_KEY: &str = "calendar.ics_url";
const REFRESH_KEY: &str = "calendar.refresh_minutes";
const REFRESH_DEFAULT: u32 = 15;
const REFRESH_RANGE: std::ops::RangeInclusive<u32> = 1..=1440;
const REFRESH_PRESETS: [u32; 4] = [5, 15, 30, 60];
const REFRESH_STEP: u32 = 5;
const ADDRESS_WIDTH: f32 = 190.0;

struct View {
    word: &'static str,
    label: fn() -> &'static str,
    description: fn() -> &'static str,
}

const VIEWS: [View; 2] = [
    View { word: "month", label: || tr!("calendar.view_month"), description: || tr!("calendar.view_month_note") },
    View { word: "week", label: || tr!("calendar.view_week"), description: || tr!("calendar.view_week_note") },
];

/// How often the notch downloads the calendar, read as the page reads it.
fn refresh_minutes(settings: &Settings) -> u32 {
    stored_whole_number(settings, REFRESH_KEY, REFRESH_DEFAULT)
}

pub fn page() -> Page {
    Page::plain(
        tr!("calendar.title"),
        vec![
            Group::titled(
                tr!("calendar.group_google"),
                vec![Item::picture_row(flow::flow), address_item(), refresh_item()],
            )
            .description(tr!("calendar.group_google_note")),
            Group::titled(tr!("calendar.group_steps"), steps::items()).description(tr!("calendar.group_steps_note")),
            Group::loose(vec![
                Item::custom(|_window, cx| warning_box(icons::WARNING, tr!("calendar.google_note1"), cx)),
                Item::custom(|_window, cx| info_box(icons::ABOUT, tr!("calendar.google_note2"), cx)),
            ]),
            Group::loose(vec![view_item()]),
        ],
    )
}

fn address_item() -> Item {
    let address = field::input(LINK_KEY, Kind::Secret { width: ADDRESS_WIDTH }, tr!("calendar.address_placeholder"));
    let reset: Handler = Rc::new(|_window, cx| field::write(LINK_KEY, "", cx));
    let control: Render = Rc::new(move |window, cx| {
        h_flex()
            .gap(px(8.0))
            .items_center()
            .child(address(window, cx))
            .child(pill("calendar-reset", tr!("calendar.reset"), None, Some(reset.clone()), cx))
            .into_any_element()
    });
    Item::new(tr!("calendar.address"), control)
}

fn refresh_item() -> Item {
    let stepper = stepper::minutes(REFRESH_KEY, REFRESH_DEFAULT, REFRESH_RANGE, REFRESH_STEP, REGULAR);
    let control: Render = Rc::new(move |window, cx| {
        h_flex()
            .gap(px(12.0))
            .items_center()
            .child(chips::presets(REFRESH_KEY, REFRESH_DEFAULT, &REFRESH_PRESETS, cx))
            .child(stepper(window, cx))
            .into_any_element()
    });
    Item::new(tr!("calendar.refresh"), control).description(tr!("calendar.refresh_note"))
}

fn view_card(view: &'static View, selected: bool, cx: &App) -> AnyElement {
    let word = view.word;
    card::frame(SharedString::from(format!("{VIEW_KEY}-{word}")), selected, cx)
        .cursor_pointer()
        .on_click(move |_event, _window, cx| field::write(VIEW_KEY, word, cx))
        .child(card::header((view.label)(), Some((view.description)()), selected, cx))
        .into_any_element()
}

fn view_item() -> Item {
    Item::custom(|_window, cx| {
        let settings = Settings::load();
        // The notch shows the month for a missing or unknown word.
        let shown = if settings.get_string(VIEW_KEY) == Some("week") { "week" } else { "month" };
        let cards = VIEWS.iter().map(|view| view_card(view, view.word == shown, cx)).collect();
        v_flex()
            .gap_3()
            .child(card::described(tr!("calendar.view_title"), tr!("calendar.view_description"), cx))
            .child(card::grid(2, cards))
            .into_any_element()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings_with(text: &str, name: &str) -> Settings {
        let path = std::env::temp_dir().join(format!("notch-settings-calendar-{name}-{}.txt", std::process::id()));
        std::fs::write(&path, text).expect("write settings");
        let settings = Settings::at(Some(path.clone()));
        let _ = std::fs::remove_file(&path);
        settings
    }

    #[test]
    fn each_card_writes_a_word_the_notch_reads() {
        let words: Vec<&str> = VIEWS.iter().map(|view| view.word).collect();
        assert_eq!(words, ["month", "week"]);
        let source =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../notch/src/pages/calendar/view.rs"))
                .expect("notch view source");
        assert!(source.contains("Some(\"month\")") && source.contains("Some(\"week\")"));
    }

    #[test]
    fn the_keys_are_the_ones_the_notch_reads() {
        let source =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../notch/src/pages/calendar/mod.rs"))
                .expect("notch calendar source");
        for key in [LINK_KEY, REFRESH_KEY] {
            assert!(source.contains(&format!("\"{key}\"")), "{key} missing in the notch");
        }
    }

    #[test]
    fn a_missing_refresh_is_fifteen_minutes() {
        assert_eq!(refresh_minutes(&settings_with("", "missing")), 15);
    }

    #[test]
    fn a_stored_refresh_is_shown_as_stored() {
        assert_eq!(refresh_minutes(&settings_with("calendar.refresh_minutes=45\n", "stored")), 45);
    }

    #[test]
    fn every_preset_lies_in_the_refresh_range() {
        assert!(REFRESH_PRESETS.iter().all(|minutes| REFRESH_RANGE.contains(minutes)));
    }
}
