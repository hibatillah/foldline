//! The Pause item of the General page: four lengths while the notch runs, a status and Resume while it is paused.

use std::rc::Rc;

use gpui::{AnyElement, App, div, prelude::*};
use gpui_component::h_flex;
use store::{
    Settings,
    pause::{PAUSED_UNTIL_KEY, Pause, PauseChoice, clock_text, now_unix},
};
use windows::Win32::System::Time::{GetTimeZoneInformation, TIME_ZONE_INFORMATION};

use super::about::widgets::{Handler, pill};
use crate::{field, i18n::tr, layout::Item};

/// `GetTimeZoneInformation` returns this while daylight saving time applies.
const TIME_ZONE_ID_DAYLIGHT: u32 = 2;

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

/// Seconds east of UTC from Windows' bias values, which are minutes west of UTC.
fn utc_offset_from_bias(bias_minutes: i32, daylight_bias_minutes: i32, daylight_applies: bool) -> i32 {
    let west_minutes = if daylight_applies { bias_minutes + daylight_bias_minutes } else { bias_minutes };
    -west_minutes * 60
}

fn local_utc_offset_seconds() -> i32 {
    let mut zone = TIME_ZONE_INFORMATION::default();
    // SAFETY: `zone` is a valid, writable structure for the call.
    let zone_id = unsafe { GetTimeZoneInformation(&mut zone) };
    utc_offset_from_bias(zone.Bias, zone.DaylightBias, zone_id == TIME_ZONE_ID_DAYLIGHT)
}

fn stored_pause() -> Pause {
    Pause::parse(Settings::load().get_string(PAUSED_UNTIL_KEY), now_unix())
}

fn start_pause(choice: PauseChoice, cx: &mut App) {
    field::write(PAUSED_UNTIL_KEY, &choice.pause_from(now_unix()).as_text(), cx);
}

fn resume(cx: &mut App) {
    field::write(PAUSED_UNTIL_KEY, &Pause::Off.as_text(), cx);
}

fn controls(cx: &App) -> AnyElement {
    let pause = stored_pause();
    match status_text(pause, local_utc_offset_seconds()) {
        None => h_flex()
            .gap_2()
            .children(PauseChoice::ALL.into_iter().enumerate().map(|(position, choice)| {
                let start: Handler = Rc::new(move |_window, cx| start_pause(choice, cx));
                pill(&format!("pause-choice-{position}"), choice_label(choice), None, Some(start), cx)
            }))
            .into_any_element(),
        Some(status) => h_flex()
            .gap_3()
            .items_center()
            .child(div().text_sm().child(status))
            .child(pill("pause-resume", tr!("pause.resume"), None, Some(Rc::new(|_window, cx| resume(cx))), cx))
            .into_any_element(),
    }
}

pub fn item() -> Item {
    Item::new(tr!("pause.title"), Rc::new(|_window, cx| controls(cx))).description(tr!("pause.note"))
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
        let path = std::env::temp_dir().join(format!("notch-settings-pause-{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut settings = Settings::at(Some(path.clone()));
        crate::sections::apply(&mut settings, PAUSED_UNTIL_KEY, &Pause::Manual.as_text());
        let text = std::fs::read_to_string(&path).expect("settings file written");
        let _ = std::fs::remove_file(&path);
        assert!(text.contains("notch.paused_until=manual"), "{text}");
    }

    #[test]
    fn the_clock_offset_is_minutes_west_of_utc_turned_into_seconds_east() {
        assert_eq!(utc_offset_from_bias(-420, 0, false), 7 * 3600);
        assert_eq!(utc_offset_from_bias(300, -60, true), -4 * 3600);
        assert_eq!(utc_offset_from_bias(300, -60, false), -5 * 3600);
    }

    #[test]
    fn each_choice_has_its_own_label() {
        let labels: Vec<&str> = PauseChoice::ALL.into_iter().map(choice_label).collect();
        assert_eq!(labels, ["10 min", "30 min", "1 hour", "Until I resume"]);
    }
}
