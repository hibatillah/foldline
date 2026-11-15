//! Small pictures of the top of the screen with the notch drawn on it, one per Behavior choice.

use gpui::{
    AnyElement, Hsla, SharedString, canvas, div, linear_color_stop, linear_gradient, point, prelude::*, px, rgb, rgba,
};

/// What is behind the notch: the situation the choice applies to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Backdrop {
    MaximizedWindow,
    FullScreenApp,
}

/// What the pill shows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pill {
    Activity,
    Lyric,
    Clock,
}

/// The pill a choice word draws, or `None` when the notch is hidden.
pub fn pill_for(word: &str) -> Option<Pill> {
    match word {
        "activities" => Some(Pill::Activity),
        "lyrics" => Some(Pill::Lyric),
        "music" => Some(Pill::Clock),
        _ => None,
    }
}

const PICTURE_HEIGHT: f32 = 78.0;
const PICTURE_RADIUS: f32 = 7.0;
const PILL_HEIGHT: f32 = 28.0;
const PILL_CORNER: f32 = 10.0;
const PILL_PADDING_X: f32 = 14.0;
const PILL_PADDING_BOTTOM: f32 = 3.0;
/// Between the pill's pieces (an icon, a text, a ring).
const PILL_GAP: f32 = 10.0;
const PILL_TEXT_SIZE: f32 = 12.0;
const RING_SIZE: f32 = 14.0;
const RING_WIDTH: f32 = 2.0;
const ARTWORK_SIZE: f32 = 16.0;
// Already trimmed: GPUI 0.2.2 draws no ellipsis inside this content-sized pill, even with a definite width.
const LYRIC_LINE: &str = "Wherever you …";
/// The clock pill keeps at least this much room between the time and the ring.
const CLOCK_SPACER: f32 = 26.0;
/// The volume message of the Activities picture: a bar 42 % full.
const VOLUME_BAR_WIDTH: f32 = 58.0;
const VOLUME_LEVEL: f32 = 0.42;
// The idle battery ring's color, from `docs/reference/design-rules.md`.
pub const BATTERY_GREEN: u32 = 0x2f9e44;
const RING_TRACK: u32 = 0x2c2c30;

pub fn color(hex: u32) -> Hsla {
    rgb(hex).into()
}

/// The picture of one choice: the backdrop at the card's width, with the pill on top.
pub fn picture(backdrop: Backdrop, pill: Option<Pill>) -> AnyElement {
    let frame = div()
        .relative()
        .w_full()
        .h(px(PICTURE_HEIGHT))
        .rounded(px(PICTURE_RADIUS))
        .overflow_hidden()
        .bg(color(0x000000));
    let frame = match backdrop {
        Backdrop::MaximizedWindow => frame.child(window_backdrop()),
        Backdrop::FullScreenApp => frame.child(game_backdrop()),
    };
    frame.children(pill.map(draw_pill)).into_any_element()
}

/// A maximized app: a title bar with its three buttons, a sidebar of items and a page of panels and lines.
fn window_backdrop() -> impl IntoElement {
    let block = |left: f32, top: f32, width: f32, height: f32, radius: f32, fill: u32| {
        div().absolute().left(px(left)).top(px(top)).w(px(width)).h(px(height)).rounded(px(radius)).bg(color(fill))
    };
    let button = |fill: u32| div().size(px(6.0)).rounded(px(3.0)).bg(color(fill));
    div()
        .absolute()
        .size_full()
        .bg(linear_gradient(135.0, linear_color_stop(color(0x232834), 0.0), linear_color_stop(color(0x181b23), 1.0)))
        .child(div().absolute().top_0().left_0().right_0().h(px(14.0)).bg(color(0x2d323e)))
        .child(
            div()
                .absolute()
                .right(px(6.0))
                .top(px(4.0))
                .flex()
                .gap(px(4.0))
                .child(button(0x59606f))
                .child(button(0x59606f))
                .child(button(0xe0584f)),
        )
        .child(div().absolute().left_0().top(px(14.0)).bottom_0().w(px(30.0)).bg(color(0x1b1e27)))
        .child(block(8.0, 22.0, 14.0, 5.0, 2.0, 0x4a6fd6))
        .child(block(8.0, 33.0, 14.0, 5.0, 2.0, 0x3a3f4d))
        .child(block(8.0, 44.0, 14.0, 5.0, 2.0, 0x3a3f4d))
        .child(block(8.0, 55.0, 14.0, 5.0, 2.0, 0x3a3f4d))
        .child(block(38.0, 22.0, 62.0, 28.0, 5.0, 0x32406a))
        .child(
            div()
                .absolute()
                .left(px(106.0))
                .right(px(8.0))
                .top(px(22.0))
                .h(px(28.0))
                .rounded(px(5.0))
                .bg(color(0x2c323f)),
        )
        .child(
            div()
                .absolute()
                .left(px(38.0))
                .right(px(8.0))
                .top(px(56.0))
                .h(px(5.0))
                .rounded(px(2.0))
                .bg(color(0x3a4050)),
        )
        .child(block(38.0, 67.0, 90.0, 5.0, 2.0, 0x323744))
}

