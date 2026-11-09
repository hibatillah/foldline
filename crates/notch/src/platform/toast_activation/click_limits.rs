//! The limits on one Open click: a deadline its waits share, and at most one activator thread per app at a time.

use std::{
    sync::{Mutex, PoisonError},
    time::{Duration, Instant},
};

/// Activator threads left running at once over all apps; a stuck one ends only when its app answers or COM gives up.
const MAX_OUTSTANDING_ACTIVATORS: usize = 3;

/// The end of one click's time; the routes wait in turn but never past it.
#[derive(Clone, Copy, Debug)]
pub struct ClickDeadline {
    ends_at: Instant,
}

impl ClickDeadline {
    pub fn new(started: Instant, limit: Duration) -> Self {
        Self { ends_at: started + limit }
    }

    pub fn remaining(&self, now: Instant) -> Duration {
        self.ends_at.saturating_duration_since(now)
    }

    /// When a wait of `limit` that starts `now` must end.
    pub fn wait_until(&self, now: Instant, limit: Duration) -> Instant {
        (now + limit).min(self.ends_at)
    }

    /// Starting the app, the last way, is tried even after the deadline: it is not waited for.
    pub fn allows(&self, now: Instant, is_last_route: bool) -> bool {
        is_last_route || !self.remaining(now).is_zero()
    }
}

/// The apps whose activator thread is still waiting on a call.
pub struct OutstandingActivators {
    apps: Mutex<Vec<String>>,
}

impl OutstandingActivators {
    pub const fn new() -> Self {
        Self { apps: Mutex::new(Vec::new()) }
    }

    /// A claim while no activator thread of `app_user_model_id` runs and fewer than the cap run at all.
    pub fn claim(&self, app_user_model_id: &str) -> Option<ActivatorClaim<'_>> {
        let mut apps = self.apps.lock().unwrap_or_else(PoisonError::into_inner);
        let is_running = apps.iter().any(|app| app.eq_ignore_ascii_case(app_user_model_id));
        if is_running || apps.len() >= MAX_OUTSTANDING_ACTIVATORS {
            return None;
        }
        apps.push(app_user_model_id.to_owned());
        Some(ActivatorClaim { owner: self, app_user_model_id: app_user_model_id.to_owned() })
    }

    #[cfg(test)]
    fn count(&self) -> usize {
        self.apps.lock().unwrap_or_else(PoisonError::into_inner).len()
    }
}

/// Held by an activator thread; the app may start another one once it drops.
pub struct ActivatorClaim<'a> {
    owner: &'a OutstandingActivators,
    app_user_model_id: String,
}

impl Drop for ActivatorClaim<'_> {
    fn drop(&mut self) {
        let mut apps = self.owner.apps.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(position) = apps.iter().position(|app| *app == self.app_user_model_id) {
            apps.swap_remove(position);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECOND: Duration = Duration::from_secs(1);

    #[test]
    fn a_route_wait_ends_at_the_click_deadline_when_that_comes_first() {
        let start = Instant::now();
        let deadline = ClickDeadline::new(start, 5 * SECOND);
        assert_eq!(deadline.wait_until(start + 4 * SECOND, 3 * SECOND), start + 5 * SECOND);
    }

    #[test]
    fn a_route_wait_keeps_its_own_limit_while_the_click_has_more_time() {
        let start = Instant::now();
        let deadline = ClickDeadline::new(start, 5 * SECOND);
        assert_eq!(deadline.wait_until(start + SECOND, 3 * SECOND), start + 4 * SECOND);
    }

    #[test]
    fn no_time_remains_once_the_click_deadline_passed() {
        let start = Instant::now();
        let deadline = ClickDeadline::new(start, 5 * SECOND);
        assert_eq!(deadline.remaining(start + 6 * SECOND), Duration::ZERO);
    }

    #[test]
    fn after_the_deadline_only_starting_the_app_is_still_tried() {
        let start = Instant::now();
        let deadline = ClickDeadline::new(start, 5 * SECOND);
        let after = start + 5 * SECOND;
        assert_eq!((deadline.allows(after, false), deadline.allows(after, true)), (false, true));
    }

    #[test]
    fn every_route_is_tried_before_the_deadline() {
        let start = Instant::now();
        let deadline = ClickDeadline::new(start, 5 * SECOND);
        assert!(deadline.allows(start + SECOND, false));
    }

    #[test]
    fn a_second_click_gets_no_activator_thread_while_the_first_for_that_app_is_stuck() {
        let outstanding = OutstandingActivators::new();
        let _first = outstanding.claim("Discord");
        assert!(outstanding.claim("discord").is_none());
    }

    #[test]
    fn an_app_gets_an_activator_thread_again_once_its_stuck_one_ended() {
        let outstanding = OutstandingActivators::new();
        drop(outstanding.claim("Discord"));
        assert!(outstanding.claim("Discord").is_some());
    }

    #[test]
    fn another_app_gets_an_activator_thread_while_one_app_is_stuck() {
        let outstanding = OutstandingActivators::new();
        let _first = outstanding.claim("Discord");
        assert!(outstanding.claim("WhatsApp").is_some());
    }

    #[test]
    fn stuck_activator_threads_stop_accumulating_at_the_cap() {
        let outstanding = OutstandingActivators::new();
        let claims: Vec<_> = ["a", "b", "c", "d", "e"].iter().filter_map(|app| outstanding.claim(app)).collect();
        assert_eq!((claims.len(), outstanding.count()), (MAX_OUTSTANDING_ACTIVATORS, MAX_OUTSTANDING_ACTIVATORS));
    }
}
