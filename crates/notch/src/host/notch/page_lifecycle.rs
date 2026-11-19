/// Which pages are started. A page that is switched off is never started and holds nothing; a page switched on is started
/// again from the settings, so it needs no state of its own across the two.
pub(super) struct PageLifecycle {
    started: Vec<bool>,
}

impl PageLifecycle {
    /// Every page counts as started until `Notch::start` decides which are on, so a test that never starts the notch still
    /// reaches its pages.
    pub(super) fn all_started(count: usize) -> Self {
        Self { started: vec![true; count] }
    }

    pub(super) fn is_started(&self, index: usize) -> bool {
        self.started.get(index).copied().unwrap_or(false)
    }

    pub(super) fn set_started(&mut self, index: usize, is_started: bool) {
        if let Some(slot) = self.started.get_mut(index) {
            *slot = is_started;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_page_counts_as_started_until_the_notch_decides() {
        let lifecycle = PageLifecycle::all_started(3);
        assert!((0..3).all(|index| lifecycle.is_started(index)));
    }

    #[test]
    fn a_page_that_is_stopped_stays_stopped_until_started_again() {
        let mut lifecycle = PageLifecycle::all_started(2);
        lifecycle.set_started(1, false);
        assert!(lifecycle.is_started(0));
        assert!(!lifecycle.is_started(1));
        lifecycle.set_started(1, true);
        assert!(lifecycle.is_started(1));
    }

    #[test]
    fn an_unknown_page_is_not_started() {
        let lifecycle = PageLifecycle::all_started(1);
        assert!(!lifecycle.is_started(5));
    }
}
