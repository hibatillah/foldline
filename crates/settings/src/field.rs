use std::{ops::RangeInclusive, rc::Rc};

use gpui::{
    AnyElement, App, AppContext as _, ElementId, Entity, Focusable as _, SharedString, Subscription, Window, div,
    prelude::*, px,
};
use gpui_component::{
    ActiveTheme as _,
    input::{Input, InputEvent, InputState},
    switch::Switch,
};
use store::Settings;

use crate::{layout::Render, sections::apply};

pub fn on_off(enabled: bool) -> &'static str {
    if enabled { "on" } else { "off" }
}

pub fn parse_whole_number(input: &str, range: &RangeInclusive<u32>) -> Option<u32> {
    input.trim().parse::<u32>().ok().filter(|number| range.contains(number))
}

/// Saves the key and repaints, so a control shows its new state at once.
pub fn write(key: &str, value: &str, cx: &mut App) {
    apply(&mut Settings::load(), key, value);
    cx.refresh_windows();
}

/// A bare switch; `on_click` gets the new state.
pub fn bare_switch(
    id: impl Into<ElementId>,
    checked: bool,
    on_click: impl Fn(bool, &mut Window, &mut App) + 'static,
) -> Switch {
    Switch::new(id).checked(checked).on_click(move |checked: &bool, window, cx| on_click(*checked, window, cx))
}

/// A switch whose state is read again on every paint, so the tray's own toggles show up too.
pub fn switch(id: &'static str, is_on: impl Fn() -> bool + 'static, set: impl Fn(bool, &mut App) + 'static) -> Render {
    let set = Rc::new(set);
    Rc::new(move |_window, _cx| {
        let set = set.clone();
        bare_switch(id, is_on(), move |enabled, _window, cx| set(enabled, cx)).into_any_element()
    })
}

pub fn toggle(key: &'static str, default: bool) -> Render {
    switch(key, move || Settings::load().get_bool(key, default), move |enabled, cx| write(key, on_off(enabled), cx))
}

#[derive(Clone)]
pub enum Kind {
    Text {
        width: f32,
    },
    /// Shown as dots, with a button that reveals it.
    Secret {
        width: f32,
    },
    WholeNumber {
        default: u32,
        range: RangeInclusive<u32>,
    },
}

impl Kind {
    fn width(&self) -> f32 {
        match self {
            Kind::Text { width } | Kind::Secret { width } => *width,
            Kind::WholeNumber { .. } => 96.0,
        }
    }
}

fn shown_value(settings: &Settings, key: &str, kind: &Kind) -> String {
    match kind {
        Kind::Text { .. } | Kind::Secret { .. } => settings.get_string(key).unwrap_or_default().to_owned(),
        Kind::WholeNumber { default, .. } => {
            // The notch accepts any positive value, so a hand edit outside the field's range is shown as stored.
            settings
                .get_string(key)
                .and_then(|text| text.trim().parse::<u32>().ok())
                .filter(|number| *number > 0)
                .unwrap_or(*default)
                .to_string()
        }
    }
}

struct InputBox {
    input: Entity<InputState>,
    _subscription: Subscription,
}

/// A text or number field that saves on Enter or when it loses focus; a rejected value reverts.
pub fn input(key: &'static str, kind: Kind, placeholder: &'static str) -> Render {
    Rc::new(move |window, cx| input_element(key, &kind, placeholder, window, cx))
}

fn input_element(
    key: &'static str,
    kind: &Kind,
    placeholder: &'static str,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let stored = shown_value(&Settings::load(), key, kind);
    let state = window.use_keyed_state(SharedString::from(format!("input-{key}")), cx, {
        let (kind, stored) = (kind.clone(), stored.clone());
        move |window, cx| {
            let secret = matches!(kind, Kind::Secret { .. });
            let input = cx.new(|cx| {
                let state = InputState::new(window, cx).placeholder(placeholder).default_value(stored);
                if secret { state.masked(true) } else { state }
            });
            let _subscription =
                cx.subscribe_in(&input, window, move |_box: &mut InputBox, input, event, window, cx| {
                    if matches!(event, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                        commit(key, &kind, input, window, cx);
                    }
                });
            InputBox { input, _subscription }
        }
    });
    let input = state.read(cx).input.clone();
    // A hand edit or the tray's own write shows up here instead of being overwritten by a stale value.
    if !input.focus_handle(cx).is_focused(window) && input.read(cx).value() != stored {
        input.update(cx, |input, cx| input.set_value(stored, window, cx));
    }
    framed(
        kind.width(),
        match kind {
            Kind::Secret { .. } => Input::new(&input).appearance(false).mask_toggle(),
            _ => Input::new(&input).appearance(false),
        },
        cx,
    )
    .into_any_element()
}

/// The box around a text input: a border and a field background a step lighter than the group.
pub fn framed(width: f32, input: Input, cx: &App) -> gpui::Div {
    div()
        .w(px(width))
        .rounded(cx.theme().radius)
        .border_1()
        .border_color(cx.theme().input)
        .bg(crate::input_background(cx))
        .child(input)
}

fn commit(key: &'static str, kind: &Kind, input: &Entity<InputState>, window: &mut Window, cx: &mut App) {
    let typed = input.read(cx).value();
    let accepted = match kind {
        Kind::Text { .. } | Kind::Secret { .. } => Some(typed.trim().to_owned()),
        Kind::WholeNumber { range, .. } => parse_whole_number(&typed, range).map(|number| number.to_string()),
    };
    let stored = shown_value(&Settings::load(), key, kind);
    let shown = match accepted {
        Some(value) => {
            if value != stored {
                write(key, &value, cx);
            }
            value
        }
        None => stored,
    };
    input.update(cx, |input, cx| input.set_value(shown, window, cx));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggles_write_on_and_off_words() {
        assert_eq!(on_off(true), "on");
        assert_eq!(on_off(false), "off");
    }

    #[test]
    fn numbers_outside_the_range_are_rejected() {
        let range = 5..=120;
        assert_eq!(parse_whole_number("4", &range), None);
        assert_eq!(parse_whole_number("121", &range), None);
        assert_eq!(parse_whole_number("abc", &range), None);
        assert_eq!(parse_whole_number("", &range), None);
        assert_eq!(parse_whole_number(" 5 ", &range), Some(5));
        assert_eq!(parse_whole_number("120", &range), Some(120));
    }

    #[test]
    fn a_stored_value_outside_the_range_is_shown_not_replaced_by_the_default() {
        let path = std::env::temp_dir().join(format!("notch-settings-shown-{}.txt", std::process::id()));
        std::fs::write(&path, "timer.short_break_minutes=180\ntimer.long_break_minutes=0\ntimer.focus_minutes=abc\n")
            .unwrap();
        let settings = Settings::at(Some(path.clone()));
        let _ = std::fs::remove_file(&path);
        let kind = |default| Kind::WholeNumber { default, range: 1..=120 };
        assert_eq!(shown_value(&settings, "timer.short_break_minutes", &kind(5)), "180");
        assert_eq!(shown_value(&settings, "timer.long_break_minutes", &kind(15)), "15");
        assert_eq!(shown_value(&settings, "timer.focus_minutes", &kind(25)), "25");
        assert_eq!(shown_value(&settings, "missing", &kind(7)), "7");
    }
}
