//! The Pause item of the General page: a select of lengths while the notch runs, a status and Resume while it is paused.

use std::rc::Rc;

use gpui::{AnyElement, App};
use store::{
    Settings,
    pause::{PAUSED_UNTIL_KEY, Pause, PauseChoice, clock_text, now_unix},
};

use super::countdown::{Countdown, Length, controls, local_utc_offset_seconds};
use crate::{i18n::tr, layout::Item};

fn choice_label(choice: PauseChoice) -> &'static str {
    match choice {
        PauseChoice::TenMinutes => tr!("pause.ten"),
        PauseChoice::ThirtyMinutes => tr!("pause.thirty"),
        PauseChoice::OneHour => tr!("pause.hour"),
        PauseChoice::UntilResumed => tr!("pause.manual"),
    }
}

/// The line shown while paused; `None` when the notch is not paused.
fn status_text(pause: Pause, utc_offset_seconds: i32) -> Option<String> {
    match pause {
        Pause::Off => None,
        Pause::Until(end_unix) => Some(tr!("pause.until", time = clock_text(end_unix, utc_offset_seconds))),
        Pause::Manual => Some(tr!("pause.until_resumed").to_owned()),
    }
}

fn stored_pause() -> Pause {
    Pause::parse(Settings::load().get_string(PAUSED_UNTIL_KEY), now_unix())
}

fn view(cx: &App) -> AnyElement {
    controls(
        Countdown {
            id_prefix: "pause",
            key: PAUSED_UNTIL_KEY,
            lengths: PauseChoice::ALL
                .into_iter()
                .map(|choice| Length {
                    label: choice_label(choice),
                    stored_text: Rc::new(move || choice.pause_from(now_unix()).as_text()),
                })
                .collect(),
            status: status_text(stored_pause(), local_utc_offset_seconds()),
            end_label: tr!("pause.resume"),
        },
        cx,
    )
}

pub fn item() -> Item {
    Item::new(tr!("pause.title"), Rc::new(|_window, cx| view(cx))).description(tr!("pause.note"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_status_shows_while_not_paused() {
        assert_eq!(status_text(Pause::Off, 0), None);
    }

    #[test]
    fn a_timed_pause_shows_its_end_time() {
        assert_eq!(status_text(Pause::Until(52_200), 0).as_deref(), Some("Paused until 14:30"));
    }

    #[test]
    fn a_manual_pause_says_it_waits_for_you() {
        assert_eq!(status_text(Pause::Manual, 0).as_deref(), Some("Paused until you resume"));
    }

    #[test]
    fn a_click_writes_the_pause_as_one_key() {
        let path = std::env::temp_dir().join(format!("foldline-settings-pause-{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut settings = Settings::at(Some(path.clone()));
        crate::sections::apply(&mut settings, PAUSED_UNTIL_KEY, &Pause::Manual.as_text());
        let text = std::fs::read_to_string(&path).expect("settings file written");
        let _ = std::fs::remove_file(&path);
        assert!(text.contains("notch.paused_until=manual"), "{text}");
    }

    #[test]
    fn each_choice_has_its_own_label() {
        let labels: Vec<&str> = PauseChoice::ALL.into_iter().map(choice_label).collect();
        assert_eq!(labels, ["10 min", "30 min", "1 hour", "Until I resume"]);
    }
}
