//! The data folder under `%APPDATA%` and the one-time copy from the folder an earlier build used.

use std::{
    fs,
    path::{Path, PathBuf},
};

pub const DIRECTORY_NAME: &str = "Foldline";
/// The folder builds before the rename used; it is only read, never changed or deleted.
pub const LEGACY_DIRECTORY_NAME: &str = "notch";

/// `%APPDATA%\Foldline`; `None` when `%APPDATA%` is not set or empty.
pub fn data_directory() -> Option<PathBuf> {
    data_directory_in(&app_data()?)
}

/// Copies `%APPDATA%\notch` to `%APPDATA%\Foldline` once. Returns whether a copy was made.
pub fn migrate_legacy_data_directory() -> bool {
    app_data().is_some_and(|app_data| migrate_legacy_data_directory_in(&app_data))
}

pub fn data_directory_in(app_data: &Path) -> Option<PathBuf> {
    (!app_data.as_os_str().is_empty()).then(|| app_data.join(DIRECTORY_NAME))
}

/// Does nothing when the new folder exists or the old one does not. The copy goes to a temporary sibling and is renamed
/// into place, so a failed copy never leaves a half-filled `Foldline`; the old folder is left as it is.
pub fn migrate_legacy_data_directory_in(app_data: &Path) -> bool {
    let (Some(target), legacy) = (data_directory_in(app_data), app_data.join(LEGACY_DIRECTORY_NAME)) else {
        return false;
    };
    if target.exists() || !legacy.is_dir() {
        return false;
    }
    let staging = app_data.join(format!("{DIRECTORY_NAME}.migrating-{}", std::process::id()));
    let _ = fs::remove_dir_all(&staging);
    if copy_directory(&legacy, &staging).and_then(|()| fs::rename(&staging, &target)).is_err() {
        let _ = fs::remove_dir_all(&staging);
        return false;
    }
    true
}

fn app_data() -> Option<PathBuf> {
    std::env::var_os("APPDATA").filter(|value| !value.is_empty()).map(PathBuf::from)
}

fn copy_directory(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let destination_path = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_directory(&entry.path(), &destination_path)?;
        } else {
            fs::copy(entry.path(), destination_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_data_folder(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("foldline-store-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&folder);
        fs::create_dir_all(&folder).expect("create the temporary app data folder");
        folder
    }

    #[test]
    fn the_data_directory_is_foldline_under_app_data() {
        assert_eq!(data_directory_in(Path::new("C:/Roaming")), Some(Path::new("C:/Roaming").join("Foldline")));
    }

    #[test]
    fn an_empty_app_data_gives_no_data_directory() {
        assert_eq!(data_directory_in(Path::new("")), None);
    }

    #[test]
    fn the_legacy_files_are_copied_and_the_old_folder_stays() {
        let app_data = app_data_folder("copy");
        let legacy = app_data.join("notch");
        fs::create_dir_all(legacy.join("sub")).expect("create the legacy folder");
        fs::write(legacy.join("settings.txt"), "notch.pinned=on\n").expect("write settings");
        fs::write(legacy.join("todo.txt"), "- [ ] task\n").expect("write todo");
        fs::write(legacy.join("sub").join("calendar.ics"), "BEGIN:VCALENDAR\n").expect("write calendar");

        assert!(migrate_legacy_data_directory_in(&app_data));

        let target = app_data.join("Foldline");
        assert_eq!(fs::read_to_string(target.join("settings.txt")).expect("copied"), "notch.pinned=on\n");
        assert_eq!(fs::read_to_string(target.join("todo.txt")).expect("copied"), "- [ ] task\n");
        assert_eq!(fs::read_to_string(target.join("sub").join("calendar.ics")).expect("copied"), "BEGIN:VCALENDAR\n");
        assert_eq!(fs::read_to_string(legacy.join("settings.txt")).expect("kept"), "notch.pinned=on\n");
        let _ = fs::remove_dir_all(&app_data);
    }

    #[test]
    fn an_existing_foldline_folder_is_never_touched() {
        let app_data = app_data_folder("existing");
        fs::create_dir_all(app_data.join("notch")).expect("create the legacy folder");
        fs::write(app_data.join("notch").join("settings.txt"), "old=1\n").expect("write legacy");
        fs::create_dir_all(app_data.join("Foldline")).expect("create the new folder");
        fs::write(app_data.join("Foldline").join("settings.txt"), "new=1\n").expect("write new");

        assert!(!migrate_legacy_data_directory_in(&app_data));

        assert_eq!(fs::read_to_string(app_data.join("Foldline").join("settings.txt")).expect("kept"), "new=1\n");
        let _ = fs::remove_dir_all(&app_data);
    }

    #[test]
    fn a_second_migration_does_nothing() {
        let app_data = app_data_folder("twice");
        fs::create_dir_all(app_data.join("notch")).expect("create the legacy folder");
        fs::write(app_data.join("notch").join("settings.txt"), "a=1\n").expect("write legacy");

        assert!(migrate_legacy_data_directory_in(&app_data));
        assert!(!migrate_legacy_data_directory_in(&app_data));
        let _ = fs::remove_dir_all(&app_data);
    }

    #[test]
    fn a_missing_legacy_folder_creates_nothing() {
        let app_data = app_data_folder("none");

        assert!(!migrate_legacy_data_directory_in(&app_data));

        assert!(!app_data.join("Foldline").exists());
        let _ = fs::remove_dir_all(&app_data);
    }

    #[test]
    fn a_missing_app_data_folder_does_not_panic() {
        assert!(!migrate_legacy_data_directory_in(Path::new("Z:/definitely/not/here")));
    }

    #[test]
    fn a_legacy_path_that_is_a_file_is_ignored() {
        let app_data = app_data_folder("file");
        fs::write(app_data.join("notch"), "not a folder").expect("write a file");

        assert!(!migrate_legacy_data_directory_in(&app_data));

        assert!(!app_data.join("Foldline").exists());
        let _ = fs::remove_dir_all(&app_data);
    }
}
