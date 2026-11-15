//! The window's own account check, so a plan bought on the site shows (and reaches the notch) without waiting for the
//! notch's daily check. One request when the Account page is shown or Upgrade is clicked; no timer, no loop.

use std::path::Path;

use account::{
    client,
    schedule::{CheckOutcome, settle},
    stored::{StoredAccount, account_path},
};
use gpui::{App, Global};

use crate::{notify::tell_notch_to_reload, standing};

/// A check younger than this is fresh enough; it keeps page switching from sending a request each time.
const REFRESH_AFTER_SECONDS: i64 = 120;

/// At most one check in flight, and the time of the last attempt, so a failing site is not asked on every visit.
#[derive(Default)]
struct AccountRefresh {
    in_flight: bool,
    last_attempt_at: Option<i64>,
}

impl Global for AccountRefresh {}

/// Due in an enforced build for a signed-in PC whose last check is two minutes old, never done, or dated ahead.
pub fn should_refresh(
    is_enforced: bool,
    signed_in: bool,
    last_check_at: Option<i64>,
    now_unix: i64,
    in_flight: bool,
) -> bool {
    if !is_enforced || !signed_in || in_flight {
        return false;
    }
    last_check_at.is_none_or(|last| last > now_unix || now_unix - last >= REFRESH_AFTER_SECONDS)
}

/// Checks the stored device once and saves the outcome unless `account.txt` changed device meanwhile; blocks for up to
/// 15 s. True when the file was written.
pub fn refresh(path: &Path, device_hash: &str, now_unix: i64, check: impl FnOnce(&str, &str) -> CheckOutcome) -> bool {
    let stored = StoredAccount::load(path);
    let (Some(device_id), Some(device_secret)) = (stored.device_id, stored.device_secret) else { return false };
    let outcome = check(&device_id, &device_secret);
    let on_disk = StoredAccount::load(path);
    match settle(&on_disk, Some(&device_id), outcome, device_hash, now_unix) {
        Some((next, _notice)) => next.save(path).is_ok(),
        None => false,
    }
}

/// Starts one check when it is due; the notch is told to reload once the file changes.
pub fn refresh_if_due(cx: &mut App) {
    let standing = standing::current();
    let now = standing::now_unix();
    let (in_flight, last_attempt_at) =
        cx.try_global::<AccountRefresh>().map_or((false, None), |state| (state.in_flight, state.last_attempt_at));
    let last_check_at = standing.last_check_at.max(last_attempt_at);
    if !should_refresh(standing.is_enforced, standing.signed_in, last_check_at, now, in_flight) {
        return;
    }
    let Some(device_hash) = standing::device_hash() else { return };
    let state = cx.default_global::<AccountRefresh>();
    state.in_flight = true;
    state.last_attempt_at = Some(now);
    cx.spawn(async move |cx| {
        let written = cx
            .background_executor()
            .spawn(async move {
                // A refused join is not fatal; the request joins the apartment on its own helper thread too.
                let _apartment = client::join_multithreaded_apartment();
                refresh(&account_path(), &device_hash, now, client::check)
            })
            .await;
        let _ = cx.update(|cx| {
            cx.default_global::<AccountRefresh>().in_flight = false;
            if written {
                tell_notch_to_reload();
                cx.refresh_windows();
            }
        });
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const NOW: i64 = 2_000_000_000;
    const THIS_DEVICE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn temporary_path(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!("notch-settings-refresh-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        directory.join("account.txt")
    }

    fn signed_in(device_id: &str) -> StoredAccount {
        StoredAccount {
            device_id: Some(device_id.to_owned()),
            device_secret: Some("secret".to_owned()),
            token: Some("old-token".to_owned()),
            policy: Some("old-policy".to_owned()),
            last_check_at: Some(NOW - 600),
        }
    }

    fn renewed() -> CheckOutcome {
        CheckOutcome::Renewed {
            token: Some(include_str!("../../account/src/samples/token-valid.txt").trim().to_owned()),
            policy: include_str!("../../account/src/samples/policy-launch-open.txt").trim().to_owned(),
        }
    }

    #[test]
    fn a_signed_in_pc_checked_over_two_minutes_ago_is_refreshed() {
        assert!(should_refresh(true, true, Some(NOW - 120), NOW, false));
        assert!(should_refresh(true, true, None, NOW, false));
        assert!(should_refresh(true, true, Some(NOW + 60), NOW, false), "a check dated ahead is due");
    }

    #[test]
    fn a_recent_check_is_not_repeated() {
        assert!(!should_refresh(true, true, Some(NOW - 119), NOW, false));
    }

    #[test]
    fn an_unenforced_build_a_guest_or_a_check_in_flight_sends_nothing() {
        assert!(!should_refresh(false, true, None, NOW, false));
        assert!(!should_refresh(true, false, None, NOW, false));
        assert!(!should_refresh(true, true, None, NOW, true));
    }

    #[test]
    fn a_verified_renewal_is_saved() {
        let path = temporary_path("renewed");
        signed_in("device-1").save(&path).expect("save");
        assert!(refresh(&path, THIS_DEVICE, NOW, |_, _| renewed()));
        assert_eq!(StoredAccount::load(&path).last_check_at, Some(NOW));
    }

    #[test]
    fn a_renewal_is_dropped_when_the_file_changed_device_during_the_check() {
        let path = temporary_path("replaced");
        signed_in("device-1").save(&path).expect("save");
        let written = refresh(&path, THIS_DEVICE, NOW, |_, _| {
            signed_in("device-2").save(&path).expect("save");
            renewed()
        });
        assert!(!written);
        assert_eq!(StoredAccount::load(&path), signed_in("device-2"));
    }

    #[test]
    fn a_signed_out_file_asks_the_site_nothing() {
        let path = temporary_path("signed-out");
        let written = refresh(&path, THIS_DEVICE, NOW, |_, _| panic!("the site must not be asked"));
        assert!(!written);
    }
}
