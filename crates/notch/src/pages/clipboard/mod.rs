//! Opt-in clipboard history: the last seven copies in memory, a preview and paste. Nothing is read while it is off.

mod actions;
mod capture;
mod dib;
mod draw;
mod history;
mod item;
mod layout;
mod preview;
#[cfg(test)]
mod sample;
mod state;
mod style;
#[cfg(test)]
mod tests;

use std::cell::RefCell;

use self::{
    history::History,
    layout::{CONTENT_HEIGHT, CONTENT_WIDTH, DoubleClick, Layout, Target},
    state::{Bar, Confirm, Preview},
};
use crate::{
    activity::SystemMessage,
    gfx::{Canvas, Point, Size},
    page::{
        Context, DEFAULT_CONTENT_HEIGHT, Frame, Mode, Page, PageSizes, PointerEvent, Priority, RESTING_MIN_WIDTH,
        TimerId,
    },
    ui::HoverTarget,
};

const BAR_TIMER: TimerId = TimerId(0);
const CONFIRM_TIMER: TimerId = TimerId(1);

pub struct ClipboardPage {
    history: History,
    is_enabled: bool,
    hover: HoverTarget<Target>,
    confirm: Option<Confirm>,
    bar: Option<Bar>,
    clicks: DoubleClick,
    preview: RefCell<Preview>,
}

impl ClipboardPage {
    pub fn new() -> Self {
        Self {
            history: History::default(),
            is_enabled: false,
            hover: HoverTarget::default(),
            confirm: None,
            bar: None,
            clicks: DoubleClick::default(),
            preview: RefCell::new(Preview::default()),
        }
    }

    fn layout(frame: &Frame) -> Layout {
        Layout::new(frame.bounds)
    }

    fn target_at(&self, point: Point, frame: &Frame) -> Option<Target> {
        let has_undo = self.bar.is_some_and(Bar::has_undo) && self.history.can_undo();
        Self::layout(frame).hit(point, self.history.items().len(), self.bar.is_some(), has_undo)
    }
}

impl Page for ClipboardPage {
    fn id(&self) -> &'static str {
        "clipboard"
    }

    fn uses_units(&self) -> bool {
        true
    }

    fn name(&self) -> String {
        crate::i18n::tr!("page.clipboard").into_owned()
    }

    fn is_opt_in(&self) -> bool {
        true
    }

    fn priority(&self, mode: Mode) -> Option<Priority> {
        (mode == Mode::Expanded).then_some(Priority::BACKGROUND)
    }

    fn sizes(&self) -> PageSizes {
        let height = if self.history.is_empty() { DEFAULT_CONTENT_HEIGHT } else { CONTENT_HEIGHT };
        PageSizes { resting_width: RESTING_MIN_WIDTH, expanded: Size::new(CONTENT_WIDTH, height) }
    }

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        self.draw_page(canvas, frame.bounds, opacity);
    }

    fn on_start(&mut self, context: &mut Context) {
        context.listen_to_clipboard();
    }

    fn on_enabled_changed(&mut self, is_enabled: bool, context: &mut Context) {
        self.is_enabled = is_enabled;
        if !is_enabled {
            self.forget_everything(context);
        }
    }

    fn on_system_message(&mut self, message: &SystemMessage, context: &mut Context) {
        if *message == SystemMessage::ClipboardChanged && self.is_enabled {
            self.capture_copy(context);
        }
    }

    fn on_shown_changed(&mut self, is_shown: bool, context: &mut Context) {
        if is_shown {
            return;
        }
        self.confirm = None;
        self.hover.clear();
        self.clicks.reset();
        context.kill_timer(CONFIRM_TIMER);
        // The uploaded picture is dropped while closed; it is uploaded again when the page is next drawn.
        *self.preview.get_mut() = Preview::default();
    }

    fn on_pointer(&mut self, event: PointerEvent, frame: &Frame, context: &mut Context) {
        if !frame.is_open() {
            return;
        }
        match event {
            PointerEvent::Moved(point) => {
                if self.hover.set(self.target_at(point, frame)) {
                    context.request_repaint();
                }
            }
            PointerEvent::Left | PointerEvent::CaptureLost => {
                if self.hover.clear() {
                    context.request_repaint();
                }
            }
            PointerEvent::Scrolled { .. } => {}
            PointerEvent::Pressed(point) => self.press(point, frame, context),
            PointerEvent::Released(point) => self.release(point, frame, context),
            PointerEvent::SecondaryReleased(point) => {
                if let Some(Target::Row(index)) = self.target_at(point, frame) {
                    self.remove(index, context);
                }
            }
        }
    }

    fn uses_right_click(&self, point: Point, frame: &Frame) -> bool {
        matches!(self.target_at(point, frame), Some(Target::Row(_)))
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        match timer {
            BAR_TIMER => self.expire_bar(context),
            CONFIRM_TIMER => self.expire_confirmation(context),
            _ => {}
        }
    }
}
