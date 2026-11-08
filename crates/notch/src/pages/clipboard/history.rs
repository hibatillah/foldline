//! The remembered copies, newest first, with the limits that keep them small, and the one removal that can be undone.

use std::time::Instant;

use super::item::{Content, Item, kind_of};

pub(super) const MAX_ITEMS: usize = 7;
pub(super) const MAX_TEXT_BYTES: usize = 100 * 1024;
/// About 3 MiB of pixels; a larger picture is shrunk before it is kept.
pub(super) const MAX_IMAGE_PIXELS: u64 = 786_432;
/// Text and pictures together; the oldest copies go first when a new one would pass it.
pub(super) const MAX_TOTAL_BYTES: usize = 12 * 1024 * 1024;

struct Removed {
    item: Item,
    index: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Pushed {
    Added,
    /// The same content was already in the list; it moved to the front.
    MovedToFront,
}

#[derive(Default)]
pub(super) struct History {
    items: Vec<Item>,
    selected: usize,
    removed: Option<Removed>,
    next_id: u64,
}

impl History {
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected(&self) -> Option<&Item> {
        self.items.get(self.selected)
    }

    pub fn can_undo(&self) -> bool {
        self.removed.is_some()
    }

    pub fn select(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected = index;
        }
    }

    /// Adds a copy at the front and selects it. The same content copied again moves instead of adding a second row.
    pub fn push(&mut self, content: Content, now: Instant) -> Pushed {
        if let Some(index) = self.items.iter().position(|item| item.content.is_same_as(&content)) {
            self.move_to_front(index, now);
            return Pushed::MovedToFront;
        }
        self.next_id += 1;
        let item = Item { id: self.next_id, kind: kind_of(&content), content, copied_at: now };
        self.items.insert(0, item);
        self.selected = 0;
        self.enforce_limits();
        Pushed::Added
    }

    /// Moves the item to the front, marks it as copied now and selects it.
    pub fn move_to_front(&mut self, index: usize, now: Instant) {
        if index >= self.items.len() {
            return;
        }
        let mut item = self.items.remove(index);
        item.copied_at = now;
        self.items.insert(0, item);
        self.selected = 0;
    }

