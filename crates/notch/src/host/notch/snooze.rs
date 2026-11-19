//! Snoozing notifications: the notch stays usable, but notices (banners, messages, page alerts) are dropped.

use store::snooze::{SNOOZED_UNTIL_KEY, Snooze};
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

use super::{Notch, SNOOZE_TIMER_ID, Source, pause::LONGEST_TIMER_MILLISECONDS};
use crate::activity::Presence;

pub(super) struct SnoozeState {
    snooze: Snooze,
    /// Sources whose notice was dropped; each stays hidden until that notice ends, so the snooze's end replays nothing.
    dropped: Vec<Source>,
}

impl Default for SnoozeState {
    fn default() -> Self {
        Self { snooze: Snooze::Off, dropped: Vec::new() }
    }
}

impl SnoozeState {
    pub(super) fn hides(&self, source: Source) -> bool {
        self.dropped.contains(&source)
    }
}

/// What the snooze does with one source's output this refresh.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Verdict {
    Shows,
    /// A notice that started while snoozed, held back from now on.
    NewlyDropped,
    /// A notice held back earlier that has not ended yet, even if the snooze has.
    StillDropped,
}

/// A notice that starts while snoozed is dropped and stays dropped until it ends, even past the snooze's end; anything
/// that is not a notice always shows.
pub(super) fn judge_notice(dropped: &mut Vec<Source>, source: Source, is_notice: bool, is_snoozed: bool) -> Verdict {
    let position = dropped.iter().position(|&candidate| candidate == source);
    match (is_notice, position) {
        (false, Some(position)) => {
            dropped.swap_remove(position);
            Verdict::Shows
        }
        (false, None) => Verdict::Shows,
        (true, Some(_)) => Verdict::StillDropped,
        (true, None) if is_snoozed => {
            dropped.push(source);
            Verdict::NewlyDropped
        }
        (true, None) => Verdict::Shows,
    }
}

impl Notch {
    pub(crate) fn is_snoozed(&self) -> bool {
        self.snooze.snooze.is_active()
    }

    pub(crate) fn snooze(&self) -> Snooze {
        self.snooze.snooze
    }

    /// Writes the key and applies it; the tray calls this.
    pub fn set_snooze(&mut self, snooze: Snooze, now_unix: i64) {
        self.settings.set_string(SNOOZED_UNTIL_KEY, &snooze.as_text());
        self.apply_stored_snooze(now_unix);
        self.flush();
    }

    /// Applies the key as read from settings; a stale or unknown value is cleared and counts as not snoozed.
    pub(crate) fn apply_stored_snooze(&mut self, now_unix: i64) {
        let stored = self.settings.get_string(SNOOZED_UNTIL_KEY).unwrap_or("").to_owned();
        let snooze = Snooze::parse(Some(&stored), now_unix);
        if !stored.is_empty() && snooze == Snooze::Off {
            self.settings.set_string(SNOOZED_UNTIL_KEY, "");
        }
        self.snooze.snooze = snooze;
        self.arm_snooze_timer(now_unix);
    }

    /// A timer that fires a little early re-arms for the rest.
    pub(super) fn snooze_timer_fired(&mut self, now_unix: i64) {
        self.apply_stored_snooze(now_unix);
        self.flush();
    }

    fn arm_snooze_timer(&self, now_unix: i64) {
        if let Some(milliseconds) = self.snooze.snooze.remaining_milliseconds(now_unix) {
            unsafe { SetTimer(Some(self.window), SNOOZE_TIMER_ID, milliseconds.min(LONGEST_TIMER_MILLISECONDS), None) };
        } else {
            let _ = unsafe { KillTimer(Some(self.window), SNOOZE_TIMER_ID) };
        }
    }

    /// Before presences are gathered: marks the notices to drop and forgets those that ended. A dropped activity banner
    /// is dismissed as if it had timed out, so the activity keeps nothing to show later.
    pub(super) fn drop_snoozed_notices(&mut self) {
        let is_snoozed = self.is_snoozed();
        if !is_snoozed && self.snooze.dropped.is_empty() {
            return;
        }
        for index in 0..self.activities.len() {
            let slot = &self.activities[index];
            let source = Source::Activity(index);
            let presence = slot.activity.presence();
            let is_notice = slot.is_showing() && slot.activity.is_notice(presence);
            let verdict = judge_notice(&mut self.snooze.dropped, source, is_notice, is_snoozed);
            if verdict == Verdict::NewlyDropped && matches!(presence, Some(Presence::Banner { .. })) {
                self.with_activity(index, |activity, context| activity.dismiss_banner(context));
            }
        }
        for index in 0..self.pages.len() {
            let page = &self.pages[index];
            let is_notice = self.page_list.is_enabled(index) && page.is_notice(page.presence());
            judge_notice(&mut self.snooze.dropped, Source::Page(index), is_notice, is_snoozed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE: Source = Source::Page(0);

    #[test]
    fn a_notice_that_starts_while_snoozed_is_newly_dropped() {
        let mut dropped = Vec::new();
        assert_eq!(judge_notice(&mut dropped, PAGE, true, true), Verdict::NewlyDropped);
        assert_eq!(dropped, vec![PAGE]);
    }

    #[test]
    fn a_dropped_notice_is_newly_dropped_only_once() {
        let mut dropped = Vec::new();
        judge_notice(&mut dropped, PAGE, true, true);
        assert_eq!(judge_notice(&mut dropped, PAGE, true, true), Verdict::StillDropped);
    }

    #[test]
    fn a_notice_shows_when_not_snoozed() {
        let mut dropped = Vec::new();
        assert_eq!(judge_notice(&mut dropped, PAGE, true, false), Verdict::Shows);
        assert!(dropped.is_empty());
    }

    #[test]
    fn an_indicator_shows_while_snoozed() {
        let mut dropped = Vec::new();
        assert_eq!(judge_notice(&mut dropped, PAGE, false, true), Verdict::Shows);
    }

    #[test]
    fn a_dropped_notice_stays_hidden_after_the_snooze_ends_until_it_ends() {
        let mut dropped = vec![PAGE];
        assert_eq!(judge_notice(&mut dropped, PAGE, true, false), Verdict::StillDropped, "not replayed at the end");
        assert_eq!(judge_notice(&mut dropped, PAGE, false, false), Verdict::Shows, "its end forgets it");
        assert!(dropped.is_empty());
        assert_eq!(judge_notice(&mut dropped, PAGE, true, false), Verdict::Shows, "the next notice shows");
    }
}
