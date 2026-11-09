//! The two Asr cards: the standard rule and the Hanafi one, each drawing a stick with its shadow.

use gpui::{AnyElement, App, SharedString, prelude::*};
use gpui_component::v_flex;
use store::Settings;

use super::{HANAFI_KEY, pictures};
use crate::{card, field, layout::Item};

struct Rule {
    hanafi: bool,
    label: &'static str,
    schools: &'static str,
    shadow_in_heights: f32,
    caption: &'static str,
}

const RULES: [Rule; 2] = [
    Rule {
        hanafi: false,
        label: "Standard",
        schools: "Shafi'i, Maliki and Hanbali",
        shadow_in_heights: 1.0,
        caption: "Shadow = 1× its height",
    },
    Rule {
        hanafi: true,
        label: "Hanafi",
        schools: "Hanafi school: a later Asr",
        shadow_in_heights: 2.0,
        caption: "Shadow = 2× its height",
    },
];

fn rule_card(rule: &'static Rule, selected: bool, cx: &App) -> AnyElement {
    let hanafi = rule.hanafi;
    card::frame(SharedString::from(format!("{HANAFI_KEY}-{}", rule.label)), selected, 10.0, 10.0, cx)
        .cursor_pointer()
        .on_click(move |_event, _window, cx| field::write(HANAFI_KEY, field::on_off(hanafi), cx))
        .child(card::header(rule.label, Some(rule.schools), selected, cx))
        .child(pictures::stick_and_shadow(rule.shadow_in_heights, rule.caption))
        .into_any_element()
}

pub fn item() -> Item {
    Item::custom(|_window, cx| {
        let hanafi = Settings::load().get_bool(HANAFI_KEY, false);
        let cards = RULES.iter().map(|rule| rule_card(rule, rule.hanafi == hanafi, cx)).collect();
        v_flex()
            .gap_3()
            .child(card::described(
                "When Asr begins",
                "Standard or Hanafi: the length of an object's shadow that marks Asr.",
                cx,
            ))
            .child(card::grid(2, cards))
            .into_any_element()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_card_writes_the_word_the_notch_reads() {
        assert_eq!(field::on_off(RULES[0].hanafi), "off");
        assert_eq!(field::on_off(RULES[1].hanafi), "on");
    }

    #[test]
    fn the_notch_treats_on_as_the_hanafi_rule() {
        let source =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../notch/src/pages/shalat/config.rs"))
                .expect("notch config source");
        assert!(source.contains("\"shalat.hanafi\""));
        assert!(source.contains("Some(\"on\") => true"));
    }
}
