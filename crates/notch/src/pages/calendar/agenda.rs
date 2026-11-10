//! Calendar presentation uses local seconds.

use super::{
    dates::{SECONDS_PER_DAY, SECONDS_PER_MINUTE, clock_text},
    ics::Occurrence,
};

pub const REMINDER_LEAD_SECONDS: i64 = 15 * SECONDS_PER_MINUTE;
const REMINDER_TRAIL_MINUTES: i64 = 5;

pub struct Row {
    pub occurrence: Occurrence,
    /// "All day" or the start time in the time column.
    pub time: String,
}

pub struct Agenda {
    pub rows: Vec<Row>,
    /// Today had events and all of them are over.
    pub is_done: bool,
}

fn row_for(occurrence: &Occurrence) -> Row {
    let time = if occurrence.is_all_day {
        crate::i18n::tr!("calendar.all_day").into_owned()
    } else {
        clock_text(occurrence.start.rem_euclid(SECONDS_PER_DAY))
    };
    Row { occurrence: occurrence.clone(), time }
}

/// All-day events first, then by start time.
fn in_listing_order(occurrences: &[Occurrence]) -> Vec<&Occurrence> {
    let mut ordered: Vec<&Occurrence> = occurrences.iter().collect();
    ordered.sort_by_key(|occurrence| (!occurrence.is_all_day, occurrence.start));
    ordered
}

/// Today: the events that are not over yet.
pub fn build(occurrences: &[Occurrence], now: i64) -> Agenda {
    let upcoming: Vec<&Occurrence> = in_listing_order(occurrences)
        .into_iter()
        .filter(|occurrence| occurrence.end.max(occurrence.start + 1) > now)
        .collect();
    let rows = upcoming.into_iter().map(row_for).collect::<Vec<_>>();
    Agenda { is_done: rows.is_empty() && !occurrences.is_empty(), rows }
}

