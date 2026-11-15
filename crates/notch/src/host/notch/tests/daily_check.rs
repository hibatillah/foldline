//! The daily account check: when it starts, and what its outcome does to the file, the access and the pill.

use std::{
    cell::RefCell,
    path::Path,
    sync::{Arc, Mutex},
};

use account::{schedule::CheckOutcome, stored::StoredAccount};

use super::*;
use crate::host::{account_check::Outcome, notch::DailyCheck};

const NOW: i64 = 2_000_000_000;
const DAY: i64 = 86_400;

/// A check that is started but never hears back, so a test never reaches the network.
pub(super) fn never_answers() -> DailyCheck {
    let path = std::env::temp_dir().join(format!("notch-no-account-{}.txt", std::process::id()));
    DailyCheck::new(path, String::new(), Box::new(|_, _| Outcome::default()))
}

/// `account.txt` in the temp folder, removed when the test ends.
struct AccountFileOnDisk(PathBuf);

impl AccountFileOnDisk {
    fn new(name: &str, account: &StoredAccount) -> Self {
        let path = std::env::temp_dir().join(format!("notch-account-{}-{name}.txt", std::process::id()));
        account.save(&path).unwrap();
        Self(path)
    }

    fn read(&self) -> String {
        fs::read_to_string(&self.0).unwrap()
    }
}

impl Drop for AccountFileOnDisk {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_file(&self.0) {
            eprintln!("could not remove test account {}: {error}", self.0.display());
        }
    }
}

fn signed_in(last_check_at: i64) -> StoredAccount {
    StoredAccount {
        device_id: Some("device-1".into()),
        // A device id without a secret reads back as signed out.
        device_secret: Some("secret".into()),
        token: Some("token".into()),
        policy: Some("policy".into()),
        last_check_at: Some(last_check_at),
    }
}

/// A check whose request answers `answer` at once, and the accounts it was started for.
fn answering(path: &Path, answer: CheckOutcome) -> (DailyCheck, Rc<RefCell<Vec<StoredAccount>>>) {
    let started = Rc::new(RefCell::new(Vec::new()));
    let seen = Rc::clone(&started);
    let start_request = Box::new(move |stored: StoredAccount, _| {
        seen.borrow_mut().push(stored);
        Arc::new(Mutex::new(Some(answer.clone())))
    });
    (DailyCheck::new(path.to_path_buf(), "a".repeat(64), start_request), started)
}

/// A rig with the account notice started and the daily check over `file`.
fn checked_rig(file: &AccountFileOnDisk, answer: CheckOutcome) -> (Notch, Rc<RefCell<Vec<StoredAccount>>>) {
    let activities: Vec<Box<dyn Activity>> = vec![crate::activities::sample_account_notice()];
    let mut notch = configured_rig(SampleOptions::default(), Settings::at(None), activities);
    notch.lock_activities();
    let (check, started) = answering(&file.0, answer);
    notch.daily_check = check;
    (notch, started)
}

fn shows_the_notice(notch: &mut Notch) -> bool {
    notch.refresh_presences();
    notch.presences.transients.iter().any(|&(source, _)| source == Source::Activity(0))
}

#[test]
fn no_check_starts_within_a_day_of_the_last_one() {
    let file = AccountFileOnDisk::new("within-a-day", &signed_in(NOW - DAY + 60));
    let (mut notch, started) = checked_rig(&file, CheckOutcome::Failed);
    notch.start_daily_check_if_due(NOW);
    assert!(started.borrow().is_empty());
}

#[test]
fn a_check_starts_once_a_day_has_passed_and_not_again_while_it_runs() {
    let file = AccountFileOnDisk::new("a-day-passed", &signed_in(NOW - DAY));
    let (mut notch, started) = checked_rig(&file, CheckOutcome::Failed);
    notch.start_daily_check_if_due(NOW);
    notch.start_daily_check_if_due(NOW);
    assert_eq!(started.borrow().len(), 1);
}

#[test]
fn a_failed_check_keeps_access_and_last_check_time() {
    let file = AccountFileOnDisk::new("failed", &signed_in(NOW - DAY));
    let before = file.read();
    let (mut notch, _) = checked_rig(&file, CheckOutcome::Failed);
    let status_before = notch.access.access.status;
    notch.start_daily_check_if_due(NOW);
    notch.daily_check_finished();
    assert_eq!((file.read(), notch.access.access.status), (before, status_before));
}

#[test]
fn a_failed_check_is_tried_again_at_the_next_resume() {
    let file = AccountFileOnDisk::new("failed-again", &signed_in(NOW - DAY));
    let (mut notch, started) = checked_rig(&file, CheckOutcome::Failed);
    notch.start_daily_check_if_due(NOW);
    notch.daily_check_finished();
    notch.start_daily_check_if_due(NOW + 60);
    assert_eq!(started.borrow().len(), 2);
}

#[test]
fn signed_out_elsewhere_shows_one_pill_message() {
    let file = AccountFileOnDisk::new("signed-out", &signed_in(NOW - DAY));
    let (mut notch, _) = checked_rig(&file, CheckOutcome::SignedOut);
    assert!(!shows_the_notice(&mut notch));
    notch.start_daily_check_if_due(NOW);
    notch.daily_check_finished();
    assert!(shows_the_notice(&mut notch));
}

#[test]
fn signed_out_elsewhere_forgets_the_device_and_needs_no_check_for_a_day() {
    let file = AccountFileOnDisk::new("signed-out-file", &signed_in(NOW - DAY));
    let (mut notch, started) = checked_rig(&file, CheckOutcome::SignedOut);
    notch.start_daily_check_if_due(store::pause::now_unix());
    notch.daily_check_finished();
    let stored = StoredAccount::load(&file.0);
    notch.start_daily_check_if_due(store::pause::now_unix());
    assert_eq!((stored.device_id, stored.token, started.borrow().len()), (None, None, 1));
}

#[test]
fn an_unknown_device_signs_out_without_a_pill_message() {
    let file = AccountFileOnDisk::new("unknown-device", &signed_in(NOW - DAY));
    let (mut notch, _) = checked_rig(&file, CheckOutcome::UnknownDevice);
    notch.start_daily_check_if_due(NOW);
    notch.daily_check_finished();
    assert!(!shows_the_notice(&mut notch));
}

#[test]
fn a_sign_in_during_the_check_keeps_the_new_account_and_shows_no_message() {
    let file = AccountFileOnDisk::new("signed-in-meanwhile", &signed_in(NOW - DAY));
    let (mut notch, _) = checked_rig(&file, CheckOutcome::SignedOut);
    notch.start_daily_check_if_due(NOW);
    let new_account = StoredAccount { device_id: Some("device-2".into()), ..signed_in(NOW) };
    new_account.save(&file.0).unwrap();
    let written = file.read();
    notch.daily_check_finished();
    assert_eq!((file.read(), shows_the_notice(&mut notch)), (written, false));
}
