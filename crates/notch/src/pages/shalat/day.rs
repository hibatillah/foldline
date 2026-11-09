//! The open page: the next prayer at the left and the day's arc at the right.

use salah::{DateTime, Utc};

use super::{
    LIGHT, ShalatPage,
    arc::{self, Marker, PRAYER_COUNT, PRAYER_SLOTS},
    schedule::Schedule,
    text::{clock_text, remaining_text},
    times::PRAYERS,
};
use crate::ui::palette::{MINT_GREEN, TEXT_DIM, TEXT_MUTED};
use crate::ui::type_styles::{MEDIUM_11_CENTER, MEDIUM_12, REGULAR_10, REGULAR_11};
use crate::{
    gfx::{Align, Canvas, Color, FontStyle, Rect, UI_FONT, Weight},
    page::Frame,
};

const SIDE_WIDTH: f32 = 124.0;
const SIDE_GAP: f32 = 16.0;
const SOON_SECONDS: i64 = 10 * 60;
const CITY_GAP: f32 = 3.0;
const LINE_GAP: f32 = 1.0;
const LABEL_HEIGHT: f32 = 13.0;
const NAME_HEIGHT: f32 = 30.0;
const COUNTDOWN_HEIGHT: f32 = 15.0;
const CITY_HEIGHT: f32 = 12.0;
const MARKER_LABEL_TOP: f32 = 54.0;
const MARKER_LABEL_BOTTOM: f32 = 66.0;
const MARKER_TIME_TOP: f32 = 70.0;
const MARKER_TIME_BOTTOM: f32 = 84.0;

const NAME: FontStyle = FontStyle::new(UI_FONT, 26.0, Weight::Medium, Align::Leading);
const MARKER_NAME: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Center);
const MARKER_TIME: FontStyle = FontStyle::new(UI_FONT, 12.0, Weight::Regular, Align::Center).tabular();

/// "Next", or "Starts soon" in the last ten minutes, or "Tomorrow" once the day's prayers are over.
pub(super) fn heading(is_tomorrow: bool, seconds_left: i64) -> &'static str {
    if is_tomorrow {
        "Tomorrow"
    } else if seconds_left < SOON_SECONDS {
        "Starts soon"
    } else {
        "Next"
    }
}

/// How each of the five prayers looks at `now`; the next prayer is the one at `next_time`, which is tomorrow's Fajr
/// when the day's prayers are over.
pub(super) fn markers(
    today: &[DateTime<Utc>; 6],
    now: DateTime<Utc>,
    next_time: DateTime<Utc>,
    is_tomorrow: bool,
) -> [Marker; PRAYER_COUNT] {
    std::array::from_fn(|index| {
        let time = today[PRAYER_SLOTS[index]];
        if is_tomorrow {
            if index == 0 { Marker::Next } else { Marker::Passed }
        } else if time == next_time {
            Marker::Next
        } else if time <= now {
            Marker::Passed
        } else {
            Marker::Ahead
        }
    })
}

