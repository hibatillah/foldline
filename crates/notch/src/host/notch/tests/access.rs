//! What the account's access locks, and how the host hands it to pages.

use std::{cell::RefCell, collections::VecDeque};

use account::{access::Access, grants::Grants, status::Status};

use super::*;
use crate::{
    gfx::Size,
    host::notch::access::{
        AccessSource, AccessState, Evaluation, PopupPermits, locks_activity, locks_page, parse_forced_status,
        popup_call,
    },
    page::{Frame, Mode, PageSizes, Priority},
};

/// Hands out its evaluations in order and keeps repeating the last one.
pub(super) struct Scripted {
    evaluations: VecDeque<(Status, Option<i64>)>,
}

impl Scripted {
    pub(super) fn always(status: Status) -> Self {
        Self::new(&[(status, None)])
    }

    fn new(evaluations: &[(Status, Option<i64>)]) -> Self {
        Self { evaluations: evaluations.iter().copied().collect() }
    }
}

impl AccessSource for Scripted {
    fn reload(&mut self) {}

    fn evaluate(&mut self, _now_unix: i64) -> Evaluation {
        let next =
            if self.evaluations.len() > 1 { self.evaluations.pop_front() } else { self.evaluations.front().copied() };
        let (status, changes_at) = next.unwrap_or((Status::Launch, None));
        Evaluation { access: Access { status, grants: Grants::built_in() }, changes_at }
    }
}

fn access(status: Status) -> Access {
    Access { status, grants: Grants::built_in() }
}

/// Settings with both Windows-popup switches off, so a test that changes access never touches the real popups.
fn popups_left_alone(name: &str) -> SettingsFile {
    let settings_file = SettingsFile::new(name);
    fs::write(&settings_file.0, "notch.hide_windows_volume_popup=off\nnotch.hide_windows_notifications=off\n").unwrap();
    settings_file
}

#[test]
fn opening_after_the_trial_ended_locks_the_shown_page_and_moves_to_an_allowed_one() {
    let settings_file = popups_left_alone("access-trial-ended");
    let mut notch = configured_rig(SampleOptions::default(), settings_file.load(), Vec::new());
    // The trial ended at second 1, long before the test's clock; the next evaluation reads as a member.
    notch.access = AccessState::new(Box::new(Scripted::new(&[(Status::Trial, Some(1)), (Status::Member, None)])), 0);
    open(&mut notch, "weather");
    let weather = notch.selected_page.unwrap();
    assert_eq!(notch.expanded_index, Some(weather));
    close(&mut notch);

    notch.set_open(true);
    notch.springs.expansion.target = 1.0;
    settle(&mut notch);

    assert!(!notch.page_list.is_enabled(weather), "the ended trial locks the weather page");
    assert!(notch.page_list.is_chosen(weather), "the user's choice stays for when access returns");
    let shown = notch.expanded_index.expect("an allowed page shows instead");
    assert!(notch.access.access.allows_page(notch.pages[shown].id()), "{} is not allowed", notch.pages[shown].id());
}

/// An activity with a catalog id that guests may not use.
struct Bluetooth;

impl Activity for Bluetooth {
    fn id(&self) -> &'static str {
        "bluetooth"
    }

    fn presence(&self) -> Option<Presence> {
        None
    }

    fn draw(&self, _canvas: &Canvas, _area: Rect, _opacity: f32) {}
}

#[test]
fn a_locked_activity_is_never_started() {
    let settings_file = popups_left_alone("access-locked-activity");
    let mut notch = configured_rig(SampleOptions::default(), settings_file.load(), vec![Box::new(Bluetooth)]);
    notch.access = AccessState::new(Box::new(Scripted::new(&[(Status::Guest, None), (Status::Paid, None)])), 0);
    notch.lock_activities();
    assert!(notch.activities[0].is_locked);
    assert!(!notch.activities[0].is_started, "a guest's bluetooth activity is not started");

    notch.reevaluate_access(0);
    assert!(!notch.activities[0].is_locked);
    assert!(notch.activities[0].is_started, "it starts once paid");
}

#[test]
fn unpinning_after_the_trial_ended_locks_the_shown_page() {
    let settings_file = popups_left_alone("access-unpin-trial-ended");
    let mut notch = configured_rig(SampleOptions::default(), settings_file.load(), Vec::new());
    notch.screen_position = POINT { x: -100_000, y: -100_000 };
    open(&mut notch, "weather");
    let weather = notch.selected_page.unwrap();
    notch.set_pinned(true);
    // The trial ends at second 1 while pinned; nothing opens or closes until the pin is released.
    notch.access = AccessState::new(Box::new(Scripted::new(&[(Status::Trial, Some(1)), (Status::Member, None)])), 0);
    assert!(notch.page_list.is_enabled(weather), "the trial still counts while pinned");

    notch.set_pinned(false);

    assert!(!notch.page_list.is_enabled(weather), "the ended trial locks the weather page on unpin");
}

/// A page with the weather id that counts each `on_enabled_changed` it hears.
struct EnabledCounter {
    calls: Rc<RefCell<Vec<bool>>>,
}

