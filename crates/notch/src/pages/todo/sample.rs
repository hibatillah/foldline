//! The design's example lists, one scenario per state of the page, for the snapshots.

use super::{
    TodoPage, TodoTarget,
    list::{Item, TodoList},
};
use crate::page::KeyInput;

/// A list with a task just deleted, so the toolbar offers Undo, and the first row hovered with its copy and delete.
pub(super) const DELETED: u8 = 6;
/// The list just emptied by Clear all, so the toolbar offers Undo over the empty page.
pub(super) const CLEARED: u8 = 7;

impl TodoPage {
    pub fn sample(scenario: u8) -> Self {
        let item = |text: &str, is_done| Item { text: text.to_owned(), is_done };
        let open = || {
            vec![
                item("Check hover on the 1440p monitor", false),
                item("Write the v0.5 release notes", false),
                item("Buy a code-signing certificate", false),
            ]
        };
        let items = match scenario {
            1 | 2 => Vec::new(),
            3 => [open(), vec![item("Send the invoice to the client", false), item("Book the Jakarta flight", false)]]
                .concat(),
            4 => [open(), vec![item("Send the invoice to the client", false)]].concat(),
            5 => [
                open(),
                vec![item("Send the invoice to the client", true), item("Reply to Rafi about the deploy", true)],
            ]
            .concat(),
            _ => [open(), vec![item("Reply to Rafi about the deploy", true)]].concat(),
        };

        let mut page = Self::with_list(TodoList::with_items(items));
        page.is_adding = matches!(scenario, 2 | 4);
        if scenario == 4 {
            page.field.apply(&KeyInput::Text("Call the notary".to_owned()));
            page.has_keyboard = true;
        }
        if scenario == DELETED {
            page.list.remove(1);
        }
        if scenario == CLEARED {
            page.list.clear();
        }
        let hovered_row = match scenario {
            0 => Some(TodoTarget::Row(1)),
            DELETED => Some(TodoTarget::Row(0)),
            _ => None,
        };
        page.hover.set(hovered_row);
        page.refresh_texts();
        page
    }
}
