use crate::i18n::tr;
use gpui::{AnyElement, App, SharedString, Window, div, prelude::*};
use gpui_component::{ActiveTheme as _, v_flex};
use store::Settings;

use crate::{
    card,
    field::{self, Kind},
    layout::{self, Group, Item, Page},
};

const VIEW_KEY: &str = "calendar.view";

struct View {
    word: &'static str,
    label: fn() -> &'static str,
    description: fn() -> &'static str,
}

const VIEWS: [View; 2] = [
    View { word: "month", label: || tr!("calendar.view_month"), description: || tr!("calendar.view_month_note") },
    View { word: "week", label: || tr!("calendar.view_week"), description: || tr!("calendar.view_week_note") },
];

// Wording follows Google's "Sync your calendar with computer programs" page (support.google.com/calendar/answer/37648).
fn google_steps() -> [&'static str; 5] {
    [
        tr!("calendar.google_step1"),
        tr!("calendar.google_step2"),
        tr!("calendar.google_step3"),
        tr!("calendar.google_step4"),
        tr!("calendar.google_step5"),
    ]
}

fn google_notes() -> [&'static str; 2] {
    [tr!("calendar.google_note1"), tr!("calendar.google_note2")]
}

pub fn page() -> Page {
    Page::new(
        tr!("calendar.title"),
        tr!("calendar.description"),
        vec![
            Group::titled(
                tr!("calendar.group_google"),
                vec![
                    Item::new(
                        tr!("calendar.address"),
                        field::input(
                            "calendar.ics_url",
                            Kind::Secret { width: 280.0 },
                            tr!("calendar.address_placeholder"),
                        ),
                    )
                    .description(tr!("calendar.address_note")),
                    Item::new(
                        tr!("calendar.refresh"),
                        field::input(
                            "calendar.refresh_minutes",
                            Kind::WholeNumber { default: 15, range: 1..=1440 },
                            "15",
                        ),
                    )
                    .description(tr!("calendar.refresh_note")),
                ],
            )
            .icon(crate::icons::CALENDAR),
            Group::loose(vec![Item::custom(guide)]),
            Group::loose(vec![view_item()]),
        ],
    )
}

fn guide(window: &mut Window, cx: &mut App) -> AnyElement {
    let steps =
        google_steps().into_iter().enumerate().map(|(index, step)| div().child(format!("{}. {step}", index + 1)));
    let notes = google_notes().into_iter().map(|note| div().text_color(cx.theme().muted_foreground).child(note));
    let body = v_flex().gap_1().text_sm().children(steps).children(notes).into_any_element();
    layout::accordion("google-steps", false, vec![(SharedString::from(tr!("calendar.guide_title")), body)], window, cx)
}

fn view_card(view: &'static View, selected: bool, cx: &App) -> AnyElement {
    let word = view.word;
    card::frame(SharedString::from(format!("{VIEW_KEY}-{word}")), selected, 10.0, 10.0, cx)
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

    #[test]
    fn each_card_writes_a_word_the_notch_reads() {
        let words: Vec<&str> = VIEWS.iter().map(|view| view.word).collect();
        assert_eq!(words, ["month", "week"]);
        let source =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../notch/src/pages/calendar/view.rs"))
                .expect("notch view source");
        assert!(source.contains("Some(\"month\")") && source.contains("Some(\"week\")"));
    }
}
