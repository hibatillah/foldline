//! The collapsed pill while a page claims it: the page's content, the group and the dot column; sizes in DIP.

use super::pill_row::{ITEM_GAP, PILL_EDGE, Run};

/// Between the artwork and the page's line, and between the line and the group.
pub(super) const CONTENT_GAP: f32 = 24.0;

/// `[edge][lead][24][content][24][group][8][dots][edge]`: the dot column ends the row where it ends the idle row.
/// The group and its 24 take room only with a group, the dots and their 8 only while a dot shows. Without a lead the
/// content starts at the pill's left edge. The battery ring never shows here, so the dots keep `ITEM_GAP`.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct ClaimedRow {
    lead_width: f32,
    /// Zero while no dot shows.
    dots_width: f32,
    /// In the order the group is drawn, left to right.
    group_widths: Vec<f32>,
    group_gaps: Vec<f32>,
}

impl ClaimedRow {
    pub fn new(lead_width: f32, dots_width: f32, group_widths: &[f32]) -> Self {
        let group_gaps = vec![ITEM_GAP; group_widths.len().saturating_sub(1)];
        Self { lead_width, dots_width, group_widths: group_widths.to_vec(), group_gaps }
    }

    fn has_lead(&self) -> bool {
        self.lead_width > 0.0
    }

