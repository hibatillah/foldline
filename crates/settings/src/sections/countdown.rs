//! The view the Pause and Snooze items of General share: a select of lengths, or a status line and a button to end it.

use std::rc::Rc;

use gpui::{AnyElement, App, SharedString, div, prelude::*, px};
use gpui_component::{
    button::Button,
    h_flex,
    menu::{DropdownMenu as _, PopupMenuItem},
};
use windows::Win32::System::Time::{GetTimeZoneInformation, TIME_ZONE_INFORMATION};

use super::about::widgets::{Handler, pill};
use crate::{field, i18n::tr};

/// `GetTimeZoneInformation` returns this while daylight saving time applies.
const TIME_ZONE_ID_DAYLIGHT: u32 = 2;

/// One length the user can pick: its label and the text to store when it is picked.
pub(super) struct Length {
    pub label: &'static str,
    pub stored_text: Rc<dyn Fn() -> String>,
}

/// The select's width, wide enough for the longest length ("Until I turn it off").
const SELECT_WIDTH: f32 = 190.0;

/// What a countdown item needs to draw itself; `status` is `Some` while the countdown runs.
pub(super) struct Countdown {
    pub id_prefix: &'static str,
    pub key: &'static str,
    pub lengths: Vec<Length>,
    pub status: Option<String>,
    pub end_label: &'static str,
}

/// Seconds east of UTC from Windows' bias values, which are minutes west of UTC.
fn utc_offset_from_bias(bias_minutes: i32, daylight_bias_minutes: i32, daylight_applies: bool) -> i32 {
    let west_minutes = if daylight_applies { bias_minutes + daylight_bias_minutes } else { bias_minutes };
    -west_minutes * 60
}

pub(super) fn local_utc_offset_seconds() -> i32 {
    let mut zone = TIME_ZONE_INFORMATION::default();
    // SAFETY: `zone` is a valid, writable structure for the call.
    let zone_id = unsafe { GetTimeZoneInformation(&mut zone) };
    utc_offset_from_bias(zone.Bias, zone.DaylightBias, zone_id == TIME_ZONE_ID_DAYLIGHT)
}

pub(super) fn controls(countdown: Countdown, cx: &App) -> AnyElement {
    let Countdown { id_prefix, key, lengths, status, end_label } = countdown;
    match status {
        None => {
            let lengths = Rc::new(lengths);
            Button::new(SharedString::from(format!("{id_prefix}-choose")))
                .outline()
                .w(px(SELECT_WIDTH))
                .label(tr!("countdown.choose"))
                .dropdown_caret(true)
                .dropdown_menu(move |menu, _window, _cx| {
                    lengths.iter().fold(menu, |menu, length| {
                        let stored_text = length.stored_text.clone();
                        menu.item(
                            PopupMenuItem::new(length.label)
                                .on_click(move |_event, _window, cx| field::write(key, &stored_text(), cx)),
                        )
                    })
                })
                .into_any_element()
        }
        Some(status) => {
            let end: Handler = Rc::new(move |_window, cx| field::write(key, "", cx));
            h_flex()
                .gap_3()
                .items_center()
                .child(div().text_sm().child(status))
                .child(pill(&format!("{id_prefix}-end"), end_label, None, end, cx))
                .into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_select_asks_for_a_length_in_both_languages() {
        use store::language::{Language, with_override};
        assert_eq!(with_override(Language::English, || tr!("countdown.choose")), "Choose how long");
        assert_eq!(with_override(Language::Indonesian, || tr!("countdown.choose")), "Pilih durasi");
    }

    #[test]
    fn the_clock_offset_is_minutes_west_of_utc_turned_into_seconds_east() {
        assert_eq!(utc_offset_from_bias(-420, 0, false), 7 * 3600);
        assert_eq!(utc_offset_from_bias(300, -60, true), -4 * 3600);
        assert_eq!(utc_offset_from_bias(300, -60, false), -5 * 3600);
    }
}
