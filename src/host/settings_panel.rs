//! The settings panel inside the notch: tabs in the header, controls below; sizes are device-independent pixels.

use crate::{
    gfx::{Align, Canvas, Color, FontStyle, Point, Rect, UI_FONT, Weight},
    page::Control,
    ui::{FieldLook, TextField},
};

pub const PANEL_WIDTH: f32 = 540.0;
const GAP: f32 = 8.0;
const CARD_RADIUS: f32 = 12.0;
const CARD_PADDING: f32 = 12.0;
const SWITCH_ROW: f32 = 44.0;
const STEPPER_ROW: f32 = 58.0;
const LABELLED_ROW: f32 = 62.0;
const TEXT_ROW: f32 = 40.0;
const SEARCH_ROW: f32 = 96.0;
const SEARCH_FIELD_TOP: f32 = 24.0;
const SEARCH_FIELD_HEIGHT: f32 = 28.0;
const SEARCH_RESULT_TOP: f32 = 60.0;
const SWITCH_SIZE: (f32, f32) = (30.0, 18.0);
const ROUND_BUTTON: f32 = 24.0;
const SEGMENT_HEIGHT: f32 = 26.0;
const CHIP_HEIGHT: f32 = 26.0;
const CHIP_PADDING: f32 = 12.0;
const TAB_HEIGHT: f32 = 22.0;
const TAB_PADDING: f32 = 12.0;
const DONE_WIDTH: f32 = 54.0;
const SWITCHES_PER_ROW: usize = 2;
const STEPPERS_PER_ROW: usize = 3;
pub const TEXT_LIMIT: usize = 400;

const LABEL: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Regular, Align::Leading);
const SMALL_LABEL: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Leading);
const HINT: FontStyle = FontStyle::new(UI_FONT, 10.0, Weight::Regular, Align::Leading);
const VALUE: FontStyle = FontStyle::new("Consolas", 14.0, Weight::Medium, Align::Center);
const CENTERED: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Medium, Align::Center);
const BUTTON: FontStyle = FontStyle::new(UI_FONT, 14.0, Weight::Medium, Align::Center);

const CARD: Color = Color::rgb(0.078, 0.078, 0.094);
const SUNKEN: Color = Color::rgb(0.043, 0.043, 0.055);
const RAISED: Color = Color::rgb(0.165, 0.165, 0.188);
const GRAY: Color = Color::rgb(0.604, 0.604, 0.651);
const DIM: Color = Color::rgb(0.431, 0.431, 0.478);
const OFF_TRACK: Color = Color::rgb(0.227, 0.227, 0.267);
const ON_TRACK: Color = Color::rgb(0.19, 0.82, 0.35);
const PICKED: Color = Color::rgb(0.071, 0.188, 0.106);
const PICKED_BORDER: Color = Color::rgb(0.251, 0.753, 0.341);
const FIELD_BORDER: Color = Color::rgb(0.251, 0.561, 1.0);

/// Startup lives in the registry, not in settings.txt; the host reads and writes it through `startup`.
pub const START_WITH_WINDOWS_KEY: &str = "notch.start_with_windows";

const SHOWING_OPTIONS: &[(&str, &str)] =
    &[("hidden", "Hidden"), ("activities", "Activities"), ("lyrics", "Lyrics"), ("music", "Always")];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    /// The settings of the page that was open.
    /// One of the open page's settings tabs, by position.
    Page(usize),
    General,
    Behavior,
    Pages,
}

pub fn general_controls() -> Vec<Control> {
    vec![
        Control::Switch {
            key: START_WITH_WINDOWS_KEY,
            label: "Start with Windows",
            hint: "when you sign in",
            default: true,
        },
        Control::Switch {
            key: "notch.hide_when_idle",
            label: "Hide when idle",
            hint: "point at the top to show it",
            default: false,
        },
        Control::Switch {
            key: "activity.notifications",
            label: "Notifications",
            hint: "banners from Windows",
            default: true,
        },
        Control::Switch {
            key: "notifications.show_text",
            label: "Notification text",
            hint: "off reads \"New message\"",
            default: true,
        },
    ]
}

pub fn behavior_controls() -> Vec<Control> {
    vec![
        Control::Choice {
            key: "notch.maximized",
            label: "Over a maximized window",
            options: SHOWING_OPTIONS,
            default: "lyrics",
        },
        Control::Choice {
            key: "notch.fullscreen",
            label: "Over a full-screen app",
            options: SHOWING_OPTIONS,
            default: "hidden",
        },
        Control::Switch {
            key: "notch.hover_maximized",
            label: "Open on hover",
            hint: "over a maximized window",
            default: true,
        },
        Control::Switch {
            key: "notch.hover_fullscreen",
            label: "Open on hover",
            hint: "over a full-screen app",
            default: true,
        },
    ]
}

