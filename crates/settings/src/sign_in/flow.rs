//! The window's one sign-in at a time: open the browser, wait for its callback off the UI thread, claim the code,
//! save, and tell the notch.

use std::time::Duration;

use account::{client, keys, pkce::Pkce, stored::account_path};
use gpui::{App, Global};

use super::{Completion, Listener, LoginRequest, Stopper, Waited, complete, login_url, sign_out};
use crate::{i18n::tr, notify::tell_notch_to_reload, standing};

/// The browser has five minutes to come back.
const WAIT_LIMIT: Duration = Duration::from_secs(300);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Idle,
    /// The browser is open on the site's login page.
    Waiting,
    /// The code came back and is being claimed.
    Finishing,
}

/// Kept as a global, so leaving the Account page and coming back keeps a sign-in that is under way.
#[derive(Default)]
pub struct SignInFlow {
    phase: Phase,
    /// Counts sign-ins, so a late answer from a cancelled one changes nothing.
    attempt: u64,
    stopper: Option<Stopper>,
    message: Option<&'static str>,
}

impl Global for SignInFlow {}

impl SignInFlow {
    pub fn phase(cx: &App) -> Phase {
        cx.try_global::<SignInFlow>().map_or(Phase::Idle, |flow| flow.phase)
    }

    /// The line under the account row after a sign-in that did not finish.
    pub fn message(cx: &App) -> Option<&'static str> {
        cx.try_global::<SignInFlow>().and_then(|flow| flow.message)
    }

    /// Does nothing in a build that does not enforce signatures: it has no site to sign in to yet.
    pub fn start(cx: &mut App) {
        if !keys::is_enforced() || Self::phase(cx) != Phase::Idle {
            return;
        }
        let Some(device_hash) = standing::device_hash() else {
            return Self::show(tr!("account.no_device"), cx);
        };
        let (Some(pkce), Ok((listener, stopper))) = (Pkce::new(), Listener::open()) else {
            return Self::show(tr!("account.no_listener"), cx);
        };
        let request = LoginRequest {
            port: stopper.port(),
            challenge: &pkce.challenge,
            state: &pkce.state,
            device_hash: &device_hash,
            device_name: &account::device::device_name(),
        };
        let url = login_url(&client::site_url(), &request);
        let flow = cx.default_global::<SignInFlow>();
        flow.attempt += 1;
        flow.phase = Phase::Waiting;
        flow.stopper = Some(stopper.clone());
        flow.message = None;
        let attempt = flow.attempt;
        cx.open_url(&url);
        cx.refresh_windows();

        cx.spawn(async move |cx| {
            cx.background_executor().timer(WAIT_LIMIT).await;
            stopper.stop();
        })
        .detach();
        let page = tr!("account.browser_done");
        cx.spawn(async move |cx| {
            let Pkce { verifier, state, .. } = pkce;
            let waited = cx.background_executor().spawn(async move { listener.wait(&state, page) }).await;
            let callback = match waited {
                Waited::Answered(callback) => callback,
                // A Cancel click has already moved the attempt on, so only the time limit reaches the message.
                Waited::Stopped | Waited::Failed => {
                    let _ = cx.update(|cx| Self::end(attempt, Some(message_for_wait(&waited)), cx));
                    return;
                }
            };
            let still_wanted = cx.update(|cx| Self::finishing(attempt, cx)).unwrap_or(false);
            if !still_wanted {
                return;
            }
            let completion = cx
                .background_executor()
                .spawn(async move {
                    let now = standing::now_unix();
                    complete(
                        callback,
                        &verifier,
                        &device_hash,
                        &account_path(),
                        now,
                        keys::is_enforced(),
                        client::claim,
                    )
                })
                .await;
            let _ = cx.update(|cx| {
                if completion == Completion::SignedIn {
                    tell_notch_to_reload();
                }
                Self::end(attempt, message_for(completion), cx);
            });
        })
        .detach();
    }

    /// The Cancel button: the browser tab stays, but its answer is no longer awaited.
    pub fn cancel(cx: &mut App) {
        let flow = cx.default_global::<SignInFlow>();
        if flow.phase != Phase::Waiting {
            return;
        }
        flow.attempt += 1;
        flow.phase = Phase::Idle;
        flow.message = None;
        if let Some(stopper) = flow.stopper.take() {
            stopper.stop();
        }
        cx.refresh_windows();
    }

    /// Forgets the device here first, then tells the site in the background, which is best effort; a build that does
    /// not enforce signatures only clears the file.
    pub fn sign_out(cx: &mut App) {
        match sign_out(&account_path()) {
            Ok(identity) => {
                tell_notch_to_reload();
                Self::show_none(cx);
                if let Some((device_id, device_secret)) = identity.filter(|_| keys::is_enforced()) {
                    cx.background_executor()
                        .spawn(async move { client::sign_out(&device_id, &device_secret) })
                        .detach();
                }
            }
            Err(_) => Self::show(tr!("account.not_saved"), cx),
        }
    }

    fn finishing(attempt: u64, cx: &mut App) -> bool {
        let flow = cx.default_global::<SignInFlow>();
        if flow.attempt != attempt || flow.phase != Phase::Waiting {
            return false;
        }
        flow.phase = Phase::Finishing;
        flow.stopper = None;
        cx.refresh_windows();
        true
    }

    fn end(attempt: u64, message: Option<&'static str>, cx: &mut App) {
        let flow = cx.default_global::<SignInFlow>();
        if flow.attempt != attempt {
            return;
        }
        flow.phase = Phase::Idle;
        flow.stopper = None;
        flow.message = message;
        cx.refresh_windows();
    }

    fn show(message: &'static str, cx: &mut App) {
        cx.default_global::<SignInFlow>().message = Some(message);
        cx.refresh_windows();
    }

    fn show_none(cx: &mut App) {
        cx.default_global::<SignInFlow>().message = None;
        cx.refresh_windows();
    }
}

