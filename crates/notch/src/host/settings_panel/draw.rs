//! Drawing the panel's tabs and controls.

use super::{
    BUTTON, CARD, CARD_PADDING, CARD_RADIUS, CENTERED, DIM, FIELD_BORDER, GRAY, HINT, LABEL, OFF_TRACK, ON_TRACK,
    PICKED, PICKED_BORDER, Placed, RAISED, SMALL_LABEL, SUNKEN, SWITCH_SIZE, Tab, TabsLayout, VALUE, is_on, number,
    picked,
};
use crate::{
    gfx::{Canvas, Color, Point, Rect},
    page::Control,
    ui::{FieldLook, TextField},
};

pub fn draw_tabs(canvas: &Canvas, layout: &TabsLayout, labels: &[(Tab, &str)], selected: Tab, opacity: f32) {
    let scale = canvas.scale();
    if let (Some(first), Some(last)) = (layout.tabs.first(), layout.tabs.last()) {
        let track = Rect::new(
            first.1.left - 2.0 * scale,
            first.1.top - 2.0 * scale,
            last.1.right + 2.0 * scale,
            last.1.bottom + 2.0 * scale,
        );
        canvas.fill_rounded_rect(track, track.height() / 2.0, CARD.faded(opacity));
    }
    for ((tab, rect), (_, label)) in layout.tabs.iter().zip(labels) {
        let is_selected = *tab == selected;
        if is_selected {
            canvas.fill_rounded_rect(*rect, rect.height() / 2.0, RAISED.faded(opacity));
        }
        canvas.text(&CENTERED, label, *rect, if is_selected { Color::WHITE } else { GRAY }.faded(opacity));
    }
    canvas.fill_rounded_rect(layout.done, layout.done.height() / 2.0, Color::WHITE.faded(opacity));
    canvas.text(&CENTERED, "Done", layout.done, Color::gray(0.05).faded(opacity));
}

pub struct Editing {
    pub control: usize,
    pub field: TextField,
}

