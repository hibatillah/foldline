//! Where the items of the idle pill's row and of the open header sit; all sizes in DIP.
//!
//! The group is one right-anchored run in the order the caller gives: badges by rank, the battery ring, the dot
//! column. A new item joins between the clock and the items already there, so nothing right of it moves.

/// The least room between the idle clock and the group.
pub(super) const GAP: f32 = 24.0;
/// Between two items of the idle group.
pub(super) const ITEM_GAP: f32 = 8.0;
pub(super) const PILL_EDGE: f32 = 12.0;
/// Room on each side of the clock in the empty pill, which hugs the clock instead of keeping the usual minimum.
pub(super) const EMPTY_SIDE: f32 = 28.0;
/// Between two items of the open header's group, and between the group and the gear.
pub(super) const HEADER_ITEM_GAP: f32 = 10.0;
/// The least room between the header clock and the nearest icon; icons nearer than this are dropped.
pub(super) const HEADER_CLOCK_GAP: f32 = 14.0;
/// The least room between the page dots and the header clock.
pub(super) const PAGE_DOTS_CLOCK_GAP: f32 = 12.0;
pub(super) const GEAR_SIZE: f32 = 14.0;
/// Edge of the square a click on a header icon lands in; neighbours 10 apart never overlap it.
pub(super) const HIT_SIZE: f32 = 24.0;

pub(super) struct RowInput<'a> {
    pub clock_width: f32,
    /// Left to right: badges lowest rank first, then the battery ring, then the dot column.
    pub group_widths: &'a [f32],
    pub minimum_width: f32,
}

/// Offsets are from the pill's left edge.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Row {
    pub width: f32,
    pub clock_left: f32,
    /// In the order of `RowInput::group_widths`.
    pub group_lefts: Vec<f32>,
}

fn run_width(widths: &[f32], gap: f32) -> f32 {
    widths.iter().sum::<f32>() + widths.len().saturating_sub(1) as f32 * gap
}

/// Left edges of a run of `widths`, `gap` apart, whose last item ends at `right`.
fn lefts_ending_at(right: f32, widths: &[f32], gap: f32) -> Vec<f32> {
    let mut x = right - run_width(widths, gap);
    widths
        .iter()
        .map(|&width| {
            let left = x;
            x += width + gap;
            left
        })
        .collect()
}

/// The idle row. With a group: the clock at the left edge and the group right-anchored, in a pill of at least
/// `minimum_width`. Without one: the clock in the middle of a pill that hugs it with `EMPTY_SIDE` on each side.
pub(super) fn idle_row(input: &RowInput) -> Row {
    if input.group_widths.is_empty() {
        return Row { width: input.clock_width + 2.0 * EMPTY_SIDE, clock_left: EMPTY_SIDE, group_lefts: Vec::new() };
    }
    let content = input.clock_width + GAP + run_width(input.group_widths, ITEM_GAP);
    let width = input.minimum_width.max(content + 2.0 * PILL_EDGE);
    Row { width, clock_left: PILL_EDGE, group_lefts: lefts_ending_at(width - PILL_EDGE, input.group_widths, ITEM_GAP) }
}

pub(super) struct HeaderInput<'a> {
    /// The header's left and right edge.
    pub left: f32,
    pub right: f32,
    pub clock_width: f32,
    /// The ranked icons, lowest rank (nearest the clock) first; the first ones are dropped when room runs out.
    pub icon_widths: &'a [f32],
    /// The battery ring and the dot column after the icons; never dropped, `header_content_width` keeps room for them.
    pub end_widths: &'a [f32],
}

/// Positions in the coordinates of the edges it was given.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct HeaderRow {
    pub clock_center: f32,
    /// How many icons, counted from the clock side, have no room and are not shown.
    pub dropped_icons: usize,
    /// The shown icons, then the end items, left to right.
    pub group_lefts: Vec<f32>,
    pub gear_left: f32,
}

