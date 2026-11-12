//! The todo list and its file, `%APPDATA%\notch\todo.txt`, one markdown checkbox per line so it reads in any editor.
//! The last removal (one task or a clear) is held in memory for undo; the file never holds removed tasks.

use std::{fs, io::ErrorKind, path::PathBuf};

use crate::log;

const OPEN_MARK: &str = "- [ ]";
const DONE_MARKS: [&str; 2] = ["- [x]", "- [X]"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub text: String,
    pub is_done: bool,
}

#[derive(Debug, Default)]
pub struct TodoList {
    /// In file order: the order items were added.
    items: Vec<Item>,
    path: Option<PathBuf>,
    /// The tasks the last removal took, with their places in `items`, in ascending order.
    removed: Vec<(usize, Item)>,
}

impl TodoList {
    pub fn load() -> Self {
        let path = std::env::var_os("APPDATA").map(|directory| PathBuf::from(directory).join("notch").join("todo.txt"));
        Self::at(path)
    }

    pub fn at(path: Option<PathBuf>) -> Self {
        let items = match path.as_ref().map(fs::read_to_string) {
            Some(Ok(text)) => parse(&text),
            Some(Err(error)) if error.kind() != ErrorKind::NotFound => {
                log::debug(&format!("could not read the todo list: {error}"));
                Vec::new()
            }
            _ => Vec::new(),
        };
        Self { items, path, removed: Vec::new() }
    }

    #[cfg(test)]
    pub fn with_items(items: Vec<Item>) -> Self {
        Self { items, path: None, removed: Vec::new() }
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn open_count(&self) -> usize {
        self.items.iter().filter(|item| !item.is_done).count()
    }

    pub fn done_count(&self) -> usize {
        self.items.len() - self.open_count()
    }

    /// Open items first, then finished ones, each in the order they were added; the values index `items`.
    pub fn shown_order(&self) -> Vec<usize> {
        let open = (0..self.items.len()).filter(|&index| !self.items[index].is_done);
        let done = (0..self.items.len()).filter(|&index| self.items[index].is_done);
        open.chain(done).collect()
    }

    pub fn item(&self, index: usize) -> Option<&Item> {
        self.items.get(index)
    }

    /// Adds an open item; blank text adds nothing and returns false.
    pub fn add(&mut self, text: &str) -> bool {
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if text.is_empty() {
            return false;
        }
        self.items.push(Item { text, is_done: false });
        self.save();
        true
    }

    pub fn toggle(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            item.is_done = !item.is_done;
            self.save();
        }
    }

    /// Removes the task and keeps it for `undo`, replacing what an earlier removal kept.
    pub fn remove(&mut self, index: usize) {
        if index < self.items.len() {
            self.removed = vec![(index, self.items.remove(index))];
            self.save();
        }
    }

    /// Removes every task, done or not, and keeps them all for `undo`.
    pub fn clear(&mut self) {
        if !self.items.is_empty() {
            self.removed = std::mem::take(&mut self.items).into_iter().enumerate().collect();
            self.save();
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.removed.is_empty()
    }

    /// Puts the tasks the last removal took back at their places (or at the end when the list got shorter) and saves.
    pub fn undo(&mut self) -> bool {
        if self.removed.is_empty() {
            return false;
        }
        for (index, item) in std::mem::take(&mut self.removed) {
            let index = index.min(self.items.len());
            self.items.insert(index, item);
        }
        self.save();
        true
    }

    pub fn forget_removed(&mut self) {
        self.removed.clear();
    }

    fn save(&self) {
        let Some(path) = &self.path else { return };
        if let Some(directory) = path.parent()
            && let Err(error) = fs::create_dir_all(directory)
        {
            log::debug(&format!("could not create the todo folder: {error}"));
            return;
        }
        if let Err(error) = fs::write(path, serialize(&self.items)) {
            log::debug(&format!("could not save the todo list: {error}"));
        }
    }
}

/// Reads checkbox lines; any other non-blank line becomes an open item, so a hand-written list still loads.
pub fn parse(text: &str) -> Vec<Item> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            if let Some(rest) = line.strip_prefix(OPEN_MARK) {
                Item { text: rest.trim().to_owned(), is_done: false }
            } else if let Some(rest) = DONE_MARKS.iter().find_map(|mark| line.strip_prefix(mark)) {
                Item { text: rest.trim().to_owned(), is_done: true }
            } else {
                Item { text: line.trim_start_matches("- ").to_owned(), is_done: false }
            }
        })
        .filter(|item| !item.text.is_empty())
        .collect()
}

