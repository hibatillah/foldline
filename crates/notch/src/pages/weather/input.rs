use super::{WeatherPage, WeatherTarget};
use crate::page::{Context, Frame, PointerEvent};

impl WeatherPage {
    pub(super) fn handle_pointer(&mut self, event: PointerEvent, frame: &Frame, context: &mut Context) {
        let is_available = self.forecast.is_some() && !self.is_offline;
        let toggle = Self::toggle_area(frame.bounds);
        match event {
            PointerEvent::Moved(point) => {
                let target = if !is_available {
                    None
                } else if toggle.contains(point) {
                    Some(WeatherTarget::Toggle)
                } else {
                    self.cell_at(frame, point).map(WeatherTarget::Cell)
                };
                if self.hover.set(target) {
                    context.request_repaint();
                }
            }
            PointerEvent::Pressed(point) if frame.is_open() && is_available => {
                if toggle.contains(point) {
                    self.shows_days = !self.shows_days;
                    if matches!(self.hover.get(), Some(WeatherTarget::Cell(_))) {
                        self.hover.clear();
                    }
                    context.request_repaint();
                } else if let Some(url) = self.cell_at(frame, point).and_then(|cell| self.cell_url(cell)) {
                    crate::platform::open(&url);
                }
            }
            PointerEvent::Left | PointerEvent::CaptureLost if self.hover.clear() => context.request_repaint(),
            _ => {}
        }
    }
}
