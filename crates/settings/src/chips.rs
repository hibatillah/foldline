//! A row of round-number buttons for a length in minutes; the one equal to the stored value is the accent.

use gpui::{AnyElement, App, ElementId, SharedString, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, h_flex};
use store::Settings;

use crate::field::stored_whole_number;

const WIDTH: f32 = 34.0;
const HEIGHT: f32 = 26.0;
const RADIUS: f32 = 7.0;
const GAP: f32 = 6.0;
const TEXT_SIZE: f32 = 13.0;

/// One chip per value; a click writes that value under `key`.
pub fn presets(key: &'static str, default: u32, values: &'static [u32], cx: &App) -> AnyElement {
    let current = stored_whole_number(&Settings::load(), key, default);
    let chips = values.iter().map(|&value| {
        let chosen = value == current;
        let ink = if chosen { crate::palette::card(cx) } else { crate::palette::heading(cx) };
        let fill = if chosen { crate::palette::accent(cx) } else { cx.theme().foreground.opacity(0.09) };
        div()
            .id(ElementId::Name(SharedString::from(format!("{key}-preset-{value}"))))
            .flex_none()
            .w(px(WIDTH))
            .h(px(HEIGHT))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(RADIUS))
            .bg(fill)
            .text_size(px(TEXT_SIZE))
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(ink)
            .cursor_pointer()
            .on_click(move |_event, _window, cx| crate::field::write(key, &value.to_string(), cx))
            .child(value.to_string())
    });
    h_flex().gap(px(GAP)).items_center().children(chips).into_any_element()
}
