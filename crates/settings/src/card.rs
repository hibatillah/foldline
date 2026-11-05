//! The option card shared by the pages that choose with pictures: a radio indicator, the title and the fill and outline
//! of the selected state, laid out in equal-width columns.

use gpui::{AnyElement, App, ElementId, Stateful, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, h_flex, v_flex};

use crate::{option_border, option_fill};

/// Columns of a block with many options (the calculation methods).
pub const COLUMNS: u16 = 4;
const GAP: f32 = 10.0;
const RADIO_SIZE: f32 = 16.0;
const RADIO_DOT: f32 = 8.0;

/// A card with the fill and outline of its state: lighter and brighter when selected, darker and quieter otherwise.
pub fn frame(id: impl Into<ElementId>, selected: bool, padding: f32, gap: f32, cx: &App) -> Stateful<gpui::Div> {
    div()
        .id(id)
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(gap))
        .p(px(padding))
        .border_1()
        .border_color(option_border(selected, cx))
        .rounded(px(10.0))
        .bg(option_fill(selected, cx))
}

/// The radio ring of a single-choice card: an empty ring, or a white ring with a white dot when selected.
pub fn radio(selected: bool, cx: &App) -> gpui::Div {
    let theme = cx.theme();
    let ring = if selected { theme.foreground } else { theme.muted_foreground.opacity(0.6) };
    div()
        .flex_none()
        .size(px(RADIO_SIZE))
        .rounded_full()
        .border(px(1.5))
        .border_color(ring)
        .flex()
        .items_center()
        .justify_center()
        .when(selected, |ring_box| ring_box.child(div().size(px(RADIO_DOT)).rounded_full().bg(theme.foreground)))
}

/// The top of a card: the radio, then the title in full foreground with an optional description under it.
pub fn header(title: &'static str, description: Option<&'static str>, selected: bool, cx: &App) -> gpui::Div {
    let theme = cx.theme();
    let text = v_flex()
        .flex_1()
        .min_w_0()
        .gap(px(1.0))
        .child(
            div().text_size(px(13.0)).font_weight(gpui::FontWeight::SEMIBOLD).text_color(theme.foreground).child(title),
        )
        .children(description.map(|note| {
            div().text_size(px(11.0)).line_height(px(14.0)).text_color(theme.muted_foreground).child(note)
        }));
    h_flex().items_start().gap(px(8.0)).child(div().pt(px(1.0)).child(radio(selected, cx))).child(text)
}

/// Cards in `columns` equal columns: every cell has the same width whatever its card holds, and a partial last row
/// keeps that width and starts at the left.
pub fn grid(columns: u16, cards: Vec<AnyElement>) -> gpui::Div {
    div().grid().grid_cols(columns).gap(px(GAP)).children(cards)
}

/// A title with its one-line explanation above a block of cards.
pub fn described(title: &'static str, description: impl Into<gpui::SharedString>, cx: &App) -> gpui::Div {
    v_flex()
        .gap_0p5()
        .child(div().text_sm().font_weight(gpui::FontWeight::MEDIUM).text_color(cx.theme().foreground).child(title))
        .child(div().text_sm().text_color(cx.theme().muted_foreground).child(description.into()))
}
