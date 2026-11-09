use super::{
    BADGE, BADGE_ICON_GAP, BADGE_ICON_SIZE, BADGE_TIME, BADGE_TITLE_GAP, BLUE, CalendarPage, REMINDER_TITLE_WIDTH,
};
use crate::{
    gfx::{Canvas, Color, Point, Rect},
    ui::Icon,
};

impl CalendarPage {
    pub(super) fn draw_badge(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32) {
        let Some(reminder) = &self.reminder else { return };
        let icon_center = Point::new(area.left + BADGE_ICON_SIZE * scale / 2.0, area.center_y());
        Icon::Calendar.draw(canvas, icon_center, BADGE_ICON_SIZE * scale, Color::WHITE, opacity);
        let title_width = (canvas.measure(&BADGE, &reminder.title).min(REMINDER_TITLE_WIDTH)).ceil() * scale;
        let title_left = area.left + (BADGE_ICON_SIZE + BADGE_ICON_GAP) * scale;
        canvas.text(
            &BADGE,
            &reminder.title,
            Rect::new(title_left, area.top, title_left + title_width + 2.0 * scale, area.bottom),
            Color::WHITE.faded(opacity),
        );
        let time_left = title_left + title_width + BADGE_TITLE_GAP * scale;
        canvas.text(
            &BADGE_TIME,
            &reminder.text,
            Rect::new(time_left, area.top, area.right, area.bottom),
            BLUE.faded(opacity),
        );
    }

    pub(super) fn draw_short_badge(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32) {
        let Some(reminder) = &self.reminder else { return };
        let icon_center = Point::new(area.left + BADGE_ICON_SIZE * scale / 2.0, area.center_y());
        Icon::Calendar.draw(canvas, icon_center, BADGE_ICON_SIZE * scale, Color::WHITE, opacity);
        let time_area =
            Rect::new(area.left + (BADGE_ICON_SIZE + BADGE_ICON_GAP) * scale, area.top, area.right, area.bottom);
        canvas.text(&BADGE_TIME, reminder.short_text(), time_area, BLUE.faded(opacity));
    }
}
