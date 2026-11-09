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

const PICTURE_HEIGHT: f32 = 84.0;
const PILL_HEIGHT: f32 = 36.0;
const PILL_CORNER: f32 = 12.0;
const RING_SIZE: f32 = 14.0;
const RING_WIDTH: f32 = 2.0;
// Already trimmed: a nowrap text in a content-sized pill never gets GPUI's own ellipsis, which needs a definite width.
const LYRIC_LINE: &str = "Wherever you are, wherever …";
// The activity color of the timer and the idle battery ring, from `docs/reference/design-rules.md`.
const TIMER_ORANGE: u32 = 0xff9e0a;
const BATTERY_GREEN: u32 = 0x2f9e44;
const RING_TRACK: u32 = 0x2c2c30;

pub fn color(hex: u32) -> Hsla {
    rgb(hex).into()
}

/// The picture of one choice: the backdrop at the card's width, with the pill on top.
pub fn picture(backdrop: Backdrop, pill: Option<Pill>) -> AnyElement {
    let frame = div().relative().w_full().h(px(PICTURE_HEIGHT)).rounded(px(6.0)).overflow_hidden().bg(color(0x000000));
    let frame = match backdrop {
        Backdrop::MaximizedWindow => frame.child(window_backdrop()),
        Backdrop::FullScreenApp => frame.child(game_backdrop()),
    };
    frame.children(pill.map(draw_pill)).into_any_element()
}

fn window_backdrop() -> impl IntoElement {
    let bar = |left: f32, top: f32, width: f32, height: f32, radius: f32, fill: u32| {
        div().absolute().left(px(left)).top(px(top)).w(px(width)).h(px(height)).rounded(px(radius)).bg(rgba(fill))
    };
    let dot = || div().size(px(6.0)).rounded(px(3.0)).bg(color(0x55555f));
    div()
        .absolute()
        .size_full()
        .bg(color(0x3b3b44))
        .child(div().absolute().top_0().left_0().right_0().h(px(20.0)).bg(color(0x2c2c33)))
        .child(div().absolute().left(px(10.0)).top(px(7.0)).flex().gap(px(5.0)).child(dot()).child(dot()).child(dot()))
        .child(bar(14.0, 52.0, 120.0, 5.0, 3.0, 0xffffff29))
        .child(bar(14.0, 64.0, 200.0, 5.0, 3.0, 0xffffff1a))
        .child(
            div()
                .absolute()
                .right(px(14.0))
                .top(px(52.0))
                .w(px(70.0))
                .h(px(17.0))
                .rounded(px(5.0))
                .bg(rgba(0xffffff1a)),
        )
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

fn game_backdrop() -> impl IntoElement {
    sunset().child(div().absolute().left(px(228.0)).top(px(40.0)).size(px(28.0)).rounded(px(14.0)).bg(rgba(0xffe0a0d9)))
}

/// The pill of one choice with its content.
fn draw_pill(pill: Pill) -> impl IntoElement {
    let content = match pill {
        Pill::Activity => div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .text_color(color(TIMER_ORANGE))
            .child(ring(TIMER_ORANGE))
            .child("16m")
            .into_any_element(),
        Pill::Lyric => div()
            .flex()
            .items_center()
            .gap(px(24.0))
            .child(div().flex_none().size(px(20.0)).rounded(px(5.0)).bg(linear_gradient(
                135.0,
                linear_color_stop(color(0x2a1d12), 0.0),
                linear_color_stop(color(0x6b3c18), 1.0),
            )))
            .child(div().child(SharedString::from(LYRIC_LINE)))
            .into_any_element(),
        Pill::Clock => {
            div().flex().items_center().gap(px(24.0)).child("15:05").child(ring(BATTERY_GREEN)).into_any_element()
        }
    };
    pill_frame(content, PILL_HEIGHT, 14.0, 3.0, PILL_CORNER)
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
fn ring(arc_color: u32) -> impl IntoElement {
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
