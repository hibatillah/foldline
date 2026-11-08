//! Text the month grid and the empty message draw, built when the day changes instead of on every paint.

use super::dates::{MonthGrid, WEEKDAY_NAMES, civil_from_days, weekday};

#[derive(Default)]
pub(super) struct Labels {
    pub year: String,
    /// The dates of the month, "1" to the last day.
    pub day_numbers: Vec<String>,
    pub empty_title: String,
}

impl Labels {
    pub fn new(today: i64, selected_day: i64) -> Self {
        let (year, _, _) = civil_from_days(today);
        let month = MonthGrid::of(today);
        let (_, _, selected_date) = civil_from_days(selected_day);
        Self {
            year: year.to_string(),
            day_numbers: (1..=month.day_count).map(|date| date.to_string()).collect(),
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
        let labels = Labels::new(friday, friday);
        assert_eq!(labels.year, "2026");
        assert_eq!(labels.day_numbers.len(), 30);
        assert_eq!((labels.day_numbers[0].as_str(), labels.day_numbers[29].as_str()), ("1", "30"));
        assert_eq!(labels.empty_title, "No events on Friday 13");
        assert_eq!(Labels::new(friday, friday + 1).empty_title, "No events on Saturday 14");
    }
}
