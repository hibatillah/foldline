//! Finds a window the app already has open, so a click on a notification brings that window forward instead of
//! starting a second copy of the app (Warp opens an empty new window when it is started again).

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM},
        UI::WindowsAndMessaging::{
            EnumWindows, GW_OWNER, GetWindow, GetWindowTextLengthW, GetWindowThreadProcessId, IsIconic,
            IsWindowVisible, SW_RESTORE, SetForegroundWindow, ShowWindow,
        },
    },
    core::{BOOL, Result},
};

use crate::{log, platform::window_info::process_image_path};

/// Brings forward the first visible, titled window of a running program named after `app_user_model_id`.
pub fn bring_forward(app_user_model_id: &str) -> Result<()> {
    let Some(stem) = executable_stem(app_user_model_id) else {
        return Err(windows::core::Error::from_hresult(super::not_found()));
    };
    let mut search = Search { stem: stem.to_owned(), found: None };
    // The callback stops the walk itself, which `EnumWindows` reports as an error, so the result is not used.
    let _ = unsafe { EnumWindows(Some(visit), LPARAM(&mut search as *mut Search as isize)) };
    let Some(window) = search.found else {
        log::debug(&format!("notifications: no window of {stem} found"));
        return Err(windows::core::Error::from_hresult(super::not_found()));
    };
    log::debug(&format!("notifications: {app_user_model_id} already has a window, so it is brought forward"));
    if unsafe { IsIconic(window) }.as_bool() {
        let _ = unsafe { ShowWindow(window, SW_RESTORE) };
    }
    // A refused foreground only flashes the taskbar button; starting the app again would open a second window.
    let _ = unsafe { SetForegroundWindow(window) };
    Ok(())
}

struct Search {
    stem: String,
    found: Option<HWND>,
}

unsafe extern "system" fn visit(window: HWND, parameter: LPARAM) -> BOOL {
    // SAFETY: `bring_forward` passes a pointer to a `Search` that outlives the enumeration.
    let search = unsafe { &mut *(parameter.0 as *mut Search) };
    let is_visible = unsafe { IsWindowVisible(window) }.as_bool();
    let title_length = unsafe { GetWindowTextLengthW(window) };
    // `GetWindow` returns an error, not a null handle, for a window without an owner.
    let has_owner = unsafe { GetWindow(window, GW_OWNER) }.is_ok();
    if !is_main_window(is_visible, title_length, has_owner) {
        return true.into();
    }
    let mut process_id = 0;
    unsafe { GetWindowThreadProcessId(window, Some(&mut process_id)) };
    if process_image_path(process_id).is_some_and(|path| file_stem(&path).eq_ignore_ascii_case(&search.stem)) {
        search.found = Some(window);
        return false.into();
    }
    true.into()
}

/// A window the user works in: shown, titled and not owned by another window (a dialog or tool window is owned).
fn is_main_window(is_visible: bool, title_length: i32, has_owner: bool) -> bool {
    is_visible && title_length > 0 && !has_owner
}

/// The program name an unpackaged app's id ends with (`dev.warp.Warp` runs as `warp.exe`); a packaged app's id
/// (`Package!App`) names no executable, and the activation manager already brings those forward.
fn executable_stem(app_user_model_id: &str) -> Option<&str> {
    if app_user_model_id.contains('!') {
        return None;
    }
    app_user_model_id.rsplit('.').next().filter(|stem| !stem.is_empty())
}

fn file_stem(path: &str) -> &str {
    let name = path.rsplit(['\\', '/']).next().unwrap_or(path);
    name.strip_suffix(".exe").or_else(|| name.strip_suffix(".EXE")).unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unpackaged_id_names_its_program_by_its_last_part() {
        assert_eq!(executable_stem("dev.warp.Warp"), Some("Warp"));
        assert_eq!(executable_stem("com.squirrel.Discord.Discord"), Some("Discord"));
    }

    #[test]
    fn a_packaged_id_names_no_program() {
        assert_eq!(executable_stem("5319275A.WhatsAppDesktop_cv1g1gvanyjgm!App"), None);
    }

    #[test]
    fn a_visible_titled_window_without_an_owner_is_a_main_window() {
        assert!(is_main_window(true, 7, false));
    }

    #[test]
    fn an_owned_window_is_not_a_main_window() {
        assert!(!is_main_window(true, 7, true));
    }

    #[test]
    fn an_untitled_window_is_not_a_main_window() {
        assert!(!is_main_window(true, 0, false));
    }

    #[test]
    fn a_hidden_window_is_not_a_main_window() {
        assert!(!is_main_window(false, 7, false));
    }

    #[test]
    fn a_path_is_cut_to_its_program_name() {
        assert_eq!(file_stem(r"C:\Users\a\Programs\Warp\warp.exe"), "warp");
    }
}
