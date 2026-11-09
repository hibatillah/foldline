use super::{CalendarPage, REFRESH_SETTING};
use crate::page::Context;

pub(super) fn minutes_to_milliseconds(minutes: u64) -> u32 {
    minutes.saturating_mul(60_000).min(u64::from(u32::MAX)) as u32
}

/// A missing, unreadable or zero value keeps `fallback`.
pub(super) fn fetch_interval(minutes_text: Option<&str>, fallback: u32) -> u32 {
    minutes_text
        .and_then(|text| text.parse::<u64>().ok())
        .filter(|minutes| *minutes > 0)
        .map_or(fallback, minutes_to_milliseconds)
}

#[derive(Debug, PartialEq)]
pub(super) enum LinkChange {
    Keep,
    Remove,
    Replace(String),
}

/// A missing key keeps the link and an empty one removes it, as `remove_link` saves it.
pub(super) fn link_change(current: Option<&str>, saved: Option<&str>) -> LinkChange {
    match saved {
        None => LinkChange::Keep,
        Some("") if current.is_none() => LinkChange::Keep,
        Some("") => LinkChange::Remove,
        Some(link) if Some(link) == current => LinkChange::Keep,
        Some(link) => LinkChange::Replace(link.to_owned()),
    }
}

impl CalendarPage {
    pub(super) fn read_fetch_interval(&mut self, context: &Context) {
        self.fetch_interval_milliseconds =
            fetch_interval(context.settings.get_string(REFRESH_SETTING), self.fetch_interval_milliseconds);
    }

    pub(super) fn forget_link(&mut self) {
        self.link = None;
        self.events.clear();
        if let Some(storage) = &self.storage {
            let _ = std::fs::remove_file(storage);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::*, *};

    #[test]
    fn a_reloaded_link_is_kept_when_missing_removed_when_empty_and_replaced_when_new() {
        let link = Some("https://a.test/cal.ics");
        assert_eq!(link_change(link, None), LinkChange::Keep);
        assert_eq!(link_change(link, link), LinkChange::Keep);
        assert_eq!(link_change(link, Some("")), LinkChange::Remove);
        assert_eq!(link_change(None, Some("")), LinkChange::Keep);
        assert_eq!(link_change(link, Some("https://b.test")), LinkChange::Replace("https://b.test".to_owned()));
        assert_eq!(link_change(None, Some("https://b.test")), LinkChange::Replace("https://b.test".to_owned()));
    }

    #[test]
    fn an_unreadable_refresh_interval_keeps_the_current_one() {
        assert_eq!(fetch_interval(Some("5"), 1), 300_000);
        for text in [None, Some("soon"), Some("0"), Some("-3")] {
            assert_eq!(fetch_interval(text, 42), 42, "{text:?}");
        }
    }
}
