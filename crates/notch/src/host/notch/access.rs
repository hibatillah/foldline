//! What the account allows: pages and activities it locks, and which of Windows' popups the notch may keep hidden.
//! Re-evaluated at start-up, on resume, after `WM_SETTINGS_CHANGED`, and on open, close or unpin once a trial or launch has ended.

use std::path::PathBuf;

use account::{
    access::Access,
    catalog::is_known,
    device::device_hash,
    grants::Grants,
    payload::{read_policy, read_token},
    status::Status,
    stored::{StoredAccount, account_path},
};

use super::{Notch, pages::ACTIVITY_RESOURCE};
use crate::{
    host::{notification_banners, volume_flyout},
    log,
};

const PAGE_ACTION: &str = "page";
/// Debug builds only: forces the status (the grants stay built in).
const FORCED_STATUS_VARIABLE: &str = "NOTCH_ACCOUNT_STATUS";

/// The access now, and the moment it can change without any file changing (a trial or free launch ending).
pub(in crate::host) struct Evaluation {
    pub(in crate::host) access: Access,
    pub(in crate::host) changes_at: Option<i64>,
}

pub(in crate::host) trait AccessSource {
    /// Reads the stored account again, after another program may have written it.
    fn reload(&mut self);
    fn evaluate(&mut self, now_unix: i64) -> Evaluation;
}

pub(super) struct AccessState {
    pub(super) access: Access,
    pub(super) changes_at: Option<i64>,
    pub(super) source: Box<dyn AccessSource>,
}

impl AccessState {
    pub(super) fn new(mut source: Box<dyn AccessSource>, now_unix: i64) -> Self {
        let Evaluation { access, changes_at } = source.evaluate(now_unix);
        log::debug(&format!("account status: {:?}", access.status));
        Self { access, changes_at, source }
    }

    fn is_due(&self, now_unix: i64) -> bool {
        self.changes_at.is_some_and(|changes_at| now_unix >= changes_at)
    }

    fn popup_permits(&self) -> PopupPermits {
        PopupPermits::of(&self.access)
    }
}

/// `account.txt`, read once and again only when asked; the device hash is read once.
pub(in crate::host) struct AccountFile {
    path: PathBuf,
    device_hash: String,
    account: StoredAccount,
    forced_status: Option<Status>,
}

impl AccountFile {
    pub(in crate::host) fn open() -> Self {
        let path = account_path();
        let forced_status = if cfg!(debug_assertions) {
            parse_forced_status(std::env::var(FORCED_STATUS_VARIABLE).ok().as_deref())
        } else {
            None
        };
        // Without a hash no token can match this PC, which reads as a guest wherever signatures are enforced.
        let device_hash = device_hash().unwrap_or_default();
        let account = StoredAccount::load(&path);
        Self { path, device_hash, account, forced_status }
    }

    pub(in crate::host) fn device_hash(&self) -> &str {
        &self.device_hash
    }
}

impl AccessSource for AccountFile {
    fn reload(&mut self) {
        self.account = StoredAccount::load(&self.path);
    }

    fn evaluate(&mut self, now_unix: i64) -> Evaluation {
        if let Some(status) = self.forced_status {
            return Evaluation { access: Access { status, grants: Grants::built_in() }, changes_at: None };
        }
        let access = self.account.access(&self.device_hash, now_unix);
        let changes_at = next_change(&self.account, &self.device_hash, now_unix);
        Evaluation { access, changes_at }
    }
}

/// The earliest future trial or launch end; past ones have already been applied.
fn next_change(account: &StoredAccount, device_hash: &str, now_unix: i64) -> Option<i64> {
    let trial_ends_at = account.token.as_deref().and_then(|text| read_token(text, device_hash)?.trial_ends_at);
    let launch_ends_at = account
        .policy
        .as_deref()
        .and_then(read_policy)
        .filter(|policy| policy.launch.active)
        .and_then(|policy| policy.launch.ends_at);
    [trial_ends_at, launch_ends_at].into_iter().flatten().filter(|moment| *moment > now_unix).min()
}

pub(super) use account::status::parse_forced_status;

/// An id the catalog does not name is never locked; a defensive fallback the catalog test makes unreachable.
pub(super) fn locks_page(access: &Access, page_id: &str) -> bool {
    is_known(page_id, PAGE_ACTION) && !access.allows_page(page_id)
}

