//! Turns changes of file names in the Downloads folder into started, finished and vanished downloads.

const PARTIAL_SUFFIXES: [&str; 4] = [".crdownload", ".part", ".partial", ".opdownload"];
/// Chromium names a download `Unconfirmed <number>.crdownload` until the file name is known.
const UNCONFIRMED_PREFIX: &str = "Unconfirmed ";
/// Firefox puts this many random characters, upper case letters and digits, between the name and the extension of its `.part` file.
const FIREFOX_RANDOM_LENGTH: usize = 8;
/// One check a minute, so a download that does not grow for this many checks is dropped.
const STALE_CHECKS: u32 = 20;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FolderEvent {
    Added(String),
    Removed(String),
    Renamed {
        from: String,
        to: String,
    },
    /// The watcher lost events; `names` is the folder's listing now.
    Relisted(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Nothing,
    /// `display_name` is the real name when the partial name carries it (Firefox), else `None` (Chromium).
    Started {
        display_name: Option<String>,
    },
    /// A partial file was renamed to `file_name`.
    Finished {
        file_name: String,
    },
    /// A partial file is gone. `final_name` is where its bytes may be (Firefox); the caller checks the disk.
    Vanished {
        final_name: Option<String>,
    },
}

struct Download {
    partial_name: String,
    last_size: Option<u64>,
    has_baseline: bool,
    unchanged_checks: u32,
}

impl Download {
    fn new(partial_name: String) -> Self {
        Self { partial_name, last_size: None, has_baseline: false, unchanged_checks: 0 }
    }
}

/// The partial files seen while the notch runs; a file that was already there at start is never a download.
#[derive(Default)]
pub struct Tracker {
    downloads: Vec<Download>,
}

impl Tracker {
    pub fn apply(&mut self, event: &FolderEvent) -> Vec<Outcome> {
        match event {
            FolderEvent::Added(name) => vec![self.start(name)],
            FolderEvent::Removed(name) => vec![self.remove(name)],
            FolderEvent::Renamed { from, to } => vec![self.rename(from, to)],
            FolderEvent::Relisted(names) => self.reconcile(names),
        }
    }

    pub fn active_count(&self) -> usize {
        self.downloads.len()
    }

    #[cfg(test)]
    pub fn partial_names(&self) -> impl Iterator<Item = &str> {
        self.downloads.iter().map(|download| download.partial_name.as_str())
    }

    /// One stale check: `size_of` gives a partial file's size now. Drops a download unchanged for 20 checks in a row.
    pub fn check_stale(&mut self, size_of: impl Fn(&str) -> Option<u64>) {
        for download in &mut self.downloads {
            let size = size_of(&download.partial_name);
            if download.has_baseline && size == download.last_size {
                download.unchanged_checks += 1;
            } else {
                download.has_baseline = true;
                download.last_size = size;
                download.unchanged_checks = 0;
            }
        }
        self.downloads.retain(|download| download.unchanged_checks < STALE_CHECKS);
    }

    fn position(&self, name: &str) -> Option<usize> {
        self.downloads.iter().position(|download| download.partial_name == name)
    }

    fn start(&mut self, name: &str) -> Outcome {
        if !is_partial(name) || self.position(name).is_some() {
            return Outcome::Nothing;
        }
        self.downloads.push(Download::new(name.to_owned()));
        Outcome::Started { display_name: display_name(name) }
    }

    fn remove(&mut self, name: &str) -> Outcome {
        match self.position(name) {
            Some(index) => {
                self.downloads.remove(index);
                Outcome::Vanished { final_name: display_name(name) }
            }
            None => Outcome::Nothing,
        }
    }

    fn rename(&mut self, from: &str, to: &str) -> Outcome {
        let Some(index) = self.position(from) else { return self.start(to) };
        if is_partial(to) {
            self.downloads[index].partial_name = to.to_owned();
            return Outcome::Nothing;
        }
        self.downloads.remove(index);
        Outcome::Finished { file_name: to.to_owned() }
    }

    fn reconcile(&mut self, names: &[String]) -> Vec<Outcome> {
        let mut outcomes = Vec::new();
        self.downloads.retain(|download| {
            let is_listed = names.contains(&download.partial_name);
            if !is_listed {
                outcomes.push(Outcome::Vanished { final_name: display_name(&download.partial_name) });
            }
            is_listed
        });
        if outcomes.is_empty() {
            outcomes.push(Outcome::Nothing);
        }
        outcomes
    }
}

fn partial_suffix_length(file_name: &str) -> Option<usize> {
    let lower = file_name.to_ascii_lowercase();
    PARTIAL_SUFFIXES.iter().find(|suffix| lower.ends_with(**suffix)).map(|suffix| suffix.len())
}

pub fn is_partial(file_name: &str) -> bool {
    partial_suffix_length(file_name).is_some()
}

/// The real file name a partial name carries: none for Chromium's, the name without Firefox's random part otherwise.
pub fn display_name(partial_name: &str) -> Option<String> {
    let stripped = &partial_name[..partial_name.len() - partial_suffix_length(partial_name).unwrap_or(0)];
    if stripped.is_empty() || stripped.starts_with(UNCONFIRMED_PREFIX) {
        return None;
    }
    let mut segments: Vec<&str> = stripped.split('.').collect();
    // At least a base name, the random part and an extension.
    if segments.len() >= 3 {
        let random = segments[segments.len() - 2];
        if random.len() == FIREFOX_RANDOM_LENGTH
            && random.bytes().all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
        {
            segments.remove(segments.len() - 2);
        }
    }
    Some(segments.join("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn started(name: Option<&str>) -> Vec<Outcome> {
        vec![Outcome::Started { display_name: name.map(str::to_owned) }]
    }

    #[test]
    fn a_chromium_download_starts_without_a_name_and_finishes_at_the_rename() {
        let mut tracker = Tracker::default();
        assert_eq!(tracker.apply(&FolderEvent::Added("Unconfirmed 979726.crdownload".into())), started(None));
        assert_eq!(tracker.active_count(), 1);
        let renamed = FolderEvent::Renamed {
            from: "Unconfirmed 979726.crdownload".into(),
            to: "calendar-windows-x64.msi".into(),
        };
        assert_eq!(tracker.apply(&renamed), vec![Outcome::Finished { file_name: "calendar-windows-x64.msi".into() }]);
        assert_eq!(tracker.active_count(), 0);
    }

    #[test]
    fn braves_temporary_file_is_ignored_until_it_becomes_a_partial_file() {
        let mut tracker = Tracker::default();
        assert_eq!(
            tracker.apply(&FolderEvent::Added("568a2a1d-ea57-4947-a493-1881ce76b123.tmp".into())),
            vec![Outcome::Nothing]
        );
        let renamed = FolderEvent::Renamed {
            from: "568a2a1d-ea57-4947-a493-1881ce76b123.tmp".into(),
            to: "Unconfirmed 231036.crdownload".into(),
        };
        assert_eq!(tracker.apply(&renamed), started(None));
    }

    #[test]
    fn a_firefox_download_starts_with_its_real_name() {
        let mut tracker = Tracker::default();
        assert_eq!(tracker.apply(&FolderEvent::Added("calendar-windows-x64.msi".into())), vec![Outcome::Nothing]);
        assert_eq!(
            tracker.apply(&FolderEvent::Added("calendar-windows-x64.RD98ESXG.msi.part".into())),
            started(Some("calendar-windows-x64.msi"))
        );
    }

    #[test]
    fn a_firefox_part_file_that_disappears_points_at_the_real_file() {
        let mut tracker = Tracker::default();
        tracker.apply(&FolderEvent::Added("calendar-windows-x64.RD98ESXG.msi.part".into()));
        assert_eq!(
            tracker.apply(&FolderEvent::Removed("calendar-windows-x64.RD98ESXG.msi.part".into())),
            vec![Outcome::Vanished { final_name: Some("calendar-windows-x64.msi".into()) }]
        );
    }

    #[test]
    fn a_firefox_part_file_renamed_onto_the_real_name_is_a_finish() {
        let mut tracker = Tracker::default();
        tracker.apply(&FolderEvent::Added("calendar-windows-x64.RD98ESXG.msi.part".into()));
        let renamed = FolderEvent::Renamed {
            from: "calendar-windows-x64.RD98ESXG.msi.part".into(),
            to: "calendar-windows-x64.msi".into(),
        };
        assert_eq!(tracker.apply(&renamed), vec![Outcome::Finished { file_name: "calendar-windows-x64.msi".into() }]);
    }

    #[test]
    fn a_cancelled_chromium_download_vanishes_with_no_file_to_look_for() {
        let mut tracker = Tracker::default();
        tracker.apply(&FolderEvent::Added("Unconfirmed 1.crdownload".into()));
        assert_eq!(
            tracker.apply(&FolderEvent::Removed("Unconfirmed 1.crdownload".into())),
            vec![Outcome::Vanished { final_name: None }]
        );
        assert_eq!(tracker.active_count(), 0);
    }

    #[test]
    fn files_that_are_not_downloads_change_nothing() {
        let mut tracker = Tracker::default();
        for event in [
            FolderEvent::Added("notes.txt".into()),
            FolderEvent::Renamed { from: "a.txt".into(), to: "b.txt".into() },
            FolderEvent::Removed("b.txt".into()),
            FolderEvent::Removed("Unconfirmed 9.crdownload".into()),
        ] {
            assert_eq!(tracker.apply(&event), vec![Outcome::Nothing]);
        }
        assert_eq!(tracker.active_count(), 0);
    }

    #[test]
    fn a_relisting_ends_downloads_whose_file_is_gone_and_starts_none() {
        let mut tracker = Tracker::default();
        tracker.apply(&FolderEvent::Added("Unconfirmed 1.crdownload".into()));
        tracker.apply(&FolderEvent::Added("Unconfirmed 2.crdownload".into()));
        let listing = FolderEvent::Relisted(vec!["Unconfirmed 2.crdownload".into(), "Unconfirmed 3.crdownload".into()]);
        assert_eq!(tracker.apply(&listing), vec![Outcome::Vanished { final_name: None }]);
        assert_eq!(tracker.partial_names().collect::<Vec<_>>(), ["Unconfirmed 2.crdownload"]);
    }

    #[test]
    fn a_relisting_that_ends_nothing_changes_nothing() {
        let mut tracker = Tracker::default();
        tracker.apply(&FolderEvent::Added("Unconfirmed 1.crdownload".into()));
        let listing = FolderEvent::Relisted(vec!["Unconfirmed 1.crdownload".into()]);
        assert_eq!(tracker.apply(&listing), vec![Outcome::Nothing]);
        assert_eq!(tracker.active_count(), 1);
    }

    #[test]
    fn a_new_tracker_tracks_nothing_whatever_is_in_the_folder() {
        let tracker = Tracker::default();
        assert_eq!(tracker.active_count(), 0);
        assert_eq!(tracker.partial_names().count(), 0);
    }

    #[test]
    fn a_partial_file_renamed_to_another_partial_name_stays_one_download() {
        let mut tracker = Tracker::default();
        tracker.apply(&FolderEvent::Added("Unconfirmed 1.crdownload".into()));
        let renamed = FolderEvent::Renamed { from: "Unconfirmed 1.crdownload".into(), to: "a.zip.crdownload".into() };
        assert_eq!(tracker.apply(&renamed), vec![Outcome::Nothing]);
        assert_eq!(tracker.partial_names().collect::<Vec<_>>(), ["a.zip.crdownload"]);
    }

    #[test]
    fn two_downloads_are_counted_separately() {
        let mut tracker = Tracker::default();
        tracker.apply(&FolderEvent::Added("Unconfirmed 1.crdownload".into()));
        tracker.apply(&FolderEvent::Added("b.ABCDEFGH.zip.part".into()));
        assert_eq!(tracker.active_count(), 2);
    }

    #[test]
    fn a_partial_name_is_recognized_by_its_suffix_in_any_case() {
        for name in ["a.crdownload", "a.PART", "a.partial", "a.opdownload"] {
            assert!(is_partial(name), "{name}");
        }
        for name in ["a.tmp", "a.msi", "crdownload", "part"] {
            assert!(!is_partial(name), "{name}");
        }
    }

    #[test]
    fn the_display_name_drops_only_an_eight_character_random_segment() {
        assert_eq!(display_name("Unconfirmed 542510.crdownload"), None);
        assert_eq!(display_name("calendar-windows-x64.RD98ESXG.msi.part").as_deref(), Some("calendar-windows-x64.msi"));
        assert_eq!(display_name("archive.tar.gz.part").as_deref(), Some("archive.tar.gz"));
        assert_eq!(display_name("setup.exe.crdownload").as_deref(), Some("setup.exe"));
    }

    #[test]
    fn an_eight_letter_lowercase_middle_segment_is_part_of_the_name() {
        assert_eq!(display_name("document.tutorial.pdf.part").as_deref(), Some("document.tutorial.pdf"));
        assert_eq!(display_name("b.ABCDEFGH.zip.part").as_deref(), Some("b.zip"));
        assert_eq!(display_name("b.12345678.zip.part").as_deref(), Some("b.zip"));
    }

    #[test]
    fn a_two_part_name_with_an_eight_letter_base_keeps_its_base() {
        assert_eq!(display_name("download.exe.crdownload").as_deref(), Some("download.exe"));
    }

    #[test]
    fn a_download_that_stops_growing_for_twenty_checks_is_dropped() {
        let mut tracker = Tracker::default();
        tracker.apply(&FolderEvent::Added("Unconfirmed 1.crdownload".into()));
        for _ in 0..20 {
            tracker.check_stale(|_| Some(500));
        }
        assert_eq!(tracker.active_count(), 1, "the first check only records the size");
        tracker.check_stale(|_| Some(500));
        assert_eq!(tracker.active_count(), 0);
    }

    #[test]
    fn a_download_that_grows_between_checks_is_kept() {
        let mut tracker = Tracker::default();
        tracker.apply(&FolderEvent::Added("Unconfirmed 1.crdownload".into()));
        for check in 0..40u64 {
            tracker.check_stale(|_| Some(check * 10));
        }
        assert_eq!(tracker.active_count(), 1);
    }

    #[test]
    fn a_size_that_cannot_be_read_counts_as_unchanged() {
        let mut tracker = Tracker::default();
        tracker.apply(&FolderEvent::Added("Unconfirmed 1.crdownload".into()));
        for _ in 0..21 {
            tracker.check_stale(|_| None);
        }
        assert_eq!(tracker.active_count(), 0);
    }
}
