//! The `FoldlineStartup` task of the MSIX manifest (`installer/msix/AppxManifest.xml`).
//!
//! A packaged build's Run-key writes are virtualized and never read at logon, so the task is the only startup entry.
//! Every call here blocks: run it on a background thread.

use windows::{
    ApplicationModel::{StartupTask, StartupTaskState},
    Win32::{
        Storage::Packaging::Appx::GetCurrentPackageFullName,
        System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize},
    },
    core::HSTRING,
};

use super::presentation::TaskState;

const TASK_ID: &str = "FoldlineStartup";
/// `APPMODEL_ERROR_NO_PACKAGE`: the process has no package identity.
const NO_PACKAGE: u32 = 15700;

/// Any status but "no package" means an identity exists (a packaged process answers "buffer too small" to a zero buffer).
fn identity_from_status(status: u32) -> bool {
    status != NO_PACKAGE
}

pub(super) fn has_package_identity() -> bool {
    let mut length = 0u32;
    // SAFETY: only the length is requested; no output buffer is passed.
    let status = unsafe { GetCurrentPackageFullName(&mut length, None) };
    identity_from_status(status.0)
}

fn task_state(state: StartupTaskState) -> Option<TaskState> {
    match state {
        StartupTaskState::Enabled => Some(TaskState::Enabled),
        StartupTaskState::Disabled => Some(TaskState::Disabled),
        StartupTaskState::DisabledByUser => Some(TaskState::DisabledByUser),
        StartupTaskState::DisabledByPolicy => Some(TaskState::DisabledByPolicy),
        StartupTaskState::EnabledByPolicy => Some(TaskState::EnabledByPolicy),
        _ => None,
    }
}

fn task() -> windows::core::Result<StartupTask> {
    // A worker thread has no apartment yet; a thread that already has one answers with an error that is safe to ignore.
    // SAFETY: plain initialization with no pointers.
    let _ = unsafe { RoInitialize(RO_INIT_MULTITHREADED) };
    StartupTask::GetAsync(&HSTRING::from(TASK_ID))?.join()
}

pub(super) fn read_state() -> Option<TaskState> {
    task_state(task().ok()?.State().ok()?)
}

/// Enables or disables the task and returns the state Windows reports afterwards.
pub(super) fn write_state(enabled: bool) -> Option<TaskState> {
    let task = task().ok()?;
    if enabled {
        // The answer is the new state; a refusal (the user or policy) comes back as a disabled state.
        task_state(task.RequestEnableAsync().ok()?.join().ok()?)
    } else {
        task.Disable().ok()?;
        task_state(task.State().ok()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_package_status_means_unpackaged_and_any_other_status_means_packaged() {
        assert!(!identity_from_status(NO_PACKAGE));
        assert!(identity_from_status(122));
        assert!(identity_from_status(0));
    }

    #[test]
    fn every_windows_state_maps_to_its_own_state() {
        assert_eq!(task_state(StartupTaskState::DisabledByUser), Some(TaskState::DisabledByUser));
        assert_eq!(task_state(StartupTaskState::EnabledByPolicy), Some(TaskState::EnabledByPolicy));
        assert_eq!(task_state(StartupTaskState(99)), None);
    }

    #[test]
    fn this_test_process_has_no_package_identity() {
        assert!(!has_package_identity());
    }
}
