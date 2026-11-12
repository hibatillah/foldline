//! The idle pill's row and the claimed pill's row; the open header's is in `header_row.rs`.

use std::cmp::Reverse;

use super::{HEADER_STAGE, Notch, Source, row_motion::RowItem, units};
use crate::host::{
    chrome,
    claimed_row::ClaimedRow,
    clock,
    pill_row::{self, Row, RowInput, Run},
};
use crate::{
    gfx::{Canvas, Point, Rect, lerp},
    page::{Frame, RESTING_HEIGHT, RESTING_MIN_WIDTH},
};

/// Companions and badges of the collapsed pill and the rows they give, rebuilt after every activity event.
#[derive(Default)]
pub(super) struct RestingRow {
    /// The battery ring; empty on a PC.
    pub(super) ring: Vec<(Source, f32)>,
    pub(super) dots: Vec<(Source, f32)>,
    pub(super) layout: Row,
    pub(super) claim: ClaimedRow,
    /// What the claimed pill shows between the line and the dot column: the statics, highest rank first, then one
    /// countdown.
    pub(super) claimed_group: Vec<(Source, f32)>,
    /// Several badges with text share the idle pill, so each draws its short form.
    pub(super) uses_short_badges: bool,
}

pub(super) struct RowOpacity {
    pub(super) clock: f32,
    pub(super) dots: f32,
    pub(super) header: f32,
}

impl RestingRow {
    /// Zero while no dot shows: the column reserves no room.
    pub(super) fn dots_width(&self) -> f32 {
        self.dots.iter().map(|&(_, width)| width).fold(0.0, f32::max)
    }

    /// The battery ring and the dot column, the fixed right end of the idle and header groups.
    pub(super) fn right_end(&self) -> Vec<(RowItem, f32)> {
        let ring = self.ring.iter().map(|&(source, width)| (RowItem::Source(source), width));
        let dots = (!self.dots.is_empty()).then(|| (RowItem::Dots, self.dots_width()));
        ring.chain(dots).collect()
    }

    /// The widths of `items` and the room between each pair of neighbours: `item_gap`, or `RING_DOTS_GAP` from the
    /// battery ring to the dot column right after it.
    pub(super) fn spacing(&self, items: &[(RowItem, f32)], item_gap: f32) -> (Vec<f32>, Vec<f32>) {
        let widths = items.iter().map(|&(_, width)| width).collect();
        let gaps = items
            .windows(2)
            .map(|pair| pill_row::gap_between(self.is_ring(pair[0].0), pair[1].0 == RowItem::Dots, item_gap))
            .collect();
        (widths, gaps)
    }

    fn is_ring(&self, item: RowItem) -> bool {
        matches!(item, RowItem::Source(source) if self.ring.iter().any(|&(ring, _)| ring == source))
    }
}

/// The claimed pill's group: the statics (badges without a countdown, the music wave left out) highest rank first,
/// then the one countdown with the least time left. `countdown` gives a badge's seconds left, `None` for a static.
pub(super) fn claimed_group(
    badges: &[(Source, f32)],
    rank: impl Fn(Source) -> u8,
    countdown: impl Fn(Source) -> Option<u64>,
    is_music: impl Fn(Source) -> bool,
) -> Vec<(Source, f32)> {
    let mut statics: Vec<(Source, f32)> =
        badges.iter().copied().filter(|&(source, _)| !is_music(source) && countdown(source).is_none()).collect();
    statics.sort_by_key(|&(source, _)| Reverse(rank(source)));
    let nearest = badges
        .iter()
        .copied()
        .filter_map(|(source, width)| countdown(source).map(|seconds| (seconds, source, width)))
        .min_by_key(|&(seconds, _, _)| seconds);
    statics.extend(nearest.map(|(_, source, width)| (source, width)));
    statics
}

impl Notch {
    /// Header status and pill badge share one order: lower ranks sit nearer the time.
    pub(super) fn rank_of(&self, source: Source) -> u8 {
        match source {
            Source::Page(index) => self.pages[index].header_status_rank(),
            Source::Activity(index) => self.activities[index].activity.header_status_rank(),
        }
    }

    /// The music wave is the badge of a page with a lead (artwork) width; it is not an activity.
    pub(super) fn is_music_badge(&self, source: Source) -> bool {
        matches!(source, Source::Page(index) if self.pages[index].resting_lead_width() > 0.0)
    }

    pub(super) fn has_music_badge(&self) -> bool {
        self.presences.badges.iter().any(|&(source, _)| self.is_music_badge(source))
    }

    pub(super) fn countdown_of(&self, source: Source) -> Option<u64> {
        match source {
            Source::Page(index) => self.pages[index].countdown_seconds(),
            Source::Activity(_) => None,
        }
    }

