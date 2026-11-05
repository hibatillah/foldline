//! Building events from an ICS file's text.

use super::super::dates::SECONDS_PER_DAY;
use super::{
    Frequency, Property, Rule, WEEKDAY_CODES, Zone, parse_duration, parse_moment, parse_offset, parse_property,
    unescape, unfold,
};
use crate::pages::calendar::Event;
use std::collections::HashMap;

pub(super) type ZoneBlock = (String, Option<i64>, Option<i64>);

pub(super) fn offset_of(zone: Zone) -> i64 {
    match zone {
        Zone::Offset(offset) => offset,
        _ => 0,
    }
}

pub(super) struct Moment {
    pub(super) wall: i64,
    pub(super) zone: Zone,
    pub(super) is_date: bool,
}

pub(super) fn read_moment(property: &Property, zones: &HashMap<String, i64>) -> Option<Moment> {
    let (wall, is_utc, is_date) = parse_moment(&property.value)?;
    let zone = if is_utc {
        Zone::Utc
    } else if is_date {
        Zone::Floating
    } else {
        property
            .parameter("TZID")
            .and_then(|name| zones.get(name))
            .map_or(Zone::Floating, |&offset| Zone::Offset(offset))
    };
    Some(Moment { wall, zone, is_date })
}

/// Expresses a moment as wall seconds in `target`'s zone. A floating moment is already on the target's wall.
pub(super) fn in_zone(moment: &Moment, target: Zone) -> i64 {
    if moment.zone == Zone::Floating || target == Zone::Floating {
        return moment.wall;
    }
    let utc = moment.wall - offset_of(moment.zone);
    utc + offset_of(target)
}

pub(super) fn parse_rule(value: &str, zone: Zone) -> Option<Rule> {
    let mut rule = Rule {
        frequency: Frequency::Daily,
        interval: 1,
        count: None,
        until: None,
        by_day: Vec::new(),
        by_month_day: Vec::new(),
    };
    let mut has_frequency = false;
    for part in value.split(';') {
        let Some((key, text)) = part.split_once('=') else { continue };
        match key.to_uppercase().as_str() {
            "FREQ" => {
                rule.frequency = match text.to_uppercase().as_str() {
                    "DAILY" => Frequency::Daily,
                    "WEEKLY" => Frequency::Weekly,
                    "MONTHLY" => Frequency::Monthly,
                    "YEARLY" => Frequency::Yearly,
                    _ => return None,
                };
                has_frequency = true;
            }
            "INTERVAL" => rule.interval = text.parse().ok().filter(|interval| *interval > 0)?,
            "COUNT" => rule.count = text.parse().ok(),
            "UNTIL" => {
                let (seconds, is_utc, is_date) = parse_moment(text)?;
                let until = Moment {
                    wall: seconds + if is_date { SECONDS_PER_DAY - 1 } else { 0 },
                    zone: if is_utc { Zone::Utc } else { Zone::Floating },
                    is_date,
                };
                rule.until = Some(in_zone(&until, zone));
            }
            "BYDAY" => rule.by_day = text.split(',').filter_map(parse_by_day).collect(),
            "BYMONTHDAY" => rule.by_month_day = text.split(',').filter_map(|day| day.trim().parse().ok()).collect(),
            _ => {}
        }
    }
    has_frequency.then_some(rule)
}

pub(super) fn parse_by_day(text: &str) -> Option<(i32, u32)> {
    let text = text.trim().to_uppercase();
    let split = text.len().checked_sub(2)?;
    let code = text.get(split..)?;
    let weekday = WEEKDAY_CODES.iter().position(|known| *known == code)? as u32;
    let ordinal = match text.get(..split)? {
        "" => 0,
        prefix => prefix.parse().ok()?,
    };
    Some((ordinal, weekday))
}

#[derive(Default)]
pub(super) struct Draft {
    pub(super) properties: Vec<Property>,
}

pub(super) fn build_event(draft: &Draft, zones: &HashMap<String, i64>) -> Option<Event> {
    let find = |name: &str| draft.properties.iter().find(|property| property.name == name);
    let start_property = find("DTSTART")?;
    let start = read_moment(start_property, zones)?;
    let zone = start.zone;
    let is_all_day = start.is_date;
    let end = match (find("DTEND"), find("DURATION")) {
        (Some(property), _) => in_zone(&read_moment(property, zones)?, zone),
        (None, Some(property)) => start.wall + parse_duration(&property.value)?,
        (None, None) if is_all_day => start.wall + SECONDS_PER_DAY,
        (None, None) => start.wall,
    };
    let text = |name: &str| find(name).map(|property| unescape(&property.value)).unwrap_or_default();
    let exceptions = draft
        .properties
        .iter()
        .filter(|property| property.name == "EXDATE")
        .flat_map(|property| {
            property.value.split(',').filter_map(|value| {
                let single = Property {
                    name: property.name.clone(),
                    parameters: property.parameters.clone(),
                    value: value.to_owned(),
                };
                Some(in_zone(&read_moment(&single, zones)?, zone))
            })
        })
        .collect();
    Some(Event {
        uid: text("UID"),
        summary: text("SUMMARY"),
        location: text("LOCATION"),
        description: text("DESCRIPTION"),
        url: text("URL"),
        start: start.wall,
        end: end.max(start.wall),
        zone,
        is_all_day,
        rule: find("RRULE").and_then(|property| parse_rule(&property.value, zone)),
        exceptions,
        replaces: find("RECURRENCE-ID").and_then(|property| Some(in_zone(&read_moment(property, zones)?, zone))),
        is_cancelled: find("STATUS").is_some_and(|property| property.value.trim().eq_ignore_ascii_case("CANCELLED")),
    })
}

/// Reads every event of an iCalendar file; what cannot be read is skipped.
pub fn parse(text: &str) -> Vec<Event> {
    let mut zones: HashMap<String, i64> = HashMap::new();
    let mut drafts: Vec<Draft> = Vec::new();
    let (mut current_event, mut current_zone): (Option<Draft>, Option<ZoneBlock>) = (None, None);
    let mut section = String::new();
    for line in unfold(text) {
        let Some(property) = parse_property(&line) else { continue };
        match (property.name.as_str(), property.value.as_str()) {
            ("BEGIN", "VEVENT") => current_event = Some(Draft::default()),
            ("END", "VEVENT") => drafts.extend(current_event.take()),
            ("BEGIN", "VTIMEZONE") => current_zone = Some((String::new(), None, None)),
            ("END", "VTIMEZONE") => {
                if let Some((name, standard, daylight)) = current_zone.take()
                    && let Some(offset) = standard.or(daylight)
                {
                    zones.insert(name, offset);
                }
            }
            ("BEGIN", name) => section = name.to_uppercase(),
            ("END", _) => section.clear(),
            _ => {
                if let Some(draft) = current_event.as_mut() {
                    draft.properties.push(property);
                } else if let Some((name, standard, daylight)) = current_zone.as_mut() {
                    match (property.name.as_str(), section.as_str()) {
                        ("TZID", _) => *name = property.value.clone(),
                        ("TZOFFSETTO", "STANDARD") => *standard = standard.or(parse_offset(&property.value)),
                        ("TZOFFSETTO", "DAYLIGHT") => *daylight = daylight.or(parse_offset(&property.value)),
                        _ => {}
                    }
                }
            }
        }
    }
    drafts.iter().filter_map(|draft| build_event(draft, &zones)).collect()
}
