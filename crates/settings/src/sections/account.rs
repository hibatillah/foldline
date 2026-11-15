//! The Account page: sign in with Google through the browser, the account and its status, and sign out.

use std::rc::Rc;

use account::status::Status;
use gpui::{AnyElement, App, IntoElement, SharedString, Window, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, IconName, spinner::Spinner};

use super::about::widgets::{Handler, pill};
use crate::{
    account_refresh,
    i18n::tr,
    layout::{Group, Item, Page, control_row},
    sign_in::{Phase, SignInFlow},
    standing::{self, Standing},
};

/// The site page a button opens, after the site's address.
fn site_page(path: &str) -> String {
    format!("{}{path}", account::client::site_url())
}

fn open_site_page(path: &'static str) -> Handler {
    Rc::new(move |_window, cx| cx.open_url(&site_page(path)))
}

/// Opens the site's pricing page, then checks the account, so a purchase shows when the user comes back.
pub fn open_pricing(cx: &mut App) {
    cx.open_url(&site_page("/pricing"));
    account_refresh::refresh_if_due(cx);
}

/// Only a build that enforces signatures talks to the site; any other has no accounts to sign in to yet.
fn account_page_offers_sign_in(is_enforced: bool) -> bool {
    is_enforced
}

/// The status under the email; a build that locks nothing shows none.
fn status_line(is_enforced: bool, status: Status, trial_ends_at: Option<i64>) -> Option<String> {
    if !is_enforced {
        return None;
    }
    Some(match (status, trial_ends_at) {
        (Status::Guest, _) => tr!("account.status_inactive").to_owned(),
        (Status::Member, _) | (Status::Trial, None) => tr!("account.status_free").to_owned(),
        (Status::Trial, Some(ends_at)) => tr!("account.status_trial", date = iso_date(ends_at)),
        (Status::Paid, _) => tr!("account.status_paid").to_owned(),
        (Status::Launch, _) => tr!("account.status_launch").to_owned(),
    })
}