    /// Takes the item out and keeps it for `undo`; an earlier removal that was not undone is dropped for good.
    pub fn remove(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }
        let item = self.items.remove(index);
        self.removed = Some(Removed { item, index });
        if self.selected > index {
            self.selected -= 1;
        }
        self.selected = self.selected.min(self.items.len().saturating_sub(1));
    }

    /// Puts the last removed item back where it was and selects it.
    pub fn undo(&mut self) -> bool {
        let Some(Removed { item, index }) = self.removed.take() else { return false };
        let index = index.min(self.items.len());
        self.items.insert(index, item);
        self.selected = index;
        self.enforce_limits();
        true
    }

    pub fn forget_removed(&mut self) {
        self.removed = None;
    }

    /// Drops everything, including what could be undone.
    pub fn clear(&mut self) {
        self.items = Vec::new();
        self.removed = None;
        self.selected = 0;
    }

    fn total_bytes(&self) -> usize {
        self.items.iter().map(|item| item.content.byte_size()).sum()
    }

    fn enforce_limits(&mut self) {
        let keep_selected = self.selected;
        self.items.truncate(MAX_ITEMS);
        while self.items.len() > 1 && self.total_bytes() > MAX_TOTAL_BYTES {
            self.items.pop();
        }
        self.selected = keep_selected.min(self.items.len().saturating_sub(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{gfx::ImageData, pages::clipboard::item::StoredImage};

    fn text(value: &str) -> Content {
        Content::Text(value.to_owned())
    }

    fn texts(history: &History) -> Vec<String> {
        history
            .items()
            .iter()
            .map(|item| match &item.content {
                Content::Text(text) => text.clone(),
                Content::Image(_) => "image".to_owned(),
            })
            .collect()
    }

    #[test]
    fn the_newest_copy_goes_on_top_and_is_selected() {
        let mut history = History::default();
        history.push(text("one"), Instant::now());
        history.push(text("two"), Instant::now());
        assert_eq!(texts(&history), ["two", "one"]);
        assert_eq!(history.selected_index(), 0);
    }

    #[test]
    fn only_the_last_seven_copies_are_kept() {
        let mut history = History::default();
        for number in 0..10 {
            history.push(text(&format!("copy {number}")), Instant::now());
        }
        assert_eq!(history.items().len(), MAX_ITEMS);
        assert_eq!(texts(&history)[0], "copy 9");
        assert_eq!(texts(&history)[6], "copy 3");
    }

    #[test]
    fn copying_the_same_text_again_moves_it_to_the_front_instead_of_adding_it() {
        let mut history = History::default();
        history.push(text("a"), Instant::now());
        history.push(text("b"), Instant::now());
        history.push(text("c"), Instant::now());
        assert_eq!(history.push(text("a"), Instant::now()), Pushed::MovedToFront);
        assert_eq!(texts(&history), ["a", "c", "b"]);
        assert_eq!(history.selected_index(), 0);
    }

    #[test]
    fn moving_an_item_to_the_front_refreshes_its_time_and_selects_it() {
        let mut history = History::default();
        let old = Instant::now();
        history.push(text("a"), old);
        history.push(text("b"), old);
        let later = old + std::time::Duration::from_secs(60);
        history.move_to_front(1, later);
        assert_eq!(texts(&history), ["a", "b"]);
        assert_eq!(history.items()[0].copied_at, later);
        assert_eq!(history.selected_index(), 0);
    }

    #[test]
    fn removing_closes_the_gap_and_undo_restores_the_place_and_selection() {
        let mut history = History::default();
        for value in ["a", "b", "c", "d"] {
            history.push(text(value), Instant::now());
        }
        history.remove(1);
        assert_eq!(texts(&history), ["d", "b", "a"]);
        assert!(history.can_undo());
        assert!(history.undo());
        assert_eq!(texts(&history), ["d", "c", "b", "a"]);
        assert_eq!(history.selected_index(), 1);
        assert!(!history.can_undo());
        assert!(!history.undo());
    }

    #[test]
    fn a_second_removal_replaces_what_the_first_could_have_undone() {
        let mut history = History::default();
        for value in ["a", "b", "c"] {
            history.push(text(value), Instant::now());
        }
        history.remove(0);
        history.remove(0);
        assert!(history.undo());
        assert_eq!(texts(&history), ["b", "a"]);
    }

    #[test]
    fn removing_the_last_item_leaves_an_empty_history_that_can_still_undo() {
        let mut history = History::default();
        history.push(text("only"), Instant::now());
        history.remove(0);
        assert!(history.is_empty());
        assert!(history.selected().is_none());
        assert!(history.undo());
        assert_eq!(texts(&history), ["only"]);
    }

    #[test]
    fn the_selection_stays_on_the_same_item_when_one_above_it_is_removed() {
        let mut history = History::default();
        for value in ["a", "b", "c"] {
            history.push(text(value), Instant::now());
        }
        history.select(2);
        history.remove(0);
        assert_eq!(history.selected_index(), 1);
        assert!(matches!(&history.selected().unwrap().content, Content::Text(text) if text == "a"));
    }

    #[test]
    fn pictures_over_the_total_limit_push_the_oldest_copies_out() {
        let mut history = History::default();
        history.push(text("old"), Instant::now());
        let big = |value: u8| {
            let side = 1024u32;
            let pixels = ImageData::new(side, side, vec![value; (side * side * 4) as usize]);
            Content::Image(StoredImage::new(pixels, side, side, "PNG"))
        };
        for value in 1..=4 {
            history.push(big(value), Instant::now());
        }
        // Three 4 MiB pictures fill the 12 MiB limit, so the text and the first picture are gone.
        assert_eq!(history.items().len(), 3);
        assert!(history.total_bytes() <= MAX_TOTAL_BYTES);
    }

    #[test]
    fn clearing_forgets_the_items_and_the_undo() {
        let mut history = History::default();
        history.push(text("a"), Instant::now());
        history.remove(0);
        history.push(text("b"), Instant::now());
        history.clear();
        assert!(history.is_empty());
        assert!(!history.can_undo());
    }
}
