use std::ops::RangeInclusive;

use store::Settings;

use crate::i18n::tr;
use crate::{
    field::{self, Kind},
    layout::{Group, Item, Page},
    location, startup,
};

const MONITOR_KEY: &str = "notch.monitor";
const DEFAULT_MONITOR: &str = "main";

pub struct MonitorChoice {
    pub word: &'static str,
    label: fn() -> &'static str,
}

const MONITOR_CHOICES: [MonitorChoice; 3] = [
    MonitorChoice { word: "main", label: || tr!("general.monitor_main") },
    MonitorChoice { word: "focused", label: || tr!("general.monitor_focused") },
    MonitorChoice { word: "chosen", label: || tr!("general.monitor_chosen") },
];

/// Whether the dropdown shows "A chosen screen", which brings the screen list with it.
pub(super) fn chosen_is_selected(settings: &Settings) -> bool {
    shown_monitor_word(settings) == "chosen"
}

/// The stored word when it is one of the choices, else the default (the notch reads any other word as `main`).
fn shown_monitor_word(settings: &Settings) -> String {
    let stored = settings.get_string(MONITOR_KEY);
    MONITOR_CHOICES
        .iter()
        .find(|choice| Some(choice.word) == stored)
        .map_or(DEFAULT_MONITOR, |choice| choice.word)
        .to_owned()
}

fn minutes(
    title: &'static str,
    key: &'static str,
    default: u32,
    range: RangeInclusive<u32>,
    placeholder: &'static str,
) -> Item {
    Item::new(title, field::input(key, Kind::WholeNumber { default, range }, placeholder))
}

pub fn page() -> Page {
    let start_with_windows = field::switch("start-with-windows", startup::is_enabled, |enabled, cx| {
        startup::set_enabled(enabled);
        cx.refresh_windows();
    });
    Page::new(
        tr!("general.title"),
        tr!("general.description"),
        vec![
            Group::titled(tr!("general.language_title"), vec![super::language::item()]),
            Group::titled(
                tr!("general.group_startup"),
                vec![
                    Item::new(tr!("general.start_with_windows"), start_with_windows)
                        .description(tr!("general.start_with_windows_note")),
                    Item::new(tr!("general.hide_when_idle"), field::toggle("notch.hide_when_idle", false))
                        .description(tr!("general.hide_when_idle_note")),
                ],
            ),
            Group::titled(tr!("pause.group"), vec![super::pause::item()]),
            Group::titled(
                tr!("general.group_screen"),
                vec![
                    Item::new(
                        tr!("general.monitor"),
                        field::choice(
                            MONITOR_KEY,
                            160.0,
                            MONITOR_CHOICES
                                .iter()
                                .map(|choice| field::Choice { word: choice.word, label: (choice.label)().into() })
                                .collect(),
                            || shown_monitor_word(&Settings::load()),
                        ),
                    )
                    .description(tr!("general.monitor_note")),
                    super::screen::item(),
                ],
            ),
            Group::titled(
                tr!("general.group_location"),
                vec![
                    Item::new(tr!("general.track_location"), field::toggle(places::TRACK_KEY, true))
                        .description(tr!("general.track_location_note")),
                    Item::new(tr!("general.city"), location::city_search("general-city"))
                        .description(tr!("general.city_note"))
                        .top_aligned(),
                ],
            ),
            Group::titled(
                tr!("general.group_notifications"),
                vec![
                    Item::new(tr!("general.show_notifications"), field::toggle("activity.notifications", true))
                        .description(tr!("general.show_notifications_note")),
                    Item::new(tr!("general.show_notification_text"), field::toggle("notifications.show_text", true))
                        .description(tr!("general.show_notification_text_note")),
                    Item::new(tr!("general.device_battery"), field::toggle("activity.device_battery", true))
                        .description(tr!("general.device_battery_note")),
                    Item::new(tr!("general.downloads"), field::toggle("activity.downloads", true))
                        .description(tr!("general.downloads_note")),
                ],
            ),
            Group::titled(
                tr!("general.group_windows_popups"),
                vec![
                    Item::new(tr!("general.hide_volume_popup"), field::toggle("notch.hide_windows_volume_popup", true))
                        .description(tr!("general.hide_volume_popup_note")),
                    Item::new(tr!("general.hide_banners"), field::toggle("notch.hide_windows_notifications", true))
                        .description(tr!("general.hide_banners_note")),
                ],
            ),
            Group::titled(
                tr!("general.group_timer"),
                vec![
                    minutes(tr!("general.focus_minutes"), "timer.focus_minutes", 25, 5..=120, "25")
                        .description(tr!("general.focus_minutes_note")),
                    minutes(tr!("general.short_break_minutes"), "timer.short_break_minutes", 5, 1..=120, "5")
                        .description(tr!("general.short_break_note")),
                    minutes(tr!("general.long_break_minutes"), "timer.long_break_minutes", 15, 1..=120, "15")
                        .description(tr!("general.long_break_note")),
                ],
            ),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use store::Settings;

    #[test]
    fn the_general_page_offers_main_focused_and_chosen_for_the_monitor_key() {
        let words: Vec<&str> = MONITOR_CHOICES.iter().map(|choice| choice.word).collect();
        assert_eq!(words, ["main", "focused", "chosen"]);
        assert_eq!(MONITOR_KEY, "notch.monitor");
        assert_eq!(MONITOR_CHOICES[0].word, DEFAULT_MONITOR);
    }

    #[test]
    fn a_missing_or_unknown_monitor_word_shows_main() {
        let path = std::env::temp_dir().join(format!("notch-settings-monitor-{}.txt", std::process::id()));
        std::fs::write(&path, "notch.monitor=banana\n").unwrap();
        let unknown = Settings::at(Some(path.clone()));
        std::fs::write(&path, "notch.monitor=focused\n").unwrap();
        let focused = Settings::at(Some(path.clone()));
        let _ = std::fs::remove_file(&path);
        let missing = Settings::at(Some(path));
        assert_eq!(shown_monitor_word(&unknown), "main");
        assert_eq!(shown_monitor_word(&focused), "focused");
        assert_eq!(shown_monitor_word(&missing), "main");
    }

    #[test]
    fn the_screen_list_shows_only_while_a_chosen_screen_is_selected() {
        let path = std::env::temp_dir().join(format!("notch-settings-chosen-{}.txt", std::process::id()));
        std::fs::write(
            &path,
            "notch.monitor=chosen
",
        )
        .unwrap();
        let chosen = Settings::at(Some(path.clone()));
        std::fs::write(
            &path,
            "notch.monitor=focused
",
        )
        .unwrap();
        let focused = Settings::at(Some(path.clone()));
        let _ = std::fs::remove_file(&path);
        assert!(chosen_is_selected(&chosen));
        assert!(!chosen_is_selected(&focused));
    }
}
