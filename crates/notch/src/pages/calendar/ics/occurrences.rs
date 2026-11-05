//! Which events fall on a day, with repeats expanded.

use super::super::dates::{
    SECONDS_PER_DAY, civil_from_days, day_of, days_from_civil, days_in_month, nth_weekday_of_month, same_day_in_month,
    week_start, weekday,
};
use super::{Clock, Frequency, Zone};
use crate::pages::calendar::Event;
use crate::pages::calendar::Occurrence;

pub(super) const MAX_STEPS: i64 = 200_000;

/// The start (wall seconds) of every repeat of `event` from `from` to `to`, the first one included.
pub(super) fn repeat_starts(event: &Event, from: i64, to: i64) -> Vec<i64> {
    let Some(rule) = &event.rule else { return vec![event.start] };
    let time_of_day = event.start.rem_euclid(SECONDS_PER_DAY);
    let start_day = day_of(event.start);
    let (from_day, to_day) = (day_of(from), day_of(to));
    let mut days: Vec<i64> = Vec::new();
    let mut produced = 0i64;
    let limit = rule.count.unwrap_or(i64::MAX);
    let has_ended = |day: i64| rule.until.is_some_and(|until| day * SECONDS_PER_DAY + time_of_day > until);

    match rule.frequency {
        Frequency::Daily => {
            let first = if rule.count.is_none() { ((from_day - start_day) / rule.interval - 1).max(0) } else { 0 };
            for step in first..first + MAX_STEPS {
                let day = start_day + step * rule.interval;
                if step >= limit || day > to_day || has_ended(day) {
                    break;
                }
                days.push(day);
            }
        }
        Frequency::Weekly => {
            let base = week_start(start_day);
            let weekdays: Vec<u32> = if rule.by_day.is_empty() {
                vec![weekday(start_day)]
            } else {
                let mut listed: Vec<u32> = rule.by_day.iter().map(|&(_, weekday)| weekday).collect();
                listed.sort_unstable();
                listed
            };
            let first = if rule.count.is_none() { ((from_day - base) / (7 * rule.interval) - 1).max(0) } else { 0 };
            'weeks: for week in first..first + MAX_STEPS {
                let week_day = base + week * 7 * rule.interval;
                if week_day > to_day {
                    break;
                }
                for &listed in &weekdays {
                    let day = week_day + i64::from(listed);
                    if day < start_day {
                        continue;
                    }
                    if has_ended(day) || produced >= limit {
                        break 'weeks;
                    }
                    produced += 1;
                    days.push(day);
                }
            }
        }
        Frequency::Monthly | Frequency::Yearly => {
            let (start_year, start_month, start_date) = civil_from_days(start_day);
            let months_per_step = if rule.frequency == Frequency::Yearly { 12 * rule.interval } else { rule.interval };
            let months_before = (civil_from_days(from_day).0 - start_year) * 12
                + i64::from(civil_from_days(from_day).1)
                - i64::from(start_month);
            let first = if rule.count.is_none() { (months_before / months_per_step - 1).max(0) } else { 0 };
            'months: for step in first..first + MAX_STEPS {
                let months = step * months_per_step;
                let index = start_year * 12 + i64::from(start_month) - 1 + months;
                let (year, month) = (index.div_euclid(12), (index.rem_euclid(12) + 1) as u32);
                if days_from_civil(year, month, 1) > to_day {
                    break;
                }
                let mut in_month: Vec<i64> = Vec::new();
                if rule.frequency == Frequency::Monthly && !rule.by_day.is_empty() {
                    for &(ordinal, listed) in &rule.by_day {
                        if ordinal == 0 {
                            let first_day = days_from_civil(year, month, 1);
                            in_month.extend(
                                (0..i64::from(days_in_month(year, month)))
                                    .map(|offset| first_day + offset)
                                    .filter(|&day| weekday(day) == listed),
                            );
                        } else {
                            in_month.extend(nth_weekday_of_month(year, month, listed, ordinal));
                        }
                    }
                } else if rule.frequency == Frequency::Monthly && !rule.by_month_day.is_empty() {
                    for &number in &rule.by_month_day {
                        let length = days_in_month(year, month) as i32;
                        let day = if number > 0 { number } else { length + 1 + number };
                        if (1..=length).contains(&day) {
                            in_month.push(days_from_civil(year, month, day as u32));
                        }
                    }
                } else {
                    in_month.extend(same_day_in_month(start_year, start_month, start_date, months));
                }
                in_month.sort_unstable();
                for day in in_month {
                    if day < start_day {
                        continue;
                    }
                    if has_ended(day) || produced >= limit {
                        break 'months;
                    }
                    produced += 1;
                    days.push(day);
                }
            }
        }
    }
    days.into_iter()
        .map(|day| day * SECONDS_PER_DAY + time_of_day)
        .filter(|&start| start >= from && start <= to)
        .collect()
}

pub(super) fn to_local(wall: i64, zone: Zone, clock: &dyn Clock) -> i64 {
    match zone {
        Zone::Floating => wall,
        Zone::Utc => clock.utc_to_local(wall),
        Zone::Offset(offset) => clock.utc_to_local(wall - offset),
    }
}

pub fn occurrences_on(events: &[Event], day: i64, clock: &dyn Clock) -> Vec<Occurrence> {
    let (window_start, window_end) = (day * SECONDS_PER_DAY, (day + 1) * SECONDS_PER_DAY);
    let replaced: Vec<(&str, i64)> =
        events.iter().filter_map(|event| event.replaces.map(|start| (event.uid.as_str(), start))).collect();
    let mut found = Vec::new();
    for event in events.iter().filter(|event| !event.is_cancelled) {
        let length = event.end - event.start;
        // Wall time can differ from local time by a day either way, so look a day further on both sides.
        let from = window_start - length - 2 * SECONDS_PER_DAY;
        let to = window_end + 2 * SECONDS_PER_DAY;
        for start in repeat_starts(event, from, to) {
            let is_removed = event.exceptions.contains(&start) || replaced.contains(&(event.uid.as_str(), start));
            if is_removed && event.replaces.is_none() {
                continue;
            }
            let local_start = if event.is_all_day { start } else { to_local(start, event.zone, clock) };
            let local_end = local_start + length;
            let overlaps = local_start < window_end && local_end > window_start;
            let is_instant_inside = length == 0 && (window_start..window_end).contains(&local_start);
            if overlaps || is_instant_inside {
                found.push(Occurrence {
                    start: local_start,
                    end: local_end,
                    is_all_day: event.is_all_day,
                    summary: event.summary.clone(),
                    location: event.location.clone(),
                    description: event.description.clone(),
                    url: event.url.clone(),
                });
            }
        }
    }
    found.sort_by_key(|occurrence| (!occurrence.is_all_day, occurrence.start, occurrence.end));
    found
}
