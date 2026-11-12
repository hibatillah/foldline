use super::{dates::DayGrid, geometry::MONTH_VIEWPORT_HEIGHT};
use crate::page::DEFAULT_CONTENT_HEIGHT;

/// How much of the calendar the left column shows, set by `calendar.view`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum View {
    #[default]
    Month,
    Week,
}

impl View {
    /// A missing or unknown word keeps `current`.
    pub fn from_setting(text: Option<&str>, current: View) -> View {
        match text.map(str::trim) {
            Some("month") => View::Month,
            Some("week") => View::Week,
            _ => current,
        }
    }

    pub fn grid_of(self, day: i64) -> DayGrid {
        match self {
            View::Month => DayGrid::month_of(day),
            View::Week => DayGrid::week_of(day),
        }
    }

    /// The height of the events list; it is also the page's least height.
    pub fn viewport_height(self) -> f32 {
        match self {
            View::Month => MONTH_VIEWPORT_HEIGHT,
            View::Week => DEFAULT_CONTENT_HEIGHT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_setting_picks_a_view_and_an_unknown_word_keeps_the_current_one() {
        assert_eq!(View::from_setting(Some("week"), View::Month), View::Week);
        assert_eq!(View::from_setting(Some(" month "), View::Week), View::Month);
        assert_eq!(View::from_setting(Some("year"), View::Week), View::Week);
        assert_eq!(View::from_setting(None, View::Week), View::Week);
    }

    #[test]
    fn the_week_view_is_as_tall_as_the_default_page_and_the_month_view_keeps_its_height() {
        assert_eq!(View::Week.viewport_height(), DEFAULT_CONTENT_HEIGHT);
        assert_eq!(View::Month.viewport_height(), 168.0);
    }

    #[test]
    fn each_view_builds_its_own_grid() {
        let today = 20_400;
        assert_eq!(View::Week.grid_of(today).day_count, 7);
        assert!(View::Month.grid_of(today).day_count >= 28);
    }
}
