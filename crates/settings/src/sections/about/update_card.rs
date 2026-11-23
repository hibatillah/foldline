//! What the app card's right side shows and does for each state of the update check.

use crate::{i18n::tr, updates::CheckState};

/// What a click on the card's button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum UpdateAction {
    Check,
    Download,
    /// A check is running, so a click changes nothing.
    None,
}

/// The note beside the button, the button's text and what the button does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct UpdateView {
    pub note: Option<String>,
    pub button: &'static str,
    pub action: UpdateAction,
}

pub(super) fn update_view(state: &CheckState) -> UpdateView {
    let check = tr!("about.check_updates");
    match state {
        CheckState::Idle => UpdateView { note: None, button: check, action: UpdateAction::Check },
        CheckState::Checking => {
            UpdateView { note: Some(tr!("about.checking").to_owned()), button: check, action: UpdateAction::None }
        }
        CheckState::UpToDate => {
            UpdateView { note: Some(tr!("about.up_to_date").to_owned()), button: check, action: UpdateAction::Check }
        }
        CheckState::Available { version } => UpdateView {
            note: Some(tr!("about.update_available", version = version)),
            button: tr!("about.download"),
            action: UpdateAction::Download,
        },
        CheckState::Failed => {
            UpdateView { note: Some(tr!("about.check_failed").to_owned()), button: check, action: UpdateAction::Check }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use store::language::{Language, with_override};

    fn view(state: CheckState) -> UpdateView {
        update_view(&state)
    }

    #[test]
    fn idle_shows_no_note_and_a_check_button() {
        let shown = view(CheckState::Idle);
        assert_eq!(shown, UpdateView { note: None, button: "Check for updates", action: UpdateAction::Check });
    }

    #[test]
    fn checking_says_so_and_a_click_does_nothing() {
        let shown = view(CheckState::Checking);
        assert_eq!(shown.note.as_deref(), Some("Checking..."));
        assert_eq!(shown.button, "Check for updates");
        assert_eq!(shown.action, UpdateAction::None);
    }

    #[test]
    fn up_to_date_says_so_and_the_button_checks_again() {
        let shown = view(CheckState::UpToDate);
        assert_eq!(shown.note.as_deref(), Some("Foldline is up to date"));
        assert_eq!(shown.action, UpdateAction::Check);
    }

    #[test]
    fn an_available_version_is_named_and_the_button_downloads() {
        let shown = view(CheckState::Available { version: "0.7.0".into() });
        assert_eq!(shown.note.as_deref(), Some("Version 0.7.0 is available"));
        assert_eq!(shown.button, "Download");
        assert_eq!(shown.action, UpdateAction::Download);
    }

    #[test]
    fn a_failed_check_says_so_and_the_button_retries() {
        let shown = view(CheckState::Failed);
        assert_eq!(shown.note.as_deref(), Some("Could not check for updates"));
        assert_eq!(shown.button, "Check for updates");
        assert_eq!(shown.action, UpdateAction::Check);
    }

    #[test]
    fn the_note_follows_the_language_on_every_call() {
        let english = view(CheckState::UpToDate).note;
        let indonesian = with_override(Language::Indonesian, || view(CheckState::UpToDate).note);
        assert_eq!(english.as_deref(), Some("Foldline is up to date"));
        assert_eq!(indonesian.as_deref(), Some("Foldline sudah yang terbaru"));
    }

    #[test]
    fn the_locale_file_no_longer_holds_the_store_texts() {
        let locale = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/locales/app.yml")).unwrap();
        assert!(!locale.contains("about.updates_from_store:"));
        assert!(!locale.contains("about.open_in_store:"));
    }
}
