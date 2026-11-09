use gpui::{App, SharedString, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, Icon, IconName, Sizable as _, h_flex, v_flex};
use store::Settings;

use crate::{
    card, field,
    layout::{Group, Item, Page},
    picture::{Backdrop, picture, pill_for},
};

struct Choice {
    word: &'static str,
    label: &'static str,
    description: &'static str,
}

// The words `Showing::parse` accepts in `src/host/visibility.rs`; a test compares them, the labels are this window's own.
const CHOICES: [Choice; 4] = [
    Choice { word: "hidden", label: "0 · Hidden", description: "Nothing is drawn. Not even messages." },
    Choice {
        word: "activities",
        label: "1 · Activities",
        description: "On-demand messages only, such as volume. Then it hides again.",
    },
    Choice {
        word: "lyrics",
        label: "2 · Lyrics",
        description: "Lyrics while they play, plus everything from level 1.",
    },
    Choice {
        word: "music",
        label: "3 · Always visible",
        description: "The idle pill with clock and battery, plus everything from levels 1 and 2.",
    },
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

const INTRO: &str = "Behavior works like a log level. You pick how much the notch is allowed to show, and every level also shows everything below it. Level 2 does not mean lyrics only: it shows lyrics and messages. Level 3 shows all of it.";
const MATRIX_COLUMNS: [&str; 3] = ["On-demand messages", "Lyrics", "Idle pill"];
const LABEL_WIDTH: f32 = 170.0;

/// Whether a level (its index in `CHOICES`) shows a column: column 0 needs level 1, column 1 level 2, column 2 level 3.
fn level_shows(level: usize, column: usize) -> bool {
    level > column
}

/// One matrix row: the level label, then a check or a dash for each column.
fn matrix_row(level: usize, cx: &App) -> gpui::Div {
    let cell = move |column: usize| {
        let mark = if level_shows(level, column) {
            Icon::new(IconName::Check).small().text_color(cx.theme().primary).into_any_element()
        } else {
            div().w(px(10.0)).h(px(2.0)).rounded_full().bg(cx.theme().muted_foreground.opacity(0.4)).into_any_element()
        };
        div().flex_1().flex().justify_center().child(mark)
    };
    h_flex()
        .h(px(32.0))
        .items_center()
        .child(div().w(px(LABEL_WIDTH)).text_sm().child(CHOICES[level].label))
        .children((0..MATRIX_COLUMNS.len()).map(cell))
}

/// The intro sentence and the read-only matrix of what each level shows.
fn levels_item() -> Item {
    Item::custom(move |_window, cx| {
        let muted = cx.theme().muted_foreground;
        let header = h_flex().child(div().w(px(LABEL_WIDTH))).children(MATRIX_COLUMNS.iter().map(move |name| {
            div().flex_1().flex().justify_center().text_size(px(12.0)).text_color(muted).child(*name)
        }));
        v_flex()
            .gap_3()
            .child(div().text_sm().text_color(muted).child(INTRO))
            .child(v_flex().child(header).children((0..CHOICES.len()).map(|level| matrix_row(level, cx))))
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
        "Choose what the notch shows, as a level, while a maximized or full-screen window is in front of it.",
        vec![
            Group::titled("Levels", vec![levels_item()]),
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
    fn each_level_shows_its_own_column_and_every_one_below() {
        let rows: Vec<Vec<bool>> =
            (0..4).map(|level| (0..3).map(|column| level_shows(level, column)).collect()).collect();
        assert_eq!(
            rows,
            [vec![false, false, false], vec![true, false, false], vec![true, true, false], vec![true, true, true],]
        );
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