impl Page for EnabledCounter {
    fn id(&self) -> &'static str {
        "weather"
    }

    fn priority(&self, _mode: Mode) -> Option<Priority> {
        Some(Priority::BACKGROUND)
    }

    fn sizes(&self) -> PageSizes {
        PageSizes { resting_width: 100.0, expanded: Size::new(100.0, 40.0) }
    }

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn on_enabled_changed(&mut self, is_enabled: bool, _context: &mut Context) {
        self.calls.borrow_mut().push(is_enabled);
    }
}

#[test]
fn a_page_hears_once_when_it_is_locked_once_when_unlocked_and_never_when_its_lock_stays() {
    let settings_file = popups_left_alone("access-enabled-count");
    let calls = Rc::new(RefCell::new(Vec::new()));
    let page = EnabledCounter { calls: calls.clone() };
    let mut notch = custom_rig(vec![Box::new(page)], settings_file.load(), Vec::new());
    let statuses = [(Status::Paid, None), (Status::Guest, None), (Status::Paid, None), (Status::Trial, None)];
    notch.access = AccessState::new(Box::new(Scripted::new(&statuses)), 0);

    notch.reevaluate_access(0);
    assert_eq!(*calls.borrow(), vec![false], "locking tells the page once");
    notch.reevaluate_access(0);
    assert_eq!(*calls.borrow(), vec![false, true], "unlocking tells the page once");
    notch.reevaluate_access(0);
    assert_eq!(*calls.borrow(), vec![false, true], "paid to trial leaves weather allowed, so the page hears nothing");
}

// Only the permit logic is tested here; the Windows wiring of the popups is checked by running the notch.
#[test]
fn locking_notifications_restores_the_windows_banners() {
    let permits_before = PopupPermits::of(&access(Status::Paid));
    let permits_after = PopupPermits::of(&access(Status::Guest));
    let call = popup_call(permits_before.banners, permits_after.banners, true, true);
    assert_eq!(call, Some(false));
}

#[test]
fn regaining_notifications_hides_the_banners_again_only_when_the_user_wants_it() {
    assert_eq!(popup_call(false, true, true, false), Some(true));
    assert_eq!(popup_call(false, true, false, false), None);
}

#[test]
fn an_unchanged_permit_leaves_the_popup_alone() {
    assert_eq!(popup_call(true, true, true, false), None);
    assert_eq!(popup_call(false, false, true, true), None);
}

#[test]
fn every_registered_page_and_activity_is_in_the_catalog() {
    let page_ids = crate::pages::all().iter().map(|page| page.id()).collect::<Vec<_>>();
    let activity_ids = crate::activities::all().iter().map(|activity| activity.id()).collect::<Vec<_>>();
    let missing: Vec<String> = page_ids
        .iter()
        .filter(|id| !account::catalog::is_known(id, "page"))
        .map(|id| format!("page {id}"))
        .chain(
            activity_ids
                .iter()
                .filter(|id| !account::catalog::is_known("activity", id))
                .map(|id| format!("activity {id}")),
        )
        .collect();
    assert!(missing.is_empty(), "add these to crates/account/src/catalog.rs: {missing:?}");
}

#[test]
fn an_id_the_catalog_does_not_name_is_never_locked() {
    let guest = access(Status::Guest);
    assert!(!locks_activity(&guest, "unlisted"));
    assert!(!locks_page(&guest, "unlisted"));
    assert!(locks_page(&guest, "weather"));
    assert!(locks_activity(&guest, "notifications"));
}

#[test]
fn the_forced_status_reads_every_status_and_ignores_anything_else() {
    assert_eq!(parse_forced_status(Some("guest")), Some(Status::Guest));
    assert_eq!(parse_forced_status(Some(" Paid ")), Some(Status::Paid));
    assert_eq!(parse_forced_status(Some("member")), Some(Status::Member));
    assert_eq!(parse_forced_status(Some("trial")), Some(Status::Trial));
    assert_eq!(parse_forced_status(Some("launch")), Some(Status::Launch));
    assert_eq!(parse_forced_status(Some("premium")), None);
    assert_eq!(parse_forced_status(None), None);
}

/// A page with the media id that records what `Context::allows` answers when access changes.
struct LyricsAsker {
    allows_lyrics: Rc<Cell<Option<bool>>>,
}

impl Page for LyricsAsker {
    fn id(&self) -> &'static str {
        "media"
    }

    fn priority(&self, _mode: Mode) -> Option<Priority> {
        Some(Priority::BACKGROUND)
    }

    fn sizes(&self) -> PageSizes {
        PageSizes { resting_width: 100.0, expanded: Size::new(100.0, 40.0) }
    }

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn on_access_changed(&mut self, context: &mut Context) {
        self.allows_lyrics.set(Some(context.allows("lyrics")));
    }
}

#[test]
fn a_page_hears_of_new_access_and_asks_about_its_own_sub_features() {
    let settings_file = popups_left_alone("access-page-asks");
    let allows_lyrics = Rc::new(Cell::new(None));
    let page = LyricsAsker { allows_lyrics: allows_lyrics.clone() };
    let mut notch = custom_rig(vec![Box::new(page)], settings_file.load(), Vec::new());
    notch.access = AccessState::new(Box::new(Scripted::new(&[(Status::Paid, None), (Status::Member, None)])), 0);
    notch.reevaluate_access(0);
    assert_eq!(allows_lyrics.get(), Some(false), "a member has no lyrics");
}
