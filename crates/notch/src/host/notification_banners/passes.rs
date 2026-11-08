//! One pass over every app's banner value: turning banners off, or putting the saved values back.

use std::{io, path::PathBuf};

use super::backup::{BANNER_OFF, Backup, plan_hiding, should_restore};
use crate::{
    log,
    platform::notification_settings::{ExclusiveAccess, app_ids, set_show_banner, show_banner},
};

fn backup_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|directory| PathBuf::from(directory).join("notch").join("notification-banners.txt"))
}

fn read_backup() -> Backup {
    backup_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|text| Backup::parse(&text))
        .unwrap_or_default()
}

/// Written through a temporary file and a rename, so a crash mid-write never leaves half a backup.
fn write_backup(backup: &Backup) -> io::Result<()> {
    let path = backup_path().ok_or_else(|| io::Error::other("APPDATA is not set"))?;
    if backup.originals.is_empty() {
        return match std::fs::remove_file(&path) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        };
    }
    if let Some(directory) = path.parent() {
        std::fs::create_dir_all(directory)?;
    }
    let temporary = path.with_extension("tmp");
    std::fs::write(&temporary, backup.to_text())?;
    std::fs::rename(&temporary, &path)
}

/// Turns off the banner of every app that can still show one, after saving what each app had.
pub(super) fn hide_banners() {
    let Some(_access) = ExclusiveAccess::acquire() else {
        log::debug("notification banners: another notch holds the lock; skipping this pass");
        return;
    };
    let mut backup = read_backup();
    let apps: Vec<(String, Option<u32>)> =
        app_ids().into_iter().map(|app_id| (app_id.clone(), show_banner(&app_id))).collect();
    let plan = plan_hiding(&apps, &backup);
    if plan.apps_to_hide.is_empty() {
        return;
    }
    if !plan.new_originals.is_empty() {
        backup.originals.extend(plan.new_originals);
        // A value that could not be saved is never changed, so it never needs restoring.
        if let Err(error) = write_backup(&backup) {
            log::debug(&format!("could not save the notification banner backup: {error}"));
            return;
        }
    }
    for app_id in &plan.apps_to_hide {
        if let Err(error) = set_show_banner(app_id, Some(BANNER_OFF)) {
            log::debug(&format!("could not turn off the banner of {app_id}: {error:?}"));
        }
    }
    log::debug(&format!("notification banners: turned off for {} apps", plan.apps_to_hide.len()));
}

/// Puts back every value the notch changed; one it cannot write stays in the backup for the next start.
pub(super) fn restore_banners() {
    let backup = read_backup();
    if backup.originals.is_empty() {
        return;
    }
    let Some(_access) = ExclusiveAccess::acquire() else {
        log::debug("notification banners: another notch holds the lock; restoring at the next start");
        return;
    };
    // Read again under the lock: another notch may have restored meanwhile.
    let backup = read_backup();
    let mut left = Backup::default();
    let mut restored = 0;
    for (app_id, original) in backup.originals {
        if !should_restore(show_banner(&app_id)) {
            continue;
        }
        match set_show_banner(&app_id, original.to_value()) {
            Ok(()) => restored += 1,
            Err(error) => {
                log::debug(&format!("could not restore the banner of {app_id}: {error:?}"));
                left.originals.insert(app_id, original);
            }
        }
    }
    if let Err(error) = write_backup(&left) {
        log::debug(&format!("could not update the notification banner backup: {error}"));
    }
    log::debug(&format!("notification banners: restored {restored} apps"));
}