pub(super) fn locks_activity(access: &Access, activity_id: &str) -> bool {
    is_known(ACTIVITY_RESOURCE, activity_id) && !access.allows_activity(activity_id)
}

/// Which of Windows' popups the account lets the notch keep hidden.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PopupPermits {
    pub(super) flyout: bool,
    pub(super) banners: bool,
}

impl PopupPermits {
    pub(super) fn of(access: &Access) -> Self {
        Self { flyout: access.may_hide_volume_flyout(), banners: access.may_hide_notification_banners() }
    }
}

/// The `set_hidden` call one popup needs after its permit changed; `None` leaves it, since repeating a restore rewrites it.
pub(super) fn popup_call(was_permitted: bool, is_permitted: bool, wants_hidden: bool, is_hidden: bool) -> Option<bool> {
    if was_permitted == is_permitted {
        return None;
    }
    let should_hide = is_permitted && wants_hidden;
    (should_hide != is_hidden).then_some(should_hide)
}

impl Notch {
    /// After `WM_SETTINGS_CHANGED` or a resume: `account.txt` may have changed.
    pub(super) fn reload_access(&mut self) {
        self.access.source.reload();
        self.reevaluate_access(store::pause::now_unix());
    }

    /// On resume: a trial may have ended while asleep, or another program may have written `account.txt`.
    pub fn account_may_have_changed(&mut self) {
        self.reload_access();
        self.flush();
    }

    /// When the notch opens or closes and when the pin is released: a time comparison unless a trial or launch has just ended.
    pub(super) fn reevaluate_access_if_due(&mut self, now_unix: i64) {
        if self.access.is_due(now_unix) {
            self.reevaluate_access(now_unix);
        }
    }

    pub(super) fn reevaluate_access(&mut self, now_unix: i64) {
        let Evaluation { access, changes_at } = self.access.source.evaluate(now_unix);
        self.access.changes_at = changes_at;
        let is_unchanged = access.status == self.access.access.status && access.grants == self.access.access.grants;
        if is_unchanged {
            return;
        }
        log::debug(&format!("account status: {:?}", access.status));
        let permits_before = self.access.popup_permits();
        self.access.access = access;
        self.apply_access(permits_before);
    }

    pub(super) fn page_locks(&self) -> Vec<bool> {
        self.pages.iter().map(|page| locks_page(&self.access.access, page.id())).collect()
    }

    /// Marks locked activities; an unlocked one the user has on starts now.
    pub(super) fn lock_activities(&mut self) {
        for index in 0..self.activities.len() {
            let slot = &mut self.activities[index];
            slot.is_locked = locks_activity(&self.access.access, slot.activity.id());
            if slot.is_enabled {
                self.start_activity(index);
            }
        }
    }

    fn apply_access(&mut self, permits_before: PopupPermits) {
        let navigable_before = self.navigable.clone();
        let locks = self.page_locks();
        self.change_page_list(|page_list| page_list.set_locked(locks));
        self.lock_activities();
        // While paused the popups belong to Windows; the resume applies the permits.
        if !self.is_paused() {
            self.apply_popup_permits(permits_before);
        }
        for index in 0..self.pages.len() {
            self.with_page(index, |page, context| page.on_access_changed(context));
        }
        self.keep_shown_page_in_place(&navigable_before);
        self.pending.repaint = true;
    }

    fn apply_popup_permits(&mut self, before: PopupPermits) {
        let after = self.access.popup_permits();
        let wants_flyout_hidden = volume_flyout::wants_hidden(&self.settings);
        if let Some(is_hidden) =
            popup_call(before.flyout, after.flyout, wants_flyout_hidden, volume_flyout::is_hidden())
        {
            volume_flyout::set_hidden(is_hidden);
        }
        let wants_banners_hidden = notification_banners::wants_hidden(&self.settings);
        let banners_hidden = notification_banners::is_hidden();
        if let Some(is_hidden) = popup_call(before.banners, after.banners, wants_banners_hidden, banners_hidden) {
            notification_banners::set_hidden(is_hidden);
        }
    }

    /// The flyout may be hidden only while the account allows both of its activities.
    pub(super) fn may_hide_flyout(&self) -> bool {
        self.access.popup_permits().flyout
    }

    pub(super) fn may_hide_banners(&self) -> bool {
        self.access.popup_permits().banners
    }
}
