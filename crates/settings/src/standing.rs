//! What the window shows about the account: who is signed in, the status, and which pages carry a lock. Display only;
//! the notch enforces access itself.

use std::{cell::RefCell, path::Path, rc::Rc, time::SystemTime};

use account::{
    access::Access,
    base64url, catalog,
    grants::Grants,
    json,
    payload::{read_policy, read_token},
    status::{Status, forced_status},
    stored::{StoredAccount, account_path},
};

pub struct Standing {
    pub signed_in: bool,
    pub email: Option<String>,
    pub status: Status,
    pub trial_ends_at: Option<i64>,
    /// False in a release build without a production key: nothing is locked and no status is shown.
    pub is_enforced: bool,
    /// The next moment the status changes by itself (a trial or the launch ends); the cache is read again after it.
    pub changes_at: Option<i64>,
    /// When the site last answered a check, as written in `account.txt`.
    pub last_check_at: Option<i64>,
    access: Access,
}

/// What a locked page's button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lock {
    SignIn,
    /// Signed in, but the stored token does not verify on this PC.
    SignInAgain,
    Upgrade,
}

impl Standing {
    pub fn of(stored: &StoredAccount, device_hash: Option<&str>, now_unix: i64, is_enforced: bool) -> Standing {
        let device_hash = device_hash.unwrap_or_default();
        let access = stored.access(device_hash, now_unix);
        let verified = stored.token.as_deref().and_then(|token| read_token(token, device_hash));
        let email = match &verified {
            Some(token) => Some(token.email.clone()),
            None if !is_enforced => stored.token.as_deref().and_then(unverified_email),
            None => None,
        };
        // Only a running trial has an end worth showing; an ended one reads as a free account.
        let token_trial_end = verified.as_ref().and_then(|token| token.trial_ends_at);
        let launch_end = stored.policy.as_deref().and_then(read_policy).and_then(|policy| policy.launch.ends_at);
        let changes_at = [token_trial_end, launch_end].into_iter().flatten().filter(|moment| *moment > now_unix).min();
        let trial_ends_at = token_trial_end.filter(|_| access.status == Status::Trial);
        Standing {
            signed_in: stored.device_id.is_some() && stored.device_secret.is_some(),
            email,
            status: access.status,
            trial_ends_at,
            is_enforced,
            changes_at,
            last_check_at: stored.last_check_at,
            access,
        }
    }

    /// What `NOTCH_ACCOUNT_STATUS` forces in a debug build, read the way the notch reads it, so the locks agree.
    pub fn with_forced_status(self, forced: Option<Status>) -> Standing {
        let Some(status) = forced else { return self };
        Standing {
            status,
            trial_ends_at: None,
            changes_at: None,
            access: Access { status, grants: Grants::built_in() },
            ..self
        }
    }

    /// `None` for a page outside the catalog or one the status allows.
    pub fn lock_for(&self, page_id: &str) -> Option<Lock> {
        self.lock_for_feature(page_id, "page")
    }

    /// The lock on one row: a page (`weather`, `page`), a sub-feature (`weather`, `rain_alert`) or an activity
    /// (`activity`, `notifications`); `None` outside the catalog, when allowed, or in a build that locks nothing.
    pub fn lock_for_feature(&self, resource: &str, action: &str) -> Option<Lock> {
        if !self.is_enforced || !catalog::is_known(resource, action) || self.access.allows(resource, action) {
            return None;
        }
        Some(match (self.status, self.signed_in) {
            (Status::Guest, false) => Lock::SignIn,
            (Status::Guest, true) => Lock::SignInAgain,
            _ => Lock::Upgrade,
        })
    }

    /// Upgrading means nothing to a paid account or to a build that locks nothing.
    pub fn offers_upgrade(&self) -> bool {
        self.signed_in && self.is_enforced && self.status != Status::Paid
    }
}

