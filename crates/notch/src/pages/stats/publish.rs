//! The disks and GPUs this machine has, written to settings.txt for the settings window to offer as choices.

use super::sampler::Meter;
use crate::host::Settings;

pub(super) const KNOWN_DISKS_SETTING: &str = "stats.known_disks";
pub(super) const KNOWN_GPUS_SETTING: &str = "stats.known_gpus";
/// A GPU name never contains it; a comma could.
pub(super) const GPU_SEPARATOR: char = '|';
const DISK_SEPARATOR: char = ',';

/// Drive names such as `C:,G:`.
pub(super) fn known_disks_text(disks: &[Meter]) -> String {
    join(disks, DISK_SEPARATOR)
}

/// GPU names joined by `|`, such as `RTX 4050|Radeon 760M`.
pub(super) fn known_gpus_text(gpus: &[Meter]) -> String {
    join(gpus, GPU_SEPARATOR)
}

/// Names keep to one line and never hold their own separator.
fn join(meters: &[Meter], separator: char) -> String {
    let names: Vec<String> = meters
        .iter()
        .map(|meter| {
            meter
                .name
                .chars()
                .map(|character| if character == separator || character.is_control() { ' ' } else { character })
                .collect::<String>()
                .trim()
                .to_owned()
        })
        .collect();
    names.join(&separator.to_string())
}

/// Writes each list only when it differs from the stored one, so a repeat opening writes nothing.
pub(super) fn publish_known(settings: &mut Settings, disks: &[Meter], gpus: &[Meter]) {
    for (key, text) in [(KNOWN_DISKS_SETTING, known_disks_text(disks)), (KNOWN_GPUS_SETTING, known_gpus_text(gpus))] {
        if settings.get_string(key) != Some(text.as_str()) {
            settings.set_string(key, &text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meters(names: &[&str]) -> Vec<Meter> {
        names.iter().map(|name| Meter { name: (*name).to_owned(), capacity: None, percent: None }).collect()
    }

    #[test]
    fn the_lists_use_their_own_separators() {
        assert_eq!(known_disks_text(&meters(&["C:", "G:"])), "C:,G:");
        assert_eq!(known_gpus_text(&meters(&["RTX 4050", "Radeon 760M"])), "RTX 4050|Radeon 760M");
        assert_eq!(known_gpus_text(&[]), "");
    }

    #[test]
    fn a_name_cannot_break_the_line_or_the_list() {
        assert_eq!(known_gpus_text(&meters(&["A|B\nC"])), "A B C");
    }

    #[test]
    fn a_list_is_written_only_when_it_changes() {
        let path = std::env::temp_dir().join(format!("notch-stats-known-{}", std::process::id())).join("settings.txt");
        let mut settings = Settings::at(Some(path.clone()));
        publish_known(&mut settings, &meters(&["C:"]), &meters(&["RTX 4050"]));
        assert_eq!(settings.get_string(KNOWN_DISKS_SETTING), Some("C:"));
        let first_write = std::fs::metadata(&path).and_then(|data| data.modified()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(30));
        publish_known(&mut settings, &meters(&["C:"]), &meters(&["RTX 4050"]));
        assert_eq!(std::fs::metadata(&path).and_then(|data| data.modified()).unwrap(), first_write);
        publish_known(&mut settings, &meters(&["C:", "G:"]), &meters(&["RTX 4050"]));
        assert_eq!(settings.get_string(KNOWN_DISKS_SETTING), Some("C:,G:"));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn no_gpu_is_written_as_an_empty_list_so_absent_still_means_never_opened() {
        let path = std::env::temp_dir().join(format!("notch-stats-empty-{}", std::process::id())).join("settings.txt");
        let mut settings = Settings::at(Some(path.clone()));
        publish_known(&mut settings, &meters(&["C:"]), &[]);
        assert_eq!(settings.get_string(KNOWN_GPUS_SETTING), Some(""));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}