/// Purple to orange. GPUI gradients have two stops, so the middle color is a second layer that fades in.
pub fn sunset() -> gpui::Div {
    let (top, middle, bottom) = (color(0x1b1042), color(0x6a2f7a), color(0xe0642f));
    let angle = 160.0;
    div()
        .absolute()
        .size_full()
        .bg(linear_gradient(angle, linear_color_stop(top, 0.0), linear_color_stop(middle, 0.55)))
        .child(div().absolute().size_full().bg(linear_gradient(
            angle,
            linear_color_stop(middle.opacity(0.0), 0.55),
            linear_color_stop(bottom, 1.0),
        )))
}

/// A game: a dusk sky with the sun, dark mountains, a green health bar at the bottom left and a ring at the right.
fn game_backdrop() -> impl IntoElement {
    let (top, middle, bottom) = (color(0x1a1245), color(0x6b2f7a), color(0xf08a4b));
    let sky = div()
        .absolute()
        .size_full()
        .bg(linear_gradient(180.0, linear_color_stop(top, 0.0), linear_color_stop(middle, 0.52)))
        .child(div().absolute().size_full().bg(linear_gradient(
            180.0,
            linear_color_stop(middle.opacity(0.0), 0.52),
            linear_color_stop(bottom, 1.0),
        )));
    let sun_glow =
        div().absolute().right(px(20.0)).bottom(px(20.0)).size(px(34.0)).rounded(px(17.0)).bg(rgba(0xffb36b40));
    let sun = div().absolute().right(px(26.0)).bottom(px(26.0)).size(px(22.0)).rounded(px(11.0)).bg(color(0xffd9a0));
    let health = div()
        .absolute()
        .left(px(8.0))
        .bottom(px(6.0))
        .w(px(46.0))
        .h(px(5.0))
        .rounded(px(3.0))
        .bg(rgba(0xffffff33))
        .child(div().w(gpui::relative(0.7)).h_full().rounded(px(3.0)).bg(color(0x5fd16f)));
    let aim = div()
        .absolute()
        .right(px(8.0))
        .bottom(px(6.0))
        .size(px(16.0))
        .rounded(px(8.0))
        .border(px(1.5))
        .border_color(rgba(0xffffff8c));
    sky.child(sun_glow).child(sun).child(mountains()).child(health).child(aim)
}

/// Two dark peaks along the bottom of the game picture, as fractions of its width and height.
const MOUNTAIN_POINTS: [(f32, f32); 7] =
    [(0.0, 1.0), (0.0, 0.78), (0.2, 0.55), (0.4, 0.8), (0.62, 0.5), (1.0, 0.85), (1.0, 1.0)];

fn mountains() -> impl IntoElement {
    canvas(
        |_bounds, _window, _cx| (),
        |bounds, _, window, _cx| {
            let mut builder = gpui::PathBuilder::fill();
            for (index, (x, y)) in MOUNTAIN_POINTS.iter().enumerate() {
                let at = point(bounds.origin.x + bounds.size.width * *x, bounds.origin.y + bounds.size.height * *y);
                if index == 0 {
                    builder.move_to(at);
                } else {
                    builder.line_to(at);
                }
            }
            builder.close();
            if let Ok(path) = builder.build() {
                window.paint_path(path, color(0x1c1230));
            }
        },
    )
    .absolute()
    .size_full()
}

