//! The Timer page's drawings: the open timer page for a focus session and each break, and one round as a bar.

use gpui::{AnyElement, App, Hsla, SharedString, Window, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, h_flex, v_flex};
use store::Settings;

use super::{Lengths, SESSIONS_PER_ROUND};
use crate::{i18n::tr, illustration, picture::color};

const SCENE_HEIGHT: f32 = 76.0;
/// The open page inside the scene: flush with the top edge as the real notch is, inset at both sides, and running past the
/// bottom edge so the scene cuts it off.
const PAGE_INSET_X: f32 = 10.0;
const PAGE_BLEED_BOTTOM: f32 = 10.0;
const TIME_SIZE: f32 = 22.0;
const LABEL_SIZE: f32 = 10.5;
const BUTTON_SIZE: f32 = 10.0;
const LABEL_GRAY: u32 = 0x9a9aa6;
const BUTTON_FILL: u32 = 0x2a2a2d;
const STOP_RED: u32 = 0xb02a2a;

/// The colors of a round's parts; focus is the accent.
const SHORT_BREAK_GREEN: u32 = 0x4dd4ac;
const LONG_BREAK_PURPLE: u32 = 0x9b7bff;
const BAR_HEIGHT: f32 = 10.0;
const BAR_GAP: f32 = 4.0;
const LEGEND_DOT: f32 = 8.0;

/// One part of a round: its length in minutes and its color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Focus(u32),
    ShortBreak(u32),
    LongBreak(u32),
}

/// A round in order: focus and a short break by turns, the long break after the last focus session.
pub fn round_parts(lengths: Lengths) -> Vec<Part> {
    let mut parts = Vec::new();
    for session in 1..=SESSIONS_PER_ROUND {
        parts.push(Part::Focus(lengths.focus));
        parts.push(if session < SESSIONS_PER_ROUND {
            Part::ShortBreak(lengths.short_break)
        } else {
            Part::LongBreak(lengths.long_break)
        });
    }
    parts
}

/// "25:00": the page shows the minutes with two digits at least.
pub fn clock(minutes: u32) -> String {
    format!("{minutes:02}:00")
}

fn page_button(text: &'static str, fill: u32) -> gpui::Div {
    div()
        .px(px(10.0))
        .py(px(2.0))
        .rounded(px(8.0))
        .bg(color(fill))
        .text_size(px(BUTTON_SIZE))
        .text_color(color(0xffffff))
        .child(text)
}

/// The open timer page with a session's time, its label and the running page's buttons.
fn open_page(minutes: u32, label: SharedString) -> AnyElement {
    let page = v_flex()
        .absolute()
        .top_0()
        .left(px(PAGE_INSET_X))
        .right(px(PAGE_INSET_X))
        .bottom(px(-PAGE_BLEED_BOTTOM))
        .px(px(12.0))
        .pt(px(8.0))
        .gap(px(4.0))
        .bg(color(0x000000))
        .text_color(color(0xffffff))
        .child(
            h_flex()
                .items_baseline()
                .gap(px(8.0))
                .whitespace_nowrap()
                .child(
                    div()
                        .text_size(px(TIME_SIZE))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(SharedString::from(clock(minutes))),
                )
                .child(div().text_size(px(LABEL_SIZE)).text_color(color(LABEL_GRAY)).truncate().child(label)),
        )
        .child(
            h_flex()
                .gap(px(5.0))
                .child(page_button(tr!("timer.pause"), BUTTON_FILL))
                .child(page_button(tr!("timer.extend"), BUTTON_FILL))
                .child(page_button(tr!("timer.stop"), STOP_RED)),
        );
    illustration::scene(SCENE_HEIGHT).child(page).into_any_element()
}

