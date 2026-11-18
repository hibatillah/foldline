//! The collapsible help on the Calendar page: the five numbered steps that lead to the secret address in Google
//! Calendar, then two notes. It is closed until the heading row is clicked.

use gpui::{AnyElement, App, ElementId, Window, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, Icon, IconName, h_flex, v_flex};

use crate::{
    i18n::tr,
    icons,
    info_box::{info_box, warning_box},
    layout::{Item, block_frame},
    palette,
};

const NUMBER_SIZE: f32 = 20.0;
const NUMBER_TEXT: f32 = 11.0;
const HEADING_PADDING_X: f32 = 14.0;
const HEADING_PADDING_Y: f32 = 12.0;
const BODY_PADDING: f32 = 14.0;
const CHEVRON_SIZE: f32 = 15.0;
const OPEN_STATE_ID: &str = "calendar-steps-open";

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

fn step_row(index: usize, text: &'static str, cx: &App) -> AnyElement {
    let accent = palette::accent(cx);
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
        .child(div().flex_1().min_w_0().text_sm().text_color(palette::heading(cx)).child(text))
        .into_any_element()
}

fn body(cx: &App) -> AnyElement {
    v_flex()
        .gap(px(10.0))
        .p(px(BODY_PADDING))
        .border_t_1()
        .border_color(palette::row_divider(cx))
        .children(steps().into_iter().enumerate().map(|(index, text)| step_row(index, text, cx)))
        .child(warning_box(icons::WARNING, tr!("calendar.google_note1"), cx))
        .child(info_box(icons::ABOUT, tr!("calendar.google_note2"), cx))
        .into_any_element()
}

fn dropdown(window: &mut Window, cx: &mut App) -> AnyElement {
    let open_state = window.use_keyed_state(ElementId::Name(OPEN_STATE_ID.into()), cx, |_, _| false);
    let open = *open_state.read(cx);
    let chevron = if open { IconName::ChevronUp } else { IconName::ChevronDown };
    let heading = h_flex()
        .id("calendar-steps-heading")
        .w_full()
        .px(px(HEADING_PADDING_X))
        .py(px(HEADING_PADDING_Y))
        .gap_3()
        .items_center()
        .justify_between()
        .cursor_pointer()
        .child(
            v_flex()
                .child(div().text_base().font_weight(gpui::FontWeight::MEDIUM).child(tr!("calendar.group_steps")))
                .child(div().text_sm().text_color(cx.theme().muted_foreground).child(tr!("calendar.group_steps_note"))),
        )
        .child(Icon::new(chevron).size(px(CHEVRON_SIZE)).text_color(palette::accent(cx)))
        .on_click(move |_event, window, cx| {
            open_state.update(cx, |open, _| *open = !*open);
            window.refresh();
        });
    block_frame(cx).child(heading).when(open, |frame| frame.child(body(cx))).into_any_element()
}

pub fn item() -> Item {
    Item::custom(dropdown).outside_block()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_help_has_five_steps_each_with_a_sentence() {
        let texts = steps();
        assert!(texts.iter().all(|text| text.len() > 10));
        assert_eq!(texts.len(), 5);
    }
}