/// A part of a control the pointer can press.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Toggle,
    Minus,
    Plus,
    Option(usize),
    Field,
    Pick(usize),
    Result(usize),
}

pub struct Placed {
    pub control: usize,
    pub card: Rect,
    pub parts: Vec<(Part, Rect)>,
}

fn row_height(control: &Control) -> f32 {
    match control {
        Control::Switch { .. } => SWITCH_ROW,
        Control::Stepper { .. } => STEPPER_ROW,
        Control::Choice { .. } | Control::Picks { .. } => LABELLED_ROW,
        Control::Text { .. } => TEXT_ROW,
        Control::Search { .. } => SEARCH_ROW,
    }
}

fn per_row(control: &Control) -> usize {
    match control {
        Control::Switch { .. } => SWITCHES_PER_ROW,
        Control::Stepper { .. } => STEPPERS_PER_ROW,
        _ => 1,
    }
}

/// Groups the controls into rows: neighbouring switches pair up, neighbouring steppers share a row of three.
fn rows(controls: &[Control]) -> Vec<Vec<usize>> {
    let mut rows: Vec<Vec<usize>> = Vec::new();
    for (index, control) in controls.iter().enumerate() {
        let open_row = rows.last_mut().filter(|row| {
            std::mem::discriminant(&controls[row[0]]) == std::mem::discriminant(control) && row.len() < per_row(control)
        });
        match open_row {
            Some(row) => row.push(index),
            None => rows.push(vec![index]),
        }
    }
    rows
}

pub fn content_height(controls: &[Control]) -> f32 {
    let rows = rows(controls);
    let heights: f32 = rows.iter().map(|row| row_height(&controls[row[0]])).sum();
    heights + GAP * rows.len().saturating_sub(1) as f32
}

/// Places every control inside `content` (device pixels); `measure` gives a label's width in DIP.
pub fn layout(
    controls: &[Control],
    content: Rect,
    scale: f32,
    measure: &dyn Fn(&str) -> f32,
    results: &[&str],
) -> Vec<Placed> {
    let mut placed = Vec::new();
    let mut top = content.top;
    for row in rows(controls) {
        let height = row_height(&controls[row[0]]) * scale;
        let slots = per_row(&controls[row[0]]) as f32;
        let width = (content.width() - GAP * scale * (slots - 1.0)) / slots;
        for (column, &index) in row.iter().enumerate() {
            let left = content.left + column as f32 * (width + GAP * scale);
            let card = Rect::new(left, top, left + width, top + height);
            placed.push(Placed { control: index, card, parts: parts(&controls[index], card, scale, measure, results) });
        }
        top += height + GAP * scale;
    }
    placed
}

fn parts(
    control: &Control,
    card: Rect,
    scale: f32,
    measure: &dyn Fn(&str) -> f32,
    results: &[&str],
) -> Vec<(Part, Rect)> {
    let padding = CARD_PADDING * scale;
    match control {
        Control::Switch { .. } => vec![(Part::Toggle, card)],
        Control::Stepper { .. } => {
            let size = ROUND_BUTTON * scale;
            let top = card.bottom - padding * 0.8 - size;
            let minus = Rect::new(card.left + padding, top, card.left + padding + size, top + size);
            let plus = Rect::new(card.right - padding - size, top, card.right - padding, top + size);
            vec![(Part::Minus, minus), (Part::Plus, plus)]
        }
        Control::Choice { options, .. } => {
            let top = card.bottom - padding * 0.8 - SEGMENT_HEIGHT * scale;
            let track = Rect::new(card.left + padding, top, card.right - padding, top + SEGMENT_HEIGHT * scale);
            let width = track.width() / options.len() as f32;
            (0..options.len())
                .map(|option| {
                    let left = track.left + option as f32 * width;
                    (Part::Option(option), Rect { left, right: left + width, ..track })
                })
                .collect()
        }
        Control::Text { .. } => vec![(Part::Field, card)],
        Control::Search { .. } => {
            let field_top = card.top + SEARCH_FIELD_TOP * scale;
            let field = Rect::new(
                card.left + padding,
                field_top,
                card.right - padding,
                field_top + SEARCH_FIELD_HEIGHT * scale,
            );
            let top = card.top + SEARCH_RESULT_TOP * scale;
            let mut left = card.left + padding;
            let chips = results.iter().enumerate().map(|(result, name)| {
                let width = (measure(name) + 2.0 * CHIP_PADDING) * scale;
                let rect = Rect::new(left, top, left + width, top + CHIP_HEIGHT * scale);
                left = rect.right + 6.0 * scale;
                (Part::Result(result), rect)
            });
            std::iter::once((Part::Field, field)).chain(chips).collect()
        }
        Control::Picks { options, .. } => {
            let top = card.bottom - padding * 0.8 - CHIP_HEIGHT * scale;
            let mut left = card.left + padding;
            options
                .iter()
                .enumerate()
                .map(|(option, name)| {
                    let width = (measure(name) + 2.0 * CHIP_PADDING + 12.0) * scale;
                    let rect = Rect::new(left, top, left + width, top + CHIP_HEIGHT * scale);
                    left = rect.right + 6.0 * scale;
                    (Part::Pick(option), rect)
                })
                .collect()
        }
    }
}

