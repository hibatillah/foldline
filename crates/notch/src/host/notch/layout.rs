use super::{HIDDEN_WIDTH, Notch, PAGE_GAP, RESTING_BOTTOM_PADDING, RESTING_PILL_HEIGHT, Source, resting_row};
use crate::host::{activity_layout, chrome, navigation};
use crate::{
    activity::{Presence, Slot},
    gfx::{Point, Size, Spring},
    log,
    page::{Frame, Mode, Priority, RESTING_MIN_WIDTH},
};

const DRAG_GROWTH: Size = Size::new(28.0, 14.0);
/// The page area of the open notch while no page has anything to show: only the header remains.
const EMPTY_PAGE_SIZE: Size = Size::new(220.0, 0.0);
/// The page whose open width every page opens at least as wide as, while it is switched on.
pub(super) const WIDTH_FLOOR_PAGE_ID: &str = "timer";

/// A page's own content height, capped to what the window holds; nothing reported means the shared default.
pub(super) fn content_height(reported: f32) -> f32 {
    if reported > 0.0 { reported.min(crate::page::MAX_CONTENT_HEIGHT) } else { crate::page::DEFAULT_CONTENT_HEIGHT }
}

pub(super) fn highest_priority(priorities: impl IntoIterator<Item = Option<Priority>>) -> Option<usize> {
    let mut best: Option<(usize, Priority)> = None;
    for (index, priority) in priorities.into_iter().enumerate() {
        let Some(priority) = priority else { continue };
        if best.is_none_or(|(_, best_priority)| priority > best_priority) {
            best = Some((index, priority));
        }
    }
    best.map(|(index, _)| index)
}

impl Notch {
    pub(super) fn page_frame(&self, frame: &Frame) -> Frame {
        Frame { bounds: chrome::content_rect(frame.bounds, self.scale), ..*frame }
    }

    /// Whether the open page has its own use for a right-click at `point`.
    pub(super) fn page_uses_right_click(&self, point: Point) -> bool {
        let Some(index) = self.expanded_index else { return false };
        let viewport = self.frame();
        let index = if self.navigable.len() < 2 { index } else { self.page_at(self.track.position) };
        let frame = self.natural_page_frame(index, &viewport, 0.0);
        self.pages[index].uses_right_click(point, &frame)
    }

    /// Whether the open page scrolls something at `point` with the vertical wheel.
    pub(super) fn page_claims_scroll(&self, point: Point) -> bool {
        let Some(index) = self.expanded_index else { return false };
        let viewport = self.frame();
        if !viewport.is_open() || !crate::host::chrome::content_rect(viewport.bounds, self.scale).contains(point) {
            return false;
        }
        let index = if self.navigable.len() < 2 { index } else { self.page_at(self.track.position) };
        let frame = self.natural_page_frame(index, &viewport, 0.0);
        self.pages[index].claims_scroll(point, &frame)
    }

    pub(super) fn natural_page_frame(&self, index: usize, viewport: &Frame, center_offset: f32) -> Frame {
        let mut frame = self.page_frame(viewport);
        let width = self.pages[index].sizes().expanded.width;
        let center = frame.bounds.center_x() + center_offset * self.scale;
        let left = if self.pages[index].is_left_aligned() {
            frame.bounds.left + center_offset * self.scale
        } else {
            center - width * self.scale / 2.0
        };
        frame.bounds.left = left;
        frame.bounds.right = left + width * self.scale;
        frame.expanded.width = width;
        frame
    }

    pub(super) fn page_blend(&self) -> Option<(usize, usize, f32)> {
        let count = self.navigable.len();
        if count < 2 {
            return None;
        }
        let position = self.track.position.rem_euclid(count as f32);
        let first = position.floor() as usize;
        Some((self.navigable[first], self.navigable[(first + 1) % count], position.fract()))
    }

    pub(super) fn blended_page_layout(&self) -> Option<navigation::PageLayout> {
        let (first, next, progress) = self.page_blend()?;
        Some(navigation::page_layout(
            self.pages[first].sizes().expanded.width,
            self.pages[next].sizes().expanded.width,
            progress,
            PAGE_GAP,
        ))
    }

    pub(super) fn visible_page_frames(&self, viewport: &Frame) -> Option<[(usize, Frame, f32); 2]> {
        let (first, next, progress) = self.page_blend()?;
        if progress < 0.001 || 1.0 - progress < 0.001 {
            return None;
        }
        let layout = self.blended_page_layout()?;
        Some([
            (first, self.natural_page_frame(first, viewport, layout.center_offsets[0]), progress),
            (next, self.natural_page_frame(next, viewport, layout.center_offsets[1]), 1.0 - progress),
        ])
    }

