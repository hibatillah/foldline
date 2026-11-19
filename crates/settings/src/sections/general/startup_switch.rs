//! The Start with Windows switch, with the short line that says why it cannot be changed.

use std::rc::Rc;

use gpui::{App, div, prelude::*, px};
use gpui_component::h_flex;

use crate::{field, i18n::tr, layout::Render, palette, startup, startup::Note};

const NOTE_SIZE: f32 = 12.0;
const LOCKED_OPACITY: f32 = 0.5;

fn note_text(note: Note) -> &'static str {
    match note {
        Note::TurnedOffInWindows => tr!("general.start_note_user"),
        Note::ManagedByOrganization => tr!("general.start_note_policy"),
        Note::ClassicEntry => tr!("general.start_note_classic"),
    }
}

/// Runs the blocking startup call off the UI thread, then repaints.
fn run_in_background(work: impl FnOnce() + Send + 'static, cx: &mut App) {
    cx.spawn(async move |cx| {
        cx.background_executor().spawn(async move { work() }).await;
        let _ = cx.update(|cx| cx.refresh_windows());
    })
    .detach();
}

pub fn control() -> Render {
    Rc::new(|_window, cx| {
        if startup::needs_load() {
            run_in_background(startup::load, cx);
        }
        let shown = startup::presentation();
        let bare = field::bare_switch("start-with-windows", shown.checked, move |enabled, _window, cx| {
            if shown.interactive {
                run_in_background(move || startup::set_enabled(enabled), cx);
            }
        });
        let switch = div().opacity(if shown.interactive { 1.0 } else { LOCKED_OPACITY }).child(bare);
        h_flex()
            .gap(px(10.0))
            .items_center()
            .children(
                shown
                    .note
                    .map(|note| div().text_size(px(NOTE_SIZE)).text_color(palette::muted(cx)).child(note_text(note))),
            )
            .child(switch)
            .into_any_element()
    })
}
