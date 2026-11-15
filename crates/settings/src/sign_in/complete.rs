//! Turning the browser's answer into a saved account, and forgetting it again.

use std::path::Path;

use account::{
    client::{Claimed, ClientError},
    payload::read_token,
    stored::StoredAccount,
};

use super::Callback;

/// How a sign-in ended; only `SignedIn` has written `account.txt`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Completion {
    SignedIn,
    Cancelled,
    Refused,
    InvalidCode,
    Unavailable,
    /// The token the site returned does not verify for this PC.
    NotVerified,
    NotSaved,
}

/// What the site is asked for a code; `account::client::claim` in the window, a stand-in in tests.
pub type Claim = fn(&str, &str) -> Result<Claimed, ClientError>;

/// Claims the code, checks the token when signatures are enforced and saves the account; blocks for up to 15 s.
pub fn complete(
    callback: Callback,
    verifier: &str,
    device_hash: &str,
    account_path: &Path,
    now_unix: i64,
    is_enforced: bool,
    claim: Claim,
) -> Completion {
    let code = match callback {
        Callback::Code(code) => code,
        Callback::Cancelled => return Completion::Cancelled,
        Callback::Refused => return Completion::Refused,
    };
    let claimed = match claim(&code, verifier) {
        Ok(claimed) => claimed,
        Err(ClientError::InvalidCode) => return Completion::InvalidCode,
        Err(ClientError::Unavailable) => return Completion::Unavailable,
    };
    // An unenforced build has no key to check with; nothing is locked there, so the token is kept as it came.
    if is_enforced && read_token(&claimed.token, device_hash).is_none() {
        return Completion::NotVerified;
    }
    let stored = StoredAccount {
        device_id: Some(claimed.device_id),
        device_secret: Some(claimed.device_secret),
        token: Some(claimed.token),
        policy: Some(claimed.policy),
        last_check_at: Some(now_unix),
    };
    match stored.save(account_path) {
        Ok(()) => Completion::SignedIn,
        Err(_) => Completion::NotSaved,
    }
}

/// Forgets the device on this PC and returns its identity, so the site can be told after the file is written.
pub fn sign_out(account_path: &Path) -> std::io::Result<Option<(String, String)>> {
    let mut stored = StoredAccount::load(account_path);
    let identity = stored.device_id.take().zip(stored.device_secret.take());
    stored.token = None;
    stored.save(account_path)?;
    Ok(identity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const THIS_DEVICE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const NOW: i64 = 1_800_000_000;

    fn temporary_path(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!("notch-settings-sign-in-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        directory.join("account.txt")
    }

    fn sample_claim(_code: &str, _verifier: &str) -> Result<Claimed, ClientError> {
        Ok(Claimed {
            device_id: "device-1".to_owned(),
            device_secret: "secret-1".to_owned(),
            token: include_str!("../../../account/src/samples/token-valid.txt").trim().to_owned(),
            policy: include_str!("../../../account/src/samples/policy-launch-open.txt").trim().to_owned(),
        })
    }

    fn unreachable_claim(_code: &str, _verifier: &str) -> Result<Claimed, ClientError> {
        panic!("the site must not be asked")
    }

    fn refusing_claim(_code: &str, _verifier: &str) -> Result<Claimed, ClientError> {
        Err(ClientError::InvalidCode)
    }

    fn code() -> Callback {
        Callback::Code("code".to_owned())
    }

    #[test]
    fn a_cancelled_sign_in_writes_nothing() {
        let path = temporary_path("cancelled");
        let ended = complete(Callback::Cancelled, "v", THIS_DEVICE, &path, NOW, true, unreachable_claim);
        assert_eq!(ended, Completion::Cancelled);
        assert!(!path.exists());
    }

    #[test]
    fn a_refused_callback_writes_nothing_and_asks_nobody() {
        let path = temporary_path("refused");
        let ended = complete(Callback::Refused, "v", THIS_DEVICE, &path, NOW, true, unreachable_claim);
        assert_eq!(ended, Completion::Refused);
        assert!(!path.exists());
    }

    #[test]
    fn a_claimed_code_with_a_token_for_this_pc_is_saved() {
        let path = temporary_path("signed-in");
        assert_eq!(complete(code(), "v", THIS_DEVICE, &path, NOW, true, sample_claim), Completion::SignedIn);
        let stored = StoredAccount::load(&path);
        assert_eq!(stored.device_id.as_deref(), Some("device-1"));
        assert_eq!(stored.device_secret.as_deref(), Some("secret-1"));
        assert_eq!(stored.last_check_at, Some(NOW));
    }

    #[test]
    fn a_token_for_another_pc_is_not_saved_while_signatures_are_enforced() {
        let path = temporary_path("other-pc");
        let other_device = "b".repeat(64);
        assert_eq!(complete(code(), "v", &other_device, &path, NOW, true, sample_claim), Completion::NotVerified);
        assert!(!path.exists());
        assert_eq!(complete(code(), "v", &other_device, &path, NOW, false, sample_claim), Completion::SignedIn);
    }

    #[test]
    fn a_refused_code_writes_nothing() {
        let path = temporary_path("invalid-code");
        assert_eq!(complete(code(), "v", THIS_DEVICE, &path, NOW, true, refusing_claim), Completion::InvalidCode);
        assert!(!path.exists());
    }

    #[test]
    fn signing_out_clears_the_device_and_token_and_keeps_the_policy() {
        let path = temporary_path("sign-out");
        complete(code(), "v", THIS_DEVICE, &path, NOW, true, sample_claim);
        let identity = sign_out(&path).expect("saved");
        assert_eq!(identity, Some(("device-1".to_owned(), "secret-1".to_owned())));
        let stored = StoredAccount::load(&path);
        assert_eq!((stored.device_id, stored.device_secret, stored.token), (None, None, None));
        assert!(stored.policy.is_some());
    }
}
