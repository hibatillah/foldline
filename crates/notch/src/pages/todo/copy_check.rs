//! The green check that replaces a row's copy icon for a moment after a copy.

/// How long the check stays, as on the clipboard page's Copy button.
pub(super) const CHECK_MILLISECONDS: u32 = 1500;

/// The task whose copy button shows the check, by its place in the list; there is one at most.
///
/// An index names a task for as long as the list does not shift: a toggle only changes where the row is shown, while a
/// removal, a clear and an undo move indexes, so each of them calls `clear`.
#[derive(Debug, Default)]
pub(super) struct CopyCheck {
    index: Option<usize>,
}

impl CopyCheck {
    /// Moves the check to the task just copied; the caller restarts the timer for every call.
    pub(super) fn start(&mut self, index: usize) {
        self.index = Some(index);
    }

    pub(super) fn shows(&self, index: usize) -> bool {
        self.index == Some(index)
    }

    /// Forgets the check; returns whether one was showing, so the caller stops the timer and repaints only then.
    pub(super) fn clear(&mut self) -> bool {
        self.index.take().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_check_shows_on_the_copied_task_only_and_clears_once() {
        let mut check = CopyCheck::default();
        assert!(!check.shows(0));
        check.start(2);
        assert!(check.shows(2) && !check.shows(1));
        assert!(check.clear());
        assert!(!check.shows(2));
        assert!(!check.clear(), "nothing was showing, so there is no timer to stop");
    }

    #[test]
    fn a_second_copy_moves_the_check_to_the_new_task() {
        let mut check = CopyCheck::default();
        check.start(0);
        check.start(3);
        assert!(check.shows(3) && !check.shows(0));
    }
}
