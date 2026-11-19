//! The license notices: the file the "Licenses" row opens, and where it is looked for.

use crate::i18n::tr;
use std::path::{Path, PathBuf};

use gpui::{App, Window};
use gpui_component::{WindowExt as _, notification::Notification};

/// Installed beside `foldline-settings.exe`.
const LICENSES_FILE: &str = "THIRD-PARTY-LICENSES.txt";
/// How many folders above the program's own the licenses file is looked for (`run/` and `target/.../release` sit below the repository root).
const LICENSES_SEARCH_LEVELS: usize = 6;

/// Looks for the licenses file in `start_folder`, then in each parent folder, up to `LICENSES_SEARCH_LEVELS` levels up.
fn find_licenses_file(start_folder: &Path, file_exists: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    start_folder
        .ancestors()
        .take(LICENSES_SEARCH_LEVELS + 1)
        .map(|folder| folder.join(LICENSES_FILE))
        .find(|candidate| file_exists(candidate))
}

/// Opens the license notices, or tells the user in the window when the file cannot be found.
pub fn open_licenses(window: &mut Window, cx: &mut App) {
    let found =
        std::env::current_exe().ok().and_then(|exe| find_licenses_file(exe.parent()?, |candidate| candidate.is_file()));
    match found {
        Some(path) => cx.open_with_system(&path),
        None => {
            window.push_notification(Notification::warning(tr!("about.licenses_missing", file = LICENSES_FILE)), cx)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_licenses_file_next_to_the_program_wins_over_one_further_up() {
        let near = Path::new("repo/run").join(LICENSES_FILE);
        let far = Path::new("repo").join(LICENSES_FILE);
        let found = find_licenses_file(Path::new("repo/run"), |path| path == near || path == far);
        assert_eq!(found, Some(near));
    }

    #[test]
    fn the_search_walks_up_to_the_repository_root_from_the_build_folder() {
        let root_file = Path::new("repo").join(LICENSES_FILE);
        let found =
            find_licenses_file(Path::new("repo/target/x86_64-pc-windows-msvc/release"), |path| path == root_file);
        assert_eq!(found, Some(root_file));
    }

    #[test]
    fn the_search_stops_after_the_level_limit_and_reports_nothing() {
        let too_far = Path::new("a").join(LICENSES_FILE);
        let start = Path::new("a/b/c/d/e/f/g/h");
        assert_eq!(find_licenses_file(start, |path| path == too_far), None);
        assert_eq!(find_licenses_file(start, |_| false), None);
    }
}
