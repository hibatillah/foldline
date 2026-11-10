//! The hour and day cells as buttons: which cell is under the pointer and the forecast each one opens.

use super::{WeatherPage, draw::COLUMNS, link};
use crate::{
    gfx::{Point, Rect},
    page::Frame,
};

/// The cell whose area holds `point`, left to right; the gaps between cells hold none.
pub(super) fn column_at(bounds: Rect, point: Point) -> Option<usize> {
    (0..COLUMNS).find(|&column| WeatherPage::column_area(bounds, column).contains(point))
}

impl WeatherPage {
    /// The cell under `point` that has something to show.
    pub(super) fn cell_at(&self, frame: &Frame, point: Point) -> Option<usize> {
        let column = column_at(frame.bounds, point)?;
        self.cell_url(column).map(|_| column)
    }

    /// Today: the hour by hour view. Week: the ten day view.
    pub(super) fn cell_url(&self, column: usize) -> Option<String> {
        let (forecast, location) = (self.forecast.as_ref()?, self.location.as_ref()?);
        let (latitude, longitude) = (location.latitude, location.longitude);
        if self.shows_days {
            forecast.days.get(column)?;
            Some(link::tenday_url(latitude, longitude))
        } else {
            forecast.hours.get(forecast.first_hour_index(self.now()) + column)?;
            Some(link::hourly_url(latitude, longitude))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::weather::{CONTENT_HEIGHT, CONTENT_WIDTH};

    const BOUNDS: Rect = Rect::new(0.0, 0.0, CONTENT_WIDTH, CONTENT_HEIGHT);

    #[test]
    fn a_point_in_a_cell_finds_its_column() {
        for column in 0..COLUMNS {
            let area = WeatherPage::column_area(BOUNDS, column);
            assert_eq!(column_at(BOUNDS, Point::new(area.center_x(), area.center_y())), Some(column));
        }
    }

    #[test]
    fn the_left_column_and_the_gaps_hit_no_cell() {
        let first = WeatherPage::column_area(BOUNDS, 0);
        let second = WeatherPage::column_area(BOUNDS, 1);
        assert_eq!(column_at(BOUNDS, Point::new(40.0, 40.0)), None);
        assert_eq!(column_at(BOUNDS, Point::new((first.right + second.left) / 2.0, 40.0)), None);
    }

    #[test]
    fn an_hour_opens_the_hour_by_hour_view_and_a_day_opens_the_ten_day_view() {
        let mut page = WeatherPage::sample(false);
        assert_eq!(page.cell_url(0).unwrap(), "https://weather.com/weather/hourbyhour/l/0.517,101.442");
        page.shows_days = true;
        let last = page.forecast.as_ref().unwrap().days.len() - 1;
        assert_eq!(page.cell_url(last).unwrap(), "https://weather.com/weather/tenday/l/0.517,101.442");
    }

    #[test]
    fn a_cell_without_data_opens_nothing() {
        let page = WeatherPage::sample(false);
        assert_eq!(page.cell_url(COLUMNS + 40), None);
        assert_eq!(WeatherPage::new().cell_url(0), None);
    }
}
