//! What clicks and copies do to the history, the clipboard and the window in front.

use std::time::Instant;

use super::{
    BAR_TIMER, CONFIRM_TIMER, ClipboardPage,
    capture::{Captured, MAX_DIB_BYTES, capture},
    dib,
    history::MAX_TEXT_BYTES,
    item::Content,
    layout::{DOUBLE_CLICK_SLOP, Target},
    state::{Bar, Confirm},
};
use crate::{
    gfx::Point,
    page::{Context, Frame},
    platform::{self, clipboard},
};

const UNDO_MILLISECONDS: u32 = 5000;
const SKIPPED_MILLISECONDS: u32 = 3000;
const COPIED_MILLISECONDS: u32 = 1500;
/// The notch stays open this long after a paste so the green "Pasted" shows.
const PASTED_MILLISECONDS: u32 = 600;

impl ClipboardPage {
    /// A copy arrived; the history stores it, or a bar says why it did not.
    pub(super) fn capture_copy(&mut self, context: &mut Context) {
        if clipboard::is_own_write() {
            return;
        }
        match capture(clipboard::read_snapshot(MAX_TEXT_BYTES, MAX_DIB_BYTES)) {
            Captured::Nothing => {}
            Captured::Skipped(skip) => self.show_bar(Bar::Skipped(skip), SKIPPED_MILLISECONDS, context),
            Captured::Content(content) => {
                self.history.push(content, Instant::now());
                self.confirm = None;
                context.request_repaint();
            }
        }
    }

    pub(super) fn press(&mut self, point: Point, frame: &Frame, context: &mut Context) {
        match self.target_at(point, frame) {
            Some(Target::Row(index)) => {
                if self.history.selected_index() != index || self.confirm.is_some() {
                    self.history.select(index);
                    self.confirm = None;
                    context.kill_timer(CONFIRM_TIMER);
                    context.request_repaint();
                }
                let limit = platform::double_click_time();
                if self.clicks.press(Instant::now(), point, limit, DOUBLE_CLICK_SLOP) {
                    self.paste_selected(context);
                }
            }
            _ => self.clicks.reset(),
        }
    }

    pub(super) fn release(&mut self, point: Point, frame: &Frame, context: &mut Context) {
        match self.target_at(point, frame) {
            Some(Target::Delete) => self.remove(self.history.selected_index(), context),
            Some(Target::Copy) => self.copy_selected(context),
            Some(Target::Paste) => self.paste_selected(context),
            Some(Target::Undo) => self.undo(context),
            Some(Target::Pin(index)) => self.toggle_pin(index, point, frame, context),
            Some(Target::Row(_)) | None => {}
        }
    }

    /// Pins or unpins the row's copy; the rows move, so the hover is found again under the pointer.
    fn toggle_pin(&mut self, index: usize, point: Point, frame: &Frame, context: &mut Context) {
        if self.history.toggle_pin(index) {
            self.hover.set(self.target_at(point, frame));
            context.request_repaint();
        }
    }

    /// Writes the selected item to the clipboard; false when the clipboard stayed busy.
    fn write_selected(&self, context: &Context) -> bool {
        let Some(item) = self.history.selected() else { return false };
        match &item.content {
            Content::Text(text) => clipboard::write_text(context.window(), text),
            Content::Image(image) => clipboard::write_dib_v5(context.window(), &dib::encode(&image.pixels)),
        }
    }

    /// Pastes into the window in front. With nowhere to paste the item is only copied again and moves to the front.
    pub(super) fn paste_selected(&mut self, context: &mut Context) {
        if self.history.is_empty() {
            return;
        }
        let target = platform::paste_target();
        if !self.write_selected(context) {
            return;
        }
        if target.is_some_and(platform::send_paste) {
            self.confirm = Some(Confirm::Pasted);
            context.set_timer(CONFIRM_TIMER, PASTED_MILLISECONDS);
            context.request_repaint();
        } else {
            self.finish_copy(context);
        }
    }

    pub(super) fn copy_selected(&mut self, context: &mut Context) {
        if !self.history.is_empty() && self.write_selected(context) {
            self.finish_copy(context);
        }
    }

    fn finish_copy(&mut self, context: &mut Context) {
        self.history.move_to_front(self.history.selected_index(), Instant::now());
        self.confirm = Some(Confirm::Copied);
        context.set_timer(CONFIRM_TIMER, COPIED_MILLISECONDS);
        context.request_repaint();
    }

    pub(super) fn remove(&mut self, index: usize, context: &mut Context) {
        if index >= self.history.items().len() {
            return;
        }
        self.history.remove(index);
        self.confirm = None;
        context.kill_timer(CONFIRM_TIMER);
        self.show_bar(Bar::Removed, UNDO_MILLISECONDS, context);
    }

    pub(super) fn undo(&mut self, context: &mut Context) {
        if self.history.undo() {
            self.bar = None;
            context.kill_timer(BAR_TIMER);
            context.request_repaint();
        }
    }

    /// Replacing a bar that offered undo drops that undo for good.
    fn show_bar(&mut self, bar: Bar, milliseconds: u32, context: &mut Context) {
        if bar != Bar::Removed {
            self.history.forget_removed();
        }
        self.bar = Some(bar);
        context.set_timer(BAR_TIMER, milliseconds);
        context.request_repaint();
    }

    pub(super) fn expire_bar(&mut self, context: &mut Context) {
        context.kill_timer(BAR_TIMER);
        self.bar = None;
        self.history.forget_removed();
        context.request_repaint();
    }

    pub(super) fn expire_confirmation(&mut self, context: &mut Context) {
        context.kill_timer(CONFIRM_TIMER);
        if self.confirm.take() == Some(Confirm::Pasted) {
            context.request_close();
        }
        context.request_repaint();
    }

    /// The page was switched off: nothing copied stays in memory.
    pub(super) fn forget_everything(&mut self, context: &mut Context) {
        self.history.clear();
        self.confirm = None;
        self.bar = None;
        self.hover.clear();
        self.clicks.reset();
        context.kill_timer(BAR_TIMER);
        context.kill_timer(CONFIRM_TIMER);
        *self.preview.get_mut() = super::state::Preview::default();
        context.request_repaint();
    }
}
