//! The picture at the top of the Calendar page: the calendar's address reaches the notch every few minutes.

use gpui::{AnyElement, App, SharedString, Window, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, Icon, h_flex, v_flex};
use store::Settings;

use crate::i18n::tr;

const CHIP_RADIUS: f32 = 8.0;
const DAY_SIZE: f32 = 16.0;
const DAY_TEXT: f32 = 9.5;
/// The day drawn in the accent, a stand-in for today.
const MARKED_DAY: usize = 3;

/// "every 15 min".
pub fn interval_text(minutes: u32) -> String {
    tr!("calendar.flow_every", minutes = minutes)
}

fn chip(title: &'static str, note: &'static str, cx: &App) -> gpui::Div {
    v_flex()
        .flex_none()
        .px(px(10.0))
        .py(px(6.0))
        .gap(px(1.0))
        .rounded(px(CHIP_RADIUS))
        .bg(cx.theme().foreground.opacity(0.07))
        .child(
            div()
                .text_size(px(12.0))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(crate::palette::heading(cx))
                .child(title),
        )
        .child(div().text_size(px(10.5)).text_color(cx.theme().muted_foreground).child(note))
}

fn week(cx: &App) -> AnyElement {
    let letters: Vec<char> = tr!("calendar.weekday_letters").chars().collect();
    h_flex()
        .gap(px(3.0))
        .children(letters.into_iter().enumerate().map(|(index, letter)| {
            let marked = index == MARKED_DAY;
            div()
                .size(px(DAY_SIZE))
                .rounded(px(4.0))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(DAY_TEXT))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .bg(if marked { crate::palette::accent(cx) } else { cx.theme().foreground.opacity(0.09) })
                .text_color(if marked { crate::palette::card(cx) } else { cx.theme().muted_foreground })
                .child(letter.to_string())
        }))
        .into_any_element()
}

pub fn flow(_window: &mut Window, cx: &mut App) -> AnyElement {
    let minutes = super::refresh_minutes(&Settings::load());
    let accent = crate::palette::accent(cx);
    let between = v_flex()
        .items_center()
        .gap(px(2.0))
        .child(Icon::default().path("icons/arrow-right.svg").size(px(13.0)).text_color(accent))
        .child(
            div()
                .text_size(px(10.5))
                .text_color(cx.theme().muted_foreground)
                .child(SharedString::from(interval_text(minutes))),
        );
    h_flex()
        .w_full()
        .items_center()
        .justify_between()
        .child(
            h_flex()
                .gap(px(14.0))
                .items_center()
                .child(chip(tr!("calendar.group_google"), tr!("calendar.flow_source_note"), cx))
                .child(between)
                .child(chip(tr!("calendar.flow_target"), tr!("calendar.flow_target_note"), cx)),
        )
        .child(week(cx))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_interval_names_the_minutes() {
        assert_eq!(interval_text(15), "every 15 min");
    }

    #[test]
    fn the_week_has_seven_letters_in_both_languages() {
        use store::language::{Language, with_override};
        assert_eq!(tr!("calendar.weekday_letters").chars().count(), 7);
        assert_eq!(with_override(Language::Indonesian, || tr!("calendar.weekday_letters").chars().count()), 7);
    }
}
