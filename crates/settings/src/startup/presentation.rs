//! How the Start with Windows switch looks for each state of the packaged startup task; no Windows calls here.

/// `Windows.ApplicationModel.StartupTaskState`, without the Windows type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    Enabled,
    Disabled,
    DisabledByUser,
    DisabledByPolicy,
    EnabledByPolicy,
}

/// The short line shown beside a switch that cannot be changed here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Note {
    TurnedOffInWindows,
    ManagedByOrganization,
    /// The window runs from a package folder without package identity, so only the Run key is available.
    ClassicEntry,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Presentation {
    pub checked: bool,
    pub interactive: bool,
    pub note: Option<Note>,
}

/// `None` is a state not read yet: shown off and locked until the first answer arrives.
pub fn present_task(state: Option<TaskState>) -> Presentation {
    let (checked, interactive, note) = match state {
        None => (false, false, None),
        Some(TaskState::Enabled) => (true, true, None),
        Some(TaskState::Disabled) => (false, true, None),
        Some(TaskState::DisabledByUser) => (false, false, Some(Note::TurnedOffInWindows)),
        Some(TaskState::DisabledByPolicy) => (false, false, Some(Note::ManagedByOrganization)),
        Some(TaskState::EnabledByPolicy) => (true, false, Some(Note::ManagedByOrganization)),
    };
    Presentation { checked, interactive, note }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shown(state: Option<TaskState>) -> (bool, bool, Option<Note>) {
        let presentation = present_task(state);
        (presentation.checked, presentation.interactive, presentation.note)
    }

    #[test]
    fn a_state_not_read_yet_shows_off_and_locked() {
        assert_eq!(shown(None), (false, false, None));
    }

    #[test]
    fn enabled_and_disabled_are_plain_switches() {
        assert_eq!(shown(Some(TaskState::Enabled)), (true, true, None));
        assert_eq!(shown(Some(TaskState::Disabled)), (false, true, None));
    }

    #[test]
    fn a_task_turned_off_in_windows_settings_shows_off_locked_with_the_windows_note() {
        assert_eq!(shown(Some(TaskState::DisabledByUser)), (false, false, Some(Note::TurnedOffInWindows)));
    }

    #[test]
    fn policy_states_are_locked_with_the_organization_note() {
        assert_eq!(shown(Some(TaskState::DisabledByPolicy)), (false, false, Some(Note::ManagedByOrganization)));
        assert_eq!(shown(Some(TaskState::EnabledByPolicy)), (true, false, Some(Note::ManagedByOrganization)));
    }
}
