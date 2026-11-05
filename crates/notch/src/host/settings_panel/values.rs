//! The value a press on a control saves.

use super::Part;
use crate::page::Control;

pub(super) fn is_on(text: Option<&str>, default: bool) -> bool {
    match text {
        Some("on") => true,
        Some("off") => false,
        _ => default,
    }
}

pub(super) fn number(text: Option<&str>, default: i32, min: i32, max: i32) -> i32 {
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
