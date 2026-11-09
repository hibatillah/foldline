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
    label: &'static str,
    description: &'static str,
}

const VIEWS: [View; 2] = [
    View { word: "month", label: "Month", description: "The whole month. Pick any day to list its events." },
    View { word: "week", label: "Week", description: "Only the current week, on a shorter page." },
];

// Wording follows Google's "Sync your calendar with computer programs" page (support.google.com/calendar/answer/37648).
const GOOGLE_STEPS: [&str; 5] = [
    "Open calendar.google.com on a computer.",
    "Click the gear icon at the top right, then Settings.",
    "Under \"Settings for my calendars\" on the left, click the calendar to show.",
    "Click Integrate calendar.",
    "Copy the link under \"Secret address in iCal format\" and paste it into the Calendar address field.",
];

const GOOGLE_NOTES: [&str; 2] = [
    "The secret address lets anyone who has it read the whole calendar. Keep it to yourself; Reset in the same section replaces it.",
    "A work or school account may hide the address when the admin limits sharing. Ask the admin in that case.",
];

pub fn page() -> Page {
    Page::new(
        "Calendar",
        "Connect your calendar and choose how often the notch downloads it again.",
        vec![
            Group::titled(
                "Google Calendar",
                vec![
                    Item::new(
                        "Calendar address (.ics)",
                        field::input(
                            "calendar.ics_url",
                            Kind::Secret { width: 280.0 },
                            "Paste the secret iCal address",
                        ),
                    )
                    .description("The private iCal link of your calendar."),
                    Item::new(
                        "Refresh (minutes)",
                        field::input(
                            "calendar.refresh_minutes",
                            Kind::WholeNumber { default: 15, range: 1..=1440 },
                            "15",
                        ),
                    )
                    .description("How often the notch downloads it again, 1 to 1440."),
                ],
            )
            .icon(crate::icons::CALENDAR),
            Group::loose(vec![Item::custom(guide)]),
            Group::loose(vec![view_item()]),
        ],
    )
}

fn guide(window: &mut Window, cx: &mut App) -> AnyElement {
    let steps = GOOGLE_STEPS.iter().enumerate().map(|(index, step)| div().child(format!("{}. {step}", index + 1)));
    let notes = GOOGLE_NOTES.iter().map(|note| div().text_color(cx.theme().muted_foreground).child(*note));
    let body = v_flex().gap_1().text_sm().children(steps).children(notes).into_any_element();
    layout::accordion(
        "google-steps",
        false,
        vec![(SharedString::from("How to get the address from Google Calendar"), body)],
        window,
        cx,
    )
}

fn view_card(view: &'static View, selected: bool, cx: &App) -> AnyElement {
    let word = view.word;
    card::frame(SharedString::from(format!("{VIEW_KEY}-{word}")), selected, 10.0, 10.0, cx)
        .cursor_pointer()
        .on_click(move |_event, _window, cx| field::write(VIEW_KEY, word, cx))
        .child(card::header(view.label, Some(view.description), selected, cx))
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
            .child(card::described("Calendar view", "Show the whole month or only the current week.", cx))
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
