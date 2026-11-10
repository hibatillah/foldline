use super::{DRAG_START_DISTANCE, Press, ShelfPage, Target, file_index, is_double_click, target_at};
use crate::{
    page::{Context, Frame, PointerEvent},
    platform,
};
use std::time::Instant;

impl ShelfPage {
    pub(super) fn handle_pointer(&mut self, event: PointerEvent, frame: &Frame, context: &mut Context) {
        let (bounds, count) = (frame.bounds, self.shelf.len());
        match event {
            PointerEvent::Moved(point) => {
                if self.hover.set(target_at(point, bounds, count)) {
                    context.request_repaint();
                }
                let Some(press) = self.press else { return };
                let Target::Slot(position) = press.target else { return };
                let moved = (point.x - press.origin.x).hypot(point.y - press.origin.y);
                if moved >= DRAG_START_DISTANCE
                    && let Some(index) = file_index(position, count)
                {
                    self.press = None;
                    self.begin_drag(index, context);
                }
            }
            PointerEvent::Pressed(point) if frame.is_open() => {
                self.press = target_at(point, bounds, count).map(|target| Press { target, origin: point });
            }
            PointerEvent::Released(point) => {
                let Some(press) = self.press.take() else { return };
                if !frame.is_open() || target_at(point, bounds, count) != Some(press.target) {
                    return;
                }
                match press.target {
                    Target::Remove(position) => {
                        if let Some(index) = file_index(position, count) {
                            self.remove(index, context);
                        }
                    }
                    Target::Slot(position) => {
                        let Some(index) = file_index(position, count) else { return };
                        let path = self.shelf.items()[index].clone();
                        let (now, limit) = (Instant::now(), platform::double_click_time());
                        if is_double_click(self.last_click.as_ref(), &path, now, limit) {
                            self.last_click = None;
                            platform::open(&path.display().to_string());
                        } else {
                            self.last_click = Some((path, now));
                        }
                    }
                }
            }
            PointerEvent::SecondaryReleased(point) if frame.is_open() => {
                let Some(Target::Slot(position) | Target::Remove(position)) = target_at(point, bounds, count) else {
                    return;
                };
                if let Some(index) = file_index(position, count) {
                    self.remove(index, context);
                }
            }
            PointerEvent::Left | PointerEvent::CaptureLost => {
                self.press = None;
                if self.hover.clear() {
                    context.request_repaint();
                }
            }
            _ => {}
        }
    }
}