    pub(super) fn sync_shown_page(&mut self) -> bool {
        let shown = (self.hover.is_hovering || self.keyboard.owner.is_some()).then_some(self.expanded_index).flatten();
        if shown == self.shown_page {
            return false;
        }
        let previous = std::mem::replace(&mut self.shown_page, shown);
        if let Some(index) = previous {
            self.with_page(index, |page, context| page.on_shown_changed(false, context));
        }
        if let Some(index) = shown {
            self.with_page(index, |page, context| page.on_shown_changed(true, context));
        }
        true
    }

    pub(super) fn refresh_layout(&mut self) -> bool {
        let resting_index = self.best_page(Mode::Resting);
        let expanded_priorities: Vec<Option<Priority>> =
            self.pages.iter().map(|page| page.priority(Mode::Expanded)).collect();
        self.navigable = self.page_list.navigable(&expanded_priorities);
        let expanded_index = navigation::choose(self.selected_page, &self.page_list.effective(&expanded_priorities));
        let activities_changed = self.refresh_activities();
        let selection_changed =
            resting_index != self.resting_index || expanded_index != self.expanded_index || activities_changed;
        self.resting_index = resting_index;
        self.expanded_index = expanded_index;
        self.follow_open_page();

        self.refresh_header_marks();
        self.refresh_resting_row();
        let expanded = expanded_index.map_or(EMPTY_PAGE_SIZE, |index| self.pages[index].sizes().expanded);
        let expanded_width = (expanded.width + chrome::WIDTH_EXTRA).max(self.open_min_width());
        let transient_widths: Vec<f32> = self.shown_transients.iter().map(|&(_, width)| width).collect();
        let is_claimed = resting_index.is_some();
        let claim_right_inset = if is_claimed { self.resting_row.claim.right_inset() } else { 0.0 };
        let claim_lead_gap = self.resting_row.claim.lead_gap();
        let banner = self.shown_banner;
        // Only the empty idle pill may be narrower than the minimum: it hugs its centered clock.
        let resting_target = if let Some((_, width, _)) = banner {
            width.max(RESTING_MIN_WIDTH)
        } else if !transient_widths.is_empty() {
            activity_layout::total_width(&transient_widths).max(RESTING_MIN_WIDTH)
        } else if let Some(index) = resting_index {
            self.claimed_pill_width(index)
        } else {
            self.resting_row.layout.width
        };
        self.has_resting_content = resting_index.is_some() || self.has_music_badge();
        let is_hidden = self.is_hidden();
        let banner_height = banner.map_or(RESTING_PILL_HEIGHT, |(_, _, height)| height + RESTING_BOTTOM_PADDING);
        let (resting_target, resting_height_target) =
            if is_hidden { (HIDDEN_WIDTH, self.hidden_height()) } else { (resting_target, banner_height) };
        let growth = if self.hover.is_dragging_files { DRAG_GROWTH } else { Size::new(0.0, 0.0) };
        let mut targets_changed = false;
        for (spring, target) in [
            (&mut self.springs.resting_width, resting_target),
            (&mut self.springs.resting_height, resting_height_target),
            (&mut self.springs.expanded_width, expanded_width + growth.width),
            (&mut self.springs.expanded_height, content_height(expanded.height) + chrome::HEIGHT_EXTRA + growth.height),
            (&mut self.springs.activity_blend, if transient_widths.is_empty() && banner.is_none() { 0.0 } else { 1.0 }),
            (&mut self.springs.claim_right_inset, claim_right_inset),
            (&mut self.springs.claim_lead_gap, claim_lead_gap),
        ] {
            if (spring.target - target).abs() > f32::EPSILON {
                spring.target = target;
                targets_changed = true;
            }
        }
        targets_changed |= self.retarget_rows(expanded_width);
        if targets_changed {
            self.start_animation();
        }
        selection_changed
    }

    /// The least width of the open notch: the timer page's open width, or the header's content when that is wider.
    pub(super) fn open_min_width(&self) -> f32 {
        let floor_page_width = self
            .pages
            .iter()
            .enumerate()
            .find(|&(index, page)| page.id() == WIDTH_FLOOR_PAGE_ID && self.page_list.is_enabled(index))
            .map_or(0.0, |(_, page)| page.sizes().expanded.width + chrome::WIDTH_EXTRA);
        floor_page_width.max(self.header_min_width())
    }

