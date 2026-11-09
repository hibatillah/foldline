//! What a settings reload changes.

use super::*;

#[test]
fn required_activities_ignore_a_saved_disable() {
    let settings_file = SettingsFile::new("required-activities");
    let mut settings = settings_file.load();
    settings.set_bool("activity.volume", false);
    settings.set_bool("activity.notifications", false);
    let notch = configured_rig(SampleOptions::default(), settings, crate::activities::all());
    let volume_index = notch.activities.iter().position(|slot| slot.activity.id() == "volume").unwrap();
    let notifications_index = notch.activities.iter().position(|slot| slot.activity.id() == "notifications").unwrap();
    assert!(notch.activities[volume_index].is_enabled);
    assert!(!notch.activities[notifications_index].is_enabled);
}

#[test]
fn reloading_settings_applies_hide_when_idle() {
    let settings_file = SettingsFile::new("reload-hide-when-idle");
    let mut notch = configured_rig(SampleOptions::default(), settings_file.load(), Vec::new());
    settle(&mut notch);
    assert!(!notch.hiding.hides_when_idle);
    fs::write(&settings_file.0, "notch.hide_when_idle=on\nnotch.hide_for_fullscreen=off\n").unwrap();
    notch.reload_settings();
    assert!(notch.hiding.hides_when_idle);
    assert_eq!(notch.hiding.fullscreen_showing, Showing::Music);
    assert!(notch.is_hidden());
    assert_eq!(notch.springs.resting_height.target, HIDDEN_HEIGHT);
}

#[test]
fn reloading_settings_keeps_the_open_page_and_hover() {
    let settings_file = SettingsFile::new("reload-open-page");
    let options = SampleOptions { media: Some(false), timer: true, ..SampleOptions::default() };
    let mut notch = configured_rig(options, settings_file.load(), Vec::new());
    open(&mut notch, "media");
    let media = notch.selected_page.unwrap();
    assert_eq!(notch.page_at(notch.track.position), media);
    let mut reversed = notch.page_ids();
    reversed.reverse();
    fs::write(&settings_file.0, format!("navigation.order={}\nnotch.hide_when_idle=on\n", reversed.join(","))).unwrap();
    notch.reload_settings();
    assert!(notch.hiding.hides_when_idle);
    assert_eq!(notch.page_list.order().first(), notch.pages.iter().position(|page| page.id() == reversed[0]).as_ref());
    assert!(notch.hover.is_hovering);
    assert_eq!(notch.springs.expansion.target, 1.0);
    assert_eq!(notch.selected_page, Some(media));
    assert_eq!(notch.expanded_index, Some(media));
    assert_eq!(notch.page_at(notch.track.position), media, "the open page must not scroll away");
    assert!(notch.track.is_settled());
}

#[test]
fn switching_off_an_earlier_page_keeps_the_open_page_in_place() {
    let settings_file = SettingsFile::new("reload-disable-earlier");
    let options = SampleOptions { media: Some(false), timer: true, ..SampleOptions::default() };
    let mut notch = configured_rig(options, settings_file.load(), Vec::new());
    open(&mut notch, "calendar");
    let calendar = notch.selected_page.unwrap();
    let shelf = notch.pages.iter().position(|page| page.id() == "shelf").unwrap();
    let navigable_before = notch.navigable.clone();
    let shelf_row = navigable_before.iter().position(|&index| index == shelf).unwrap();
    let calendar_row = navigable_before.iter().position(|&index| index == calendar).unwrap();
    assert!(shelf_row < calendar_row, "the test needs the switched-off page before the open one");
    assert_eq!(notch.page_at(notch.track.position), calendar);
    fs::write(&settings_file.0, "navigation.disabled=shelf\n").unwrap();
    notch.reload_settings();
    assert!(!notch.page_list.is_enabled(shelf));
    assert!(notch.hover.is_hovering);
    assert_eq!(notch.expanded_index, Some(calendar));
    assert_eq!(notch.page_at(notch.track.position), calendar);
    assert_eq!(notch.page_at(notch.track.target), calendar);
    assert!(notch.track.is_settled(), "the open notch must not scroll to the page's new row");
}