/// The line shown when the wait for the browser ends without an answer.
fn message_for_wait(waited: &Waited) -> &'static str {
    match waited {
        Waited::Failed => tr!("account.listener_failed"),
        Waited::Stopped | Waited::Answered(_) => tr!("account.timed_out"),
    }
}

/// The line shown after a sign-in ends; a finished one needs none, since the row now shows the account.
fn message_for(completion: Completion) -> Option<&'static str> {
    match completion {
        Completion::SignedIn => None,
        Completion::Cancelled => Some(tr!("account.cancelled")),
        Completion::Refused => Some(tr!("account.refused")),
        Completion::InvalidCode => Some(tr!("account.invalid_code")),
        Completion::Unavailable => Some(tr!("account.unavailable")),
        Completion::NotVerified => Some(tr!("account.not_verified")),
        Completion::NotSaved => Some(tr!("account.not_saved")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_finished_sign_in_shows_no_message_and_a_cancelled_one_says_so() {
        assert_eq!(message_for(Completion::SignedIn), None);
        assert_eq!(message_for(Completion::Cancelled), Some("Sign-in cancelled."));
    }

    #[test]
    fn a_port_that_fails_is_not_reported_as_a_time_out() {
        assert_eq!(message_for_wait(&Waited::Stopped), tr!("account.timed_out"));
        assert_ne!(message_for_wait(&Waited::Failed), message_for_wait(&Waited::Stopped));
    }

    #[test]
    fn every_unfinished_sign_in_has_its_own_message() {
        let unfinished = [
            Completion::Cancelled,
            Completion::Refused,
            Completion::InvalidCode,
            Completion::Unavailable,
            Completion::NotVerified,
            Completion::NotSaved,
        ];
        let mut messages: Vec<&str> = unfinished.iter().filter_map(|completion| message_for(*completion)).collect();
        messages.sort_unstable();
        messages.dedup();
        assert_eq!(messages.len(), unfinished.len());
    }
}