pub fn hit(placed: &[Placed], point: Point) -> Option<(usize, Part)> {
    placed.iter().find_map(|item| {
        item.parts.iter().find(|(_, rect)| rect.contains(point)).map(|(part, _)| (item.control, *part))
    })
}

fn is_on(text: Option<&str>, default: bool) -> bool {
    match text {
        Some("on") => true,
        Some("off") => false,
        _ => default,
    }
}

fn number(text: Option<&str>, default: i32, min: i32, max: i32) -> i32 {
    text.and_then(|text| text.trim().parse().ok()).filter(|value| (min..=max).contains(value)).unwrap_or(default)
}

/// The picked options: the saved list (only names still offered), else the first `max`.
pub fn picked(text: Option<&str>, options: &[String], max: usize) -> Vec<String> {
    match text {
        Some(saved) => saved
            .split(',')
            .map(str::trim)
            .filter(|name| options.iter().any(|option| option == name))
            .take(max)
            .map(str::to_owned)
            .collect(),
        None => options.iter().take(max).cloned().collect(),
    }
}

/// The value to save after pressing `part`, or `None` when the press changes nothing.
pub fn pressed_value(control: &Control, part: Part, saved: Option<&str>) -> Option<String> {
    match (control, part) {
        (Control::Switch { default, .. }, Part::Toggle) => {
            Some(if is_on(saved, *default) { "off" } else { "on" }.to_owned())
        }
        (Control::Stepper { default, min, max, step, .. }, Part::Minus | Part::Plus) => {
            let current = number(saved, *default, *min, *max);
            let next = if part == Part::Minus {
                current.saturating_sub(*step).max(*min)
            } else {
                current.saturating_add(*step).min(*max)
            };
            (next != current).then(|| next.to_string())
        }
        (Control::Choice { options, default, .. }, Part::Option(option)) => {
            let word = options.get(option)?.0;
            let current = saved.filter(|saved| options.iter().any(|(known, _)| known == saved)).unwrap_or(default);
            (word != current).then(|| word.to_owned())
        }
        (Control::Picks { options, max, .. }, Part::Pick(option)) => {
            let name = options.get(option)?;
            let mut list = picked(saved, options, *max);
            if let Some(position) = list.iter().position(|picked| picked == name) {
                list.remove(position);
            } else if list.len() < *max {
                list.push(name.clone());
            } else {
                return None;
            }
            Some(list.join(","))
        }
        _ => None,
    }
}

pub struct TabsLayout {
    pub tabs: Vec<(Tab, Rect)>,
    pub done: Rect,
}

