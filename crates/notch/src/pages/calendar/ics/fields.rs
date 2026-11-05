//! Reading lines and values of an ICS file.

use super::super::dates::{SECONDS_PER_DAY, seconds_from_civil};

pub(super) fn unfold(text: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for line in text.lines() {
        match (line.strip_prefix([' ', '\t']), lines.last_mut()) {
            (Some(rest), Some(previous)) => previous.push_str(rest),
            _ => lines.push(line.to_owned()),
        }
    }
    lines
}

pub(super) struct Property {
    pub(super) name: String,
    pub(super) parameters: Vec<(String, String)>,
    pub(super) value: String,
}

pub(super) fn parse_property(line: &str) -> Option<Property> {
    let mut in_quotes = false;
    let split = line.char_indices().find(|&(_, character)| {
        if character == '"' {
            in_quotes = !in_quotes;
        }
        character == ':' && !in_quotes
    })?;
    let (head, value) = (&line[..split.0], &line[split.0 + 1..]);
    let mut parts = head.split(';');
    let name = parts.next()?.to_uppercase();
    let parameters = parts
        .filter_map(|part| part.split_once('='))
        .map(|(key, parameter)| (key.to_uppercase(), parameter.trim_matches('"').to_owned()))
        .collect();
    Some(Property { name, parameters, value: value.to_owned() })
}

impl Property {
    pub(super) fn parameter(&self, key: &str) -> Option<&str> {
        self.parameters.iter().find(|(name, _)| name == key).map(|(_, value)| value.as_str())
    }
}

pub(super) fn unescape(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            result.push(character);
            continue;
        }
        match characters.next() {
            Some('n') | Some('N') => result.push('\n'),
            Some(other) => result.push(other),
            None => {}
        }
    }
    result
}

pub(super) fn parse_moment(value: &str) -> Option<(i64, bool, bool)> {
    let value = value.trim();
    let number = |from: usize, to: usize| value.get(from..to)?.parse::<u32>().ok();
    let (year, month, day) = (i64::from(number(0, 4)?), number(4, 6)?, number(6, 8)?);
    if value.len() == 8 {
        return Some((seconds_from_civil(year, month, day, 0, 0, 0), false, true));
    }
    if value.get(8..9) != Some("T") {
        return None;
    }
    let seconds = seconds_from_civil(year, month, day, number(9, 11)?, number(11, 13)?, number(13, 15).unwrap_or(0));
    Some((seconds, value.ends_with('Z'), false))
}

pub(super) fn parse_offset(value: &str) -> Option<i64> {
    let value = value.trim();
    let sign = match value.chars().next()? {
        '+' => 1,
        '-' => -1,
        _ => return None,
    };
    let number = |from: usize, to: usize| value.get(from..to)?.parse::<i64>().ok();
    Some(sign * (number(1, 3)? * 3600 + number(3, 5)? * 60 + number(5, 7).unwrap_or(0)))
}

pub(super) fn parse_duration(value: &str) -> Option<i64> {
    let value = value.trim();
    let (negative, value) = match value.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, value.strip_prefix('+').unwrap_or(value)),
    };
    let value = value.strip_prefix('P')?;
    let (mut total, mut digits, mut in_time) = (0i64, String::new(), false);
    for character in value.chars() {
        match character {
            'T' => in_time = true,
            '0'..='9' => digits.push(character),
            unit => {
                let amount: i64 = digits.parse().ok()?;
                digits.clear();
                total += amount
                    * match (unit, in_time) {
                        ('W', _) => 7 * SECONDS_PER_DAY,
                        ('D', _) => SECONDS_PER_DAY,
                        ('H', true) => 3600,
                        ('M', true) => 60,
                        ('S', true) => 1,
                        _ => return None,
                    };
            }
        }
    }
    Some(if negative { -total } else { total })
}
