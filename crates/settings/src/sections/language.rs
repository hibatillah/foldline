//! The language cards on the General page: Automatic, English and Bahasa Indonesia.

use gpui::{AnyElement, App, SharedString, prelude::*, px};
use gpui_component::v_flex;
use store::{
    Settings,
    language::{LANGUAGE_KEY, Language, LanguageChoice},
};

use crate::{
    card,
    i18n::{apply_language, tr},
    layout::{Item, row_description},
};

/// Between the description lines and the cards.
const CARDS_GAP: f32 = 10.0;

pub struct Card {
    pub choice: LanguageChoice,
    label: fn() -> &'static str,
}

pub const CHOICES: [Card; 3] = [
    Card { choice: LanguageChoice::Auto, label: || tr!("language.auto") },
    Card { choice: LanguageChoice::English, label: || tr!("language.english") },
    Card { choice: LanguageChoice::Indonesian, label: || tr!("language.indonesian") },
];

/// What the Automatic card says: the Windows language it follows now.
pub fn auto_description(system: Language) -> &'static str {
    match system {
        Language::English => tr!("language.auto_english"),
        Language::Indonesian => tr!("language.auto_indonesian"),
    }
}

fn card_for(card: &'static Card, selected: bool, cx: &App) -> AnyElement {
    let choice = card.choice;
    card::frame(SharedString::from(format!("{LANGUAGE_KEY}-{}", choice.setting_word())), selected, cx)
        .cursor_pointer()
        .on_click(move |_event, _window, cx| apply_language(choice, cx))
        .child(card::header((card.label)(), None, selected, cx))
        .into_any_element()
}

/// What the section says above its cards, and what Automatic follows now; each card holds only a radio and its name.
pub fn description_lines() -> [SharedString; 2] {
    [tr!("general.language_description").into(), auto_description(store::language::system_language()).into()]
}

pub fn item() -> Item {
    Item::picture_row(|_window, cx| {
        let picked = LanguageChoice::from_setting(Settings::load().get_string(LANGUAGE_KEY));
        let cards = CHOICES.iter().map(|card| card_for(card, card.choice == picked, cx)).collect();
        let [description, automatic] = description_lines();
        v_flex()
            .gap(px(CARDS_GAP))
            .child(v_flex().gap(px(1.0)).child(row_description(description, cx)).child(row_description(automatic, cx)))
            .child(card::grid(3, cards))
            .into_any_element()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use store::language::LanguageChoice;

    #[test]
    fn each_card_writes_the_word_the_notch_reads() {
        let words: Vec<&str> = CHOICES.iter().map(|card| card.choice.setting_word()).collect();
        assert_eq!(words, ["auto", "en", "id"]);
        for card in CHOICES {
            assert_eq!(LanguageChoice::from_setting(Some(card.choice.setting_word())), card.choice);
        }
    }

    #[test]
    fn the_description_lines_say_what_the_section_is_for_then_what_automatic_follows() {
        let [purpose, automatic] = description_lines();
        assert_eq!(purpose, tr!("general.language_description"));
        assert_eq!(automatic, auto_description(store::language::system_language()));
    }

    #[test]
    fn the_automatic_card_names_the_windows_language_it_follows() {
        assert_eq!(auto_description(store::language::Language::English), tr!("language.auto_english"));
        assert_eq!(auto_description(store::language::Language::Indonesian), tr!("language.auto_indonesian"));
    }
}
