//! Starts the daily account check when due, at start-up and on resume, and applies what it brings back.

use std::path::PathBuf;

use account::{
    schedule::{CheckOutcome, Notice, should_check},
    stored::StoredAccount,
};

use super::Notch;
use crate::{
    host::account_check::{Outcome, WM_ACCOUNT_CHECKED, settle},
    log,
    page::Notifier,
};

/// Sends the request off the UI thread; `account_check::start` outside tests.
pub(in crate::host) type StartRequest = Box<dyn FnMut(StoredAccount, Notifier) -> Outcome>;

/// The request under way and the device it was made for.
struct InFlight {
    device_id: Option<String>,
    outcome: Outcome,
}

pub(in crate::host) struct DailyCheck {
    path: PathBuf,
    device_hash: String,
    start_request: StartRequest,
    in_flight: Option<InFlight>,
}

impl DailyCheck {
    pub(in crate::host) fn new(path: PathBuf, device_hash: String, start_request: StartRequest) -> Self {
        Self { path, device_hash, start_request, in_flight: None }
    }

    /// The checked device and the outcome, once the request has one; the request stays in flight until then.
    fn take_finished(&mut self) -> Option<(Option<String>, CheckOutcome)> {
        let outcome = self.in_flight.as_ref()?.outcome.lock().ok()?.take()?;
        let in_flight = self.in_flight.take()?;
        Some((in_flight.device_id, outcome))
    }
}

/// Without a production key no renewal can be verified, so an unenforced build makes no account request at all.
fn should_start_check(is_enforced: bool, last_check_at: Option<i64>, now_unix: i64, is_in_flight: bool) -> bool {
    is_enforced && !is_in_flight && should_check(last_check_at, now_unix)
}

impl Notch {
    /// One request when a day has passed since the last check; none while one is under way, and never a retry.
    pub(in crate::host) fn start_daily_check_if_due(&mut self, now_unix: i64) {
        let check = &mut self.daily_check;
        let stored = StoredAccount::load(&check.path);
        let is_due =
            should_start_check(account::keys::is_enforced(), stored.last_check_at, now_unix, check.in_flight.is_some());
        if !is_due {
            return;
        }
        log::debug("account check started");
        let device_id = stored.device_id.clone();
        let outcome = (check.start_request)(stored, Notifier::for_host(self.window, WM_ACCOUNT_CHECKED));
        check.in_flight = Some(InFlight { device_id, outcome });
    }

    /// The worker posted `WM_ACCOUNT_CHECKED`.
    pub fn daily_check_finished(&mut self) {
        let Some((device_id, outcome)) = self.daily_check.take_finished() else { return };
        self.apply_check_outcome(device_id.as_deref(), outcome, store::pause::now_unix());
        self.flush();
    }

    pub(super) fn apply_check_outcome(
        &mut self,
        checked_device_id: Option<&str>,
        outcome: CheckOutcome,
        now_unix: i64,
    ) {
        let path = &self.daily_check.path;
        // The settings window may have signed in or out while the request ran, so the file is read again here.
        let on_disk = StoredAccount::load(path);
        let Some((next, notice)) =
            settle(&on_disk, checked_device_id, outcome, &self.daily_check.device_hash, now_unix)
        else {
            log::debug("account check changed nothing");
            return;
        };
        if let Err(error) = next.save(path) {
            log::debug(&format!("could not save the account check: {error}"));
            return;
        }
        log::debug("account check saved");
        self.reload_access();
        if notice == Notice::SignedOutElsewhere {
            self.show_account_notice();
        }
    }

    fn show_account_notice(&mut self) {
        for index in 0..self.activities.len() {
            if self.activities[index].is_started {
                self.with_activity(index, |activity, context| activity.on_account_signed_out_elsewhere(context));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 2_000_000_000;
    const DAY: i64 = 86_400;

    #[test]
    fn an_unenforced_build_never_asks_the_site() {
        assert!(!should_start_check(false, None, NOW, false));
    }

    #[test]
    fn an_unenforced_build_does_not_ask_even_after_a_day() {
        assert!(!should_start_check(false, Some(NOW - DAY), NOW, false));
    }

    #[test]
    fn an_enforced_build_asks_when_it_never_checked() {
        assert!(should_start_check(true, None, NOW, false));
    }

    #[test]
    fn an_enforced_build_asks_once_a_day_has_passed() {
        assert!(should_start_check(true, Some(NOW - DAY), NOW, false));
    }

    #[test]
    fn an_enforced_build_does_not_ask_within_a_day() {
        assert!(!should_start_check(true, Some(NOW - DAY + 60), NOW, false));
    }

    #[test]
    fn an_enforced_build_does_not_ask_while_a_request_is_under_way() {
        assert!(!should_start_check(true, None, NOW, true));
    }
}
