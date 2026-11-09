//! The System stats page's choices: which disks and GPUs it shows. The notch publishes what exists
//! (`stats.known_disks`, `stats.known_gpus`) when the page is shown; this page writes `stats.disks` and `stats.gpus`.

use gpui::{AnyElement, App, ElementId, SharedString, Window, div, prelude::*};
use gpui_component::{ActiveTheme as _, Disableable as _, checkbox::Checkbox, v_flex};
use store::Settings;

use crate::{
    field,
    layout::{Group, Item, Page},
};

const MAX_DISKS: usize = 3;
const MAX_GPUS: usize = 2;
const DISKS_KEY: &str = "stats.disks";
const GPUS_KEY: &str = "stats.gpus";
const KNOWN_DISKS_KEY: &str = "stats.known_disks";
const KNOWN_GPUS_KEY: &str = "stats.known_gpus";

/// One kind of device the page can show.
#[derive(Clone, Copy)]
struct Kind {
    key: &'static str,
    known_key: &'static str,
    /// Splits both lists; the GPU list also accepts the old comma form when it has no `|`.
    separator: char,
    limit: usize,
    id_prefix: &'static str,
    /// "Up to 3 disks".
    limit_hint: &'static str,
    /// Shown while the notch has not published the list.
    unlisted: &'static str,
    /// Shown when the list is published and empty.
    none_found: &'static str,
}

const DISKS: Kind = Kind {
    key: DISKS_KEY,
    known_key: KNOWN_DISKS_KEY,
    separator: ',',
    limit: MAX_DISKS,
    id_prefix: "stats-disk",
    limit_hint: "Up to 3 disks",
    unlisted: "Open the System stats page in the notch once to list your disks",
    none_found: "No disk found",
};

const GPUS: Kind = Kind {
    key: GPUS_KEY,
    known_key: KNOWN_GPUS_KEY,
    separator: '|',
    limit: MAX_GPUS,
    id_prefix: "stats-gpu",
    limit_hint: "Up to 2 GPUs",
    unlisted: "Open the System stats page in the notch once to list your GPUs",
    none_found: "No GPU found",
};

/// What the notch has published for one kind of device.
#[derive(Debug, PartialEq, Eq)]
enum Availability {
    /// The key is absent: the stats page was never shown.
    Unlisted,
    /// The key is present and empty.
    NoneFound,
    Devices(Vec<String>),
}

fn split_names(text: &str, separator: char) -> Vec<String> {
    text.split(separator).map(str::trim).filter(|name| !name.is_empty()).map(str::to_owned).collect()
}

fn availability(settings: &Settings, kind: Kind) -> Availability {
    match settings.get_string(kind.known_key) {
        None => Availability::Unlisted,
        Some(text) => match split_names(text, kind.separator) {
            names if names.is_empty() => Availability::NoneFound,
            names => Availability::Devices(names),
        },
    }
}

/// The saved choice read like the notch does: GPU lists written with commas before the `|` format still split.
fn saved_names(text: &str, kind: Kind) -> Vec<String> {
    let separator = if kind.separator == '|' && !text.contains('|') { ',' } else { kind.separator };
    split_names(text, separator)
}

/// The known devices that are on, in the known order. An unset key means the first `limit` devices.
fn selected(known: &[String], saved: Option<&str>, kind: Kind) -> Vec<String> {
    match saved {
        None => known.iter().take(kind.limit).cloned().collect(),
        Some(text) => {
            let saved = saved_names(text, kind);
            let mut chosen: Vec<String> = Vec::new();
            for name in saved.into_iter().take(kind.limit) {
                if known.contains(&name) && !chosen.contains(&name) {
                    chosen.push(name);
                }
            }
            known.iter().filter(|name| chosen.contains(name)).cloned().collect()
        }
    }
}

