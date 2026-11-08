//! The `ShowBanner` values the notch replaced, kept in a file so they can be put back even after a crash.

use std::collections::BTreeMap;

/// The value that turns an app's banner off.
pub(super) const BANNER_OFF: u32 = 0;
const MISSING_WORD: &str = "missing";

/// An app's `ShowBanner` before the notch changed it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Original {
    Missing,
    Value(u32),
}

impl Original {
    fn from_current(current: Option<u32>) -> Self {
        current.map_or(Original::Missing, Original::Value)
    }

    /// The value to write back; `None` deletes it.
    pub(super) fn to_value(self) -> Option<u32> {
        match self {
            Original::Missing => None,
            Original::Value(value) => Some(value),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Backup {
    pub originals: BTreeMap<String, Original>,
}

impl Backup {
    /// One `app id<TAB>value` line per app; a line that does not parse is skipped.
    pub(super) fn parse(text: &str) -> Self {
        let originals = text
            .lines()
            .filter_map(|line| {
                let (app_id, value) = line.rsplit_once('\t')?;
                let original = match value.trim() {
                    MISSING_WORD => Original::Missing,
                    number => Original::Value(number.parse().ok()?),
                };
                (!app_id.is_empty()).then(|| (app_id.to_owned(), original))
            })
            .collect();
        Self { originals }
    }

    pub(super) fn to_text(&self) -> String {
        self.originals
            .iter()
            .map(|(app_id, original)| match original {
                Original::Missing => format!("{app_id}\t{MISSING_WORD}\n"),
                Original::Value(value) => format!("{app_id}\t{value}\n"),
            })
            .collect()
    }
}

/// What one hiding pass does: save these originals first, then turn these apps' banners off.
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct HidingPlan {
    pub new_originals: Vec<(String, Original)>,
    pub apps_to_hide: Vec<String>,
}

/// Every app whose banner can show is turned off; an app already off by the user's choice is left out of the backup.
pub(super) fn plan_hiding(apps: &[(String, Option<u32>)], backup: &Backup) -> HidingPlan {
    let mut plan = HidingPlan::default();
    for (app_id, current) in apps {
        if *current == Some(BANNER_OFF) {
            continue;
        }
        if !backup.originals.contains_key(app_id) {
            plan.new_originals.push((app_id.clone(), Original::from_current(*current)));
        }
        plan.apps_to_hide.push(app_id.clone());
    }
    plan
}

/// Only a value the notch still has in place is put back; a value changed since (or an app removed) is left alone.
pub(super) fn should_restore(current: Option<u32>) -> bool {
    current == Some(BANNER_OFF)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn backup_of(entries: &[(&str, Original)]) -> Backup {
        Backup { originals: entries.iter().map(|(app_id, original)| ((*app_id).to_owned(), *original)).collect() }
    }

    fn apps(entries: &[(&str, Option<u32>)]) -> Vec<(String, Option<u32>)> {
        entries.iter().map(|(app_id, value)| ((*app_id).to_owned(), *value)).collect()
    }

    #[test]
    fn a_backup_reads_back_exactly_what_was_written() {
        let backup = backup_of(&[
            ("Microsoft.WindowsStore_8wekyb3d8bbwe!App", Original::Missing),
            ("Microsoft.MicrosoftEdge.Stable_8wekyb3d8bbwe!https://claude.ai/", Original::Value(1)),
        ]);
        assert_eq!(Backup::parse(&backup.to_text()), backup);
    }

    #[test]
    fn a_damaged_backup_line_is_skipped_and_the_others_are_kept() {
        let backup = Backup::parse("Chrome\tmissing\nno tab here\nSpotify\tlots\n\tmissing\nDiscord\t1\n");
        assert_eq!(backup, backup_of(&[("Chrome", Original::Missing), ("Discord", Original::Value(1))]));
    }

    #[test]
    fn an_app_without_a_banner_value_is_saved_as_missing_and_turned_off() {
        let plan = plan_hiding(&apps(&[("Chrome", None)]), &Backup::default());
        assert_eq!(
            plan,
            HidingPlan {
                new_originals: vec![("Chrome".to_owned(), Original::Missing)],
                apps_to_hide: vec!["Chrome".to_owned()],
            }
        );
    }

    #[test]
    fn an_app_the_user_already_silenced_is_neither_saved_nor_written() {
        let plan = plan_hiding(&apps(&[("Microsoft.YourPhone_8wekyb3d8bbwe!App", Some(0))]), &Backup::default());
        assert_eq!(plan, HidingPlan::default());
    }

    #[test]
    fn an_app_turned_back_on_while_hidden_keeps_its_first_original() {
        let backup = backup_of(&[("Chrome", Original::Missing)]);
        let plan = plan_hiding(&apps(&[("Chrome", Some(1))]), &backup);
        assert_eq!(plan, HidingPlan { new_originals: Vec::new(), apps_to_hide: vec!["Chrome".to_owned()] });
    }

    #[test]
    fn a_pass_after_everything_is_hidden_changes_nothing() {
        let backup = backup_of(&[("Chrome", Original::Missing), ("Discord", Original::Value(1))]);
        let plan = plan_hiding(&apps(&[("Chrome", Some(0)), ("Discord", Some(0))]), &backup);
        assert_eq!(plan, HidingPlan::default());
    }

    #[test]
    fn only_a_value_the_notch_left_in_place_is_restored() {
        assert!(should_restore(Some(BANNER_OFF)));
        assert!(!should_restore(Some(1)));
        assert!(!should_restore(None));
    }

    #[test]
    fn a_missing_original_is_restored_by_deleting_the_value() {
        assert_eq!(Original::Missing.to_value(), None);
        assert_eq!(Original::Value(1).to_value(), Some(1));
    }
}
