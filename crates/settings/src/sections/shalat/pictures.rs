//! The pictures on the Shalat cards, drawn from plain elements: the sun below the horizon, a globe and a stick with its shadow.

use gpui::{AnyElement, Div, SharedString, div, linear_color_stop, linear_gradient, prelude::*, px, rgba};
use gpui_component::{Icon, IconName, h_flex, v_flex};

use crate::picture::color;

const HORIZON_HEIGHT: f32 = 26.0;
const HORIZON_PICTURE_HEIGHT: f32 = 74.0;
// Pixels of picture per degree of depth, and a gap that keeps the whole sun below the line.
const PIXELS_PER_DEGREE: f32 = 0.85;
const SUN_SIZE: f32 = 10.0;
const SUN_GAP: f32 = 6.0;
const SUN_COLOR: u32 = 0xffd27a;
const LABEL_COLOR: u32 = 0xe6e6eb;
const DASH_LENGTH: f32 = 2.0;

/// One of the two suns of a method card: its label and how far below the horizon it is drawn.
pub struct Sun {
    pub label: String,
    pub depth_degrees: f32,
}

fn picture_frame(height: f32) -> Div {
    div().relative().w_full().h(px(height)).rounded(px(6.0)).overflow_hidden()
}

/// A full-width row at `top` that centers its one child.
fn centered_at(top: f32) -> Div {
    div().absolute().top(px(top)).left_0().w_full().flex().justify_center()
}

fn sun_column(sun: &Sun) -> impl IntoElement {
    let sun_top = HORIZON_HEIGHT + sun.depth_degrees * PIXELS_PER_DEGREE + SUN_GAP - SUN_SIZE / 2.0;
    let dash_count = ((sun_top - HORIZON_HEIGHT) / (DASH_LENGTH * 2.0)).ceil().max(0.0) as usize;
    div()
        .relative()
        .flex_1()
        .h_full()
        .child(
            centered_at(HORIZON_HEIGHT).child(
                v_flex()
                    .gap(px(DASH_LENGTH))
                    .children((0..dash_count).map(|_| div().w(px(1.0)).h(px(DASH_LENGTH)).bg(color(SUN_COLOR)))),
            ),
        )
        .child(centered_at(sun_top).child(div().size(px(SUN_SIZE)).rounded_full().bg(color(SUN_COLOR))))
        .child(
            centered_at(sun_top + SUN_SIZE + 3.0)
                .text_size(px(9.0))
                .text_color(color(LABEL_COLOR))
                .child(SharedString::from(sun.label.clone())),
        )
}

/// The sky above the horizon and the ground below it, with the sun at Fajr on the left and at Isha on the right.
pub fn horizon(fajr: &Sun, isha: &Sun) -> AnyElement {
    picture_frame(HORIZON_PICTURE_HEIGHT)
        .bg(color(0x14141a))
        .child(div().absolute().top_0().left_0().w_full().h(px(HORIZON_HEIGHT)).bg(linear_gradient(
            180.0,
            linear_color_stop(color(0x0d1b3d), 0.0),
            linear_color_stop(color(0xe0824a), 1.0),
        )))
        .child(div().absolute().top(px(HORIZON_HEIGHT)).left_0().w_full().h(px(1.0)).bg(rgba(0xffffff55)))
        .child(h_flex().absolute().size_full().child(sun_column(fajr)).child(sun_column(isha)))
        .into_any_element()
}

/// A small globe, an arrow and the name of the method the country maps to.
pub fn globe_to(method_name: &'static str) -> AnyElement {
    let line = color(0x6a7a90);
    let globe = div()
        .relative()
        .flex_none()
        .size(px(28.0))
        .rounded_full()
        .border(px(1.5))
        .border_color(line)
        .child(div().absolute().top(px(14.0)).left_0().w_full().h(px(1.0)).bg(line))
        .child(
            div().absolute().top_0().left(px(7.0)).w(px(15.0)).h(px(29.0)).rounded_full().border_1().border_color(line),
        )
        .child(div().absolute().top(px(5.0)).left(px(19.0)).size(px(6.0)).rounded_full().bg(color(0xff6b6b)));
    picture_frame(HORIZON_PICTURE_HEIGHT)
        .bg(color(0x1c2430))
        .child(
            h_flex()
                .size_full()
                .items_center()
                .px(px(10.0))
                .gap(px(5.0))
                .child(globe)
                .child(Icon::new(IconName::ArrowRight).size(px(12.0)).text_color(color(0x9aa8bc)))
                .child(div().text_size(px(10.0)).text_color(color(LABEL_COLOR)).child(method_name)),
        )
        .into_any_element()
}