impl ShalatPage {
    pub(super) fn draw_day(&self, canvas: &Canvas, schedule: &Schedule, frame: &Frame, opacity: f32) {
        let (bounds, scale) = (frame.bounds, frame.scale);
        let now = self.now();
        let (prayer, time) = schedule.next;
        let seconds_left = (time - now).num_seconds();
        let is_tomorrow = time > schedule.today[5];
        let lines: [(&FontStyle, String, Color, f32); 4] = [
            (&REGULAR_11, heading(is_tomorrow, seconds_left).to_owned(), TEXT_MUTED, LABEL_HEIGHT),
            (&NAME, prayer.name().to_owned(), Color::WHITE, NAME_HEIGHT),
            (&MEDIUM_12, remaining_text(seconds_left), MINT_GREEN, COUNTDOWN_HEIGHT),
            (&REGULAR_10, self.config.city.clone(), TEXT_DIM, CITY_HEIGHT),
        ];
        let total = lines.iter().map(|line| line.3).sum::<f32>() + LINE_GAP * 3.0 + CITY_GAP;
        let mut top = bounds.center_y() - total * scale / 2.0;
        for (index, (style, text, color, height)) in lines.into_iter().enumerate() {
            if index == 3 {
                top += CITY_GAP * scale;
            }
            let line = Rect::new(bounds.left, top, bounds.left + SIDE_WIDTH * scale, top + height * scale);
            canvas.text(style, &text, line, color.faded(opacity));
            top = line.bottom + LINE_GAP * scale;
        }

        let chart = Rect { left: bounds.left + (SIDE_WIDTH + SIDE_GAP) * scale, ..bounds };
        let chart_width = chart.width() / scale;
        let states = markers(&schedule.today, now, time, is_tomorrow);
        arc::draw(canvas, chart, arc::position(&schedule.today, now), states, opacity);
        let slot_width = chart_width / PRAYER_COUNT as f32;
        for (index, state) in states.into_iter().enumerate() {
            let center_x = chart.left + arc::marker_x(index, chart_width) * scale;
            let column = |top: f32, bottom: f32| {
                Rect::new(
                    center_x - slot_width * scale / 2.0,
                    chart.top + top * scale,
                    center_x + slot_width * scale / 2.0,
                    chart.top + bottom * scale,
                )
            };
            let slot = PRAYER_SLOTS[index];
            let (name_style, name_color, time_color) = match state {
                Marker::Next => (&MEDIUM_11_CENTER, MINT_GREEN, Color::WHITE),
                Marker::Passed => (&MARKER_NAME, TEXT_DIM, TEXT_DIM),
                Marker::Ahead => (&MARKER_NAME, TEXT_MUTED, LIGHT),
            };
            canvas.text(
                name_style,
                PRAYERS[slot].name(),
                column(MARKER_LABEL_TOP, MARKER_LABEL_BOTTOM),
                name_color.faded(opacity),
            );
            let shown_time = if is_tomorrow && index == 0 { time } else { schedule.today[slot] };
            canvas.text(
                &MARKER_TIME,
                &clock_text(shown_time),
                column(MARKER_TIME_TOP, MARKER_TIME_BOTTOM),
                time_color.faded(opacity),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salah::Duration;

    fn day() -> [DateTime<Utc>; 6] {
        let start = DateTime::parse_from_rfc3339("2026-11-05T00:00:00Z").unwrap().with_timezone(&Utc);
        [4, 6, 12, 15, 18, 19].map(|hour| start + Duration::hours(hour))
    }

    #[test]
    fn the_heading_says_soon_in_the_last_ten_minutes_and_tomorrow_after_isha() {
        assert_eq!(heading(false, 3 * 3600), "Next");
        assert_eq!(heading(false, 8 * 60), "Starts soon");
        assert_eq!(heading(false, 10 * 60), "Next");
        assert_eq!(heading(true, 8 * 3600), "Tomorrow");
    }

    #[test]
    fn the_markers_split_into_passed_next_and_ahead() {
        let times = day();
        let now = times[2] + Duration::minutes(30);
        assert_eq!(
            markers(&times, now, times[3], false),
            [Marker::Passed, Marker::Passed, Marker::Next, Marker::Ahead, Marker::Ahead]
        );
    }

    #[test]
    fn after_isha_every_marker_is_passed_but_tomorrows_fajr() {
        let times = day();
        let now = times[5] + Duration::hours(1);
        assert_eq!(
            markers(&times, now, times[0] + Duration::days(1), true),
            [Marker::Next, Marker::Passed, Marker::Passed, Marker::Passed, Marker::Passed]
        );
    }

    #[test]
    fn before_fajr_the_morning_prayer_is_next_and_the_rest_are_ahead() {
        let times = day();
        let now = times[0] - Duration::hours(1);
        assert_eq!(
            markers(&times, now, times[0], false),
            [Marker::Next, Marker::Ahead, Marker::Ahead, Marker::Ahead, Marker::Ahead]
        );
    }
}
