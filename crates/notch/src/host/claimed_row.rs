//! The collapsed pill while a page claims it: the page's content, the group and the dot column; sizes in DIP.

use super::pill_row::{ITEM_GAP, PILL_EDGE};

/// Between the artwork (or the dots after it) and the page's line, and between the line and the group.
pub const CONTENT_GAP: f32 = 24.0;
/// Between the artwork and the dot column beside it.
pub const LEAD_DOTS_GAP: f32 = 10.0;

/// `[edge][lead][10][dots][24][content][24][group][edge]`. The dots and their 10 take room only while a dot shows.
/// Without a lead the content starts at the pill's left edge and the dots end the row: `[group][8][dots][edge]`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClaimedRow {
    lead_width: f32,
    /// Zero while no dot shows.
    dots_width: f32,
    /// In the order the group is drawn, left to right.
    group_widths: Vec<f32>,
}

impl ClaimedRow {
    pub fn new(lead_width: f32, dots_width: f32, group_widths: &[f32]) -> Self {
        Self { lead_width, dots_width, group_widths: group_widths.to_vec() }
    }

    fn has_lead(&self) -> bool {
        self.lead_width > 0.0
    }

    /// The group with `ITEM_GAP` between its items, without the gap before it.
    fn group_width(&self) -> f32 {
        let count = self.group_widths.len();
        self.group_widths.iter().sum::<f32>() + count.saturating_sub(1) as f32 * ITEM_GAP
    }

    /// Room the group takes right of the content: its gap and the items; 0 without a group.
    fn group_room(&self) -> f32 {
        if self.group_widths.is_empty() { 0.0 } else { CONTENT_GAP + self.group_width() }
    }

    fn shows_dots(&self) -> bool {
        self.dots_width > 0.0
    }

    /// Room the dot column takes at the right end, only without a lead: the gap before it and the dots.
    fn trailing_dots(&self) -> f32 {
        if self.shows_dots() && !self.has_lead() { ITEM_GAP + self.dots_width } else { 0.0 }
    }

    /// Distance of the page's content from the pill's left edge.
    pub fn left_inset(&self) -> f32 {
        if self.has_lead() { PILL_EDGE } else { 0.0 }
    }

    /// What the page leaves free between its lead and its line: the dot column beside the lead while a dot shows,
    /// then `CONTENT_GAP`.
    pub fn lead_gap(&self) -> f32 {
        match (self.has_lead(), self.shows_dots()) {
            (false, _) => 0.0,
            (true, false) => CONTENT_GAP,
            (true, true) => LEAD_DOTS_GAP + self.dots_width + CONTENT_GAP,
        }
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
        if !self.shows_dots() {
            return None;
        }
        Some(if self.has_lead() {
            PILL_EDGE + self.lead_width + LEAD_DOTS_GAP
        } else {
            pill_width - PILL_EDGE - self.dots_width
        })
    }

    /// Left edges of the group items from the pill's left edge, in the order of the widths given to `new`.
    pub fn group_lefts(&self, pill_width: f32) -> Vec<f32> {
        let mut left = pill_width - PILL_EDGE - self.trailing_dots() - self.group_width();
        self.group_widths
            .iter()
            .map(|&width| {
                let item_left = left;
                left += width + ITEM_GAP;
                item_left
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOTS: f32 = 6.0;

    #[test]
    fn the_lyric_pill_reads_edge_artwork_10_dots_24_line_24_group_edge() {
        let claimed = ClaimedRow::new(20.0, DOTS, &[16.0, 12.0]);
        assert_eq!(claimed.left_inset(), PILL_EDGE);
        assert_eq!(claimed.lead_gap(), LEAD_DOTS_GAP + DOTS + CONTENT_GAP);
        let group = 16.0 + ITEM_GAP + 12.0;
        assert_eq!(claimed.right_inset(), PILL_EDGE + group + CONTENT_GAP);
        let width = claimed.pill_width(200.0);
        assert_eq!(width, 12.0 + 20.0 + 10.0 + DOTS + 24.0 + 200.0 + 24.0 + group + 12.0);
    }

    #[test]
    fn the_dots_sit_10_after_the_artwork() {
        let claimed = ClaimedRow::new(20.0, DOTS, &[16.0, 12.0, 30.0]);
        let width = claimed.pill_width(150.0);
        assert_eq!(claimed.dots_left(width), Some(PILL_EDGE + 20.0 + LEAD_DOTS_GAP));
    }

    #[test]
    fn the_line_starts_24_after_the_dots_while_a_dot_shows() {
        let claimed = ClaimedRow::new(20.0, DOTS, &[16.0]);
        let width = claimed.pill_width(200.0);
        let line_left = claimed.left_inset() + 20.0 + claimed.lead_gap();
        assert_eq!(line_left - (claimed.dots_left(width).unwrap() + DOTS), CONTENT_GAP);
    }

    #[test]
    fn without_a_dot_the_column_takes_no_width_and_the_line_starts_24_after_the_artwork() {
        let claimed = ClaimedRow::new(20.0, 0.0, &[16.0]);
        assert_eq!(claimed.dots_left(300.0), None);
        assert_eq!(claimed.lead_gap(), CONTENT_GAP);
        assert_eq!(claimed.pill_width(200.0), 12.0 + 20.0 + 24.0 + 200.0 + 24.0 + 16.0 + 12.0);
        let width = claimed.pill_width(200.0);
        assert_eq!(claimed.group_lefts(width)[0] + 16.0, width - PILL_EDGE);
    }

    #[test]
    fn a_dot_appearing_widens_the_lyric_pill_by_the_dot_and_its_10() {
        let without = ClaimedRow::new(20.0, 0.0, &[16.0]).pill_width(200.0);
        let with = ClaimedRow::new(20.0, DOTS, &[16.0]).pill_width(200.0);
        assert_eq!(with - without, LEAD_DOTS_GAP + DOTS);
    }

    #[test]
    fn a_dot_beside_the_artwork_never_moves_the_group() {
        let without = ClaimedRow::new(20.0, 0.0, &[16.0]);
        let with = ClaimedRow::new(20.0, DOTS, &[16.0]);
        assert_eq!(with.right_inset(), without.right_inset());
    }

    #[test]
    fn the_lyric_pill_without_a_group_keeps_only_the_edge_at_the_right() {
        let claimed = ClaimedRow::new(20.0, DOTS, &[]);
        assert_eq!(claimed.right_inset(), PILL_EDGE);
        assert!(claimed.group_lefts(270.0).is_empty());
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