const ASR_HEIGHT: f32 = 126.0;
const STICK_LEFT: f32 = 70.0;
const STICK_TOP: f32 = 42.0;
const GROUND_TOP: f32 = 88.0;
const STICK_HEIGHT: f32 = GROUND_TOP - STICK_TOP;
const SUN_CENTER: (f32, f32) = (28.0, 20.0);
const ASR_SUN_RADIUS: f32 = 9.0;
const RAY_COLOR: u32 = 0xffe6a3;

/// A stick, the sun's ray over its top and the shadow, `shadow_in_heights` times as long as the stick is high.
pub fn stick_and_shadow(shadow_in_heights: f32, caption: &'static str) -> AnyElement {
    let shadow_length = STICK_HEIGHT * shadow_in_heights;
    let (ray_from_x, ray_from_y) = (SUN_CENTER.0, SUN_CENTER.1 + ASR_SUN_RADIUS);
    let (ray_x, ray_y) = (STICK_LEFT - ray_from_x, STICK_TOP - ray_from_y);
    let ray_length = ray_x.hypot(ray_y);
    let dots = (0..(ray_length / 6.0).floor() as usize).map(move |step| {
        let along = (step as f32 * 6.0 + 1.5) / ray_length;
        div()
            .absolute()
            .left(px(ray_from_x + ray_x * along - 1.0))
            .top(px(ray_from_y + ray_y * along - 1.0))
            .size(px(2.0))
            .bg(color(RAY_COLOR).opacity(0.8))
    });
    let tick = |left: f32| div().absolute().left(px(left)).top(px(96.0)).w(px(1.0)).h(px(6.0)).bg(color(LABEL_COLOR));
    picture_frame(ASR_HEIGHT)
        .child(div().absolute().top_0().left_0().w_full().h(px(GROUND_TOP)).bg(linear_gradient(
            180.0,
            linear_color_stop(color(0x2c5d9a), 0.0),
            linear_color_stop(color(0x9ac3e8), 1.0),
        )))
        .child(
            div().absolute().top(px(GROUND_TOP)).left_0().w_full().h(px(ASR_HEIGHT - GROUND_TOP)).bg(color(0x3c3c46)),
        )
        .child(
            div()
                .absolute()
                .left(px(SUN_CENTER.0 - ASR_SUN_RADIUS))
                .top(px(SUN_CENTER.1 - ASR_SUN_RADIUS))
                .size(px(ASR_SUN_RADIUS * 2.0))
                .rounded_full()
                .bg(color(RAY_COLOR)),
        )
        .children(dots)
        .child(
            div()
                .absolute()
                .left(px(STICK_LEFT - 1.5))
                .top(px(STICK_TOP - 1.5))
                .w(px(3.0))
                .h(px(STICK_HEIGHT + 3.0))
                .rounded(px(1.5))
                .bg(color(0xf2f2f5)),
        )
        .child(
            div()
                .absolute()
                .left(px(STICK_LEFT - 3.0))
                .top(px(GROUND_TOP - 1.0))
                .w(px(shadow_length + 6.0))
                .h(px(6.0))
                .rounded(px(3.0))
                .bg(color(0x0b0b0e)),
        )
        .child(
            div().absolute().left(px(STICK_LEFT)).top(px(99.0)).w(px(shadow_length)).h(px(1.0)).bg(color(LABEL_COLOR)),
        )
        .child(tick(STICK_LEFT))
        .child(tick(STICK_LEFT + shadow_length))
        .child(
            div()
                .absolute()
                .left(px(STICK_LEFT))
                .top(px(108.0))
                .w(px(shadow_length))
                .text_center()
                .text_size(px(10.0))
                .text_color(color(LABEL_COLOR))
                .child(SharedString::from(format!("{shadow_in_heights}×"))),
        )
        .child(
            div()
                .absolute()
                .top(px(5.0))
                .right(px(36.0))
                .text_size(px(10.0))
                .text_color(color(0xffffff))
                .child(caption),
        )
        .into_any_element()
}
