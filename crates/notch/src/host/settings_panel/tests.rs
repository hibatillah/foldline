//! What the panel lays out and saves.

use crate::page::Control;

use super::{layout::rows, *};

fn switch(key: &'static str) -> Control {
    Control::Switch { key, label: "Label", hint: "hint", default: true }
}

fn stepper(key: &'static str) -> Control {
    Control::Stepper { key, label: "Focus", unit: "min", default: 25, min: 5, max: 120, step: 5 }
}

#[test]
fn switches_pair_up_and_steppers_share_a_row_of_three() {
    let controls = vec![switch("a"), switch("b"), switch("c"), stepper("d"), stepper("e"), stepper("f"), stepper("g")];
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