    fn group(&self) -> Run<'_> {
        Run { widths: &self.group_widths, gaps: &self.group_gaps }
    }

    /// Room the group takes right of the content: its gap and the items; 0 without a group.
    fn group_room(&self) -> f32 {
        if self.group_widths.is_empty() { 0.0 } else { CONTENT_GAP + self.group().width() }
    }

    fn shows_dots(&self) -> bool {
        self.dots_width > 0.0
    }

    /// Room the dot column takes at the right end: the gap before it and the dots; 0 while no dot shows.
    fn trailing_dots(&self) -> f32 {
        if self.shows_dots() { ITEM_GAP + self.dots_width } else { 0.0 }
    }

    /// Distance of the page's content from the pill's left edge.
    pub fn left_inset(&self) -> f32 {
        if self.has_lead() { PILL_EDGE } else { 0.0 }
    }

    /// What the page leaves free between its lead and its line.
    pub fn lead_gap(&self) -> f32 {
        if self.has_lead() { CONTENT_GAP } else { 0.0 }
    }

    /// Distance of the page's content from the pill's right edge.
    pub fn right_inset(&self) -> f32 {
        PILL_EDGE + self.trailing_dots() + self.group_room()
    }

    /// The pill width for a page whose own width is `content_width` (without the lead gap, the edges and the group).
    pub fn pill_width(&self, content_width: f32) -> f32 {
        self.left_inset() + self.lead_width + self.lead_gap() + content_width + self.right_inset()
    }

    /// The dot column's left edge from the pill's left edge; `None` while no dot shows.
    pub fn dots_left(&self, pill_width: f32) -> Option<f32> {
        self.shows_dots().then_some(pill_width - PILL_EDGE - self.dots_width)
    }

    /// Left edges of the group items from the pill's left edge, in the order of the widths given to `new`.
    pub fn group_lefts(&self, pill_width: f32) -> Vec<f32> {
        self.group().lefts_ending_at(pill_width - PILL_EDGE - self.trailing_dots())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOTS: f32 = 6.0;

    #[test]
    fn the_lyric_pill_reads_edge_artwork_24_line_24_group_8_dots_edge() {
        let claimed = ClaimedRow::new(20.0, DOTS, &[16.0, 12.0]);
        assert_eq!(claimed.left_inset(), PILL_EDGE);
        assert_eq!(claimed.lead_gap(), CONTENT_GAP);
        let group = 16.0 + ITEM_GAP + 12.0;
        assert_eq!(claimed.right_inset(), PILL_EDGE + DOTS + ITEM_GAP + group + CONTENT_GAP);
        let width = claimed.pill_width(200.0);
        assert_eq!(width, 12.0 + 20.0 + 24.0 + 200.0 + 24.0 + group + 8.0 + DOTS + 12.0);
    }

    #[test]
    fn the_dots_end_the_lyric_pill_12_from_its_right_edge_as_in_the_idle_row() {
        let claimed = ClaimedRow::new(20.0, DOTS, &[16.0, 12.0, 30.0]);
        let width = claimed.pill_width(150.0);
        assert_eq!(claimed.dots_left(width), Some(width - PILL_EDGE - DOTS));
    }

    #[test]
    fn the_dots_sit_8_after_the_group() {
        let claimed = ClaimedRow::new(20.0, DOTS, &[16.0, 12.0]);
        let width = claimed.pill_width(200.0);
        let lefts = claimed.group_lefts(width);
        assert_eq!(claimed.dots_left(width).unwrap() - (lefts[1] + 12.0), ITEM_GAP);
    }

    #[test]
    fn without_a_dot_the_column_takes_no_width_and_the_group_ends_the_row() {
        let claimed = ClaimedRow::new(20.0, 0.0, &[16.0]);
        assert_eq!(claimed.dots_left(300.0), None);
        assert_eq!(claimed.lead_gap(), CONTENT_GAP);
        assert_eq!(claimed.pill_width(200.0), 12.0 + 20.0 + 24.0 + 200.0 + 24.0 + 16.0 + 12.0);
        let width = claimed.pill_width(200.0);
        assert_eq!(claimed.group_lefts(width)[0] + 16.0, width - PILL_EDGE);
    }

    #[test]
    fn a_dot_appearing_widens_the_lyric_pill_by_the_dot_and_its_8() {
        let without = ClaimedRow::new(20.0, 0.0, &[16.0]).pill_width(200.0);
        let with = ClaimedRow::new(20.0, DOTS, &[16.0]).pill_width(200.0);
        assert_eq!(with - without, ITEM_GAP + DOTS);
    }

    #[test]
    fn a_dot_never_moves_the_line_away_from_the_artwork() {
        let without = ClaimedRow::new(20.0, 0.0, &[16.0]);
        let with = ClaimedRow::new(20.0, DOTS, &[16.0]);
        assert_eq!(with.lead_gap(), without.lead_gap());
    }

    #[test]
    fn the_lyric_pill_without_a_group_keeps_the_dot_and_the_edge_at_the_right() {
        let claimed = ClaimedRow::new(20.0, DOTS, &[]);
        assert_eq!(claimed.right_inset(), PILL_EDGE + DOTS + ITEM_GAP);
        assert!(claimed.group_lefts(270.0).is_empty());
        assert_eq!(ClaimedRow::new(20.0, 0.0, &[]).right_inset(), PILL_EDGE);
    }

    #[test]
    fn the_group_runs_8_apart_from_24_after_the_line() {
        let claimed = ClaimedRow::new(20.0, DOTS, &[16.0, 12.0, 30.0]);
        let width = claimed.pill_width(150.0);
        let lefts = claimed.group_lefts(width);
        assert_eq!(lefts[1] - (lefts[0] + 16.0), ITEM_GAP);
        assert_eq!(lefts[2] - (lefts[1] + 12.0), ITEM_GAP);
        let content_right = width - claimed.right_inset();
        assert_eq!(lefts[0] - content_right, CONTENT_GAP);
    }

    #[test]
    fn without_a_lead_the_content_stays_left_and_the_dot_column_takes_the_right_end() {
        let claimed = ClaimedRow::new(0.0, DOTS, &[14.0, 20.0]);
        assert_eq!(claimed.left_inset(), 0.0);
        assert_eq!(claimed.lead_gap(), 0.0);
        let width = claimed.pill_width(200.0);
        let lefts = claimed.group_lefts(width);
        assert_eq!(lefts[1] + 20.0 + ITEM_GAP, claimed.dots_left(width).unwrap());
        assert_eq!(claimed.dots_left(width).unwrap() + DOTS, width - PILL_EDGE);
    }
}
