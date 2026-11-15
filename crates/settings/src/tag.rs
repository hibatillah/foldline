//! The small rounded tag after a row title: "After restart" in muted gray, "Also key 0" in the accent blue.

use gpui::{AnyElement, App, div, prelude::*, px};
use gpui_component::ActiveTheme as _;

const TEXT_SIZE: f32 = 11.0;
const RADIUS: f32 = 6.0;
const PADDING_X: f32 = 6.0;
const PADDING_Y: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// A fact about the setting, such as when it applies.
    Muted,
    /// A fact that belongs to the accent's meaning, such as a key that also leads to the page.
    Accent,
}

pub fn tag(text: &'static str, tone: Tone, cx: &App) -> AnyElement {
    let (fill, ink) = match tone {
        Tone::Muted => (cx.theme().foreground.opacity(0.08), crate::palette::muted(cx)),
        Tone::Accent => {
            let accent = crate::palette::accent(cx);
            (accent.opacity(0.14), accent)
        }
    };
    div()
        .flex_none()
        .px(px(PADDING_X))
        .py(px(PADDING_Y))
        .rounded(px(RADIUS))
        .bg(fill)
        .text_size(px(TEXT_SIZE))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(ink)
        .whitespace_nowrap()
        .child(text)
        .into_any_element()
}
