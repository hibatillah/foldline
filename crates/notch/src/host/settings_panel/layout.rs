//! Where the panel's controls and tabs sit, and what a press hits.

use super::{
    CARD_PADDING, CHIP_HEIGHT, CHIP_PADDING, DONE_WIDTH, GAP, LABELLED_ROW, PICKS_ROW, ROUND_BUTTON,
    SEARCH_FIELD_HEIGHT, SEARCH_FIELD_TOP, SEARCH_RESULT_TOP, SEARCH_ROW, SEGMENT_HEIGHT, STEPPER_ROW,
    STEPPERS_PER_ROW, SWITCH_ROW, SWITCHES_PER_ROW, TAB_HEIGHT, TAB_PADDING, TEXT_ROW, Tab,
};
use crate::{
    gfx::{Point, Rect},
    page::Control,
};

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

pub(super) fn row_height(control: &Control) -> f32 {
    match control {
        Control::Switch { .. } => SWITCH_ROW,
        Control::Stepper { .. } => STEPPER_ROW,
        Control::Choice { .. } => LABELLED_ROW,
        Control::Picks { .. } => PICKS_ROW,
        Control::Text { .. } => TEXT_ROW,
        Control::Search { .. } => SEARCH_ROW,
    }
}

pub(super) fn per_row(control: &Control) -> usize {
    match control {
        Control::Switch { .. } => SWITCHES_PER_ROW,
        Control::Stepper { .. } => STEPPERS_PER_ROW,
        _ => 1,
    }
}

/// Groups the controls into rows: neighbouring switches pair up, neighbouring steppers share a row of three.
pub(super) fn rows(controls: &[Control]) -> Vec<Vec<usize>> {
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

pub(super) fn parts(
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
            // The choices sit right of the label, from the card's right edge inward, centered in the row.
            let top = card.center_y() - CHIP_HEIGHT * scale / 2.0;
            let mut right = card.right - padding;
            let mut parts: Vec<(Part, Rect)> = options
                .iter()
                .enumerate()
                .rev()
                .map(|(option, name)| {
                    let width = (measure(name) + 2.0 * CHIP_PADDING + 12.0) * scale;
                    let rect = Rect::new(right - width, top, right, top + CHIP_HEIGHT * scale);
                    right = rect.left - 6.0 * scale;
                    (Part::Pick(option), rect)
                })
                .collect();
            parts.reverse();
            parts
        }
    }
}

pub fn hit(placed: &[Placed], point: Point) -> Option<(usize, Part)> {
    placed.iter().find_map(|item| {
        item.parts.iter().find(|(_, rect)| rect.contains(point)).map(|(part, _)| (item.control, *part))
    })
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
