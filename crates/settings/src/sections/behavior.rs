use gpui::{App, SharedString, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, v_flex};
use store::Settings;

use crate::{
    card, field,
    layout::{Group, Item, Page},
    picture::{Backdrop, picture, pill_for},
};

pub struct Choice {
    pub word: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

// The words `Showing::parse` accepts in `src/host/visibility.rs`; a test compares them, the labels are this window's own.
pub const CHOICES: [Choice; 4] = [
    Choice {
        word: "hidden",
        label: "Hidden",
        description: "The notch hides, even for notification banners and volume changes.",
    },
    Choice {
        word: "activities",
        label: "Activities only",
        description: "The notch shows only notification banners, volume changes and badges.",
    },
    Choice {
        word: "lyrics",
        label: "Lyrics only",
        description: "The notch shows only while a lyric line is active, and for activities. Otherwise it hides.",
    },
    Choice {
        word: "music",
        label: "Always visible",
        description: "The notch always shows, with the idle clock when nothing is playing.",
    },
];

pub const MAXIMIZED_KEY: &str = "notch.maximized";
pub const FULLSCREEN_KEY: &str = "notch.fullscreen";
// The keys `src/host/visibility.rs` reads; a test compares them.
pub const HOVER_MAXIMIZED_KEY: &str = "notch.hover_maximized";
pub const HOVER_FULLSCREEN_KEY: &str = "notch.hover_fullscreen";
const DEFAULT_MAXIMIZED: &str = "lyrics";
const DEFAULT_FULLSCREEN: &str = "hidden";

/// The stored word when the notch would accept it, else the documented default.
pub fn shown_word(settings: &Settings, key: &str, default: &'static str) -> &'static str {
    settings
        .get_string(key)
        .and_then(|stored| CHOICES.iter().find(|choice| choice.word == stored))
        .map_or(default, |choice| choice.word)
}

/// One picture card; a click saves the choice, and the selected card gets a filled radio, a lighter fill and an outline.
fn choice_card(
    key: &'static str,
    backdrop: Backdrop,
    choice: &'static Choice,
    selected: bool,
    cx: &App,
) -> impl IntoElement {
    let word = choice.word;
    card::frame(SharedString::from(format!("{key}-{word}")), selected, 10.0, 10.0, cx)
        .cursor_pointer()
        .on_click(move |_event, _window, cx| field::write(key, word, cx))
        .child(card::header(choice.label, None, selected, cx))
        .child(picture(backdrop, pill_for(word)))
}

/// The label, the four picture cards two by two and the sentence of the selected choice.
fn choice_item(
    title: &'static str,
    description: &'static str,
    key: &'static str,
    default: &'static str,
    backdrop: Backdrop,
) -> Item {
    Item::custom(move |_window, cx| {
        let shown = shown_word(&Settings::load(), key, default);
        let cards: Vec<_> = CHOICES
            .iter()
            .map(|choice| choice_card(key, backdrop, choice, choice.word == shown, cx).into_any_element())
            .collect();
        let sentence = CHOICES.iter().find(|choice| choice.word == shown).map_or("", |choice| choice.description);
        v_flex()
            .gap_3()
            .child(
                v_flex()
                    .gap_0p5()
                    .child(div().text_sm().child(title))
                    .child(div().text_sm().text_color(cx.theme().muted_foreground).child(description)),
            )
            .child(v_flex().gap(px(10.0)).child(card::grid(2, cards)).child(
                div().text_size(px(12.0)).line_height(px(16.0)).text_color(cx.theme().muted_foreground).child(sentence),
            ))
            .into_any_element()
    })
}

fn situation(
    heading: &'static str,
    description: &'static str,
    key: &'static str,
    default: &'static str,
    backdrop: Backdrop,
    hover_key: &'static str,
    hover_description: &'static str,
) -> Group {
    Group::titled(
        heading,
        vec![
            choice_item("What the notch shows", description, key, default, backdrop),
            Item::new("Open on hover", field::toggle(hover_key, true)).description(hover_description),
        ],
    )
}

pub fn page() -> Page {
    Page::new(
        "Behavior",
        "Choose what the notch shows while another window is in front.",
        vec![
            situation(
                "Over a maximized window",
                "Applies while a maximized window is in front.",
                MAXIMIZED_KEY,
                DEFAULT_MAXIMIZED,
                Backdrop::MaximizedWindow,
                HOVER_MAXIMIZED_KEY,
                "Pointing at the top edge opens the notch. Off: it never opens over a maximized window.",
            ),
            situation(
                "Over a full-screen app",
                "Applies while a full-screen app or game is in front.",
                FULLSCREEN_KEY,
                DEFAULT_FULLSCREEN,
                Backdrop::FullScreenApp,
                HOVER_FULLSCREEN_KEY,
                "Pointing at the top edge opens the notch. Off: it never opens over a full-screen app.",
            ),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings_with(name: &str, contents: &str) -> (Settings, std::path::PathBuf) {
        let path = std::env::temp_dir().join(format!("notch-settings-behavior-{}-{name}.txt", std::process::id()));
        std::fs::write(&path, contents).unwrap();
        (Settings::at(Some(path.clone())), path)
    }

    #[test]
    fn the_choices_come_in_order() {
        let offered: Vec<(&str, &str)> = CHOICES.iter().map(|choice| (choice.word, choice.label)).collect();
        assert_eq!(
            offered,
            [
                ("hidden", "Hidden"),
                ("activities", "Activities only"),
                ("lyrics", "Lyrics only"),
                ("music", "Always visible"),
            ]
        );
        assert!(CHOICES.iter().all(|choice| !choice.description.is_empty()));
    }

    #[test]
    fn a_missing_or_unknown_word_shows_the_default_for_each_key() {
        let (settings, path) = settings_with("unknown", "notch.maximized=banana\n");
        let _ = std::fs::remove_file(&path);
        assert_eq!(shown_word(&settings, MAXIMIZED_KEY, DEFAULT_MAXIMIZED), "lyrics");
        assert_eq!(shown_word(&settings, FULLSCREEN_KEY, DEFAULT_FULLSCREEN), "hidden");
    }

    #[test]
    fn a_stored_word_is_shown_as_stored() {
        let (settings, path) = settings_with("stored", "notch.maximized=music\nnotch.fullscreen=activities\n");
        let _ = std::fs::remove_file(&path);
        assert_eq!(shown_word(&settings, MAXIMIZED_KEY, DEFAULT_MAXIMIZED), "music");
        assert_eq!(shown_word(&settings, FULLSCREEN_KEY, DEFAULT_FULLSCREEN), "activities");
    }

    #[test]
    fn the_hover_keys_match_the_ones_the_host_reads() {
        let host = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../notch/src/host/visibility.rs"))
            .expect("host visibility source");
        for key in [HOVER_MAXIMIZED_KEY, HOVER_FULLSCREEN_KEY] {
            assert!(host.contains(&format!("\"{key}\"")), "{key} missing in host");
        }
    }

    #[test]
    fn every_choice_word_is_accepted_by_the_hosts_parser() {
        let host = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../notch/src/host/visibility.rs"))
            .expect("host visibility source");
        for choice in &CHOICES {
            assert!(
                host.contains(&format!("\"{}\" => Some(Showing::", choice.word)),
                "{} missing in host",
                choice.word
            );
        }
    }
}