#[test]
fn a_malformed_settings_file_keeps_the_current_values() {
    let settings_file = SettingsFile::new("reload-malformed");
    fs::write(&settings_file.0, "notch.hide_when_idle=on\nnotch.hide_for_fullscreen=off\nnavigation.disabled=stats\n")
        .unwrap();
    let options = SampleOptions { media: Some(false), timer: true, ..SampleOptions::default() };
    let mut notch = configured_rig(options, settings_file.load(), Vec::new());
    notch.reload_settings();
    let stats = notch.pages.iter().position(|page| page.id() == "stats").unwrap();
    assert!(!notch.page_list.is_enabled(stats));
    fs::write(&settings_file.0, "notch.hide_when_idle=maybe\nthis is not a setting\nnotch.hide_for_fullscreen\n")
        .unwrap();
    notch.reload_settings();
    assert!(notch.hiding.hides_when_idle);
    assert_eq!(notch.hiding.fullscreen_showing, Showing::Music);
    assert_eq!(notch.hiding.maximized_showing, Showing::MusicWithLyrics);
    assert!(!notch.page_list.is_enabled(stats));

    let every_page_off = notch.page_ids().join(",");
    fs::write(&settings_file.0, format!("navigation.disabled={every_page_off}\n")).unwrap();
    notch.reload_settings();
    assert!(!notch.page_list.is_enabled(stats), "a list with no page switched on is ignored");
    assert!((0..notch.pages.len()).any(|page| notch.page_list.is_enabled(page)));

    fs::remove_file(&settings_file.0).unwrap();
    fs::create_dir(&settings_file.0).unwrap();
    notch.reload_settings();
    assert!(notch.hiding.hides_when_idle, "an unreadable file keeps every value");
    assert_eq!(notch.hiding.fullscreen_showing, Showing::Music);
    fs::remove_dir(&settings_file.0).unwrap();
    fs::write(&settings_file.0, "").unwrap();
}

#[test]
fn reloading_settings_tells_the_media_page_to_switch_lyrics() {
    let settings_file = SettingsFile::new("reload-lyrics");
    let options = SampleOptions { media: Some(true), ..SampleOptions::default() };
    let mut notch = configured_rig(options, settings_file.load(), Vec::new());
    let media = notch.pages.iter().position(|page| page.id() == "media").unwrap();
    assert!(notch.pages[media].shows_live_text());
    fs::write(&settings_file.0, "media.lyrics=off\n").unwrap();
    notch.reload_settings();
    assert!(!notch.pages[media].shows_live_text());
    fs::write(&settings_file.0, "media.lyrics=on\n").unwrap();
    notch.reload_settings();
    assert!(notch.pages[media].shows_live_text());
    assert!(settings_file.load().get_bool("media.lyrics", false), "a reload never writes the file");
}

struct CountingActivity {
    starts: std::rc::Rc<Cell<u32>>,
    settings_changes: std::rc::Rc<Cell<u32>>,
}

impl Activity for CountingActivity {
    fn id(&self) -> &'static str {
        "counting"
    }

    fn is_optional(&self) -> bool {
        true
    }

    fn presence(&self) -> Option<Presence> {
        Some(Presence::Badge { width: 20.0 })
    }

    fn draw(&self, _canvas: &Canvas, _area: Rect, _scale: f32, _opacity: f32) {}

    fn on_start(&mut self, _context: &mut Context) {
        self.starts.set(self.starts.get() + 1);
    }

    fn on_settings_changed(&mut self, _context: &mut Context) {
        self.settings_changes.set(self.settings_changes.get() + 1);
    }
}

#[test]
fn reloading_settings_hides_a_switched_off_activity_and_starts_a_switched_on_one_once() {
    let settings_file = SettingsFile::new("reload-activity");
    fs::write(&settings_file.0, "activity.counting=off\n").unwrap();
    let (starts, settings_changes) = (std::rc::Rc::new(Cell::new(0)), std::rc::Rc::new(Cell::new(0)));
    let activity = CountingActivity { starts: starts.clone(), settings_changes: settings_changes.clone() };
    let mut notch = configured_rig(SampleOptions::default(), settings_file.load(), vec![Box::new(activity)]);
    assert_eq!(starts.get(), 0);
    notch.reload_settings();
    assert_eq!(settings_changes.get(), 0, "an activity that never started is not told");

    fs::write(&settings_file.0, "activity.counting=on\n").unwrap();
    notch.reload_settings();
    notch.reload_settings();
    assert_eq!(starts.get(), 1);
    assert_eq!(settings_changes.get(), 2);
    assert_eq!(notch.presences.badges.len(), 1);

    fs::write(&settings_file.0, "activity.counting=off\n").unwrap();
    notch.reload_settings();
    assert!(notch.presences.badges.is_empty());
    assert_eq!(starts.get(), 1);
    assert_eq!(settings_changes.get(), 3, "a started activity still hears about changes");
}
