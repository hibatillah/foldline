use super::{
    CalendarPage,
    geometry::{WHEEL_NOTCH_DISTANCE, day_at, events_area, max_scroll},
};
use crate::page::{Context, Frame, PointerEvent};

impl CalendarPage {
    pub(super) fn handle_pointer(&mut self, event: PointerEvent, frame: &Frame, context: &mut Context) {
        let (bounds, scale) = (frame.bounds, frame.scale);
        let scroll_limit = max_scroll(self.agenda.rows.len(), self.view);
        match event {
            PointerEvent::Moved(point) => {
                if let Some(last_y) = self.drag_last_y {
                    self.drag_last_y = Some(point.y);
                    self.scroll_by((last_y - point.y) / scale, scroll_limit, context);
                }
                let hovered_cell = day_at(point, &self.grid, bounds, scale);
                if hovered_cell != self.hovered_cell {
                    self.hovered_cell = hovered_cell;
                    context.request_repaint();
                }
            }
            PointerEvent::Pressed(point) if frame.is_open() => {
                if let Some(cell) = day_at(point, &self.grid, bounds, scale) {
                    let day = self.grid.first_day + (cell - self.grid.leading) as i64;
                    self.selected_day = (day != self.today).then_some(day);
                    self.scroll = 0.0;
                    self.refresh(context.fonts());
                    context.request_repaint();
                } else if scroll_limit > 0.0 && events_area(bounds, scale, self.view).contains(point) {
                    self.drag_last_y = Some(point.y);
                }
            }
            PointerEvent::Scrolled { notches } => {
                self.scroll_by(notches * WHEEL_NOTCH_DISTANCE, scroll_limit, context);
            }
            PointerEvent::Released(_) => self.drag_last_y = None,
            PointerEvent::Left | PointerEvent::CaptureLost => {
                self.drag_last_y = None;
                if self.hovered_cell.take().is_some() {
                    context.request_repaint();
                }
            }
            _ => {}
        }
    }

    fn scroll_by(&mut self, distance: f32, max_scroll: f32, context: &mut Context) {
        let scroll = (self.scroll + distance).clamp(0.0, max_scroll);
        if scroll != self.scroll {
            self.scroll = scroll;
            context.request_repaint();
        }
    }
}