/// The open header: the clock alone in the exact middle, the gear at the right edge with the group
/// `HEADER_ITEM_GAP` before it. Icons that would come nearer the clock than `HEADER_CLOCK_GAP` are dropped, nearest
/// the clock first, so the clock never moves.
pub(super) fn header_row(input: &HeaderInput) -> HeaderRow {
    let gear_left = input.right - GEAR_SIZE;
    let clock_center = (input.left + input.right) / 2.0;
    let nearest_allowed = clock_center + input.clock_width / 2.0 + HEADER_CLOCK_GAP;
    let mut dropped_icons = 0;
    loop {
        let shown: Vec<f32> = input.icon_widths[dropped_icons..].iter().chain(input.end_widths).copied().collect();
        let group_lefts = lefts_ending_at(gear_left - HEADER_ITEM_GAP, &shown, HEADER_ITEM_GAP);
        let fits = group_lefts.first().is_none_or(|&left| left >= nearest_allowed - 0.001);
        if fits || dropped_icons == input.icon_widths.len() {
            return HeaderRow { clock_center, dropped_icons, group_lefts, gear_left };
        }
        dropped_icons += 1;
    }
}

/// The least width the header's content needs with the clock in the middle: the page dots and their gap on one
/// side, the end items (battery ring, dot column) and the gear on the other, whichever is wider, on both sides.
pub(super) fn header_content_width(page_dots_width: f32, clock_width: f32, end_widths: &[f32]) -> f32 {
    let page_dots_room = if page_dots_width > 0.0 { page_dots_width + PAGE_DOTS_CLOCK_GAP } else { 0.0 };
    let end_run = if end_widths.is_empty() { 0.0 } else { run_width(end_widths, HEADER_ITEM_GAP) + HEADER_ITEM_GAP };
    let end_room = HEADER_CLOCK_GAP + end_run + GEAR_SIZE;
    clock_width + 2.0 * page_dots_room.max(end_room)
}

