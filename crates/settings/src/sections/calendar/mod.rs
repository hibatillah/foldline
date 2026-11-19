//! The Calendar page: the month or week view, the address and refresh rows, and the collapsible help
//! that finds the address.

mod steps;

use std::rc::Rc;

use gpui::{AnyElement, App, SharedString, prelude::*, px};
use gpui_component::{h_flex, v_flex};
use store::Settings;

use super::about::widgets::{Handler, pill};
use crate::{
    card, chips,
    field::{self, Kind},
    i18n::tr,
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

pub fn page() -> Page {
    Page::plain(
        tr!("calendar.title"),
        vec![
            Group::loose(vec![view_item()]),
            Group::titled(tr!("calendar.group_google"), vec![address_item(), refresh_item()])
                .description(tr!("calendar.group_google_note")),
            Group::loose(vec![steps::item()]),
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
            .child(pill("calendar-reset", tr!("calendar.reset"), None, reset.clone(), cx))
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
    use crate::field::stored_whole_number;

    fn settings_with(text: &str, name: &str) -> Settings {
        let path = std::env::temp_dir().join(format!("foldline-settings-calendar-{name}-{}.txt", std::process::id()));
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
        assert_eq!(stored_whole_number(&settings_with("", "missing"), REFRESH_KEY, REFRESH_DEFAULT), 15);
    }

    #[test]
    fn a_stored_refresh_is_shown_as_stored() {
        assert_eq!(
            stored_whole_number(
                &settings_with("calendar.refresh_minutes=45\n", "stored"),
                REFRESH_KEY,
                REFRESH_DEFAULT
            ),
            45
        );
    }

    #[test]
    fn every_preset_lies_in_the_refresh_range() {
        assert!(REFRESH_PRESETS.iter().all(|minutes| REFRESH_RANGE.contains(minutes)));
    }
}
