//! The Snooze item of the General page: a select of lengths while notifications show, a status and Turn off while snoozed.

use std::rc::Rc;

use gpui::{AnyElement, App};
use store::{
    Settings,
    snooze::{SNOOZED_UNTIL_KEY, Snooze, SnoozeChoice, clock_text, now_unix},
};

use super::countdown::{Countdown, Length, controls, local_utc_offset_seconds};
use crate::{i18n::tr, layout::Item};

fn choice_label(choice: SnoozeChoice) -> &'static str {
    match choice {
        SnoozeChoice::TenMinutes => tr!("snooze.ten"),
        SnoozeChoice::ThirtyMinutes => tr!("snooze.thirty"),
        SnoozeChoice::OneHour => tr!("snooze.hour"),
        SnoozeChoice::UntilTurnedOff => tr!("snooze.manual"),
    }
}

/// The line shown while snoozed; `None` when notifications are not snoozed.
fn status_text(snooze: Snooze, utc_offset_seconds: i32) -> Option<String> {
    match snooze {
        Snooze::Off => None,
        Snooze::Until(end_unix) => Some(tr!("snooze.until", time = clock_text(end_unix, utc_offset_seconds))),
        Snooze::Manual => Some(tr!("snooze.until_turned_off").to_owned()),
    }
}

fn stored_snooze() -> Snooze {
    Snooze::parse(Settings::load().get_string(SNOOZED_UNTIL_KEY), now_unix())
}

fn view(cx: &App) -> AnyElement {
    controls(
        Countdown {
            id_prefix: "snooze",
            key: SNOOZED_UNTIL_KEY,
            lengths: SnoozeChoice::ALL
                .into_iter()
                .map(|choice| Length {
                    label: choice_label(choice),
                    stored_text: Rc::new(move || choice.snooze_from(now_unix()).as_text()),
                })
                .collect(),
            status: status_text(stored_snooze(), local_utc_offset_seconds()),
            end_label: tr!("snooze.turn_off"),
        },
        cx,
    )
}

pub fn item() -> Item {
    Item::new(tr!("snooze.title"), Rc::new(|_window, cx| view(cx))).description(tr!("snooze.note"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_status_shows_while_notifications_are_not_snoozed() {
        assert_eq!(status_text(Snooze::Off, 0), None);
    }

    #[test]
    fn a_timed_snooze_shows_its_end_time() {
        assert_eq!(status_text(Snooze::Until(52_200), 0).as_deref(), Some("Snoozed until 14:30"));
    }

    #[test]
    fn a_manual_snooze_says_it_waits_for_you() {
        assert_eq!(status_text(Snooze::Manual, 0).as_deref(), Some("Snoozed until you turn it off"));
    }

    #[test]
    fn a_click_writes_the_snooze_as_one_key_and_leaves_the_pause_alone() {
        let path = std::env::temp_dir().join(format!("foldline-settings-snooze-{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut settings = Settings::at(Some(path.clone()));
        crate::sections::apply(&mut settings, SNOOZED_UNTIL_KEY, &Snooze::Manual.as_text());
        let text = std::fs::read_to_string(&path).expect("settings file written");
        let _ = std::fs::remove_file(&path);
        assert_eq!(text, "notch.snoozed_until=manual\n");
    }

    #[test]
    fn each_choice_has_its_own_label() {
        let labels: Vec<&str> = SnoozeChoice::ALL.into_iter().map(choice_label).collect();
        assert_eq!(labels, ["10 min", "30 min", "1 hour", "Until I turn it off"]);
    }
}