    /// With two or more badges that have a short form, every such badge takes it; true when that happened.
    pub(super) fn shortens_badges(&self, badges: &mut [(Source, f32)]) -> bool {
        let short_width = |source: Source| match source {
            Source::Page(index) => self.pages[index].short_presence_width(),
            Source::Activity(_) => None,
        };
        if badges.iter().filter(|&&(source, _)| short_width(source).is_some()).count() < 2 {
            return false;
        }
        for (source, width) in badges.iter_mut() {
            if let Some(short) = short_width(*source) {
                *width = short;
            }
        }
        true
    }

    /// The idle group left to right: badges by rank, the battery ring, the dot column.
    pub(super) fn idle_items(&self) -> Vec<(RowItem, f32)> {
        let badges = self.presences.badges.iter().map(|&(source, width)| (RowItem::Source(source), width));
        badges.chain(self.resting_row.right_end()).collect()
    }

    pub(super) fn claimed_items(&self) -> Vec<(RowItem, f32)> {
        self.resting_row.claimed_group.iter().map(|&(source, width)| (RowItem::Source(source), width)).collect()
    }

    pub(super) fn claimed_pill_width(&self, index: usize) -> f32 {
        self.resting_row.claim.pill_width(self.pages[index].sizes().resting_width).max(RESTING_MIN_WIDTH)
    }

    pub(super) fn refresh_resting_row(&mut self) {
        let (widths, gaps) = self.resting_row.spacing(&self.idle_items(), pill_row::ITEM_GAP);
        self.resting_row.layout = pill_row::idle_row(&RowInput {
            clock_width: clock::layout_width(&self.fonts),
            group: Run { widths: &widths, gaps: &gaps },
            minimum_width: clock::RESTING_WIDTH,
        });
        let lead_width = self.resting_index.map_or(0.0, |index| self.pages[index].resting_lead_width());
        let claimed_widths: Vec<f32> = self.resting_row.claimed_group.iter().map(|&(_, width)| width).collect();
        self.resting_row.claim = ClaimedRow::new(lead_width, self.resting_row.dots_width(), &claimed_widths);
    }

    /// The rect a badge draws into: the place its spring holds, measured from the pill's right edge.
    fn badge_rect(&self, bounds: Rect, source: Source, width: f32) -> Option<Rect> {
        let inset = self.row_motion.resting.position(RowItem::Source(source))?;
        let left = bounds.right - inset * self.scale;
        Some(Rect::new(left, bounds.top, left + width * self.scale, bounds.top + RESTING_HEIGHT * self.scale))
    }

    /// The page badges the collapsed pill draws now, as `(page_index, area)`; their pointer input is measured there.
    pub(super) fn page_badge_areas(&self, frame: &Frame) -> Vec<(usize, Rect)> {
        let badges =
            if self.resting_index.is_some() { &self.resting_row.claimed_group } else { &self.presences.badges };
        badges
            .iter()
            .filter_map(|&(source, width)| match source {
                Source::Page(index) => self.badge_rect(frame.bounds, source, width).map(|area| (index, area)),
                Source::Activity(_) => None,
            })
            .collect()
    }

    fn draw_badges_at_springs(
        &self,
        canvas: &Canvas,
        frame: &Frame,
        badges: &[(Source, f32)],
        short: bool,
        opacity: f32,
    ) {
        for &(source, width) in badges {
            if let Some(area) = self.badge_rect(frame.bounds, source, width) {
                self.draw_badge(source, canvas, area, short, opacity);
            }
        }
    }

    /// The idle group's badges; the ring and the dots travel to the header, so `draw_row` draws them.
    pub(super) fn draw_group(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        self.draw_badges_at_springs(canvas, frame, &self.presences.badges, self.resting_row.uses_short_badges, opacity);
    }

    /// The width of a countdown's icon alone, which the claimed pill shows in place of its text; `None` for any other badge.
    pub(super) fn countdown_icon_width(&self, source: Source) -> Option<f32> {
        match source {
            Source::Page(index) if self.pages[index].countdown_seconds().is_some() => {
                self.pages[index].header_status_width()
            }
            _ => None,
        }
    }

    /// The claimed pill's group: a countdown shows only its icon, the same mark the header shows; other badges keep
    /// their long form.
    pub(super) fn draw_claimed_group(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        for &(source, width) in &self.resting_row.claimed_group {
            let Some(area) = self.badge_rect(frame.bounds, source, width) else { continue };
            match source {
                Source::Page(index) if self.countdown_icon_width(source).is_some() => {
                    units::draw_page_header_status(self.pages[index].as_ref(), canvas, area, opacity);
                }
                _ => self.draw_badge(source, canvas, area, false, opacity),
            }
        }
    }

