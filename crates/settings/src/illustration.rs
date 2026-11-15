//! The small drawings of the desktop that explain a setting: a dark scene with a window on it and the notch at the top,
//! shown in a strip of equal pictures with a caption under each.

use gpui::{AnyElement, Hsla, SharedString, div, linear_color_stop, linear_gradient, prelude::*, px, rgba};
use gpui_component::{h_flex, v_flex};

use crate::picture::color;

const SCENE_RADIUS: f32 = 9.0;
const STRIP_GAP: f32 = 12.0;
const CAPTION_GAP: f32 = 6.0;
const CAPTION_SIZE: f32 = 12.0;
const CAPTION_LINE_HEIGHT: f32 = 16.0;
/// The scene's two colors, dark blue fading to near black.
const SCENE_TOP: u32 = 0x1d2438;
const SCENE_BOTTOM: u32 = 0x12151f;
/// A line of text in a drawn window, as two widths of faint bar.
const BAR_HEIGHT: f32 = 4.0;

/// An empty scene of the given height at its container's width; children are placed absolutely on it.
pub fn scene(height: f32) -> gpui::Div {
    div()
        .relative()
        .w_full()
        .h(px(height))
        .flex_none()
        .rounded(px(SCENE_RADIUS))
        .overflow_hidden()
        .border_1()
        .border_color(rgba(0xffffff0f))
        .bg(linear_gradient(
            160.0,
            linear_color_stop(color(SCENE_TOP), 0.0),
            linear_color_stop(color(SCENE_BOTTOM), 1.0),
        ))
}

/// A faint horizontal bar, the stand-in for a line of text: `strong` is the brighter title line.
pub fn text_bar(left: f32, top: f32, width: f32, strong: bool) -> gpui::Div {
    div()
        .absolute()
        .left(px(left))
        .top(px(top))
        .w(px(width))
        .h(px(BAR_HEIGHT))
        .rounded(px(BAR_HEIGHT / 2.0))
        .bg(rgba(if strong { 0xffffff29 } else { 0xffffff17 }))
}

/// A window standing on the bottom edge of a scene, inset from both sides, with three lines of text; its top is at `top`.
pub fn standing_window(top: f32) -> gpui::Div {
    div()
        .absolute()
        .left(gpui::relative(0.14))
        .right(gpui::relative(0.14))
        .top(px(top))
        .bottom(px(-6.0))
        .rounded_t(px(7.0))
        .border_1()
        .border_color(rgba(0xffffff0f))
        .bg(rgba(0xffffff0f))
        .child(text_bar(10.0, 10.0, 90.0, true))
        .child(text_bar(10.0, 20.0, 130.0, false))
        .child(text_bar(10.0, 30.0, 70.0, false))
}

/// A black notch pill flush with the top of a scene, centered, with rounded bottom corners.
pub fn notch(content: impl IntoElement, height: f32, corner: f32, padding_x: f32) -> gpui::Div {
    div().absolute().top_0().left_0().w_full().flex().justify_center().child(
        h_flex()
            .h(px(height))
            .px(px(padding_x))
            .pb(px(3.0))
            .gap(px(10.0))
            .items_center()
            .bg(color(0x000000))
            .rounded_b(px(corner))
            .text_color(color(0xffffff))
            .font_weight(gpui::FontWeight::MEDIUM)
            .whitespace_nowrap()
            .child(content),
    )
}

/// The muted caption under a picture, centered.
pub fn caption(text: impl Into<SharedString>, muted: Hsla) -> gpui::Div {
    div()
        .w_full()
        .text_center()
        .text_size(px(CAPTION_SIZE))
        .line_height(px(CAPTION_LINE_HEIGHT))
        .text_color(muted)
        .child(text.into())
}

/// Pictures side by side at equal widths, each with its caption under it.
pub fn strip(pictures: Vec<(AnyElement, SharedString)>, muted: Hsla) -> AnyElement {
    h_flex()
        .w_full()
        .items_start()
        .gap(px(STRIP_GAP))
        .children(pictures.into_iter().map(|(picture, text)| {
            v_flex().flex_1().min_w_0().gap(px(CAPTION_GAP)).child(picture).child(caption(text, muted))
        }))
        .into_any_element()
}
