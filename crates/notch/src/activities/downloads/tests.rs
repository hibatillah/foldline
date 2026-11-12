use super::*;
use crate::{gfx::Fonts, ui::BANNER_BUTTONS_HEIGHT};

fn temporary_folder(name: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("notch-downloads-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

#[test]
fn nothing_shows_until_a_download_starts() {
    let activity = DownloadsActivity::new();
    assert_eq!(activity.presence(), None);
    assert_eq!(activity.header_status_width(), None);
}

#[test]
fn a_running_download_is_a_badge_as_wide_as_the_icon_in_the_pill_and_the_header() {
    let activity = DownloadsActivity::sample_running();
    assert_eq!(activity.presence(), Some(Presence::Badge { width: 18.0 }));
    assert_eq!(activity.header_status_width(), Some(18.0));
}

#[test]
fn the_start_banner_is_340_wide_and_has_no_buttons_under_the_pointer() {
    let fonts = Fonts::new(1.0).unwrap();
    let banner = Some(Presence::Banner { width: 340.0, height: 58.0 });
    assert_eq!(DownloadsActivity::sample_started(&fonts).presence(), banner);
    assert_eq!(DownloadsActivity::sample_started(&fonts).hovered().presence(), banner);
}

#[test]
fn the_pointer_adds_the_buttons_to_the_finish_banner() {
    let fonts = Fonts::new(1.0).unwrap();
    let Some(Presence::Banner { height: calm, .. }) = DownloadsActivity::sample_finished(&fonts).presence() else {
        panic!("the finish sample has a banner")
    };
    assert_eq!(calm, 58.0);
    let Some(Presence::Banner { height, .. }) = DownloadsActivity::sample_finished(&fonts).hovered().presence() else {
        panic!("the hovered finish sample has a banner")
    };
    assert_eq!(height, 58.0 + BANNER_BUTTONS_HEIGHT);
}

#[test]
fn the_banner_wins_over_the_badge_while_it_shows() {
    let fonts = Fonts::new(1.0).unwrap();
    let activity = DownloadsActivity::sample_started(&fonts);
    assert_eq!(activity.tracker.active_count(), 1);
    assert!(matches!(activity.presence(), Some(Presence::Banner { .. })));
    assert_eq!(activity.header_status_width(), Some(18.0), "the header keeps its icon under a banner");
}

#[test]
fn the_downloads_activity_is_optional_and_ranks_seventh() {
    let activity = DownloadsActivity::new();
    assert_eq!(activity.id(), "downloads");
    assert!(activity.is_optional());
    assert_eq!(activity.header_status_rank(), 7);
}

#[test]
fn a_vanished_download_with_no_file_shows_no_finish_banner() {
    let folder = temporary_folder("cancelled");
    assert_eq!(finished_path(&folder, None), None);
    assert_eq!(finished_path(&folder, Some("a.msi")), None, "the real file does not exist");
    assert_eq!(react(Outcome::Vanished { final_name: Some("a.msi".into()) }, &folder), Reaction::Nothing);
    std::fs::remove_dir_all(&folder).unwrap();
}

#[test]
fn a_vanished_firefox_download_finishes_when_its_file_has_bytes() {
    let folder = temporary_folder("firefox");
    std::fs::write(folder.join("a.msi"), b"abc").unwrap();
    assert_eq!(finished_path(&folder, Some("a.msi")), Some(folder.join("a.msi")));
    assert_eq!(
        react(Outcome::Vanished { final_name: Some("a.msi".into()) }, &folder),
        Reaction::ShowFinished { path: folder.join("a.msi") }
    );
    std::fs::remove_dir_all(&folder).unwrap();
}

#[test]
fn a_vanished_firefox_download_whose_file_is_empty_shows_no_finish_banner() {
    let folder = temporary_folder("firefox-empty");
    std::fs::write(folder.join("a.msi"), b"").unwrap();
    assert_eq!(finished_path(&folder, Some("a.msi")), None);
    std::fs::remove_dir_all(&folder).unwrap();
}

#[test]
fn a_start_and_a_rename_become_a_start_banner_and_a_finish_banner() {
    let folder = Path::new("C:/Downloads");
    assert_eq!(
        react(Outcome::Started { display_name: Some("a.zip".into()) }, folder),
        Reaction::ShowStarted { title: Some("a.zip".into()) }
    );
    assert_eq!(
        react(Outcome::Finished { file_name: "a.zip".into() }, folder),
        Reaction::ShowFinished { path: folder.join("a.zip") }
    );
    assert_eq!(react(Outcome::Nothing, folder), Reaction::Nothing);
}
