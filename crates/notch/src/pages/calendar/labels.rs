//! Text the day grid and the empty message draw, built when the day changes instead of on every paint.

use super::dates::{DayGrid, civil_from_days, month_name, weekday, weekday_letter, weekday_name};
use crate::i18n::tr;

#[derive(Default)]
pub(super) struct Labels {
    pub year: String,
    /// The date of each day of the grid, in order.
    pub day_numbers: Vec<String>,
    pub empty_title: String,
    /// The month of today, its name in full.
    pub month: String,
    pub weekday_letters: Vec<String>,
    pub no_link_title: String,
    pub no_link_hint: String,
    pub done_title: String,
    pub done_detail: String,
    pub empty_detail: String,
}

impl Labels {
    pub fn new(today: i64, selected_day: i64, grid: &DayGrid) -> Self {
        let (year, month_number, _) = civil_from_days(today);
        let (_, _, selected_date) = civil_from_days(selected_day);
        Self {
            year: year.to_string(),
            day_numbers: (0..i64::from(grid.day_count))
                .map(|offset| civil_from_days(grid.first_day + offset).2.to_string())
                .collect(),
            empty_title: tr!(
                "calendar.empty_title",
                weekday = weekday_name(weekday(selected_day)),
                date = selected_date
            )
            .into_owned(),
            month: month_name(month_number),
            weekday_letters: (0..7).map(weekday_letter).collect(),
            no_link_title: tr!("calendar.no_link_title").into_owned(),
            no_link_hint: tr!("calendar.no_link_hint").into_owned(),
            done_title: tr!("calendar.done_title").into_owned(),
            done_detail: tr!("calendar.done_detail").into_owned(),
            empty_detail: tr!("calendar.empty_detail").into_owned(),
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

    #[test]
    fn the_empty_day_message_reads_in_indonesian() {
        let friday = days_from_civil(2026, 11, 13);
        store::language::with_override(store::language::Language::Indonesian, || {
            let labels = Labels::new(friday, friday, &DayGrid::month_of(friday));
            assert_eq!(labels.empty_title, "Tidak ada acara hari Jumat 13");
        });
    }
}
