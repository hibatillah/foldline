use super::super::dates::{SECONDS_PER_DAY, days_from_civil};

use super::*;

const CLOCK: FixedClock = FixedClock(7 * 3600);

fn calendar(body: &str) -> String {
    format!("BEGIN:VCALENDAR\r\nVERSION:2.0\r\n{body}END:VCALENDAR\r\n")
}

fn day(year: i64, month: u32, date: u32) -> i64 {
    days_from_civil(year, month, date)
}

fn titles(events: &[Event], when: i64) -> Vec<String> {
    occurrences_on(events, when, &CLOCK).into_iter().map(|occurrence| occurrence.summary).collect()
}

#[test]
fn a_utc_event_shows_in_local_time() {
    let events = parse(&calendar(
        "BEGIN:VEVENT\r\nUID:a\r\nDTSTART:20261024T063000Z\r\nDTEND:20261024T073000Z\r\nSUMMARY:Design review\r\nEND:VEVENT\r\n",
    ));
    let found = occurrences_on(&events, day(2026, 10, 24), &CLOCK);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].summary, "Design review");
    assert_eq!(found[0].start % SECONDS_PER_DAY, 13 * 3600 + 30 * 60);
    assert_eq!(found[0].end - found[0].start, 3600);
    assert!(occurrences_on(&events, day(2026, 10, 25), &CLOCK).is_empty());
}

#[test]
fn a_late_utc_event_belongs_to_the_next_local_day() {
    let events = parse(&calendar("BEGIN:VEVENT\r\nDTSTART:20261024T200000Z\r\nSUMMARY:Late\r\nEND:VEVENT\r\n"));
    assert!(titles(&events, day(2026, 10, 24)).is_empty());
    assert_eq!(titles(&events, day(2026, 10, 25)), vec!["Late"]);
}

#[test]
fn a_zoned_event_uses_the_offset_of_its_timezone_block() {
    let events = parse(&calendar(
        "BEGIN:VTIMEZONE\r\nTZID:Asia/Jakarta\r\nBEGIN:STANDARD\r\nDTSTART:19700101T000000\r\nTZOFFSETFROM:+0700\r\nTZOFFSETTO:+0700\r\nEND:STANDARD\r\nEND:VTIMEZONE\r\n\
         BEGIN:VEVENT\r\nDTSTART;TZID=Asia/Jakarta:20261024T090000\r\nDTEND;TZID=Asia/Jakarta:20261024T093000\r\nSUMMARY:Standup\r\nEND:VEVENT\r\n",
    ));
    let found = occurrences_on(&events, day(2026, 10, 24), &CLOCK);
    assert_eq!(found[0].start % SECONDS_PER_DAY, 9 * 3600);
    assert_eq!(found[0].end - found[0].start, 1800);
}

#[test]
fn a_floating_time_is_the_wall_clock() {
    let events = parse(&calendar("BEGIN:VEVENT\r\nDTSTART:20261024T090000\r\nSUMMARY:Local\r\nEND:VEVENT\r\n"));
    assert_eq!(occurrences_on(&events, day(2026, 10, 24), &CLOCK)[0].start % SECONDS_PER_DAY, 9 * 3600);
}

#[test]
fn an_all_day_event_covers_its_days_only_and_comes_first() {
    let events = parse(&calendar(
        "BEGIN:VEVENT\r\nDTSTART:20261024T090000Z\r\nSUMMARY:Meeting\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nDTSTART;VALUE=DATE:20261024\r\nDTEND;VALUE=DATE:20261026\r\nSUMMARY:Trip\r\nEND:VEVENT\r\n",
    ));
    assert_eq!(titles(&events, day(2026, 10, 24)), vec!["Trip", "Meeting"]);
    assert_eq!(titles(&events, day(2026, 10, 25)), vec!["Trip"]);
    assert!(titles(&events, day(2026, 10, 26)).is_empty());
}

