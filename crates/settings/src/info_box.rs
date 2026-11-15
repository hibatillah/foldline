//! A hint drawn on the page between blocks: a bordered box with an icon and a sentence, blue for a hint and amber for
//! a warning.

use gpui::{AnyElement, App, SharedString, div, prelude::*, px};
use gpui_component::{Icon, h_flex};

const RADIUS: f32 = 10.0;
const PADDING_X: f32 = 12.0;
const PADDING_Y: f32 = 10.0;
const GAP: f32 = 10.0;
const LINE_HEIGHT: f32 = 17.0;
const ICON_SIZE: f32 = 15.0;

/// `icon` is an SVG path from `icons` or the library's bundle.
pub fn info_box(icon: &'static str, text: impl Into<SharedString>, cx: &App) -> AnyElement {
    tinted(crate::palette::accent(cx), icon, text.into(), cx)
}

/// A warning about what a setting exposes, such as the secret calendar address.
pub fn warning_box(icon: &'static str, text: impl Into<SharedString>, cx: &App) -> AnyElement {
    tinted(crate::palette::warning(cx), icon, text.into(), cx)
}

fn tinted(accent: gpui::Hsla, icon: &'static str, text: SharedString, cx: &App) -> AnyElement {
    h_flex()
        .w_full()
        .items_start()
        .gap(px(GAP))
        .px(px(PADDING_X))
        .py(px(PADDING_Y))
        .rounded(px(RADIUS))
        .border_1()
        .border_color(accent.opacity(0.35))
        .bg(accent.opacity(0.08))
        .child(div().flex_none().pt(px(1.0)).child(Icon::default().path(icon).size(px(ICON_SIZE)).text_color(accent)))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_sm()
                .line_height(px(LINE_HEIGHT))
                .text_color(crate::palette::heading(cx).opacity(0.86))
                .child(text),
        )
        .into_any_element()
}
