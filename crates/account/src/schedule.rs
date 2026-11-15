use crate::payload::{read_policy, read_token};
use crate::stored::StoredAccount;

const CHECK_INTERVAL_SECONDS: i64 = 86_400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckOutcome {
    /// `token` is `None` for a guest's policy fetch, which keeps the stored token.
    Renewed {
        token: Option<String>,
        policy: String,
    },
    SignedOut,
    UnknownDevice,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notice {
    None,
    SignedOutElsewhere,
}

/// Due when never done, a day old, or dated in the future (a clock that went back).
pub fn should_check(last_check_at: Option<i64>, now_unix: i64) -> bool {
    match last_check_at {
        None => true,
        Some(last) => last > now_unix || now_unix - last >= CHECK_INTERVAL_SECONDS,
    }
}

/// The account after an outcome; a failed check, or a renewal whose signatures do not verify, changes nothing.
pub fn apply(
    stored: &StoredAccount,
    outcome: CheckOutcome,
    this_device_hash: &str,
    now_unix: i64,
) -> (StoredAccount, Notice) {
    let mut next = stored.clone();
    match outcome {
        CheckOutcome::Renewed { token, policy } => {
            let token_verifies = token.as_deref().is_none_or(|text| read_token(text, this_device_hash).is_some());
            if !token_verifies || read_policy(&policy).is_none() {
                return (next, Notice::None);
            }
            if token.is_some() {
                next.token = token;
            }
            next.policy = Some(policy);
            next.last_check_at = Some(now_unix);
            (next, Notice::None)
        }
        CheckOutcome::SignedOut => {
            sign_out_locally(&mut next, now_unix);
            (next, Notice::SignedOutElsewhere)
        }
        CheckOutcome::UnknownDevice => {
            sign_out_locally(&mut next, now_unix);
            (next, Notice::None)
        }
        CheckOutcome::Failed => (next, Notice::None),
    }
}

/// What to write after a check, or `None` to write nothing: the outcome changed nothing, or the file on disk now
/// belongs to another device than the one checked (the other program signed in or out meanwhile).
pub fn settle(
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

/// The site answered, so the check counts as done: a guest's policy fetch then waits a day, which is fine.
fn sign_out_locally(account: &mut StoredAccount, now_unix: i64) {
    account.device_id = None;
    account.device_secret = None;
    account.token = None;
    account.last_check_at = Some(now_unix);
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 2_000_000_000;
    const VALID_TOKEN: &str = include_str!("samples/token-valid.txt");
    const VALID_POLICY: &str = include_str!("samples/policy-launch-open.txt");
    const TAMPERED_TOKEN: &str = include_str!("samples/token-tampered.txt");

    fn this_device() -> String {
        "a".repeat(64)
    }

    fn signed_in() -> StoredAccount {
        StoredAccount {
            device_id: Some("device".into()),
            device_secret: Some("secret".into()),
            token: Some("old-token".into()),
            policy: Some("old-policy".into()),
            last_check_at: Some(NOW - 90_000),
        }
    }

    fn renewed(token: Option<&str>, policy: &str) -> CheckOutcome {
        CheckOutcome::Renewed { token: token.map(|text| text.trim().to_owned()), policy: policy.trim().to_owned() }
    }

    #[test]
    fn the_check_is_due_after_a_day_and_when_never_done() {
        assert!(should_check(None, NOW));
        assert!(!should_check(Some(NOW - 86_399), NOW));
        assert!(should_check(Some(NOW - 86_400), NOW));
    }

    #[test]
    fn a_check_time_in_the_future_is_due() {
        assert!(should_check(Some(NOW + 1), NOW));
    }

    #[test]
    fn a_verified_renewal_replaces_the_token_and_policy_and_records_the_time() {
        let outcome = renewed(Some(VALID_TOKEN), VALID_POLICY);
        let (stored, notice) = apply(&signed_in(), outcome, &this_device(), NOW);
        assert_eq!(stored.token.as_deref(), Some(VALID_TOKEN.trim()));
        assert_eq!(stored.policy.as_deref(), Some(VALID_POLICY.trim()));
        assert_eq!(stored.device_id.as_deref(), Some("device"));
        assert_eq!(stored.last_check_at, Some(NOW));
        assert_eq!(notice, Notice::None);
    }

    #[test]
    fn a_policy_only_renewal_keeps_the_stored_token() {
        let (stored, _) = apply(&signed_in(), renewed(None, VALID_POLICY), &this_device(), NOW);
        assert_eq!(stored.token.as_deref(), Some("old-token"));
        assert_eq!(stored.policy.as_deref(), Some(VALID_POLICY.trim()));
        assert_eq!(stored.last_check_at, Some(NOW));
    }

    #[test]
    fn a_renewal_with_an_unverifiable_token_keeps_the_old_one_and_the_old_check_time() {
        let before = signed_in();
        let bad_tokens = [TAMPERED_TOKEN, "garbage"];
        for bad_token in bad_tokens {
            let (stored, notice) = apply(&before, renewed(Some(bad_token), VALID_POLICY), &this_device(), NOW);
            assert_eq!(stored, before);
            assert_eq!(notice, Notice::None);
        }
        // A genuine token for another PC is refused too.
        let (stored, _) = apply(&before, renewed(Some(VALID_TOKEN), VALID_POLICY), &"b".repeat(64), NOW);
        assert_eq!(stored, before);
    }

    #[test]
    fn a_renewal_with_an_unverifiable_policy_changes_nothing() {
        let before = signed_in();
        for bad_policy in [TAMPERED_TOKEN, "garbage"] {
            let (stored, _) = apply(&before, renewed(Some(VALID_TOKEN), bad_policy), &this_device(), NOW);
            assert_eq!(stored, before);
            let (stored, _) = apply(&before, renewed(None, bad_policy), &this_device(), NOW);
            assert_eq!(stored, before);
        }
    }

    #[test]
    fn signed_out_elsewhere_clears_the_device_and_keeps_the_policy() {
        let (stored, notice) = apply(&signed_in(), CheckOutcome::SignedOut, &this_device(), NOW);
        assert_eq!((stored.device_id, stored.device_secret, stored.token), (None, None, None));
        assert_eq!(stored.policy.as_deref(), Some("old-policy"));
        assert_eq!(stored.last_check_at, Some(NOW));
        assert_eq!(notice, Notice::SignedOutElsewhere);
    }

    #[test]
    fn an_unknown_device_clears_it_without_a_notice() {
        let (stored, notice) = apply(&signed_in(), CheckOutcome::UnknownDevice, &this_device(), NOW);
        assert_eq!((stored.device_id, stored.device_secret, stored.token), (None, None, None));
        assert_eq!(stored.policy.as_deref(), Some("old-policy"));
        assert_eq!(notice, Notice::None);
    }

    #[test]
    fn a_settled_check_for_a_device_replaced_meanwhile_writes_nothing() {
        let on_disk = StoredAccount { device_id: Some("new".into()), ..signed_in() };
        assert_eq!(settle(&on_disk, Some("device"), CheckOutcome::SignedOut, &this_device(), NOW), None);
    }

    #[test]
    fn a_settled_renewal_for_the_same_device_is_written() {
        let outcome = renewed(Some(VALID_TOKEN), VALID_POLICY);
        let (next, _) = settle(&signed_in(), Some("device"), outcome, &this_device(), NOW).expect("written");
        assert_eq!(next.last_check_at, Some(NOW));
    }

    #[test]
    fn a_failed_check_keeps_access_and_last_check_time() {
        let before = signed_in();
        let (stored, notice) = apply(&before, CheckOutcome::Failed, &this_device(), NOW);
        assert_eq!(stored, before);
        assert_eq!(notice, Notice::None);
    }
}
