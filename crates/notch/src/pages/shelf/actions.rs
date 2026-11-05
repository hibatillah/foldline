//! What the shelf does with its files: save, preview, remove, drag out.

use super::items::{count_text, file_name, removed_text};
use super::{MAX_SLOTS, PREVIEW_PIXELS, ShelfPage, name_lines};
use crate::{
    gfx::Fonts,
    page::Context,
    platform,
    ui::{self},
};
use std::sync::Arc;

pub(super) const ALERT_MILLISECONDS: u32 = 2000;

impl ShelfPage {
    pub(super) fn save(&self) {
        if let Some(storage) = &self.storage {
            self.shelf.save(storage);
        }
    }

    pub(super) fn announce(&mut self, title: String, context: &mut Context) {
        self.alert_value = count_text(self.shelf.len());
        self.alert_width = ui::message_width(context.fonts(), &title, Some(&self.alert_value));
        self.alert_title = title;
        self.alert.show(context, ALERT_MILLISECONDS);
    }

    pub(super) fn refresh_names(&mut self, fonts: &Fonts) {
        self.names =
            self.shelf.items().iter().take(MAX_SLOTS).map(|path| name_lines(&file_name(path), fonts)).collect();
    }

    pub(super) fn request_previews(&mut self, context: &Context) {
        let items = self.shelf.items();
        self.requested.retain(|path| items.contains(path));
        self.previews.get_mut().retain(|path, _| items.contains(path));
        let notifier = context.notifier();
        for path in items.iter().take(MAX_SLOTS) {
            if !self.requested.insert(path.clone()) {
                continue;
            }
            let (path, inbox) = (path.clone(), Arc::clone(&self.inbox));
            let _ = std::thread::Builder::new().name("shelf-preview".into()).spawn(move || {
                let image = platform::file_preview(&path, PREVIEW_PIXELS);
                if let Ok(mut inbox) = inbox.lock() {
                    inbox.previews.push((path, image));
                }
                notifier.notify();
            });
        }
    }

    pub(super) fn shelf_changed(&mut self, context: &mut Context) {
        self.save();
        self.refresh_names(context.fonts());
        self.request_previews(context);
        let items = self.shelf.items();
        self.arrivals.retain(|path, _| items.contains(path));
        self.hovered = None;
        context.request_repaint();
    }

    pub(super) fn remove(&mut self, index: usize, context: &mut Context) {
        if let Some(path) = self.shelf.remove(index) {
            self.shelf_changed(context);
            self.announce(removed_text(&file_name(&path)), context);
        }
    }

    pub(super) fn begin_drag(&mut self, index: usize, context: &mut Context) {
        let path = self.shelf.items()[index].clone();
        let (inbox, notifier) = (Arc::clone(&self.inbox), context.notifier());
        self.dragging = Some(path.clone());
        let started = platform::drag_file_out(path, move |path, was_taken| {
            if let Ok(mut inbox) = inbox.lock() {
                inbox.drags.push((path, was_taken));
            }
            notifier.notify();
        });
        if !started {
            self.dragging = None;
        }
        context.request_repaint();
    }
}