    /// The x of an item's left edge in device pixels: its collapsed place moving to its header place as the notch
    /// opens. An item missing from one row stays at its place in the other.
    fn traveling_left(&self, item: RowItem, collapsed: Option<f32>, bounds: Rect, amount: f32) -> Option<f32> {
        let header = chrome::header_rect(bounds, self.scale);
        let open = self.row_motion.header.position(item).map(|inset| header.right - inset * self.scale);
        match (collapsed, open) {
            (Some(collapsed), Some(open)) => Some(lerp(collapsed, open, amount)),
            (place, None) | (None, place) => place,
        }
    }

    pub(super) fn draw_row(&self, canvas: &Canvas, frame: &Frame, opacity: &RowOpacity) {
        let (bounds, scale) = (frame.bounds, self.scale);
        let amount = (frame.expansion / HEADER_STAGE).clamp(0.0, 1.0);
        let header = chrome::header_rect(bounds, scale);
        let collapsed_y = bounds.top + RESTING_HEIGHT * scale / 2.0;
        let center_y = lerp(collapsed_y, header.center_y(), amount);
        let half_height = RESTING_HEIGHT * scale / 2.0;
        let collapsed_inset = |item| self.row_motion.resting.position(item).map(|inset| bounds.right - inset * scale);

        if opacity.clock > 0.01 {
            let collapsed = Point::new(bounds.left + self.idle_clock_center() * scale, collapsed_y);
            let open = Point::new(header.center_x(), header.center_y());
            let center = clock::center(collapsed, open, amount);
            clock::draw_time(canvas, Rect::around(center, clock::HALF_WIDTH * scale, half_height), opacity.clock);
            for &(source, width) in &self.resting_row.ring {
                let item = RowItem::Source(source);
                let collapsed = if self.resting_index.is_none() { collapsed_inset(item) } else { None };
                let Some(left) = self.traveling_left(item, collapsed, bounds, amount) else { continue };
                let area = Rect::new(left, center_y - half_height, left + width * scale, center_y + half_height);
                self.draw_source(source, canvas, area, opacity.clock);
            }
        }
        if opacity.dots > 0.01 && !self.resting_row.dots.is_empty() {
            let width = self.resting_row.dots_width();
            if let Some(left) = self.traveling_left(RowItem::Dots, collapsed_inset(RowItem::Dots), bounds, amount) {
                let area = Rect::new(left, center_y - half_height, left + width * scale, center_y + half_height);
                for &(source, _) in &self.resting_row.dots {
                    self.draw_source(source, canvas, area, opacity.dots);
                }
            }
        }
        if opacity.header > 0.01 {
            self.draw_header_marks(canvas, bounds, opacity.header);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WAVE: Source = Source::Page(0);
    const CAPS: Source = Source::Activity(1);
    const MUTED: Source = Source::Activity(2);
    const TIMER: Source = Source::Page(3);
    const EVENT: Source = Source::Page(4);
    const PRAYER: Source = Source::Page(5);

    fn rank(source: Source) -> u8 {
        match source {
            WAVE => 0,
            CAPS => 1,
            MUTED => 2,
            TIMER => 3,
            EVENT => 4,
            _ => 5,
        }
    }

    fn group(badges: &[Source], seconds: impl Fn(Source) -> Option<u64>) -> Vec<Source> {
        let badges: Vec<(Source, f32)> = badges.iter().map(|&source| (source, 10.0)).collect();
        claimed_group(&badges, rank, seconds, |source| source == WAVE).into_iter().map(|(source, _)| source).collect()
    }

    fn countdowns(source: Source) -> Option<u64> {
        match source {
            TIMER => Some(600),
            EVENT => Some(300),
            PRAYER => Some(1500),
            _ => None,
        }
    }

    #[test]
    fn the_lyric_pill_shows_muted_then_caps_lock_then_the_countdown_with_the_least_time_left() {
        let all = [WAVE, CAPS, MUTED, TIMER, EVENT, PRAYER];
        assert_eq!(group(&all, countdowns), [MUTED, CAPS, EVENT]);
    }

    #[test]
    fn the_lyric_pill_never_shows_the_wave_and_keeps_a_lone_countdown() {
        assert_eq!(group(&[WAVE, PRAYER], countdowns), [PRAYER]);
        assert_eq!(group(&[WAVE], countdowns), Vec::<Source>::new());
    }

    #[test]
    fn a_countdown_that_runs_out_sooner_takes_the_place_of_the_other() {
        let timer_first = |source| if source == TIMER { Some(30) } else { countdowns(source) };
        assert_eq!(group(&[CAPS, TIMER, EVENT], timer_first), [CAPS, TIMER]);
    }
}
