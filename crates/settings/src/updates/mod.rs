//! Checking GitHub for a newer release: the state the About page shows and the one background run that changes it.

mod fetch;
mod version_order;

use std::sync::Mutex;

use gpui::App;

use crate::version::displayed_version;

/// What the About page shows about the last check.
#[derive(Clone, Debug, PartialEq)]
pub enum CheckState {
    Idle,
    Checking,
    UpToDate,
    /// The newer release's version, without its `v`.
    Available {
        version: String,
    },
    Failed,
}

static STATE: Mutex<CheckState> = Mutex::new(CheckState::Idle);

fn lock_state() -> std::sync::MutexGuard<'static, CheckState> {
    STATE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The state now; cheap enough to call on every paint.
pub fn state() -> CheckState {
    lock_state().clone()
}

/// Marks a check as running; `false` when one already runs.
fn begin(state: &mut CheckState) -> bool {
    if *state == CheckState::Checking {
        return false;
    }
    *state = CheckState::Checking;
    true
}

/// Turns the outcome of a request into the state to show.
fn decide(outcome: Result<String, fetch::Failure>, current_version: &str) -> CheckState {
    match outcome {
        Err(_) => CheckState::Failed,
        // A tag or a running version that cannot be read must not claim "up to date".
        Ok(tag) if !version_order::is_readable(&tag) || !version_order::is_readable(current_version) => {
            CheckState::Failed
        }
        Ok(tag) if version_order::is_newer(&tag, current_version) => {
            CheckState::Available { version: tag.trim_start_matches('v').to_owned() }
        }
        Ok(_) => CheckState::UpToDate,
    }
}

/// Starts one check on a background thread and repaints when it ends; does nothing while a check runs.
pub fn start(cx: &mut App) {
    if !begin(&mut lock_state()) {
        return;
    }
    // Repaint now so the card shows "Checking..." at once.
    cx.refresh_windows();
    cx.spawn(async move |cx| {
        let outcome = cx.background_executor().spawn(async { fetch::latest_tag() }).await;
        if let Err(reason) = &outcome {
            eprintln!("update check failed: {reason:?}");
        }
        *lock_state() = decide(outcome, displayed_version());
        let _ = cx.update(|cx| cx.refresh_windows());
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn available(version: &str) -> CheckState {
        CheckState::Available { version: version.to_owned() }
    }

    #[test]
    fn begin_refuses_a_second_check_while_one_runs() {
        let mut state = CheckState::Idle;
        assert!(begin(&mut state));
        assert_eq!(state, CheckState::Checking);
        assert!(!begin(&mut state));
        state = CheckState::Failed;
        assert!(begin(&mut state));
    }

    #[test]
    fn a_newer_tag_is_available_without_its_v() {
        assert_eq!(decide(Ok("v0.7.0".into()), "0.6.0"), available("0.7.0"));
    }

    #[test]
    fn the_same_tag_is_up_to_date() {
        assert_eq!(decide(Ok("v0.6.0".into()), "0.6.0"), CheckState::UpToDate);
    }

    #[test]
    fn a_dev_build_sees_the_stable_release() {
        assert_eq!(decide(Ok("v0.6.0".into()), "0.6.0-alpha.2"), available("0.6.0"));
    }

    #[test]
    fn any_failure_is_failed_not_up_to_date() {
        for failure in
            [fetch::Failure::Network("x".into()), fetch::Failure::Service("x".into()), fetch::Failure::Unreadable]
        {
            assert_eq!(decide(Err(failure.clone()), "0.6.0"), CheckState::Failed, "{failure:?}");
        }
    }

    #[test]
    fn a_malformed_tag_is_failed_not_up_to_date() {
        for tag in ["latest", "v1.2", "v1.2.3.4", ""] {
            assert_eq!(decide(Ok(tag.into()), "0.6.0"), CheckState::Failed, "tag: {tag:?}");
        }
    }

    #[test]
    fn an_unreadable_current_version_is_failed() {
        assert_eq!(decide(Ok("v0.7.0".into()), "dev"), CheckState::Failed);
    }

    #[test]
    fn a_tag_with_a_suffix_is_up_to_date_not_failed() {
        assert_eq!(decide(Ok("v0.7.0-rc.1".into()), "0.6.0"), CheckState::UpToDate);
    }
}