/// The pill of one choice with its content.
fn draw_pill(pill: Pill) -> impl IntoElement {
    let content = match pill {
        Pill::Activity => div()
            .flex()
            .items_center()
            .gap(px(PILL_GAP))
            .child(gpui_component::Icon::default().path(crate::icons::VOLUME).size(px(13.0)))
            .child(
                div()
                    .w(px(VOLUME_BAR_WIDTH))
                    .h(px(4.0))
                    .rounded(px(2.0))
                    .bg(color(0x3a3a40))
                    .child(div().w(gpui::relative(VOLUME_LEVEL)).h_full().rounded(px(2.0)).bg(color(0xffffff))),
            )
            .child(div().text_size(px(PILL_TEXT_SIZE)).child("42"))
            .into_any_element(),
        Pill::Lyric => div()
            .flex()
            .items_center()
            .gap(px(PILL_GAP))
            .child(div().flex_none().size(px(ARTWORK_SIZE)).rounded(px(4.0)).bg(linear_gradient(
                135.0,
                linear_color_stop(color(0x2a1d12), 0.0),
                linear_color_stop(color(0x6b3c18), 1.0),
            )))
            .child(div().text_size(px(PILL_TEXT_SIZE)).child(SharedString::from(LYRIC_LINE)))
            .into_any_element(),
        Pill::Clock => div()
            .flex()
            .items_center()
            .child(div().text_size(px(PILL_TEXT_SIZE)).child("15:05"))
            .child(div().w(px(CLOCK_SPACER)))
            .child(ring(BATTERY_GREEN))
            .into_any_element(),
    };
    pill_frame(content, PILL_HEIGHT, PILL_PADDING_X, PILL_PADDING_BOTTOM, PILL_CORNER)
}

/// A black pill flush with the top edge of its picture and centered, with rounded bottom corners.
pub fn pill_frame(
    content: impl IntoElement,
    height: f32,
    padding_x: f32,
    padding_bottom: f32,
    corner: f32,
) -> impl IntoElement {
    div().absolute().top_0().left_0().w_full().flex().justify_center().child(
        div()
            .h(px(height))
            .px(px(padding_x))
            .pb(px(padding_bottom))
            .flex()
            .items_center()
            .bg(color(0x000000))
            .rounded_b(px(corner))
            .text_color(color(0xffffff))
            .text_size(px(13.0))
            .font_weight(gpui::FontWeight::MEDIUM)
            .whitespace_nowrap()
            .child(content),
    )
}

/// A progress ring: a dark track with a colored arc over 240 degrees, starting at the top.
pub fn ring(arc_color: u32) -> impl IntoElement {
    canvas(
        |_bounds, _window, _cx| (),
        move |bounds, _, window, _cx| {
            let center = bounds.center();
            let radius = px(RING_SIZE / 2.0 - RING_WIDTH / 2.0);
            let around = |sweep_degrees: f32| {
                let mut builder = gpui::PathBuilder::stroke(px(RING_WIDTH));
                let steps = (sweep_degrees / 6.0).ceil().max(1.0) as usize;
                for step in 0..=steps {
                    let angle = (-90.0 + sweep_degrees * step as f32 / steps as f32).to_radians();
                    let at = point(center.x + radius * angle.cos(), center.y + radius * angle.sin());
                    if step == 0 {
                        builder.move_to(at);
                    } else {
                        builder.line_to(at);
                    }
                }
                builder.build()
            };
            for (sweep, fill) in [(359.0, RING_TRACK), (240.0, arc_color)] {
                if let Ok(path) = around(sweep) {
                    window.paint_path(path, color(fill));
                }
            }
        },
    )
    .size(px(RING_SIZE))
    .flex_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_draws_no_pill() {
        assert_eq!(pill_for("hidden"), None);
    }

    #[test]
    fn each_shown_choice_draws_its_own_pill() {
        assert_eq!(pill_for("activities"), Some(Pill::Activity));
        assert_eq!(pill_for("lyrics"), Some(Pill::Lyric));
        assert_eq!(pill_for("music"), Some(Pill::Clock));
    }

    #[test]
    fn an_unknown_word_draws_no_pill() {
        assert_eq!(pill_for("banana"), None);
    }
}
