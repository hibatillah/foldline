use crate::i18n::tr;
use gpui::{App, SharedString, prelude::*};
use store::Settings;

use crate::{
    card, field,
    layout::{Group, Item, Page},
    picture::{Backdrop, picture, pill_for},
};

struct Choice {
    word: &'static str,
    label: fn() -> &'static str,
    description: fn() -> &'static str,
}

// The words `Showing::parse` accepts in `src/host/visibility.rs`; a test compares them, the labels are this window's own.
const CHOICES: [Choice; 4] = [
    Choice { word: "hidden", label: || tr!("behavior.level0"), description: || tr!("behavior.level0_note") },
    Choice { word: "activities", label: || tr!("behavior.level1"), description: || tr!("behavior.level1_note") },
    Choice { word: "lyrics", label: || tr!("behavior.level2"), description: || tr!("behavior.level2_note") },
    Choice { word: "music", label: || tr!("behavior.level3"), description: || tr!("behavior.level3_note") },
];

const MAXIMIZED_KEY: &str = "notch.maximized";
const FULLSCREEN_KEY: &str = "notch.fullscreen";
// The keys `src/host/visibility.rs` reads; a test compares them.
const HOVER_MAXIMIZED_KEY: &str = "notch.hover_maximized";
const HOVER_FULLSCREEN_KEY: &str = "notch.hover_fullscreen";
const DEFAULT_MAXIMIZED: &str = "lyrics";
const DEFAULT_FULLSCREEN: &str = "hidden";

/// The stored word when the notch would accept it, else the documented default.
pub fn shown_word(settings: &Settings, key: &str, default: &'static str) -> &'static str {
    settings
        .get_string(key)
        .and_then(|stored| CHOICES.iter().find(|choice| choice.word == stored))
        .map_or(default, |choice| choice.word)
}

/// One picture card, the picture above the choice and its sentence; a click saves the choice, and the selected card gets
/// an accent check, a tinted fill and an accent outline.
fn choice_card(
    key: &'static str,
    backdrop: Backdrop,
    choice: &'static Choice,
    selected: bool,
    cx: &App,
) -> impl IntoElement {
    let word = choice.word;
    card::frame(SharedString::from(format!("{key}-{word}")), selected, cx)
        .cursor_pointer()
        .on_click(move |_event, _window, cx| field::write(key, word, cx))
        .child(picture(backdrop, pill_for(word)))
        .child(card::checked_header((choice.label)(), (choice.description)(), selected, cx))
}

/// The four picture cards in one row, drawn on the page above the section's switch row.
fn choice_item(key: &'static str, default: &'static str, backdrop: Backdrop) -> Item {
    Item::custom(move |_window, cx| {
        let shown = shown_word(&Settings::load(), key, default);
        let cards: Vec<_> = CHOICES
            .iter()
            .map(|choice| choice_card(key, backdrop, choice, choice.word == shown, cx).into_any_element())
            .collect();
        card::grid(card::COLUMNS, cards).into_any_element()
    })
    .outside_block()
}

fn situation(
    heading: &'static str,
    description: &'static str,
    key: &'static str,
    default: &'static str,
    backdrop: Backdrop,
    open_on_hover_key: &'static str,
    open_on_hover_description: &'static str,
) -> Group {
    Group::titled(
        heading,
        vec![
            choice_item(key, default, backdrop),
            Item::new(tr!("behavior.open_on_hover"), field::toggle(open_on_hover_key, true))
                .description(open_on_hover_description),
        ],
    )
    .description(description)
}

pub fn page() -> Page {
    Page::plain(
        tr!("behavior.title"),
        vec![
            situation(
                tr!("behavior.over_maximized"),
                tr!("behavior.over_maximized_note"),
                MAXIMIZED_KEY,
                DEFAULT_MAXIMIZED,
                Backdrop::MaximizedWindow,
                HOVER_MAXIMIZED_KEY,
                tr!("behavior.open_on_hover_maximized_note"),
            ),
            situation(
                tr!("behavior.over_fullscreen"),
                tr!("behavior.over_fullscreen_note"),
                FULLSCREEN_KEY,
                DEFAULT_FULLSCREEN,
                Backdrop::FullScreenApp,
                HOVER_FULLSCREEN_KEY,
                tr!("behavior.open_on_hover_fullscreen_note"),
            ),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings_with(name: &str, contents: &str) -> (Settings, std::path::PathBuf) {
        let path = std::env::temp_dir().join(format!("foldline-settings-behavior-{}-{name}.txt", std::process::id()));
        std::fs::write(&path, contents).unwrap();
        (Settings::at(Some(path.clone())), path)
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
    fn the_open_on_hover_keys_match_the_ones_the_host_reads() {
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