/// The email inside a token that cannot be checked (a build without its key); shown, never trusted for access.
fn unverified_email(token: &str) -> Option<String> {
    let (payload, _) = token.split_once('.')?;
    let text = String::from_utf8(base64url::decode(payload)?).ok()?;
    json::parse_payload(&text)?.email
}

pub fn now_unix() -> i64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_secs() as i64)
}

struct Cached {
    modified: Option<SystemTime>,
    standing: Rc<Standing>,
}

/// A cached standing holds while the file is unchanged (and exists) and no trial or launch end has passed since.
fn is_fresh(
    cached_modified: Option<SystemTime>,
    modified: Option<SystemTime>,
    changes_at: Option<i64>,
    now: i64,
) -> bool {
    modified.is_some() && cached_modified == modified && changes_at.is_none_or(|moment| now < moment)
}

thread_local! {
    static CACHE: RefCell<Option<Cached>> = const { RefCell::new(None) };
    static DEVICE_HASH: Option<String> = account::device::device_hash();
}

/// The standing read from `account.txt`, read again only when the file's modified time changes (a sign-in here, or the
/// notch's daily check) or once a trial or launch end passes, since the device secret is unprotected through DPAPI on
/// every read. Checked on paint, so no timer runs for it.
pub fn current() -> Rc<Standing> {
    let path = account_path();
    let modified = std::fs::metadata(&path).and_then(|metadata| metadata.modified()).ok();
    CACHE.with_borrow_mut(|cache| {
        let now = now_unix();
        if let Some(cached) =
            cache.as_ref().filter(|cached| is_fresh(cached.modified, modified, cached.standing.changes_at, now))
        {
            return cached.standing.clone();
        }
        let standing = Rc::new(read(&path));
        *cache = Some(Cached { modified, standing: standing.clone() });
        standing
    })
}

/// The PC's device hash, read once per thread.
pub fn device_hash() -> Option<String> {
    DEVICE_HASH.with(Clone::clone)
}

fn read(path: &Path) -> Standing {
    let device_hash = device_hash();
    Standing::of(&StoredAccount::load(path), device_hash.as_deref(), now_unix(), account::keys::is_enforced())
        .with_forced_status(forced_status())
}

#[cfg(test)]
mod tests {
    use super::*;

    const THIS_DEVICE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const NOW: i64 = 1_800_000_000;

    fn signed_in(token: &str) -> StoredAccount {
        StoredAccount {
            device_id: Some("device-1".to_owned()),
            device_secret: Some("secret".to_owned()),
            token: Some(token.trim().to_owned()),
            ..StoredAccount::default()
        }
    }

    #[test]
    fn a_guest_sees_a_sign_in_lock_on_paid_pages_only() {
        let standing = Standing::of(&StoredAccount::default(), Some(THIS_DEVICE), NOW, true);
        assert!(!standing.signed_in);
        assert_eq!(standing.lock_for("weather"), Some(Lock::SignIn));
        assert_eq!(standing.lock_for("media"), None);
    }

    #[test]
    fn a_running_trial_changes_at_its_end_and_reads_as_trial_until_then() {
        let token = include_str!("../../account/src/samples/token-trial-expired.txt");
        let ends_at = 1_767_830_400;
        let standing = Standing::of(&signed_in(token), Some(THIS_DEVICE), ends_at - 60, true);
        assert_eq!(standing.status, Status::Trial);
        assert_eq!(standing.changes_at, Some(ends_at));
        assert_eq!(Standing::of(&signed_in(token), Some(THIS_DEVICE), ends_at, true).changes_at, None);
    }