pub fn tabs_layout(tabs: &[(Tab, &str)], header: Rect, scale: f32, measure: &dyn Fn(&str) -> f32) -> TabsLayout {
    let height = TAB_HEIGHT * scale;
    let top = header.center_y() - height / 2.0;
    let mut left = header.left + 2.0 * scale;
    let placed = tabs
        .iter()
        .map(|&(tab, label)| {
            let width = (measure(label) + 2.0 * TAB_PADDING) * scale;
            let rect = Rect::new(left, top, left + width, top + height);
            left = rect.right + 4.0 * scale;
            (tab, rect)
        })
        .collect();
    let done = Rect::new(header.right - DONE_WIDTH * scale, top, header.right, top + height);
    TabsLayout { tabs: placed, done }
}

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
            canvas.text(
                &SMALL_LABEL,
                &format!("{label} \u{00b7} up to {max}"),
                label_area(8.0, 22.0),
                GRAY.faded(opacity),
            );
            let chosen = picked(saved, options, *max);
            for ((_, rect), name) in item.parts.iter().zip(options) {
                let is_picked = chosen.contains(name);
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
                        OFF_TRACK.faded(opacity),
                    );
                }
                let text = if is_picked {
                    std::borrow::Cow::Owned(format!("\u{2713} {name}"))
                } else {
                    std::borrow::Cow::Borrowed(name.as_str())
                };
                canvas.text(&CENTERED, &text, *rect, if is_picked { Color::WHITE } else { GRAY }.faded(opacity));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn switch(key: &'static str) -> Control {
        Control::Switch { key, label: "Label", hint: "hint", default: true }
    }

    fn stepper(key: &'static str) -> Control {
        Control::Stepper { key, label: "Focus", unit: "min", default: 25, min: 5, max: 120, step: 5 }
    }

    #[test]
    fn switches_pair_up_and_steppers_share_a_row_of_three() {
        let controls =
            vec![switch("a"), switch("b"), switch("c"), stepper("d"), stepper("e"), stepper("f"), stepper("g")];
        assert_eq!(rows(&controls), vec![vec![0, 1], vec![2], vec![3, 4, 5], vec![6]]);
    }

    #[test]
    fn the_general_and_behavior_tabs_fit_the_tallest_page() {
        assert!(content_height(&general_controls()) <= crate::page::MAX_CONTENT_HEIGHT);
        assert!(content_height(&behavior_controls()) <= crate::page::MAX_CONTENT_HEIGHT);
    }

    #[test]
    fn the_behavior_choices_match_the_hosts_words_and_defaults() {
        use crate::host::visibility::{DEFAULT_FULLSCREEN, DEFAULT_MAXIMIZED, Showing};
        let controls = behavior_controls();
        let defaults: Vec<&str> = controls
            .iter()
            .filter_map(|control| match control {
                Control::Choice { default, options, .. } => {
                    assert!(options.iter().all(|(word, _)| Showing::parse(word).is_some()), "{options:?}");
                    Some(*default)
                }
                _ => None,
            })
            .collect();
        assert_eq!(defaults, [DEFAULT_MAXIMIZED.as_text(), DEFAULT_FULLSCREEN.as_text()]);
    }

    #[test]
    fn a_switch_flips_from_its_saved_value_or_its_default() {
        let control = switch("a");
        assert_eq!(pressed_value(&control, Part::Toggle, None), Some("off".to_owned()));
        assert_eq!(pressed_value(&control, Part::Toggle, Some("off")), Some("on".to_owned()));
        assert_eq!(pressed_value(&control, Part::Toggle, Some("banana")), Some("off".to_owned()));
    }

    #[test]
    fn a_stepper_moves_by_its_step_and_stops_at_its_ends() {
        let control = stepper("a");
        assert_eq!(pressed_value(&control, Part::Plus, None), Some("30".to_owned()));
        assert_eq!(pressed_value(&control, Part::Minus, Some("7")), Some("5".to_owned()));
        assert_eq!(pressed_value(&control, Part::Minus, Some("5")), None);
        assert_eq!(pressed_value(&control, Part::Plus, Some("120")), None);
        assert_eq!(pressed_value(&control, Part::Plus, Some("999")), Some("30".to_owned()));
    }

    #[test]
    fn a_choice_saves_a_new_word_only() {
        let control = &behavior_controls()[0];
        assert_eq!(pressed_value(control, Part::Option(3), None), Some("music".to_owned()));
        assert_eq!(pressed_value(control, Part::Option(2), None), None);
        assert_eq!(pressed_value(control, Part::Option(9), None), None);
    }

    #[test]
    fn picks_toggle_in_order_and_refuse_past_the_limit() {
        let options = vec!["C:".to_owned(), "D:".to_owned(), "G:".to_owned(), "H:".to_owned()];
        let control = Control::Picks { key: "stats.disks", label: "Disks", options: options.clone(), max: 3 };
        assert_eq!(picked(None, &options, 3), ["C:", "D:", "G:"]);
        assert_eq!(pressed_value(&control, Part::Pick(1), None), Some("C:,G:".to_owned()));
        assert_eq!(pressed_value(&control, Part::Pick(3), None), None);
        assert_eq!(pressed_value(&control, Part::Pick(3), Some("G:,C:")), Some("G:,C:,H:".to_owned()));
        assert_eq!(picked(Some("X:,G:"), &options, 3), ["G:"]);
    }
}