/// The item whose `HIT_SIZE` wide area holds `x`; where areas meet, the nearer item wins.
/// `items` are `(left, width)` pairs.
pub(super) fn hit_item(items: &[(f32, f32)], x: f32) -> Option<usize> {
    items
        .iter()
        .enumerate()
        .filter_map(|(index, &(left, width))| {
            let center = left + width / 2.0;
            let half = HIT_SIZE.max(width) / 2.0;
            ((x - center).abs() <= half).then_some((index, (x - center).abs()))
        })
        .min_by(|first, second| first.1.total_cmp(&second.1))
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLOCK: f32 = 40.0;
    const RING: f32 = 14.0;
    const DOTS: f32 = 6.0;

    fn row(group_widths: &[f32]) -> Row {
        idle_row(&RowInput { clock_width: CLOCK, group_widths, minimum_width: 150.0 })
    }

    #[test]
    fn an_empty_pill_centers_the_clock_and_hugs_it_with_28_on_each_side() {
        let row = row(&[]);
        assert_eq!(row.width, CLOCK + 2.0 * EMPTY_SIDE);
        assert!(row.width < 150.0, "the empty pill is narrower than the usual minimum");
        assert_eq!(row.clock_left, EMPTY_SIDE);
        assert_eq!(row.clock_left + CLOCK / 2.0, row.width / 2.0);
    }

    #[test]
    fn any_item_moves_the_clock_to_the_left_edge_and_restores_the_150_minimum() {
        let row = row(&[RING]);
        assert_eq!(row.width, 150.0);
        assert_eq!(row.clock_left, PILL_EDGE);
        assert_eq!(row.group_lefts, [150.0 - PILL_EDGE - RING]);
    }

    #[test]
    fn the_group_is_right_anchored_8_apart_and_ends_12_from_the_edge() {
        let row = row(&[30.0, 40.0, RING, DOTS]);
        let lefts = &row.group_lefts;
        assert_eq!(lefts[1] - (lefts[0] + 30.0), ITEM_GAP);
        assert_eq!(lefts[2] - (lefts[1] + 40.0), ITEM_GAP);
        assert_eq!(lefts[3] - (lefts[2] + RING), ITEM_GAP);
        assert_eq!(lefts[3] + DOTS, row.width - PILL_EDGE);
    }

    #[test]
    fn a_crowded_row_keeps_24_between_the_clock_and_the_group_and_grows_the_pill() {
        let row = row(&[50.0, 50.0, RING]);
        assert_eq!(row.group_lefts[0] - (PILL_EDGE + CLOCK), GAP);
        assert_eq!(row.width, PILL_EDGE + CLOCK + GAP + 50.0 + ITEM_GAP + 50.0 + ITEM_GAP + RING + PILL_EDGE);
    }

    #[test]
    fn a_wide_minimum_leaves_the_spare_room_between_the_clock_and_the_group() {
        let row = row(&[RING]);
        assert!(row.group_lefts[0] - (PILL_EDGE + CLOCK) > GAP);
    }

    #[test]
    fn a_new_item_joins_left_of_the_group_and_nothing_right_of_it_moves() {
        let before = row(&[50.0, 50.0, RING]);
        let after = row(&[20.0, 50.0, 50.0, RING]);
        let insets = |row: &Row| row.group_lefts.iter().map(|left| row.width - left).collect::<Vec<f32>>();
        assert_eq!(insets(&after)[1..], insets(&before)[..]);
    }

    #[test]
    fn the_dot_column_reserves_no_width_so_the_group_shifts_left_by_the_dot_and_a_gap() {
        let without = row(&[50.0, 50.0, RING]);
        let with = row(&[50.0, 50.0, RING, DOTS]);
        assert_eq!(with.width - without.width, DOTS + ITEM_GAP);
        let ring_inset = |row: &Row| row.width - row.group_lefts[2];
        assert_eq!(ring_inset(&with) - ring_inset(&without), DOTS + ITEM_GAP);
    }

    const LEFT: f32 = 16.0;
    const RIGHT: f32 = 416.0;

    fn header(icon_widths: &[f32], end_widths: &[f32]) -> HeaderRow {
        header_row(&HeaderInput { left: LEFT, right: RIGHT, clock_width: CLOCK, icon_widths, end_widths })
    }

    #[test]
    fn the_gear_is_the_last_item_at_the_right_edge_of_the_header() {
        assert_eq!(header(&[16.0], &[]).gear_left + GEAR_SIZE, RIGHT);
        assert_eq!(header(&[], &[]).gear_left + GEAR_SIZE, RIGHT);
    }

    #[test]
    fn the_header_group_runs_10_apart_and_ends_10_before_the_gear_in_the_idle_order() {
        let header = header(&[16.0, 16.0], &[RING, DOTS]);
        assert_eq!(header.gear_left - (header.group_lefts[3] + DOTS), HEADER_ITEM_GAP);
        assert_eq!(header.group_lefts[3] - (header.group_lefts[2] + RING), HEADER_ITEM_GAP);
        assert_eq!(header.group_lefts[1] - (header.group_lefts[0] + 16.0), HEADER_ITEM_GAP);
    }

    #[test]
    fn the_gear_shifts_every_group_item_left_by_its_width_and_a_gap() {
        let group = [16.0, RING, DOTS];
        let with_gear = header(&group[..1], &group[1..]);
        let edge_anchored = lefts_ending_at(RIGHT, &group, HEADER_ITEM_GAP);
        for (left, edge_left) in with_gear.group_lefts.iter().zip(&edge_anchored) {
            assert_eq!(edge_left - left, GEAR_SIZE + HEADER_ITEM_GAP);
        }
    }

    #[test]
    fn the_header_clock_sits_alone_in_the_middle() {
        assert_eq!(header(&[16.0], &[RING]).clock_center, 216.0);
        assert_eq!(header(&[], &[]).clock_center, 216.0);
    }

    #[test]
    fn a_crowded_header_keeps_the_clock_in_the_middle_and_drops_the_icons_nearest_it() {
        let header = header(&[16.0; 12], &[RING, DOTS]);
        assert_eq!(header.clock_center, 216.0, "the clock never moves");
        assert!(header.dropped_icons > 0);
        assert_eq!(header.group_lefts.len(), 12 - header.dropped_icons + 2);
        assert!(header.group_lefts[0] - (header.clock_center + CLOCK / 2.0) >= HEADER_CLOCK_GAP);
    }

    #[test]
    fn the_header_drops_only_as_many_icons_as_it_must() {
        let header = header(&[16.0; 12], &[RING, DOTS]);
        let one_more_shown = header.group_lefts[0] - 16.0 - HEADER_ITEM_GAP;
        assert!(one_more_shown < header.clock_center + CLOCK / 2.0 + HEADER_CLOCK_GAP);
    }

    #[test]
    fn a_header_with_room_shows_every_icon() {
        assert_eq!(header(&[16.0, 16.0], &[RING]).dropped_icons, 0);
    }

    #[test]
    fn the_battery_ring_and_the_dots_are_never_dropped() {
        let header = header_row(&HeaderInput {
            left: 0.0,
            right: 60.0,
            clock_width: CLOCK,
            icon_widths: &[16.0, 16.0],
            end_widths: &[RING, DOTS],
        });
        assert_eq!(header.dropped_icons, 2);
        assert_eq!(header.group_lefts.len(), 2);
    }

    #[test]
    fn the_least_header_width_puts_the_wider_side_on_both_sides_of_the_clock() {
        let dots_side = 96.0 + PAGE_DOTS_CLOCK_GAP;
        assert_eq!(header_content_width(96.0, CLOCK, &[RING]), CLOCK + 2.0 * dots_side);
        let end_side = HEADER_CLOCK_GAP + RING + HEADER_ITEM_GAP + DOTS + HEADER_ITEM_GAP + GEAR_SIZE;
        assert_eq!(header_content_width(0.0, CLOCK, &[RING, DOTS]), CLOCK + 2.0 * end_side);
        assert_eq!(header_content_width(0.0, CLOCK, &[]), CLOCK + 2.0 * (HEADER_CLOCK_GAP + GEAR_SIZE));
    }

    #[test]
    fn nine_page_dots_and_a_37_px_clock_need_a_285_px_panel() {
        let nine_dots = 8.0 * 10.0 + 16.0;
        assert_eq!(header_content_width(nine_dots, 37.0, &[RING]) + 32.0, 285.0);
    }

    #[test]
    fn a_click_lands_on_the_icon_whose_24_wide_area_holds_it_and_nowhere_in_empty_space() {
        let items = [(100.0, 16.0), (124.0, 16.0)];
        assert_eq!(hit_item(&items, 108.0), Some(0));
        assert_eq!(hit_item(&items, 96.5), Some(0), "4 left of the icon is still in its area");
        assert_eq!(hit_item(&items, 95.0), None);
        assert_eq!(hit_item(&items, 132.0), Some(1));
        assert_eq!(hit_item(&items, 160.0), None);
    }

    #[test]
    fn where_two_hit_areas_meet_the_nearer_icon_wins() {
        let items = [(100.0, 16.0), (124.0, 16.0)];
        assert_eq!(hit_item(&items, 119.0), Some(0));
        assert_eq!(hit_item(&items, 121.0), Some(1));
    }

    #[test]
    fn a_ring_narrower_than_an_icon_never_steals_its_neighbour() {
        let items = [(100.0, RING), (122.0, 16.0)];
        assert_eq!(hit_item(&items, 118.0), Some(0));
        assert_eq!(hit_item(&items, 119.5), Some(1));
    }
}
