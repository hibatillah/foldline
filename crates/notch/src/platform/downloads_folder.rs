//! Where Windows keeps the user's Downloads folder.

use std::path::PathBuf;

use windows::Win32::{
    System::Com::CoTaskMemFree,
    UI::Shell::{FOLDERID_Downloads, KF_FLAG_DEFAULT, SHGetKnownFolderPath},
};

/// The Downloads folder, followed when the user moved it; `None` when Windows cannot say.
pub fn path() -> Option<PathBuf> {
    let raw = unsafe { SHGetKnownFolderPath(&FOLDERID_Downloads, KF_FLAG_DEFAULT, None) }.ok()?;
    let folder = unsafe { raw.to_string() }.ok().map(PathBuf::from);
    // SAFETY: `raw` came from `SHGetKnownFolderPath`, which hands its buffer to the caller to free exactly once.
    unsafe { CoTaskMemFree(Some(raw.as_ptr().cast())) };
    folder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_downloads_folder_is_an_existing_directory() {
        let folder = path().expect("Windows knows the Downloads folder");
        assert!(folder.is_dir());
    }
}
