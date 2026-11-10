//! "Copied" when something is put on the clipboard. It never reads what was copied.

use crate::i18n::tr;
use windows::Win32::System::DataExchange::CountClipboardFormats;

use crate::{
    activity::{Activity, Flash, Presence, SystemMessage},
    gfx::{Canvas, Color, Rect},
    page::{Context, TimerId},
    platform::clipboard,
    ui::{self, Icon},
};

const SHOW_MILLISECONDS: u32 = 1200;

pub struct ClipboardActivity {
    flash: Flash,
    width: f32,
}

impl ClipboardActivity {
    pub fn new() -> Self {
        Self { flash: Flash::default(), width: 0.0 }
    }
}

/// A copy that put something on the clipboard, as opposed to a clear, that its owner wants shown and that the notch did
/// not make itself (the Clipboard page writing an item back).
fn is_visible_copy() -> bool {
    (unsafe { CountClipboardFormats() }) > 0 && !clipboard::is_own_write() && !clipboard::is_excluded()
}

impl Activity for ClipboardActivity {
    fn id(&self) -> &'static str {
        "clipboard"
    }

    fn presence(&self) -> Option<Presence> {
        self.flash.is_visible().then_some(Presence::Transient { width: self.width })
    }

    fn draw(&self, canvas: &Canvas, area: Rect, opacity: f32) {
        ui::draw_message(canvas, area, opacity, Icon::Copy, Color::WHITE, &tr!("clipboard.copied"), None);
    }

    fn on_start(&mut self, context: &mut Context) {
        self.width = ui::message_width(context.fonts(), &tr!("clipboard.copied"), None);
        context.listen_to_clipboard();
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        self.flash.on_timer(timer, context);
    }

    fn on_system_message(&mut self, message: &SystemMessage, context: &mut Context) {
        if *message == SystemMessage::ClipboardChanged && is_visible_copy() {
            self.flash.show(context, SHOW_MILLISECONDS);
        }
    }
}