/// `YYYY-MM-DD` of a Unix time in UTC.
fn iso_date(unix: i64) -> String {
    let days = unix.div_euclid(86_400);
    // Howard Hinnant's civil_from_days.
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

fn signed_in_row(standing: &Standing, message: Option<&'static str>, cx: &App) -> AnyElement {
    let title = standing.email.clone().unwrap_or_else(|| tr!("account.signed_in").to_owned());
    let lines = status_line(standing.is_enforced, standing.status, standing.trial_ends_at)
        .into_iter()
        .map(SharedString::from)
        .chain(message.map(SharedString::from));
    let sign_out: Handler = Rc::new(|_window, cx| SignInFlow::sign_out(cx));
    let control = pill("sign-out", tr!("account.sign_out"), None, Some(sign_out), cx);
    control_row(title.into(), lines, control, false, cx)
}

fn signed_out_row(phase: Phase, message: Option<&'static str>, cx: &App) -> AnyElement {
    let (title, note, control) = match phase {
        Phase::Idle => {
            let start: Handler = Rc::new(|_window, cx| SignInFlow::start(cx));
            let button = pill("sign-in", tr!("account.sign_in_google"), Some(IconName::ExternalLink), Some(start), cx);
            (tr!("account.signed_out_title"), Some(tr!("account.signed_out_note")), button)
        }
        Phase::Waiting => {
            let cancel: Handler = Rc::new(|_window, cx| SignInFlow::cancel(cx));
            let button = pill("cancel", tr!("account.cancel"), None, Some(cancel), cx);
            (tr!("account.waiting_title"), Some(tr!("account.waiting_note")), button)
        }
        Phase::Finishing => {
            let spinner = Spinner::new().color(crate::palette::accent(cx)).into_any_element();
            (tr!("account.finishing"), Some(tr!("account.finishing_note")), spinner)
        }
    };
    let lines = note.into_iter().chain(message).map(SharedString::from);
    control_row(title.into(), lines, control, false, cx)
}

/// The one line a build without accounts shows; an account stored by an earlier build can still be signed out here.
fn unopened_row(standing: &Standing, message: Option<&'static str>, cx: &App) -> AnyElement {
    if standing.signed_in {
        return signed_in_row(standing, message, cx);
    }
    control_row(
        tr!("account.opening_soon").into(),
        message.map(SharedString::from),
        div().into_any_element(),
        false,
        cx,
    )
}

fn account_row(_window: &mut Window, cx: &mut App) -> AnyElement {
    let standing = standing::current();
    let message = SignInFlow::message(cx);
    if !account_page_offers_sign_in(standing.is_enforced) {
        return unopened_row(&standing, message, cx);
    }
    if standing.signed_in {
        signed_in_row(&standing, message, cx)
    } else {
        signed_out_row(SignInFlow::phase(cx), message, cx)
    }
}

fn site_button(id: &'static str, label: &'static str, path: &'static str) -> crate::layout::Render {
    Rc::new(move |_window, cx| pill(id, label, Some(IconName::ExternalLink), Some(open_site_page(path)), cx))
}

fn upgrade_button() -> crate::layout::Render {
    Rc::new(|_window, cx| {
        let open: Handler = Rc::new(|_window, cx| open_pricing(cx));
        pill("upgrade", tr!("account.upgrade"), Some(IconName::ExternalLink), Some(open), cx)
    })
}

pub fn page() -> Page {
    let upgrade = Item::new(tr!("account.upgrade"), upgrade_button())
        .description(tr!("account.upgrade_note"))
        .visible_when(|| standing::current().offers_upgrade());
    let manage = Item::new(tr!("account.manage"), site_button("manage", tr!("about.open"), "/account"))
        .description(tr!("account.manage_note"))
        .visible_when(|| {
            let standing = standing::current();
            standing.signed_in && account_page_offers_sign_in(standing.is_enforced)
        });
    let note = Item::custom(|_window, cx| {
        div()
            .px(px(4.0))
            .text_sm()
            .text_color(cx.theme().muted_foreground)
            .child(account_description(account::keys::is_enforced()))
            .into_any_element()
    })
    .outside_block();
    Page::plain(
        tr!("account.title"),
        vec![Group::titled(tr!("account.title"), vec![Item::custom(account_row), upgrade, manage, note])],
    )
}

/// What the account is for, under its block: signing in where accounts are open, and that none is needed otherwise.
fn account_description(is_enforced: bool) -> &'static str {
    if account_page_offers_sign_in(is_enforced) { tr!("account.description") } else { tr!("account.description_soon") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_running_trial_shows_its_end_date() {
        let ends_at = 1_799_971_200; // 2027-01-15T00:00:00Z
        assert_eq!(status_line(true, Status::Trial, Some(ends_at)).as_deref(), Some("Trial until 2027-01-15"));
    }

    #[test]
    fn the_launch_reads_free_during_launch() {
        assert_eq!(status_line(true, Status::Launch, None).as_deref(), Some("Free during launch"));
    }

    #[test]
    fn a_build_that_locks_nothing_shows_no_status() {
        assert_eq!(status_line(false, Status::Launch, None), None);
    }

    #[test]
    fn dates_are_read_in_utc_across_leap_days_and_the_epoch() {
        assert_eq!(iso_date(0), "1970-01-01");
        assert_eq!(iso_date(1_709_208_000), "2024-02-29");
        assert_eq!(iso_date(1_767_225_599), "2025-12-31");
    }

    #[test]
    fn the_buttons_open_the_pricing_and_account_pages_of_the_site() {
        assert!(site_page("/pricing").ends_with("/pricing"));
        assert_eq!(site_page("/account"), format!("{}/account", account::client::site_url()));
    }

    #[test]
    fn an_unenforced_build_offers_no_sign_in() {
        assert!(!account_page_offers_sign_in(false));
        assert!(account_page_offers_sign_in(true));
    }

    #[test]
    fn the_note_says_whether_accounts_are_open() {
        assert_eq!(account_description(true), tr!("account.description"));
        assert_eq!(account_description(false), tr!("account.description_soon"));
    }
}
