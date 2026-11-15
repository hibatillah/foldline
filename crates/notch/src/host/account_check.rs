//! The daily account check: one request on a thread of its own that ends with it, then a `Notifier` wakes the host.

use std::sync::{Arc, Mutex};

use account::{
    client,
    schedule::{CheckOutcome, Notice, apply},
    stored::StoredAccount,
};
use windows::Win32::UI::WindowsAndMessaging::WM_APP;

use crate::page::Notifier;

/// Posted to the notch window when a check has an outcome.
pub(super) const WM_ACCOUNT_CHECKED: u32 = WM_APP + 33;

pub(super) type Outcome = Arc<Mutex<Option<CheckOutcome>>>;

/// Starts the request for `stored`; the returned slot holds the outcome once `notifier` fires.
pub(super) fn start(stored: StoredAccount, notifier: Notifier) -> Outcome {
    let outcome: Outcome = Arc::default();
    let slot = Arc::clone(&outcome);
    std::thread::spawn(move || {
        // Explicit membership of the multithreaded apartment for the WinRT calls; an existing one is fine.
        let _apartment = client::join_multithreaded_apartment()
            .inspect_err(|error| crate::log::debug(&format!("could not join the apartment: {error}")));
        let result = request(&stored);
        if let Ok(mut slot) = slot.lock() {
            *slot = Some(result);
        }
        notifier.notify();
    });
    outcome
}

/// A device check when signed in, otherwise the guest's policy fetch, which sends no identifier.
fn request(stored: &StoredAccount) -> CheckOutcome {
    match (&stored.device_id, &stored.device_secret) {
        (Some(device_id), Some(device_secret)) => client::check(device_id, device_secret),
        _ => client::fetch_policy(),
    }
}

/// What to write after a check, or `None` to write nothing: the outcome changed nothing, or the file on disk now
/// belongs to another device than the one checked (the settings window signed in or out meanwhile).
pub(super) fn settle(
    on_disk: &StoredAccount,
    checked_device_id: Option<&str>,
    outcome: CheckOutcome,
    this_device_hash: &str,
    now_unix: i64,
) -> Option<(StoredAccount, Notice)> {
    if on_disk.device_id.as_deref() != checked_device_id {
        return None;
    }
    let (next, notice) = apply(on_disk, outcome, this_device_hash, now_unix);
    (next != *on_disk).then_some((next, notice))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 2_000_000_000;

    fn signed_in(device_id: &str) -> StoredAccount {
        StoredAccount {
            device_id: Some(device_id.into()),
            device_secret: Some("secret".into()),
            token: Some("token".into()),
            policy: Some("policy".into()),
            last_check_at: Some(NOW - 90_000),
        }
    }

    #[test]
    fn a_check_result_for_a_device_that_signed_out_meanwhile_is_dropped() {
        let on_disk = StoredAccount { device_id: None, ..signed_in("old") };
        assert_eq!(settle(&on_disk, Some("old"), CheckOutcome::SignedOut, "hash", NOW), None);
    }

    #[test]
    fn a_check_result_for_a_device_replaced_by_a_new_sign_in_is_dropped() {
        let on_disk = signed_in("new");
        assert_eq!(settle(&on_disk, Some("old"), CheckOutcome::SignedOut, "hash", NOW), None);
    }

    #[test]
    fn a_guest_policy_fetch_is_dropped_when_the_user_signed_in_meanwhile() {
        let on_disk = signed_in("new");
        assert_eq!(settle(&on_disk, None, CheckOutcome::UnknownDevice, "hash", NOW), None);
    }

    #[test]
    fn a_failed_check_writes_nothing() {
        assert_eq!(settle(&signed_in("same"), Some("same"), CheckOutcome::Failed, "hash", NOW), None);
    }

    #[test]
    fn signed_out_elsewhere_for_the_same_device_clears_it_and_brings_the_notice() {
        let (next, notice) = settle(&signed_in("same"), Some("same"), CheckOutcome::SignedOut, "hash", NOW).unwrap();
        assert_eq!((next.device_id, next.last_check_at, notice), (None, Some(NOW), Notice::SignedOutElsewhere));
    }
}