/// Another day: all its events, whether or not their time has passed.
pub fn list(occurrences: &[Occurrence]) -> Agenda {
    let rows = in_listing_order(occurrences).into_iter().map(row_for).collect();
    Agenda { rows, is_done: false }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reminder {
    pub title: String,
    pub text: String,
    /// `text` without the word around the minutes, for the shorter badge.
    pub short: String,
    /// When the event starts, in the same local seconds as the occurrences.
    pub start: i64,
}

impl Reminder {
    /// "12m" for "in 12m"; "now" stays.
    pub fn short_text(&self) -> &str {
        &self.short
    }
}

pub fn reminder(occurrences: &[Occurrence], now: i64) -> Option<Reminder> {
    let occurrence = occurrences.iter().filter(|occurrence| !occurrence.is_all_day).find(|occurrence| {
        occurrence.start - now <= REMINDER_LEAD_SECONDS
            && now - occurrence.start <= REMINDER_TRAIL_MINUTES * SECONDS_PER_MINUTE
    })?;
    let (text, short) = if occurrence.start <= now {
        let now_text = crate::i18n::tr!("calendar.reminder_now").into_owned();
        (now_text.clone(), now_text)
    } else {
        let minutes = (occurrence.start - now + SECONDS_PER_MINUTE - 1) / SECONDS_PER_MINUTE;
        (
            crate::i18n::tr!("calendar.reminder_in", minutes = minutes).into_owned(),
            crate::i18n::tr!("calendar.reminder_short", minutes = minutes).into_owned(),
        )
    };
    Some(Reminder { title: occurrence.summary.clone(), text, short, start: occurrence.start })
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOUR: i64 = 3600;
    const MINUTE: i64 = 60;
    const DAY_START: i64 = 20_000 * SECONDS_PER_DAY;

    fn event(start_hour: i64, minutes: i64, summary: &str) -> Occurrence {
        let start = DAY_START + start_hour * HOUR;
        Occurrence {
            start,
            end: start + minutes * MINUTE,
            is_all_day: false,
            summary: summary.to_owned(),
            location: String::new(),
            description: String::new(),
            url: String::new(),
        }
    }

    fn names(agenda: &Agenda) -> Vec<&str> {
        agenda.rows.iter().map(|row| row.occurrence.summary.as_str()).collect()
    }

    #[test]
    fn a_past_event_leaves_the_list_and_the_next_one_takes_its_place() {
        let day = [event(9, 30, "Standup"), event(13, 60, "Review"), event(16, 60, "Reading")];
        assert_eq!(names(&build(&day, DAY_START + 8 * HOUR)), vec!["Standup", "Review", "Reading"]);
        assert_eq!(names(&build(&day, DAY_START + 9 * HOUR + 40 * MINUTE)), vec!["Review", "Reading"]);
        assert_eq!(names(&build(&day, DAY_START + 13 * HOUR + 10 * MINUTE)), vec!["Review", "Reading"]);
    }

    #[test]
    fn rows_show_the_start_time_or_all_day() {
        let mut trip = event(0, 0, "Trip");
        trip.is_all_day = true;
        trip.end = trip.start + SECONDS_PER_DAY;
        let agenda = build(&[event(13, 60, "Review"), trip], DAY_START + 9 * HOUR);
        let times: Vec<&str> = agenda.rows.iter().map(|row| row.time.as_str()).collect();
        assert_eq!(times, vec!["All day", "13:00"], "an all-day event leads the list");
    }

    #[test]
    fn every_event_of_the_day_is_listed_for_the_list_to_scroll() {
        let day: Vec<Occurrence> = (8..17).map(|hour| event(hour, 30, &format!("Event {hour}"))).collect();
        let agenda = build(&day, DAY_START + 7 * HOUR);
        assert_eq!(agenda.rows.len(), 9);
        assert_eq!(agenda.rows[8].occurrence.summary, "Event 16");
        assert_eq!(list(&day).rows.len(), 9);
    }

    #[test]
    fn after_the_last_event_the_day_is_done_and_an_empty_day_is_not() {
        let done = build(&[event(9, 30, "Standup")], DAY_START + 20 * HOUR);
        assert!(done.rows.is_empty() && done.is_done);
        let empty = build(&[], DAY_START);
        assert!(empty.rows.is_empty() && !empty.is_done);
    }

    #[test]
    fn another_day_lists_all_its_events_even_when_their_time_has_passed() {
        let agenda = list(&[event(13, 60, "Review"), event(9, 30, "Standup")]);
        assert_eq!(names(&agenda), vec!["Standup", "Review"]);
        assert!(!agenda.is_done);
    }

    #[test]
    fn the_reminder_appears_fifteen_minutes_before_and_leaves_five_after() {
        let day = [event(13, 60, "Design review")];
        let start = day[0].start;
        assert_eq!(reminder(&day, start - 16 * MINUTE), None);
        assert_eq!(reminder(&day, start - 15 * MINUTE).unwrap().text, "in 15m");
        assert_eq!(reminder(&day, start - 12 * MINUTE + 10).unwrap().text, "in 12m");
        assert_eq!(reminder(&day, start).unwrap().text, "now");
        assert_eq!(reminder(&day, start + 5 * MINUTE).unwrap().text, "now");
        assert_eq!(reminder(&day, start + 5 * MINUTE + 1), None);
        assert_eq!(reminder(&day, start - 12 * MINUTE).unwrap().title, "Design review");
    }

    #[test]
    fn the_short_reminder_text_drops_the_word_in() {
        let day = [event(13, 60, "Design review")];
        let start = day[0].start;
        assert_eq!(reminder(&day, start - 12 * MINUTE).unwrap().short_text(), "12m");
        assert_eq!(reminder(&day, start).unwrap().short_text(), "now");
    }

    #[test]
    fn the_reminder_and_the_all_day_text_read_in_indonesian() {
        let day = [event(13, 60, "Design review")];
        let start = day[0].start;
        store::language::with_override(store::language::Language::Indonesian, || {
            let soon = reminder(&day, start - 12 * MINUTE).unwrap();
            assert_eq!((soon.text.as_str(), soon.short_text()), ("12m lagi", "12m"));
            assert_eq!(reminder(&day, start).unwrap().text, "sekarang");
            let mut trip = event(0, 0, "Trip");
            trip.is_all_day = true;
            assert_eq!(build(&[trip], DAY_START).rows[0].time, "Seharian");
        });
    }

    #[test]
    fn all_day_events_never_trigger_a_reminder() {
        let mut trip = event(0, 0, "Trip");
        trip.is_all_day = true;
        assert_eq!(reminder(&[trip], DAY_START), None);
    }
}