/// The explicit list after switching one device, in the known order; `None` when the limit would be passed.
fn toggled(known: &[String], saved: Option<&str>, kind: Kind, name: &str, on: bool) -> Option<Vec<String>> {
    let mut chosen = selected(known, saved, kind);
    if on && !chosen.iter().any(|chosen_name| chosen_name == name) {
        if chosen.len() >= kind.limit {
            return None;
        }
        chosen.push(name.to_owned());
    }
    if !on {
        chosen.retain(|chosen_name| chosen_name != name);
    }
    Some(known.iter().filter(|known_name| chosen.contains(known_name)).cloned().collect())
}

fn list_text(names: &[String], kind: Kind) -> String {
    names.join(&kind.separator.to_string())
}

fn muted_message(text: &'static str, cx: &App) -> AnyElement {
    div().text_sm().text_color(cx.theme().muted_foreground).child(text).into_any_element()
}

fn rows(kind: Kind, _window: &mut Window, cx: &mut App) -> AnyElement {
    let settings = Settings::load();
    let known = match availability(&settings, kind) {
        Availability::Unlisted => return muted_message(kind.unlisted, cx),
        Availability::NoneFound => return muted_message(kind.none_found, cx),
        Availability::Devices(known) => known,
    };
    let chosen = selected(&known, settings.get_string(kind.key), kind);
    let full = chosen.len() >= kind.limit;
    let any_disabled = full && known.iter().any(|name| !chosen.contains(name));
    let checkboxes = known.iter().map(|name| {
        let checked = chosen.contains(name);
        let device = name.clone();
        Checkbox::new(ElementId::Name(SharedString::from(format!("{}-{name}", kind.id_prefix))))
            .label(name.clone())
            .checked(checked)
            .disabled(full && !checked)
            .on_click(move |on: &bool, _window, cx| {
                let settings = Settings::load();
                let known = match availability(&settings, kind) {
                    Availability::Devices(known) => known,
                    _ => return,
                };
                if let Some(names) = toggled(&known, settings.get_string(kind.key), kind, &device, *on) {
                    field::write(kind.key, &list_text(&names, kind), cx);
                }
            })
    });
    v_flex()
        .gap_3()
        .children(checkboxes)
        .children(any_disabled.then(|| muted_message(kind.limit_hint, cx)))
        .into_any_element()
}

