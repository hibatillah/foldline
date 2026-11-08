//! What the page shows besides the list: a confirmation on the selected row, the bar at the bottom, the picture preview.

use super::capture::Skip;
use crate::gfx::ImageBrush;

/// The selected row and its button turn green for a moment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Confirm {
    Pasted,
    Copied,
}

impl Confirm {
    pub fn row_text(self) -> &'static str {
        match self {
            Confirm::Pasted => "Pasted",
            Confirm::Copied => "Copied, moved to front",
        }
    }
}

/// One bar at a time covers the bottom of the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Bar {
    Removed,
    Skipped(Skip),
}

impl Bar {
    pub fn text(self) -> &'static str {
        match self {
            Bar::Removed => "Removed",
            Bar::Skipped(Skip::Excluded) => "Skipped: password manager",
            Bar::Skipped(Skip::TooLarge) => "Skipped: too large",
        }
    }

    pub fn has_undo(self) -> bool {
        self == Bar::Removed
    }
}

/// The uploaded copy of the selected picture; only one is held, so memory does not grow with the history.
#[derive(Default)]
pub(super) struct Preview {
    pub item_id: u64,
    pub width: u32,
    pub brush: Option<ImageBrush>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_removed_bar_offers_undo() {
        assert!(Bar::Removed.has_undo());
        assert!(!Bar::Skipped(Skip::Excluded).has_undo());
    }

    #[test]
    fn the_bars_and_confirmations_use_the_designed_words() {
        assert_eq!(Bar::Skipped(Skip::Excluded).text(), "Skipped: password manager");
        assert_eq!(Bar::Removed.text(), "Removed");
        assert_eq!(Confirm::Copied.row_text(), "Copied, moved to front");
        assert_eq!(Confirm::Pasted.row_text(), "Pasted");
    }
}