#[test]
fn a_duration_replaces_the_end() {
    let events = parse(&calendar(
        "BEGIN:VEVENT\r\nDTSTART:20261024T060000Z\r\nDURATION:PT1H30M\r\nSUMMARY:Long\r\nEND:VEVENT\r\n",
    ));
    let found = occurrences_on(&events, day(2026, 10, 24), &CLOCK);
    assert_eq!(found[0].end - found[0].start, 5400);
}

#[test]
fn text_is_unfolded_and_unescaped() {
    let events = parse(&calendar(
        "BEGIN:VEVENT\r\nDTSTART:20261024T060000Z\r\nSUMMARY:Plan\\, review\r\n  and ship\r\nDESCRIPTION:one\\ntwo\r\nEND:VEVENT\r\n",
    ));
    assert_eq!(events[0].summary, "Plan, review and ship");
    assert_eq!(events[0].description, "one\ntwo");
}

#[test]
fn cancelled_events_and_events_without_a_start_are_left_out() {
    let events = parse(&calendar(
        "BEGIN:VEVENT\r\nDTSTART:20261024T060000Z\r\nSTATUS:CANCELLED\r\nSUMMARY:Gone\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nSUMMARY:No start\r\nEND:VEVENT\r\n",
    ));
    assert!(titles(&events, day(2026, 10, 24)).is_empty());
}

#[test]
fn a_daily_repeat_shows_on_every_day_after_it_starts() {
    let events = parse(&calendar(
        "BEGIN:VEVENT\r\nDTSTART:20261020T020000Z\r\nRRULE:FREQ=DAILY\r\nSUMMARY:Daily\r\nEND:VEVENT\r\n",
    ));
    assert!(titles(&events, day(2026, 10, 19)).is_empty());
    assert_eq!(titles(&events, day(2026, 10, 20)), vec!["Daily"]);
    assert_eq!(titles(&events, day(2027, 3, 3)), vec!["Daily"]);
}

#[test]
fn a_repeat_with_a_count_or_an_end_date_stops() {
    let counted = parse(&calendar(
        "BEGIN:VEVENT\r\nDTSTART:20261020T020000Z\r\nRRULE:FREQ=DAILY;COUNT=3\r\nSUMMARY:Three\r\nEND:VEVENT\r\n",
    ));
    assert_eq!(titles(&counted, day(2026, 10, 22)), vec!["Three"]);
    assert!(titles(&counted, day(2026, 10, 23)).is_empty());
    let ended = parse(&calendar(
        "BEGIN:VEVENT\r\nDTSTART:20261020T020000Z\r\nRRULE:FREQ=DAILY;UNTIL=20261022T020000Z\r\nSUMMARY:Until\r\nEND:VEVENT\r\n",
    ));
    assert_eq!(titles(&ended, day(2026, 10, 22)), vec!["Until"]);
    assert!(titles(&ended, day(2026, 10, 23)).is_empty());
}

#[test]
fn a_weekly_repeat_on_chosen_weekdays_skips_the_others() {
    let events = parse(&calendar(
        "BEGIN:VEVENT\r\nDTSTART:20261019T020000Z\r\nRRULE:FREQ=WEEKLY;BYDAY=MO,WE\r\nSUMMARY:Class\r\nEND:VEVENT\r\n",
    ));
    assert_eq!(titles(&events, day(2026, 10, 21)), vec!["Class"]);
    assert!(titles(&events, day(2026, 10, 22)).is_empty());
    assert_eq!(titles(&events, day(2026, 11, 2)), vec!["Class"]);
    assert_eq!(titles(&events, day(2026, 11, 4)), vec!["Class"]);
}

#[test]
fn a_two_weekly_repeat_skips_the_weeks_in_between() {
    let events = parse(&calendar(
        "BEGIN:VEVENT\r\nDTSTART:20261019T020000Z\r\nRRULE:FREQ=WEEKLY;INTERVAL=2\r\nSUMMARY:Sprint\r\nEND:VEVENT\r\n",
    ));
    assert!(titles(&events, day(2026, 10, 26)).is_empty());
    assert_eq!(titles(&events, day(2026, 11, 2)), vec!["Sprint"]);
}