pub fn draw_control(
    canvas: &Canvas,
    control: &Control,
    item: &Placed,
    saved: Option<&str>,
    editing: Option<&TextField>,
    results: &[&str],
    opacity: f32,
) {
    let scale = canvas.scale();
    let padding = CARD_PADDING * scale;
    let card = item.card;
    canvas.fill_rounded_rect(card, CARD_RADIUS * scale, CARD.faded(opacity));
    let label_area = |top: f32, bottom: f32| {
        Rect::new(card.left + padding, card.top + top * scale, card.right - padding, card.top + bottom * scale)
    };
    match control {
        Control::Switch { label, hint, default, .. } => {
            let (width, height) = (SWITCH_SIZE.0 * scale, SWITCH_SIZE.1 * scale);
            let track = Rect::new(
                card.right - padding - width,
                card.center_y() - height / 2.0,
                card.right - padding,
                card.center_y() + height / 2.0,
            );
            let text_right = track.left - 8.0 * scale;
            canvas.text(
                &LABEL,
                label,
                Rect { right: text_right, ..label_area(6.0, 24.0) },
                Color::WHITE.faded(opacity),
            );
            canvas.text(&HINT, hint, Rect { right: text_right, ..label_area(24.0, 38.0) }, GRAY.faded(opacity));
            let on = is_on(saved, *default);
            canvas.fill_rounded_rect(track, height / 2.0, if on { ON_TRACK } else { OFF_TRACK }.faded(opacity));
            let knob = height / 2.0 - 2.0 * scale;
            let knob_x = if on { track.right - height / 2.0 } else { track.left + height / 2.0 };
            canvas.fill_circle(Point::new(knob_x, track.center_y()), knob, Color::WHITE.faded(opacity));
        }
        Control::Stepper { label, unit, default, min, max, .. } => {
            canvas.text(&SMALL_LABEL, label, label_area(8.0, 22.0), GRAY.faded(opacity));
            let value = number(saved, *default, *min, *max);
            let number = if *min < 0 { format!("{value:+}") } else { value.to_string() };
            let shown = if unit.is_empty() { number } else { format!("{number} {unit}") };
            if let [(_, minus), (_, plus)] = item.parts.as_slice() {
                for (rect, sign) in [(minus, "\u{2212}"), (plus, "+")] {
                    canvas.fill_rounded_rect(*rect, rect.height() / 2.0, RAISED.faded(opacity));
                    canvas.text(&BUTTON, sign, *rect, Color::WHITE.faded(opacity));
                }
                let value_area = Rect::new(minus.right, minus.top, plus.left, minus.bottom);
                canvas.text(&VALUE, &shown, value_area, Color::WHITE.faded(opacity));
            }
        }
        Control::Choice { label, options, default, .. } => {
            canvas.text(&SMALL_LABEL, label, label_area(8.0, 22.0), Color::WHITE.faded(opacity));
            let current = saved.filter(|saved| options.iter().any(|(word, _)| word == saved)).unwrap_or(default);
            if let (Some(first), Some(last)) = (item.parts.first(), item.parts.last()) {
                let track = Rect { left: first.1.left, right: last.1.right, ..first.1 };
                canvas.fill_rounded_rect(track, 10.0 * scale, SUNKEN.faded(opacity));
            }
            for ((_, rect), (word, option_label)) in item.parts.iter().zip(options.iter()) {
                let is_current = *word == current;
                if is_current {
                    let inner = Rect::new(
                        rect.left + 2.0 * scale,
                        rect.top + 2.0 * scale,
                        rect.right - 2.0 * scale,
                        rect.bottom - 2.0 * scale,
                    );
                    canvas.fill_rounded_rect(inner, 8.0 * scale, RAISED.faded(opacity));
                }
                canvas.text(
                    &CENTERED,
                    option_label,
                    *rect,
                    if is_current { Color::WHITE } else { GRAY }.faded(opacity),
                );
            }
        }
        Control::Text { label, placeholder, is_secret, .. } => {
            let label_width = 92.0 * scale;
            let label_rect = Rect::new(card.left + padding, card.top, card.left + padding + label_width, card.bottom);
            canvas.text(&SMALL_LABEL, label, label_rect, GRAY.faded(opacity));
            let field_area = Rect::new(label_rect.right, card.top, card.right - padding, card.bottom);
            let look = FieldLook { style: &LABEL, color: Color::WHITE, placeholder };
            match editing {
                Some(field) => {
                    let border = Rect::new(card.left, card.top, card.right, card.bottom);
                    canvas.fill_rounded_rect(border, CARD_RADIUS * scale, FIELD_BORDER.faded(opacity));
                    let inner = Rect::new(card.left + scale, card.top + scale, card.right - scale, card.bottom - scale);
                    canvas.fill_rounded_rect(inner, (CARD_RADIUS - 1.0) * scale, CARD.faded(opacity));
                    canvas.text(&SMALL_LABEL, label, label_rect, GRAY.faded(opacity));
                    field.draw(canvas, field_area, &look, true, opacity);
                }
                None => {
                    let text = saved.unwrap_or_default();
                    let shown = if *is_secret && !text.is_empty() { "\u{2022}".repeat(12) } else { text.to_owned() };
                    let color = if shown.is_empty() { DIM } else { Color::WHITE };
                    let content = if shown.is_empty() { (*placeholder).to_owned() } else { shown };
                    canvas.text(&LABEL, &content, field_area, color.faded(opacity));
                }
            }
        }
        Control::Search { label, placeholder, .. } => {
            canvas.text(&SMALL_LABEL, label, label_area(8.0, 22.0), GRAY.faded(opacity));
            let Some((_, field_rect)) = item.parts.first() else { return };
            let look = FieldLook { style: &LABEL, color: Color::WHITE, placeholder };
            canvas.fill_rounded_rect(*field_rect, 8.0 * scale, SUNKEN.faded(opacity));
            let text_area =
                Rect { left: field_rect.left + 10.0 * scale, right: field_rect.right - 10.0 * scale, ..*field_rect };
            match editing {
                Some(field) => {
                    canvas.fill_rounded_rect(*field_rect, 8.0 * scale, FIELD_BORDER.faded(opacity));
                    let inner = Rect::new(
                        field_rect.left + scale,
                        field_rect.top + scale,
                        field_rect.right - scale,
                        field_rect.bottom - scale,
                    );
                    canvas.fill_rounded_rect(inner, 7.0 * scale, SUNKEN.faded(opacity));
                    field.draw(canvas, text_area, &look, true, opacity);
                }
                None => {
                    let shown = saved.filter(|text| !text.is_empty());
                    let color = if shown.is_some() { Color::WHITE } else { DIM };
                    canvas.text(&LABEL, shown.unwrap_or(placeholder), text_area, color.faded(opacity));
                }
            }
            for ((_, rect), name) in item.parts.iter().skip(1).zip(results) {
                canvas.fill_rounded_rect(*rect, rect.height() / 2.0, RAISED.faded(opacity));
                canvas.text(&CENTERED, name, *rect, Color::WHITE.faded(opacity));
            }
        }
        Control::Picks { label, options, max, .. } => {
            let chosen = picked(saved, options, *max);
            canvas.text(&LABEL, label, label_area(9.0, 25.0), Color::WHITE.faded(opacity));
            canvas.text(
                &SMALL_LABEL,
                &picks_note(label, options.len(), chosen.len(), *max),
                label_area(27.0, 41.0),
                GRAY.faded(opacity),
            );
            let is_limit_reached = chosen.len() >= *max;
            for ((_, rect), name) in item.parts.iter().zip(options) {
                let is_picked = chosen.contains(name);
                let is_dimmed = !is_picked && is_limit_reached;
                let outline = if is_dimmed { OFF_TRACK.with_alpha(0.5) } else { OFF_TRACK };
                let radius = rect.height() / 2.0;
                if is_picked {
                    canvas.fill_rounded_rect(*rect, radius, PICKED_BORDER.faded(opacity));
                    let inner = Rect::new(rect.left + scale, rect.top + scale, rect.right - scale, rect.bottom - scale);
                    canvas.fill_rounded_rect(inner, radius - scale, PICKED.faded(opacity));
                } else {
                    crate::ui::draw_dashed_outline(
                        canvas,
                        *rect,
                        radius,
                        scale,
                        4.0 * scale,
                        3.0 * scale,
                        outline.faded(opacity),
                    );
                }
                let text = if is_picked {
                    std::borrow::Cow::Owned(format!("\u{2713} {name}"))
                } else {
                    std::borrow::Cow::Borrowed(name.as_str())
                };
                let color = if is_picked {
                    Color::WHITE
                } else if is_dimmed {
                    DIM
                } else {
                    GRAY
                };
                canvas.text(&CENTERED, &text, *rect, color.faded(opacity));
            }
        }
    }
}

/// The line under a picks label: what is allowed and how many are chosen, or that only one thing exists to choose.
pub(super) fn picks_note(label: &str, found: usize, chosen: usize, max: usize) -> String {
    if found <= 1 {
        let noun = label.strip_suffix('s').unwrap_or(label);
        let noun = if noun.chars().all(char::is_uppercase) { noun.to_owned() } else { noun.to_lowercase() };
        return format!("One {noun} found");
    }
    format!("Show up to {max} \u{00b7} {chosen} chosen")
}
