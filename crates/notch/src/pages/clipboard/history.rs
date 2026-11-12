//! The remembered copies with the limits that keep them small, the pinned ones and the one removal that can be undone.
//!
//! The list is the pinned copies in the order they were pinned, then the others newest first. The limits drop the oldest
//! unpinned copy first and never a pinned one; the newest copy also stays, so pins alone can pass a limit (with three
//! pinned pictures of at most 3 MiB and one new one, the 12 MiB limit is not reached).

use std::time::Instant;

use super::item::{Content, Item, kind_of};

pub(super) const MAX_ITEMS: usize = 7;
/// Pinned copies count toward `MAX_ITEMS`.
pub(super) const MAX_PINNED: usize = 3;
pub(super) const MAX_TEXT_BYTES: usize = 100 * 1024;
/// About 3 MiB of pixels; a larger picture is shrunk before it is kept.
pub(super) const MAX_IMAGE_PIXELS: u64 = 786_432;
/// Text and pictures together; the oldest unpinned copies go first when a new one would pass it.
pub(super) const MAX_TOTAL_BYTES: usize = 12 * 1024 * 1024;

struct Removed {
    item: Item,
    index: usize,
    was_pinned: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Pushed {
    Added,
    /// The same content was already in the list; it moved to the front of the unpinned copies.
    MovedToFront,
}

#[derive(Default)]
pub(super) struct History {
    items: Vec<Item>,
    /// The first `pinned_count` items are pinned.
    pinned_count: usize,
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

    pub fn is_pinned(&self, index: usize) -> bool {
        index < self.pinned_count.min(self.items.len())
    }

    /// Whether the row at `index` has a pin button: a pinned row always, another only while fewer than three are pinned.
    pub fn has_pin_button(&self, index: usize) -> bool {
        index < self.items.len() && (self.is_pinned(index) || self.pinned_count < MAX_PINNED)
    }

    pub fn select(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected = index;
        }
    }

    /// Adds a copy on top of the unpinned ones and selects it. The same content copied again moves instead of adding a
    /// second row.
    pub fn push(&mut self, content: Content, now: Instant) -> Pushed {
        if let Some(index) = self.items.iter().position(|item| item.content.is_same_as(&content)) {
            self.move_to_front(index, now);
            return Pushed::MovedToFront;
        }
        self.next_id += 1;
        let item = Item { id: self.next_id, kind: kind_of(&content), content, copied_at: now };
        self.items.insert(self.pinned_count, item);
        self.selected = self.pinned_count;
        self.enforce_limits();
        Pushed::Added
    }

    /// Marks the item as copied now and selects it; an unpinned one moves on top of the unpinned copies, a pinned one
    /// keeps its place.
    pub fn move_to_front(&mut self, index: usize, now: Instant) {
        if index >= self.items.len() {
            return;
        }
        self.items[index].copied_at = now;
        self.selected = if self.is_pinned(index) { index } else { self.move_item(index, self.pinned_count) };
    }

    /// Pins an unpinned item under the pinned ones, or unpins a pinned one back among the others by its copy time.
    /// Returns false when nothing changed: a fourth pin is refused.
    pub fn toggle_pin(&mut self, index: usize) -> bool {
        if !self.has_pin_button(index) {
            return false;
        }
        let selected_id = self.selected().map(|item| item.id);
        if self.is_pinned(index) {
            self.pinned_count -= 1;
            let copied_at = self.items[index].copied_at;
            let later_unpinned =
                self.items[self.pinned_count + 1..].iter().take_while(|item| item.copied_at > copied_at).count();
            self.move_item(index, self.pinned_count + later_unpinned);
        } else {
            self.move_item(index, self.pinned_count);
            self.pinned_count += 1;
        }
        if let Some(position) = self.items.iter().position(|item| Some(item.id) == selected_id) {
            self.selected = position;
        }
        true
    }

    /// Takes the item out and keeps it for `undo`; an earlier removal that was not undone is dropped for good.
    pub fn remove(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }
        let was_pinned = self.is_pinned(index);
        let item = self.items.remove(index);
        if was_pinned {
            self.pinned_count -= 1;
        }
        self.removed = Some(Removed { item, index, was_pinned });
        if self.selected > index {
            self.selected -= 1;
        }
        self.selected = self.selected.min(self.items.len().saturating_sub(1));
    }

    /// Puts the last removed item back where it was, pinned if it was and three are not pinned since, and selects it.
    pub fn undo(&mut self) -> bool {
        let Some(Removed { item, index, was_pinned }) = self.removed.take() else { return false };
        let index = if was_pinned && self.pinned_count < MAX_PINNED {
            self.pinned_count += 1;
            index.min(self.pinned_count - 1)
        } else {
            index.clamp(self.pinned_count, self.items.len())
        };
        self.items.insert(index, item);
        self.selected = index;
        self.enforce_limits();
        true
    }

    pub fn forget_removed(&mut self) {
        self.removed = None;
    }

    /// Drops everything, including the pins and what could be undone.
    pub fn clear(&mut self) {
        self.items = Vec::new();
        self.pinned_count = 0;
        self.removed = None;
        self.selected = 0;
    }

    /// Moves the item at `from` to `to` (an index in the list after the move) and returns `to`.
    fn move_item(&mut self, from: usize, to: usize) -> usize {
        let item = self.items.remove(from);
        let to = to.min(self.items.len());
        self.items.insert(to, item);
        to
    }

    fn total_bytes(&self) -> usize {
        self.items.iter().map(|item| item.content.byte_size()).sum()
    }

    /// Drops the oldest unpinned copies (the end of the list) while a limit is passed, keeping the newest unpinned one.
    fn enforce_limits(&mut self) {
        let selected_id = self.selected().map(|item| item.id);
        while self.items.len() - self.pinned_count > 1
            && (self.items.len() > MAX_ITEMS || self.total_bytes() > MAX_TOTAL_BYTES)
        {
            self.items.pop();
        }
        self.selected = self
            .items
            .iter()
            .position(|item| Some(item.id) == selected_id)
            .unwrap_or_else(|| self.selected.min(self.items.len().saturating_sub(1)));
    }
}

#[cfg(test)]
mod tests;
