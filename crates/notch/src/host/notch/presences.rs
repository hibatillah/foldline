use super::{Notch, Source, resting_row};
use crate::host::activity_layout;
use crate::{
    activity::{Presence, Slot},
    log,
};

/// What activities and pages show in the collapsed pill now, rebuilt by `refresh_presences` after every event.
#[derive(Default)]
pub(super) struct ShownPresences {
    /// Every transient still asking to show, oldest first; the newest of them show.
    pub(super) transient_order: Vec<Source>,
    pub(super) transients: Vec<(Source, f32)>,
    /// The banner's owner, width and height.
    pub(super) banner: Option<(Source, f32, f32)>,
    pub(super) badges: Vec<(Source, f32)>,
    /// Whether a priority activity (`Activity::is_priority`) shows a transient level, which can override the hiding word.
    pub(super) has_priority: bool,
}

impl Notch {
    pub(super) fn refresh_presences(&mut self) -> bool {
        self.dismiss_banners_for_priority();
        let mut transients = Vec::new();
        let mut badges = Vec::new();
        let mut banners = Vec::new();
        let mut before = Vec::new();
        let mut after = Vec::new();
        let activity_presences = self
            .activities
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.is_enabled && slot.is_started)
            .map(|(index, slot)| (Source::Activity(index), slot.activity.presence()));
        let has_priority = self.activities.iter().any(|slot| {
            slot.is_enabled
                && slot.is_started
                && slot.activity.is_priority()
                && matches!(slot.activity.presence(), Some(Presence::Transient { .. }))
        });
        let page_presences = self
            .pages
            .iter()
            .enumerate()
            .filter(|(index, _)| self.page_list.is_enabled(*index))
            .map(|(index, page)| (Source::Page(index), page.presence()));
        for (source, presence) in activity_presences.chain(page_presences) {
            match presence {
                Some(Presence::Transient { width }) => transients.push((source, width)),
                Some(Presence::Badge { width }) => badges.push((source, width)),
                Some(Presence::Banner { width, height }) => banners.push((source, width, height)),
                Some(Presence::Companion { slot, width }) => {
                    if let Source::Activity(_) = source {
                        match slot {
                            Slot::Ring => before.push((source, width)),
                            Slot::Dots => after.push((source, width)),
                        }
                    }
                }
                None => {}
            }
        }
        let active: Vec<Source> = transients.iter().map(|&(source, _)| source).collect();
        activity_layout::track_order(&mut self.presences.transient_order, &active);
        let banner = banners.last().copied();
        let shown: Vec<(Source, f32)> = if banner.is_some() {
            Vec::new()
        } else {
            activity_layout::newest(&self.presences.transient_order)
                .iter()
                .filter_map(|&source| transients.iter().copied().find(|&(candidate, _)| candidate == source))
                .collect()
        };
        badges.sort_by_key(|&(source, _)| self.rank_of(source));
        let mut claimed = resting_row::claimed_group(
            &badges,
            |source| self.rank_of(source),
            |source| self.countdown_of(source),
            |source| self.is_music_badge(source),
        );
        for (source, width) in &mut claimed {
            if let Some(icon_width) = self.countdown_icon_width(*source) {
                *width = icon_width;
            }
        }
        let uses_short_badges = self.shortens_badges(&mut badges);
        let changed = shown != self.presences.transients
            || badges != self.presences.badges
            || banner != self.presences.banner
            || before != self.resting_row.ring
            || after != self.resting_row.dots
            || claimed != self.resting_row.claimed_group
            || uses_short_badges != self.resting_row.uses_short_badges
            || has_priority != self.presences.has_priority;
        if shown != self.presences.transients || badges != self.presences.badges || banner != self.presences.banner {
            let ids = |sources: &[(Source, f32)]| sources.iter().map(|&(source, _)| self.source_id(source)).collect();
            let (transient_ids, badge_ids): (Vec<_>, Vec<_>) = (ids(&shown), ids(&badges));
            let banner_id = banner.map(|(source, _, _)| self.source_id(source));
            log::debug(&activity_layout::presence_summary(&transient_ids, &badge_ids, banner_id));
        }
        self.resting_row.ring = before;
        self.resting_row.claimed_group = claimed;
        self.resting_row.uses_short_badges = uses_short_badges;
        self.resting_row.dots = after;
        self.presences.banner = banner;
        self.presences.transients = shown;
        self.presences.badges = badges;
        self.presences.has_priority = has_priority;
        changed
    }

    /// A priority transient (a volume or brightness level) brings down every activity banner at once; a lasting muted
    /// badge does not, and neither does an ordinary transient, which waits behind the banner.
    fn dismiss_banners_for_priority(&mut self) {
        let is_priority_transient = |slot: &super::ActivitySlot| {
            slot.is_enabled
                && slot.is_started
                && slot.activity.is_priority()
                && matches!(slot.activity.presence(), Some(Presence::Transient { .. }))
        };
        if !self.activities.iter().any(is_priority_transient) {
            return;
        }
        for index in 0..self.activities.len() {
            let slot = &self.activities[index];
            if slot.is_enabled && slot.is_started && matches!(slot.activity.presence(), Some(Presence::Banner { .. })) {
                self.with_activity(index, |activity, context| activity.dismiss_banner(context));
            }
        }
    }

    fn source_id(&self, source: Source) -> &'static str {
        match source {
            Source::Activity(index) => self.activities[index].activity.id(),
            Source::Page(index) => self.pages[index].id(),
        }
    }
}
