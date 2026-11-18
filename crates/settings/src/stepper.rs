//! A number between a minus and a plus button, for a length in minutes (the timer, the calendar refresh, a prayer's
//! offset). A button that would leave the range is dimmed and does nothing.

use std::{ops::RangeInclusive, rc::Rc};

use gpui::{AnyElement, App, ElementId, SharedString, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, h_flex};
use store::Settings;

use crate::{field::stored_whole_number, i18n::tr, layout::Render};

/// What a click does: it reads the stored value again and writes the next one.
pub type Step = Rc<dyn Fn(&mut App)>;

/// The size of a row's stepper.
#[derive(Clone, Copy, Debug)]
pub struct Size {
    button: f32,
    value_width: f32,
    text: f32,
    gap: f32,
}

pub const REGULAR: Size = Size { button: 26.0, value_width: 52.0, text: 14.0, gap: 8.0 };

const BUTTON_RADIUS: f32 = 7.0;
const GLYPH_SIZE: f32 = 16.0;

/// `current` moved by `delta` and kept in `range`; a value stored outside the range comes back into it.
pub fn stepped(current: i64, delta: i64, range: &RangeInclusive<i64>) -> i64 {
    (current + delta).clamp(*range.start(), *range.end())
}

fn button(id: ElementId, glyph: &'static str, step: Option<Step>, size: Size, cx: &App) -> AnyElement {
    let fill = cx.theme().foreground.opacity(0.09);
    div()
        .id(id)
        .flex_none()
        .size(px(size.button))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(BUTTON_RADIUS))
        .bg(fill)
        .text_size(px(GLYPH_SIZE))
        .text_color(crate::palette::heading(cx).opacity(0.85))
        .child(glyph)
        .map(|button| match step {
            Some(step) => button
                .cursor_pointer()
                .hover(|style| style.bg(cx.theme().foreground.opacity(0.15)))
                .on_click(move |_event, _window, cx| step(cx)),
            None => button.opacity(0.4),
        })
        .into_any_element()
}

/// `value` with its muted unit after it, between the two buttons; `None` dims a button.
pub fn stepper(
    id: &str,
    value: SharedString,
    unit: &'static str,
    down: Option<Step>,
    up: Option<Step>,
    size: Size,
    cx: &App,
) -> AnyElement {
    let shown = h_flex()
        .min_w(px(size.value_width))
        .justify_center()
        .gap(px(4.0))
        .text_size(px(size.text))
        .child(div().text_color(crate::palette::heading(cx)).child(value))
        .child(div().text_color(cx.theme().muted_foreground).child(unit));
    h_flex()
        .flex_none()
        .items_center()
        .gap(px(size.gap))
        .child(button(SharedString::from(format!("{id}-minus")).into(), "−", down, size, cx))
        .child(shown)
        .child(button(SharedString::from(format!("{id}-plus")).into(), "+", up, size, cx))
        .into_any_element()
}

/// The stepper of a length in minutes stored under `key`: each click reads the stored value again, moves it by
/// `step` within `range` and writes it.
pub fn minutes(key: &'static str, default: u32, range: RangeInclusive<u32>, step: u32, size: Size) -> Render {
    let range = i64::from(*range.start())..=i64::from(*range.end());
    Rc::new(move |_window, cx| {
        let current = i64::from(stored_whole_number(&Settings::load(), key, default));
        let step_by = |delta: i64| -> Option<Step> {
            let range = range.clone();
            (stepped(current, delta, &range) != current).then(|| {
                Rc::new(move |cx: &mut App| {
                    let stored = i64::from(stored_whole_number(&Settings::load(), key, default));
                    crate::field::write(key, &stepped(stored, delta, &range).to_string(), cx);
                }) as Step
            })
        };
        let step = i64::from(step);
        stepper(key, current.to_string().into(), tr!("stepper.minutes"), step_by(-step), step_by(step), size, cx)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_step_moves_by_its_size_inside_the_range() {
        assert_eq!(stepped(25, 5, &(5..=120)), 30);
        assert_eq!(stepped(25, -5, &(5..=120)), 20);
    }

    #[test]
    fn a_step_stops_at_either_end_of_the_range() {
        assert_eq!(stepped(5, -5, &(5..=120)), 5);
        assert_eq!(stepped(118, 5, &(5..=120)), 120);
        assert_eq!(stepped(3, -5, &(1..=1440)), 1);
    }

    #[test]
    fn a_value_stored_outside_the_range_comes_back_into_it() {
        assert_eq!(stepped(180, -5, &(1..=120)), 120);
        assert_eq!(stepped(0, 1, &(5..=120)), 5);
    }
}