    #[test]
    fn a_cached_standing_is_read_again_after_its_change_or_a_new_file() {
        let written = Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(5));
        let rewritten = Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(9));
        assert!(is_fresh(written, written, Some(100), 99));
        assert!(!is_fresh(written, written, Some(100), 100));
        assert!(!is_fresh(written, rewritten, None, 0));
        assert!(!is_fresh(None, None, None, 0), "a missing file is read again every time");
    }

    #[test]
    fn sub_features_and_activities_lock_like_pages() {
        let guest = Standing::of(&StoredAccount::default(), Some(THIS_DEVICE), NOW, true);
        assert_eq!(guest.lock_for_feature("weather", "rain_alert"), Some(Lock::SignIn));
        assert_eq!(guest.lock_for_feature("media", "lyrics"), Some(Lock::SignIn));
        assert_eq!(guest.lock_for_feature("activity", "notifications"), Some(Lock::SignIn));
        assert_eq!(guest.lock_for_feature("activity", "downloads"), None);
        assert_eq!(guest.lock_for_feature("activity", "unknown"), None);
        let paid = Standing::of(
            &signed_in(include_str!("../../account/src/samples/token-valid.txt")),
            Some(THIS_DEVICE),
            NOW,
            true,
        );
        assert_eq!(paid.lock_for_feature("activity", "notifications"), None);
    }

    #[test]
    fn a_page_outside_the_catalog_never_carries_a_lock() {
        let standing = Standing::of(&StoredAccount::default(), Some(THIS_DEVICE), NOW, true);
        assert_eq!(standing.lock_for("clock"), None);
    }

    #[test]
    fn a_paid_token_for_this_pc_unlocks_everything_and_shows_its_email() {
        let standing = Standing::of(
            &signed_in(include_str!("../../account/src/samples/token-valid.txt")),
            Some(THIS_DEVICE),
            NOW,
            true,
        );
        assert_eq!(standing.status, Status::Paid);
        assert_eq!(standing.email.as_deref(), Some("sample@example.com"));
        assert_eq!(standing.lock_for("weather"), None);
        assert!(!standing.offers_upgrade());
    }

    #[test]
    fn a_member_sees_an_upgrade_lock() {
        let token = include_str!("../../account/src/samples/token-trial-expired.txt");
        let standing = Standing::of(&signed_in(token), Some(THIS_DEVICE), NOW, true);
        assert_eq!(standing.status, Status::Member);
        assert_eq!(standing.lock_for("weather"), Some(Lock::Upgrade));
        assert_eq!(standing.trial_ends_at, None, "an ended trial shows no end date");
        assert!(standing.offers_upgrade());
    }

    #[test]
    fn a_token_for_another_pc_shows_no_email_while_enforced() {
        let token = include_str!("../../account/src/samples/token-valid.txt");
        let standing = Standing::of(&signed_in(token), Some(&"b".repeat(64)), NOW, true);
        assert!(standing.signed_in);
        assert_eq!(standing.email, None);
        assert_eq!(standing.lock_for("weather"), Some(Lock::SignInAgain));
    }

    #[test]
    fn a_forced_status_sets_the_locks_like_the_notch() {
        let guest = Standing::of(&StoredAccount::default(), Some(THIS_DEVICE), NOW, true);
        let forced = guest.with_forced_status(account::status::parse_forced_status(Some("paid")));
        assert_eq!(forced.status, Status::Paid);
        assert_eq!(forced.lock_for("weather"), None);
    }

    #[test]
    fn no_forced_status_keeps_the_standing_from_the_file() {
        let guest = Standing::of(&StoredAccount::default(), Some(THIS_DEVICE), NOW, true).with_forced_status(None);
        assert_eq!(guest.lock_for("weather"), Some(Lock::SignIn));
    }

    #[test]
    fn an_unenforced_build_shows_the_email_and_offers_no_upgrade() {
        let token = include_str!("../../account/src/samples/token-valid.txt");
        let standing = Standing::of(&signed_in(token), Some(&"b".repeat(64)), NOW, false);
        assert_eq!(standing.email.as_deref(), Some("sample@example.com"));
        assert!(!standing.offers_upgrade());
        assert_eq!(standing.lock_for("weather"), None);
    }
}