#[test]
fn a_monthly_repeat_keeps_the_day_of_the_month_or_the_nth_weekday() {
    let by_date = parse(&calendar(
        "BEGIN:VEVENT\r\nDTSTART:20261031T020000Z\r\nRRULE:FREQ=MONTHLY\r\nSUMMARY:Month end\r\nEND:VEVENT\r\n",
    ));
    assert!(titles(&by_date, day(2026, 11, 30)).is_empty());
    assert_eq!(titles(&by_date, day(2026, 12, 31)), vec!["Month end"]);
    let by_weekday = parse(&calendar(
        "BEGIN:VEVENT\r\nDTSTART:20261013T020000Z\r\nRRULE:FREQ=MONTHLY;BYDAY=2TU\r\nSUMMARY:Second Tuesday\r\nEND:VEVENT\r\n",
    ));
    assert_eq!(titles(&by_weekday, day(2026, 11, 10)), vec!["Second Tuesday"]);
    assert!(titles(&by_weekday, day(2026, 11, 17)).is_empty());
}

#[test]
fn a_yearly_repeat_comes_back_on_the_same_date() {
    let events = parse(&calendar(
        "BEGIN:VEVENT\r\nDTSTART;VALUE=DATE:20200229\r\nRRULE:FREQ=YEARLY\r\nSUMMARY:Leap\r\nEND:VEVENT\r\n",
    ));
    assert_eq!(titles(&events, day(2028, 2, 29)), vec!["Leap"]);
    assert!(titles(&events, day(2027, 2, 28)).is_empty());
}

#[test]
fn a_removed_repeat_is_skipped() {
    let events = parse(&calendar(
        "BEGIN:VEVENT\r\nDTSTART:20261020T020000Z\r\nRRULE:FREQ=DAILY\r\nEXDATE:20261022T020000Z\r\nSUMMARY:Daily\r\nEND:VEVENT\r\n",
    ));
    assert!(titles(&events, day(2026, 10, 22)).is_empty());
    assert_eq!(titles(&events, day(2026, 10, 23)), vec!["Daily"]);
}

#[test]
fn a_moved_repeat_replaces_the_original() {
    let events = parse(&calendar(
        "BEGIN:VEVENT\r\nUID:x\r\nDTSTART:20261020T020000Z\r\nRRULE:FREQ=DAILY\r\nSUMMARY:Daily\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:x\r\nRECURRENCE-ID:20261022T020000Z\r\nDTSTART:20261022T080000Z\r\nSUMMARY:Moved\r\nEND:VEVENT\r\n",
    ));
    let found = occurrences_on(&events, day(2026, 10, 22), &CLOCK);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].summary, "Moved");
    assert_eq!(found[0].start % SECONDS_PER_DAY, 15 * 3600);
}

#[test]
fn a_repeat_that_started_years_ago_is_found_quickly() {
    let events = parse(&calendar(
        "BEGIN:VEVENT\r\nDTSTART:20100104T020000Z\r\nRRULE:FREQ=WEEKLY;BYDAY=MO\r\nSUMMARY:Old\r\nEND:VEVENT\r\n",
    ));
    assert_eq!(titles(&events, day(2026, 10, 19)), vec!["Old"]);
}

#[test]
fn durations_parse_in_every_unit() {
    assert_eq!(parse_duration("PT1H30M"), Some(5400));
    assert_eq!(parse_duration("P1D"), Some(86_400));
    assert_eq!(parse_duration("P2W"), Some(1_209_600));
    assert_eq!(parse_duration("-PT15M"), Some(-900));
    assert_eq!(parse_duration("nonsense"), None);
}

#[test]
fn offsets_parse_with_either_sign() {
    assert_eq!(parse_offset("+0700"), Some(25_200));
    assert_eq!(parse_offset("-0330"), Some(-12_600));
    assert_eq!(parse_offset("0700"), None);
}

#[test]
fn garbage_gives_no_events() {
    assert!(parse("").is_empty());
    assert!(parse("not a calendar\r\nat all").is_empty());
}