/// The page for the first focus session, a short break and the long break, at the stored lengths.
pub fn pills(_window: &mut Window, cx: &mut App) -> AnyElement {
    let lengths = Lengths::read(&Settings::load());
    let focus_label = tr!("timer.focus_session", n = 1, total = SESSIONS_PER_ROUND);
    illustration::strip(
        vec![
            (open_page(lengths.focus, focus_label.into()), tr!("timer.focus").into()),
            (open_page(lengths.short_break, tr!("timer.short_break").into()), tr!("timer.short_break").into()),
            (open_page(lengths.long_break, tr!("timer.long_break").into()), tr!("timer.long_break").into()),
        ],
        cx.theme().muted_foreground,
    )
}

fn part_color(part: Part, cx: &App) -> Hsla {
    match part {
        Part::Focus(_) => crate::palette::accent(cx),
        Part::ShortBreak(_) => color(SHORT_BREAK_GREEN),
        Part::LongBreak(_) => color(LONG_BREAK_PURPLE),
    }
}

fn minutes_of(part: Part) -> u32 {
    match part {
        Part::Focus(minutes) | Part::ShortBreak(minutes) | Part::LongBreak(minutes) => minutes,
    }
}

fn legend_entry(fill: Hsla, text: &'static str, cx: &App) -> AnyElement {
    h_flex()
        .gap(px(6.0))
        .items_center()
        .text_size(px(12.0))
        .text_color(cx.theme().muted_foreground)
        .child(div().size(px(LEGEND_DOT)).rounded_full().bg(fill))
        .child(text)
        .into_any_element()
}

/// The round as one bar, each part as wide as its share of the time, with the legend and a sentence under it.
pub fn round(_window: &mut Window, cx: &mut App) -> AnyElement {
    let parts = round_parts(Lengths::read(&Settings::load()));
    let segments = parts.iter().map(|&part| {
        let mut segment =
            div().h(px(BAR_HEIGHT)).min_w(px(BAR_HEIGHT)).rounded(px(BAR_HEIGHT / 2.0)).flex_basis(px(0.0));
        segment.style().flex_grow = Some(minutes_of(part) as f32);
        segment.bg(part_color(part, cx))
    });
    v_flex()
        .w_full()
        .child(h_flex().w_full().gap(px(BAR_GAP)).children(segments))
        .child(
            h_flex()
                .mt(px(10.0))
                .gap(px(16.0))
                .child(legend_entry(crate::palette::accent(cx), tr!("timer.focus"), cx))
                .child(legend_entry(color(SHORT_BREAK_GREEN), tr!("timer.short_break"), cx))
                .child(legend_entry(color(LONG_BREAK_PURPLE), tr!("timer.long_break"), cx)),
        )
        .child(div().mt(px(8.0)).text_sm().text_color(cx.theme().muted_foreground).child(tr!("timer.round_note")))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_round_is_four_focus_sessions_with_short_breaks_between_and_the_long_break_last() {
        let lengths = Lengths { focus: 25, short_break: 5, long_break: 15 };
        assert_eq!(
            round_parts(lengths),
            [
                Part::Focus(25),
                Part::ShortBreak(5),
                Part::Focus(25),
                Part::ShortBreak(5),
                Part::Focus(25),
                Part::ShortBreak(5),
                Part::Focus(25),
                Part::LongBreak(15),
            ]
        );
    }

    #[test]
    fn the_parts_of_a_round_add_up_to_its_total() {
        let lengths = Lengths { focus: 50, short_break: 10, long_break: 30 };
        let total: u32 = round_parts(lengths).into_iter().map(minutes_of).sum();
        assert_eq!(total, lengths.round_minutes());
    }

    #[test]
    fn the_clock_shows_two_digits_of_minutes_at_least() {
        assert_eq!(clock(5), "05:00");
        assert_eq!(clock(25), "25:00");
        assert_eq!(clock(120), "120:00");
    }

    #[test]
    fn the_page_words_are_the_ones_the_notch_shows() {
        for key in [
            "timer.pause",
            "timer.extend",
            "timer.stop",
            "timer.focus_session",
            "timer.short_break",
            "timer.long_break",
        ] {
            assert!(crate::i18n::same_in_the_notch(key), "{key} differs between the two programs");
        }
    }
}
