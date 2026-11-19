//! "Show the notch on": one picture card per `notch.monitor` word, each drawing three screens with the notch on the one
//! that word picks.

use gpui::{AnyElement, App, SharedString, div, prelude::*, px, rgba};
use gpui_component::h_flex;
use store::Settings;

use crate::{card, field, i18n::tr, illustration, layout::Item, picture::color};

const MONITOR_KEY: &str = "notch.monitor";
const DEFAULT_MONITOR: &str = "main";
const PICTURE_HEIGHT: f32 = 70.0;
const SCREEN_WIDTH: f32 = 46.0;
const SCREEN_HEIGHT: f32 = 29.0;
const SCREEN_GAP: f32 = 8.0;
const NOTCH_WIDTH: f32 = 14.0;
const NOTCH_HEIGHT: f32 = 4.0;

/// Which of the three drawn screens carries the notch, and what marks why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mark {
    /// The middle screen, the main one, with a small bar under it for the taskbar.
    Main,
    /// The right screen, which holds the window in front.
    WindowInFront,
    /// The left screen, outlined in the accent: the one you picked.
    Picked,
}

pub struct MonitorChoice {
    pub word: &'static str,
    label: fn() -> &'static str,
    description: fn() -> &'static str,
    mark: Mark,
}

const MONITOR_CHOICES: [MonitorChoice; 3] = [
    MonitorChoice {
        word: "main",
        label: || tr!("general.monitor_main"),
        description: || tr!("general.monitor_main_note"),
        mark: Mark::Main,
    },
    MonitorChoice {
        word: "focused",
        label: || tr!("general.monitor_focused"),
        description: || tr!("general.monitor_focused_note"),
        mark: Mark::WindowInFront,
    },
    MonitorChoice {
        word: "chosen",
        label: || tr!("general.monitor_chosen"),
        description: || tr!("general.monitor_chosen_note"),
        mark: Mark::Picked,
    },
];

/// Whether "A chosen screen" is selected, which brings the screen picker with it.
pub(super) fn chosen_is_selected(settings: &Settings) -> bool {
    shown_monitor_word(settings) == "chosen"
}

/// The stored word when it is one of the choices, else the default (the notch reads any other word as `main`).
fn shown_monitor_word(settings: &Settings) -> &'static str {
    let stored = settings.get_string(MONITOR_KEY);
    MONITOR_CHOICES.iter().find(|choice| Some(choice.word) == stored).map_or(DEFAULT_MONITOR, |choice| choice.word)
}

/// The position (0 left, 1 middle, 2 right) of the screen that carries the notch.
fn notch_position(mark: Mark) -> usize {
    match mark {
        Mark::Picked => 0,
        Mark::Main => 1,
        Mark::WindowInFront => 2,
    }
}

fn drawn_screen(position: usize, mark: Mark, cx: &App) -> AnyElement {
    let accent = crate::palette::accent(cx);
    let has_notch = notch_position(mark) == position;
    let outlined = has_notch && mark == Mark::Picked;
    let screen = div()
        .relative()
        .w(px(SCREEN_WIDTH))
        .h(px(SCREEN_HEIGHT))
        .rounded(px(4.0))
        .border(px(if outlined { 1.5 } else { 1.0 }))
        .border_color(if outlined { accent } else { rgba(0xffffff38).into() })
        .bg(color(0x26304a))
        .overflow_hidden()
        .when(position == 1, |screen| {
            screen.child(div().absolute().left_0().right_0().bottom_0().h(px(4.0)).bg(rgba(0x0000008c)))
        })
        .when(mark == Mark::WindowInFront && position == 2, |screen| {
            screen.child(
                div()
                    .absolute()
                    .left(px(6.0))
                    .right(px(6.0))
                    .top(px(9.0))
                    .bottom(px(4.0))
                    .rounded(px(2.0))
                    .bg(accent.opacity(0.45)),
            )
        })
        .when(has_notch, |screen| {
            screen.child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .flex()
                    .justify_center()
                    .child(div().w(px(NOTCH_WIDTH)).h(px(NOTCH_HEIGHT)).rounded_b(px(2.0)).bg(color(0x000000))),
            )
        });
    let stand = div().w(px(10.0)).h(px(4.0)).bg(rgba(0xffffff26));
    gpui_component::v_flex().items_center().child(screen).child(stand).into_any_element()
}

fn picture(mark: Mark, cx: &App) -> AnyElement {
    illustration::scene(PICTURE_HEIGHT)
        .flex()
        .items_center()
        .justify_center()
        .child(
            h_flex().gap(px(SCREEN_GAP)).items_end().children((0..3).map(|position| drawn_screen(position, mark, cx))),
        )
        .into_any_element()
}

fn choice_card(choice: &'static MonitorChoice, selected: bool, cx: &App) -> AnyElement {
    let word = choice.word;
    card::frame(SharedString::from(format!("{MONITOR_KEY}-{word}")), selected, cx)
        .cursor_pointer()
        .on_click(move |_event, _window, cx| {
            if Settings::load().get_string(MONITOR_KEY) != Some(word) {
                field::write(MONITOR_KEY, word, cx);
            }
        })
        .child(picture(choice.mark, cx))
        .child(card::header((choice.label)(), Some((choice.description)()), selected, cx))
        .into_any_element()
}

/// The three cards in one row of the section's block.
pub fn item() -> Item {
    Item::picture_row(|_window, cx| {
        let shown = shown_monitor_word(&Settings::load());
        let cards = MONITOR_CHOICES.iter().map(|choice| choice_card(choice, choice.word == shown, cx)).collect();
        card::grid(3, cards).into_any_element()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings_with(name: &str, text: &str) -> Settings {
        let path = std::env::temp_dir().join(format!("foldline-settings-monitor-{name}-{}.txt", std::process::id()));
        std::fs::write(&path, text).unwrap();
        let settings = Settings::at(Some(path.clone()));
        let _ = std::fs::remove_file(&path);
        settings
    }

    #[test]
    fn the_cards_offer_main_focused_and_chosen_in_that_order() {
        let words: Vec<&str> = MONITOR_CHOICES.iter().map(|choice| choice.word).collect();
        assert_eq!(words, ["main", "focused", "chosen"]);
    }

    #[test]
    fn a_missing_or_unknown_monitor_word_shows_main() {
        assert_eq!(shown_monitor_word(&settings_with("unknown", "notch.monitor=banana\n")), "main");
        assert_eq!(shown_monitor_word(&settings_with("focused", "notch.monitor=focused\n")), "focused");
        assert_eq!(shown_monitor_word(&settings_with("missing", "")), "main");
    }

    #[test]
    fn the_screen_picker_shows_only_while_a_chosen_screen_is_selected() {
        assert!(chosen_is_selected(&settings_with("chosen", "notch.monitor=chosen\n")));
        assert!(!chosen_is_selected(&settings_with("not-chosen", "notch.monitor=focused\n")));
    }

    #[test]
    fn each_card_draws_the_notch_on_a_different_screen() {
        let mut positions: Vec<usize> = MONITOR_CHOICES.iter().map(|choice| notch_position(choice.mark)).collect();
        positions.sort_unstable();
        positions.dedup();
        assert_eq!(positions.len(), MONITOR_CHOICES.len());
    }
}
