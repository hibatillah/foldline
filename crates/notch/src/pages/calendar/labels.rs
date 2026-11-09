//! Text the day grid and the empty message draw, built when the day changes instead of on every paint.

use super::dates::{DayGrid, WEEKDAY_NAMES, civil_from_days, weekday};

#[derive(Default)]
pub(super) struct Labels {
    pub year: String,
    /// The date of each day of the grid, in order.
    pub day_numbers: Vec<String>,
    pub empty_title: String,
}

impl Labels {
    pub fn new(today: i64, selected_day: i64, grid: &DayGrid) -> Self {
        let (year, _, _) = civil_from_days(today);
        let (_, _, selected_date) = civil_from_days(selected_day);
        Self {
            year: year.to_string(),
            day_numbers: (0..i64::from(grid.day_count))
                .map(|offset| civil_from_days(grid.first_day + offset).2.to_string())
                .collect(),
            empty_title: format!("No events on {} {selected_date}", WEEKDAY_NAMES[weekday(selected_day) as usize]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::calendar::dates::days_from_civil;

    #[test]
    fn the_labels_hold_the_year_the_dates_of_the_month_and_the_selected_day() {
        let friday = days_from_civil(2026, 11, 13);
        let labels = Labels::new(friday, friday, &DayGrid::month_of(friday));
        assert_eq!(labels.year, "2026");
        assert_eq!(labels.day_numbers.len(), 30);
        assert_eq!((labels.day_numbers[0].as_str(), labels.day_numbers[29].as_str()), ("1", "30"));
        assert_eq!(labels.empty_title, "No events on Friday 13");
        assert_eq!(Labels::new(friday, friday + 1, &DayGrid::month_of(friday)).empty_title, "No events on Saturday 14");
    }

    #[test]
    fn a_week_that_crosses_a_month_edge_numbers_its_days_by_date() {
        let friday = days_from_civil(2026, 10, 30);
        let labels = Labels::new(friday, friday, &DayGrid::week_of(friday));
        assert_eq!(labels.day_numbers, ["26", "27", "28", "29", "30", "31", "1"]);
    }
}
