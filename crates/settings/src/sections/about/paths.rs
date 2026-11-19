//! Where the notch keeps its files, and what "Open" does with them.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

const SETTINGS_FILE_NAME: &str = "settings.txt";

/// Foldline's data folder and the settings file inside it, both under `%APPDATA%`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPaths {
    pub folder: PathBuf,
    pub settings_file: PathBuf,
}

impl DataPaths {
    /// From the value of `%APPDATA%`; `None` when it is not set.
    pub fn from_app_data(app_data: Option<OsString>) -> Option<Self> {
        let folder = store::data_directory::data_directory_in(Path::new(&app_data?))?;
        let settings_file = folder.join(SETTINGS_FILE_NAME);
        Some(Self { folder, settings_file })
    }

    pub fn from_environment() -> Option<Self> {
        store::migrate_legacy_data_directory();
        Self::from_app_data(std::env::var_os("APPDATA"))
    }
}

/// What the system is asked to open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opening {
    /// Open this file in its default app.
    File(PathBuf),
    /// Show this folder in Explorer.
    Folder(PathBuf),
    /// Nothing exists yet.
    Nothing,
}

/// The settings file when it exists, else its folder when that exists, else nothing.
pub fn settings_opening(paths: &DataPaths, exists: impl Fn(&Path) -> bool) -> Opening {
    if exists(&paths.settings_file) {
        Opening::File(paths.settings_file.clone())
    } else if exists(&paths.folder) {
        Opening::Folder(paths.folder.clone())
    } else {
        Opening::Nothing
    }
}

/// The data folder when it exists, else nothing.
pub fn folder_opening(paths: &DataPaths, exists: impl Fn(&Path) -> bool) -> Opening {
    if exists(&paths.folder) { Opening::Folder(paths.folder.clone()) } else { Opening::Nothing }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths() -> DataPaths {
        DataPaths::from_app_data(Some(OsString::from("C:/Users/me/AppData/Roaming"))).expect("paths")
    }

    #[test]
    fn the_paths_sit_under_appdata_foldline() {
        let paths = paths();
        assert_eq!(paths.folder, Path::new("C:/Users/me/AppData/Roaming").join("Foldline"));
        assert_eq!(paths.settings_file, paths.folder.join("settings.txt"));
    }

    #[test]
    fn a_missing_or_empty_appdata_gives_no_paths() {
        assert_eq!(DataPaths::from_app_data(None), None);
        assert_eq!(DataPaths::from_app_data(Some(OsString::new())), None);
    }

    #[test]
    fn the_settings_file_opens_when_it_exists() {
        let paths = paths();
        assert_eq!(settings_opening(&paths, |_| true), Opening::File(paths.settings_file.clone()));
    }

    #[test]
    fn the_folder_opens_when_the_settings_file_does_not_exist_yet() {
        let paths = paths();
        let folder = paths.folder.clone();
        assert_eq!(settings_opening(&paths, |path| path == folder), Opening::Folder(paths.folder.clone()));
    }

    #[test]
    fn nothing_opens_when_the_folder_is_missing() {
        let paths = paths();
        assert_eq!(settings_opening(&paths, |_| false), Opening::Nothing);
        assert_eq!(folder_opening(&paths, |_| false), Opening::Nothing);
        assert_eq!(folder_opening(&paths, |_| true), Opening::Folder(paths.folder.clone()));
    }
}