pub fn page() -> Page {
    Page::new(
        "System stats",
        "Choose which disks and graphics cards the System stats page shows.",
        vec![
            Group::titled("Disks", vec![Item::custom(|window, cx| rows(DISKS, window, cx))]),
            Group::titled("GPUs", vec![Item::custom(|window, cx| rows(GPUS, window, cx))]),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    fn settings_with(text: &str) -> Settings {
        let path = std::env::temp_dir().join(format!(
            "notch-settings-stats-{}-{}.txt",
            std::process::id(),
            text.bytes().fold(0u64, |sum, byte| sum.wrapping_mul(31).wrapping_add(u64::from(byte)))
        ));
        std::fs::write(&path, text).expect("write settings");
        let settings = Settings::at(Some(path.clone()));
        let _ = std::fs::remove_file(&path);
        settings
    }

    #[test]
    fn an_unset_key_selects_the_first_three_disks_and_the_first_two_gpus() {
        let disks = names(&["C:", "D:", "E:", "F:"]);
        assert_eq!(selected(&disks, None, DISKS), names(&["C:", "D:", "E:"]));
        let gpus = names(&["RTX 4050", "Radeon 760M", "Arc"]);
        assert_eq!(selected(&gpus, None, GPUS), names(&["RTX 4050", "Radeon 760M"]));
    }

    #[test]
    fn an_empty_value_selects_nothing() {
        assert!(selected(&names(&["C:", "D:"]), Some(""), DISKS).is_empty());
    }

    #[test]
    fn an_explicit_list_is_shown_in_the_known_order_without_gone_or_repeated_devices() {
        let disks = names(&["C:", "D:", "G:"]);
        assert_eq!(selected(&disks, Some("G:, X:, C:, C:"), DISKS), names(&["C:", "G:"]));
    }

    #[test]
    fn a_saved_list_is_cut_at_the_limit_like_the_notch_does() {
        let disks = names(&["C:", "D:", "E:", "F:"]);
        assert_eq!(selected(&disks, Some("F:,E:,D:,C:"), DISKS), names(&["D:", "E:", "F:"]));
    }

    #[test]
    fn gpu_names_split_at_the_bar_and_old_comma_lists_still_read() {
        let gpus = names(&["RTX 4050", "Radeon 760M"]);
        assert_eq!(selected(&gpus, Some("Radeon 760M|RTX 4050"), GPUS), gpus);
        assert_eq!(selected(&gpus, Some("RTX 4050,Radeon 760M"), GPUS), gpus);
        assert_eq!(selected(&gpus, Some("RTX 4050"), GPUS), names(&["RTX 4050"]));
    }

    #[test]
    fn the_first_click_writes_the_explicit_list_in_the_known_order() {
        let disks = names(&["C:", "D:", "G:"]);
        let after = toggled(&disks, None, DISKS, "D:", false).expect("allowed");
        assert_eq!(list_text(&after, DISKS), "C:,G:");
        let after = toggled(&disks, Some(""), DISKS, "G:", true).expect("allowed");
        assert_eq!(list_text(&after, DISKS), "G:");
    }

    #[test]
    fn gpus_are_written_with_a_bar() {
        let gpus = names(&["RTX 4050", "Radeon 760M"]);
        let after = toggled(&gpus, None, GPUS, "RTX 4050", true).expect("already on");
        assert_eq!(list_text(&after, GPUS), "RTX 4050|Radeon 760M");
    }

    #[test]
    fn a_device_over_the_limit_is_refused() {
        let disks = names(&["C:", "D:", "E:", "F:"]);
        assert_eq!(toggled(&disks, None, DISKS, "F:", true), None);
        let gpus = names(&["A", "B", "C"]);
        assert_eq!(toggled(&gpus, Some("A|B"), GPUS, "C", true), None);
        assert!(toggled(&gpus, Some("A|B"), GPUS, "B", false).is_some());
    }

    #[test]
    fn switching_the_last_device_off_writes_an_empty_list() {
        let gpus = names(&["RTX 4050"]);
        let after = toggled(&gpus, None, GPUS, "RTX 4050", false).expect("allowed");
        assert_eq!(list_text(&after, GPUS), "");
    }

    #[test]
    fn absent_lists_are_unlisted_and_empty_lists_mean_none_found() {
        let settings = settings_with("");
        assert_eq!(availability(&settings, DISKS), Availability::Unlisted);
        assert_eq!(availability(&settings, GPUS), Availability::Unlisted);
        let settings = settings_with("stats.known_gpus=\nstats.known_disks=C:,G:\n");
        assert_eq!(availability(&settings, GPUS), Availability::NoneFound);
        assert_eq!(availability(&settings, DISKS), Availability::Devices(names(&["C:", "G:"])));
    }

    #[test]
    fn known_gpu_names_keep_their_spaces_and_are_split_only_at_the_bar() {
        let settings = settings_with("stats.known_gpus=RTX 4050|Radeon 760M\n");
        assert_eq!(availability(&settings, GPUS), Availability::Devices(names(&["RTX 4050", "Radeon 760M"])));
    }

    #[test]
    fn the_write_goes_to_the_file_as_one_key() {
        let path = std::env::temp_dir().join(format!("notch-settings-stats-write-{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut settings = Settings::at(Some(path.clone()));
        let gpus = names(&["RTX 4050", "Radeon 760M"]);
        let after = toggled(&gpus, None, GPUS, "RTX 4050", false).expect("allowed");
        crate::sections::apply(&mut settings, GPUS_KEY, &list_text(&after, GPUS));
        let text = std::fs::read_to_string(&path).expect("settings file written");
        let _ = std::fs::remove_file(&path);
        assert_eq!(text.trim(), "stats.gpus=Radeon 760M");
    }
}
