//! The app icons of notifications: a found icon is kept for the session, a failed lookup is tried again later.

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::gfx::ImageData;

/// How long after a failed lookup the next notification of that app may try again.
pub const RETRY_AFTER: Duration = Duration::from_secs(30);
/// Lookups per app in all; after that the app stays on the fallback tile.
pub const MAX_ATTEMPTS: u32 = 5;

enum Entry {
    Found(Arc<ImageData>),
    Missing { attempts: u32, last_attempt: Instant },
}

#[derive(Default)]
pub struct LogoCache {
    entries: HashMap<String, Entry>,
}

impl LogoCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// The cached icon, else the result of `load` when the app has no attempt yet or the last failed one is old enough.
    pub fn get(
        &mut self,
        app_id: &str,
        now: Instant,
        load: impl FnOnce() -> Option<Arc<ImageData>>,
    ) -> Option<Arc<ImageData>> {
        let attempts = match self.entries.get(app_id) {
            Some(Entry::Found(image)) => return Some(image.clone()),
            Some(Entry::Missing { attempts, last_attempt }) => {
                if *attempts >= MAX_ATTEMPTS || now.saturating_duration_since(*last_attempt) < RETRY_AFTER {
                    return None;
                }
                *attempts
            }
            None => 0,
        };
        let entry = match load() {
            Some(image) => Entry::Found(image),
            None => Entry::Missing { attempts: attempts + 1, last_attempt: now },
        };
        let icon = match &entry {
            Entry::Found(image) => Some(image.clone()),
            Entry::Missing { .. } => None,
        };
        self.entries.insert(app_id.to_owned(), entry);
        icon
    }

    /// How many lookups failed for the app so far.
    pub fn failed_attempts(&self, app_id: &str) -> u32 {
        match self.entries.get(app_id) {
            Some(Entry::Missing { attempts, .. }) => *attempts,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image() -> Option<Arc<ImageData>> {
        Some(Arc::new(ImageData::new(1, 1, vec![0; 4])))
    }

    /// Runs `get` and says whether the loader was called.
    fn lookup(
        cache: &mut LogoCache,
        app_id: &str,
        now: Instant,
        result: Option<Arc<ImageData>>,
    ) -> (Option<Arc<ImageData>>, bool) {
        let mut called = false;
        let icon = cache.get(app_id, now, || {
            called = true;
            result
        });
        (icon, called)
    }

    #[test]
    fn a_failure_is_retried_after_30_seconds_but_not_before() {
        let start = Instant::now();
        let mut cache = LogoCache::new();
        assert!(lookup(&mut cache, "app", start, None).1);
        assert!(!lookup(&mut cache, "app", start + Duration::from_secs(29), None).1);
        assert!(lookup(&mut cache, "app", start + Duration::from_secs(30), None).1);
    }

    #[test]
    fn a_success_is_never_reloaded() {
        let start = Instant::now();
        let mut cache = LogoCache::new();
        assert!(lookup(&mut cache, "app", start, image()).0.is_some());
        let (icon, called) = lookup(&mut cache, "app", start + Duration::from_secs(3600), None);
        assert!(icon.is_some());
        assert!(!called);
    }

    #[test]
    fn the_sixth_attempt_is_not_made() {
        let start = Instant::now();
        let mut cache = LogoCache::new();
        for attempt in 0..MAX_ATTEMPTS {
            let now = start + RETRY_AFTER * attempt;
            assert!(lookup(&mut cache, "app", now, None).1, "attempt {attempt}");
        }
        assert_eq!(cache.failed_attempts("app"), MAX_ATTEMPTS);
        assert!(!lookup(&mut cache, "app", start + RETRY_AFTER * MAX_ATTEMPTS, image()).1);
    }

    #[test]
    fn a_retry_that_succeeds_is_cached() {
        let start = Instant::now();
        let mut cache = LogoCache::new();
        lookup(&mut cache, "app", start, None);
        let later = start + RETRY_AFTER;
        assert!(lookup(&mut cache, "app", later, image()).0.is_some());
        let (icon, called) = lookup(&mut cache, "app", later + RETRY_AFTER, None);
        assert!(icon.is_some());
        assert!(!called);
        assert_eq!(cache.failed_attempts("app"), 0);
    }

    #[test]
    fn each_app_has_its_own_state() {
        let start = Instant::now();
        let mut cache = LogoCache::new();
        lookup(&mut cache, "first", start, None);
        assert!(lookup(&mut cache, "second", start, image()).0.is_some());
        assert!(!lookup(&mut cache, "first", start + Duration::from_secs(1), None).1);
        assert_eq!(cache.failed_attempts("first"), 1);
        assert_eq!(cache.failed_attempts("second"), 0);
    }
}
