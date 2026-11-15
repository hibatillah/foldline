//! The five numbered steps that lead to the secret address in Google Calendar.

use gpui::{div, prelude::*, px};
use gpui_component::h_flex;

use crate::{i18n::tr, layout::Item};

const NUMBER_SIZE: f32 = 20.0;
const NUMBER_TEXT: f32 = 11.0;

// Wording follows Google's "Sync your calendar with computer programs" page (support.google.com/calendar/answer/37648).
fn steps() -> [&'static str; 5] {
    [
        tr!("calendar.google_step1"),
        tr!("calendar.google_step2"),
        tr!("calendar.google_step3"),
        tr!("calendar.google_step4"),
        tr!("calendar.google_step5"),
    ]
}

pub fn items() -> Vec<Item> {
    (0..steps().len())
        .map(|index| {
            Item::custom(move |_window, cx| {
                let accent = crate::palette::accent(cx);
                h_flex()
                    .gap(px(12.0))
                    .items_center()
                    .child(
                        div()
                            .flex_none()
                            .size(px(NUMBER_SIZE))
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(accent.opacity(0.2))
                            .text_size(px(NUMBER_TEXT))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(accent)
                            .child((index + 1).to_string()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_sm()
                            .text_color(crate::palette::heading(cx))
                            .child(steps()[index]),
                    )
                    .into_any_element()
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_is_one_item_for_each_step() {
        assert_eq!(items().len(), steps().len());
    }
}
