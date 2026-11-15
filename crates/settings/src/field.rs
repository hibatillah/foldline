use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext as _, ElementId, Entity, Focusable as _, SharedString, Subscription, Window, div,
    prelude::*, px,
};
use gpui_component::{
    ActiveTheme as _, Icon,
    input::{Input, InputEvent, InputState},
};
use store::Settings;

use crate::{layout::Render, sections::apply, switch::Switch};

pub fn on_off(enabled: bool) -> &'static str {
    if enabled { "on" } else { "off" }
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
    /// A file path, with a file icon before it.
    Path { width: f32 },
    /// Shown as dots, with a button that reveals it.
    Secret { width: f32 },
}

impl Kind {
    fn width(&self) -> f32 {
        match self {
            Kind::Path { width } | Kind::Secret { width } => *width,
        }
    }
}

fn shown_value(settings: &Settings, key: &str, kind: &Kind) -> String {
    match kind {
        Kind::Path { .. } | Kind::Secret { .. } => settings.get_string(key).unwrap_or_default().to_owned(),
    }
}

/// A stored positive whole number, else `default`. The notch accepts any positive value, so a hand edit outside a
/// control's range is shown as stored.
pub fn stored_whole_number(settings: &Settings, key: &str, default: u32) -> u32 {
    settings
        .get_string(key)
        .and_then(|text| text.trim().parse::<u32>().ok())
        .filter(|number| *number > 0)
        .unwrap_or(default)
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
    let focused = input.focus_handle(cx).is_focused(window);
    // A hand edit or the tray's own write shows up here instead of being overwritten by a stale value.
    if !focused && input.read(cx).value() != stored {
        input.update(cx, |input, cx| input.set_value(stored, window, cx));
    }
    let file_icon = || Icon::default().path(crate::icons::FILE).size(px(13.0)).text_color(cx.theme().muted_foreground);
    framed(
        focused,
        match kind {
            Kind::Secret { .. } => Input::new(&input).appearance(false).mask_toggle(),
            Kind::Path { .. } => Input::new(&input).appearance(false).prefix(file_icon()),
        },
        cx,
    )
    .w(px(kind.width()))
    .into_any_element()
}

const FIELD_RADIUS: f32 = 8.0;

/// The box around a text input: a thin border (the accent while it has focus) on a fill darker than the block.
pub fn framed(focused: bool, input: Input, cx: &App) -> gpui::Div {
    div()
        .rounded(px(FIELD_RADIUS))
        .border_1()
        .border_color(if focused { crate::palette::accent(cx) } else { crate::palette::input_border(cx) })
        .bg(crate::palette::input_background(cx))
        .child(input)
}

fn commit(key: &'static str, kind: &Kind, input: &Entity<InputState>, window: &mut Window, cx: &mut App) {
    let typed = input.read(cx).value();
    let accepted = match kind {
        Kind::Path { .. } | Kind::Secret { .. } => Some(typed.trim().to_owned()),
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
    fn a_stored_value_outside_a_controls_range_is_shown_not_replaced_by_the_default() {
        let path = std::env::temp_dir().join(format!("notch-settings-shown-{}.txt", std::process::id()));
        std::fs::write(&path, "timer.short_break_minutes=180\ntimer.long_break_minutes=0\ntimer.focus_minutes=abc\n")
            .unwrap();
        let settings = Settings::at(Some(path.clone()));
        let _ = std::fs::remove_file(&path);
        assert_eq!(stored_whole_number(&settings, "timer.short_break_minutes", 5), 180);
        assert_eq!(stored_whole_number(&settings, "timer.long_break_minutes", 15), 15);
        assert_eq!(stored_whole_number(&settings, "timer.focus_minutes", 25), 25);
        assert_eq!(stored_whole_number(&settings, "missing", 7), 7);
    }
}