pub fn serialize(items: &[Item]) -> String {
    items
        .iter()
        .map(|item| format!("{} {}\n", if item.is_done { DONE_MARKS[0] } else { OPEN_MARK }, item.text))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(text: &str, is_done: bool) -> Item {
        Item { text: text.to_owned(), is_done }
    }

    #[test]
    fn the_file_round_trips_through_markdown_checkboxes() {
        let items = vec![item("Write release notes", false), item("Reply to Rafi", true)];
        let text = serialize(&items);
        assert_eq!(text, "- [ ] Write release notes\n- [x] Reply to Rafi\n");
        assert_eq!(parse(&text), items);
    }

    #[test]
    fn a_hand_written_list_still_loads() {
        let parsed = parse("Buy milk\n\n- [X] Call mum\n- plain bullet\n- [ ]   \n");
        assert_eq!(parsed, vec![item("Buy milk", false), item("Call mum", true), item("plain bullet", false)]);
    }

    #[test]
    fn open_items_show_before_finished_ones_in_the_order_they_were_added() {
        let list = TodoList::with_items(vec![item("a", true), item("b", false), item("c", true), item("d", false)]);
        assert_eq!(list.shown_order(), [1, 3, 0, 2]);
        assert_eq!((list.open_count(), list.done_count()), (2, 2));
    }

    #[test]
    fn adding_tidies_spaces_and_ignores_blank_text() {
        let mut list = TodoList::with_items(Vec::new());
        assert!(!list.add("   "));
        assert!(list.add("  Ship   v0.5 "));
        assert_eq!(list.item(0), Some(&item("Ship v0.5", false)));
    }

    #[test]
    fn toggling_and_removing_change_only_the_named_item() {
        let mut list = TodoList::with_items(vec![item("a", false), item("b", false)]);
        list.toggle(1);
        assert!(list.item(1).is_some_and(|item| item.is_done));
        list.remove(0);
        assert_eq!(list.item(0), Some(&item("b", true)));
        list.remove(5);
        list.toggle(5);
        assert_eq!(list.open_count() + list.done_count(), 1);
    }

    #[test]
    fn clearing_removes_every_task_done_or_not() {
        let mut list = TodoList::with_items(vec![item("a", false), item("b", true)]);
        list.clear();
        assert!(list.is_empty());
        list.clear();
        assert_eq!((list.open_count(), list.done_count()), (0, 0));
    }

    fn texts(list: &TodoList) -> Vec<&str> {
        (0..list.len()).filter_map(|index| list.item(index)).map(|item| item.text.as_str()).collect()
    }

    #[test]
    fn undo_puts_a_removed_task_back_in_its_place_with_its_tick() {
        let mut list = TodoList::with_items(vec![item("a", false), item("b", true), item("c", false)]);
        list.remove(1);
        assert!(list.can_undo());
        assert!(list.undo());
        assert_eq!(list.item(1), Some(&item("b", true)));
        assert_eq!(texts(&list), ["a", "b", "c"]);
        assert!(!list.can_undo());
        assert!(!list.undo());
    }

    #[test]
    fn undo_after_clearing_brings_every_task_back_in_order() {
        let mut list = TodoList::with_items(vec![item("a", false), item("b", true), item("c", false)]);
        list.clear();
        assert!(list.is_empty());
        list.add("d");
        assert!(list.undo());
        assert_eq!(texts(&list), ["a", "b", "c", "d"]);
        assert_eq!(list.done_count(), 1);
    }

    #[test]
    fn a_second_removal_replaces_what_the_first_could_undo() {
        let mut list = TodoList::with_items(vec![item("a", false), item("b", false), item("c", false)]);
        list.remove(0);
        list.remove(1);
        assert!(list.undo());
        assert_eq!(texts(&list), ["b", "c"]);
        list.clear();
        list.forget_removed();
        assert!(!list.undo());
        assert!(list.is_empty());
    }

    #[test]
    fn a_removed_task_stays_off_the_file_until_it_is_undone() {
        let path = std::env::temp_dir().join(format!("notch-todo-undo-{}.txt", std::process::id()));
        let _ = fs::remove_file(&path);
        let mut list = TodoList::at(Some(path.clone()));
        list.add("keep");
        list.add("gone for now");
        list.remove(1);
        let after_remove = TodoList::at(Some(path.clone()));
        list.undo();
        let after_undo = TodoList::at(Some(path.clone()));
        list.clear();
        let after_clear = TodoList::at(Some(path.clone()));
        let _ = fs::remove_file(&path);
        assert_eq!(texts(&after_remove), ["keep"]);
        assert_eq!(texts(&after_undo), ["keep", "gone for now"]);
        assert!(after_clear.is_empty());
        assert!(!after_clear.can_undo(), "the undo lives in memory only");
    }

    #[test]
    fn a_saved_list_loads_back_from_its_file() {
        let path = std::env::temp_dir().join(format!("notch-todo-{}.txt", std::process::id()));
        let _ = fs::remove_file(&path);
        let mut list = TodoList::at(Some(path.clone()));
        list.add("Check the hover on 1440p");
        list.toggle(0);
        let loaded = TodoList::at(Some(path.clone()));
        let _ = fs::remove_file(&path);
        assert_eq!(loaded.item(0), Some(&item("Check the hover on 1440p", true)));
    }
}