    /// Slides the page row to the selected page; jumps while closed.
    pub(super) fn follow_open_page(&mut self) {
        let Some(position) = self.navigable.iter().position(|&index| Some(index) == self.expanded_index) else {
            return;
        };
        let count = self.navigable.len() as f32;
        let mut gap = (position as f32 - self.track.target).rem_euclid(count);
        if gap > count / 2.0 {
            gap -= count;
        }
        if gap.abs() < 0.01 {
            return;
        }
        if self.hover.is_hovering {
            self.track.target += gap;
            self.start_animation();
        } else {
            self.track.jump_to(position as f32);
        }
    }

    pub(super) fn refresh_activities(&mut self) -> bool {
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
        activity_layout::track_order(&mut self.transient_order, &active);
        let banner = banners.last().copied();
        let shown: Vec<(Source, f32)> = if banner.is_some() {
            Vec::new()
        } else {
            activity_layout::newest(&self.transient_order)
                .iter()
                .filter_map(|&source| transients.iter().copied().find(|&(candidate, _)| candidate == source))
                .collect()
        };
        badges.sort_by_key(|&(source, _)| self.rank_of(source));
        let claimed = resting_row::claimed_group(
            &badges,
            |source| self.rank_of(source),
            |source| self.countdown_of(source),
            |source| self.is_music_badge(source),
        );
        let uses_short_badges = self.shortens_badges(&mut badges);
        let changed = shown != self.shown_transients
            || badges != self.shown_badges
            || banner != self.shown_banner
            || before != self.resting_row.ring
            || after != self.resting_row.dots
            || claimed != self.resting_row.claimed_group
            || uses_short_badges != self.resting_row.uses_short_badges;
        if shown != self.shown_transients || badges != self.shown_badges || banner != self.shown_banner {
            let ids = |sources: &[(Source, f32)]| sources.iter().map(|&(source, _)| self.source_id(source)).collect();
            let (transient_ids, badge_ids): (Vec<_>, Vec<_>) = (ids(&shown), ids(&badges));
            let banner_id = banner.map(|(source, _, _)| self.source_id(source));
            log::debug(&activity_layout::presence_summary(&transient_ids, &badge_ids, banner_id));
        }
        self.resting_row.ring = before;
        self.resting_row.claimed_group = claimed;
        self.resting_row.uses_short_badges = uses_short_badges;
        self.resting_row.dots = after;
        self.shown_banner = banner;
        self.shown_transients = shown;
        self.shown_badges = badges;
        changed
    }

    fn source_id(&self, source: Source) -> &'static str {
        match source {
            Source::Activity(index) => self.activities[index].activity.id(),
            Source::Page(index) => self.pages[index].id(),
        }
    }

    /// Every spring of the layout but the open amount; the frame timer runs until all of them settle.
    pub(super) fn layout_springs(&mut self) -> impl Iterator<Item = &mut Spring> {
        let springs = &mut self.springs;
        [
            &mut springs.resting_width,
            &mut springs.resting_height,
            &mut springs.expanded_width,
            &mut springs.expanded_height,
            &mut springs.activity_blend,
            &mut springs.claim_right_inset,
            &mut springs.claim_lead_gap,
            &mut self.track,
        ]
        .into_iter()
        .chain(self.row_motion.springs_mut())
    }

    pub(super) fn best_page(&self, mode: Mode) -> Option<usize> {
        highest_priority(self.pages.iter().map(|page| page.priority(mode)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub(super) fn a_page_sets_its_own_open_height_up_to_what_the_window_holds() {
        assert_eq!(content_height(86.0), 86.0);
        assert_eq!(content_height(150.0), 150.0);
        assert_eq!(content_height(400.0), crate::page::MAX_CONTENT_HEIGHT);
        assert_eq!(content_height(0.0), crate::page::DEFAULT_CONTENT_HEIGHT);
    }
    const LOW: Priority = Priority(1);
    const HIGH: Priority = Priority(5);

    #[test]
    pub(super) fn no_page_wants_to_show() {
        assert_eq!(highest_priority([None, None]), None);
        assert_eq!(highest_priority(Vec::new()), None);
    }

    #[test]
    pub(super) fn the_highest_priority_wins() {
        assert_eq!(highest_priority([Some(LOW), Some(HIGH), Some(LOW)]), Some(1));
    }

    #[test]
    pub(super) fn the_earliest_page_wins_a_tie() {
        assert_eq!(highest_priority([Some(HIGH), Some(HIGH)]), Some(0));
    }

    #[test]
    pub(super) fn pages_with_nothing_to_show_are_skipped() {
        assert_eq!(highest_priority([None, Some(LOW), None]), Some(1));
    }
}
