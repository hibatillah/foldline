use super::WeatherPage;
use crate::page::{Context, Frame, PointerEvent};

impl WeatherPage {
    pub(super) fn handle_pointer(&mut self, event: PointerEvent, frame: &Frame, context: &mut Context) {
        let is_available = self.forecast.is_some() && !self.is_offline;
        let toggle = Self::toggle_area(frame.bounds, frame.scale);
        match event {
            PointerEvent::Moved(point) => {
                let is_hovered = is_available && toggle.contains(point);
                let hovered_cell = if is_available { self.cell_at(frame, point) } else { None };
                if is_hovered != self.is_toggle_hovered || hovered_cell != self.hovered_cell {
                    self.is_toggle_hovered = is_hovered;
                    self.hovered_cell = hovered_cell;
                    context.request_repaint();
                }
            }
            PointerEvent::Pressed(point) if frame.is_open() && is_available => {
                if toggle.contains(point) {
                    self.shows_days = !self.shows_days;
                    self.hovered_cell = None;
                    context.request_repaint();
                } else if let Some(url) = self.cell_at(frame, point).and_then(|cell| self.cell_url(cell)) {
                    crate::platform::open(&url);
                }
            }
            PointerEvent::Left | PointerEvent::CaptureLost => {
                let had_hover = std::mem::take(&mut self.is_toggle_hovered) | self.hovered_cell.take().is_some();
                if had_hover {
                    context.request_repaint();
                }
            }
            _ => {}
        }
    }
}
