//! The classic current-user Run value, used by a build without package identity (the Inno install and a loose exe).

use std::path::{Path, PathBuf};

use windows::{
    Win32::System::Registry::{
        HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW,
    },
    core::{HSTRING, PCWSTR, w},
};

const RUN_KEY: PCWSTR = w!(r"Software\Microsoft\Windows\CurrentVersion\Run");
const VALUE_NAME: PCWSTR = w!("Foldline");
/// The value name builds before the rename wrote; it still counts as on, and is removed when the switch changes.
const LEGACY_VALUE_NAME: PCWSTR = w!("Notch");

const HOST_EXECUTABLE: &str = "foldline.exe";

/// `foldline.exe` sits in the same folder as this program.
fn host_executable_beside(settings_executable: &Path) -> PathBuf {
    settings_executable.with_file_name(HOST_EXECUTABLE)
}

fn host_executable() -> Option<PathBuf> {
    let executable = host_executable_beside(&std::env::current_exe().ok()?);
    executable.is_file().then_some(executable)
}

fn has_value(value_name: PCWSTR) -> bool {
    let mut bytes = 0u32;
    // SAFETY: only the size of the value is requested; no output buffer is passed.
    let status =
        unsafe { RegGetValueW(HKEY_CURRENT_USER, RUN_KEY, value_name, RRF_RT_REG_SZ, None, None, Some(&mut bytes)) };
    status.is_ok() && bytes > 0
}

fn delete_value(value_name: PCWSTR) {
    // SAFETY: both strings are null-terminated and outlive the call.
    let _ = unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN_KEY, value_name) };
}

pub(super) fn is_enabled() -> bool {
    has_value(VALUE_NAME) || has_value(LEGACY_VALUE_NAME)
}

pub(super) fn set_enabled(enabled: bool) {
    if !enabled {
        delete_value(VALUE_NAME);
        delete_value(LEGACY_VALUE_NAME);
        return;
    }
    let Some(executable) = host_executable() else { return };
    let command = HSTRING::from(format!("\"{}\"", executable.display()));
    let bytes = (command.len() + 1) * size_of::<u16>();
    // SAFETY: `command` is a null-terminated UTF-16 buffer of `bytes` bytes that outlives the call.
    let written = unsafe {
        RegSetKeyValueW(HKEY_CURRENT_USER, RUN_KEY, VALUE_NAME, REG_SZ.0, Some(command.as_ptr().cast()), bytes as u32)
    };
    if written.is_ok() {
        delete_value(LEGACY_VALUE_NAME);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_run_value_points_at_foldline_exe_beside_the_settings_program() {
        let path = host_executable_beside(Path::new(r"C:\Apps\Foldline\foldline-settings.exe"));
        assert_eq!(path, PathBuf::from(r"C:\Apps\Foldline\foldline.exe"));
    }
}
