//! Every item of the pill's rows has its own spring, so an item joining or leaving slides its neighbours.

use super::{Notch, Source, animation::size_spring};
use crate::gfx::Spring;
use crate::host::{chrome, clock};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RowItem {
    Source(Source),
    /// The camera and microphone dots, one column for all of them.
    Dots,
    /// The pin left of the gear in the open header; it has no place in the collapsed pill.
    Pin,
    /// The settings gear at the right end of the open header; it has no place in the collapsed pill.
    Gear,
}

/// One spring per item of a row; positions are distances from the row's right edge to the item's left edge.
#[derive(Default)]
pub(super) struct ItemSprings(Vec<(RowItem, Spring)>);

impl ItemSprings {
    /// Drops the items that left, puts new ones straight at their place and retargets the rest; true when an item
    /// already shown has to move.
    pub(super) fn follow(&mut self, targets: &[(RowItem, f32)]) -> bool {
        self.0.retain(|(item, _)| targets.iter().any(|(target_item, _)| target_item == item));
        let mut moved = false;
        for &(item, target) in targets {
            match self.0.iter_mut().find(|(known, _)| *known == item) {
                Some((_, spring)) => {
                    if (spring.target - target).abs() > f32::EPSILON {
                        spring.target = target;
                        moved = true;
                    }
                }
                None => self.0.push((item, size_spring(target))),
            }
        }
        moved
    }

    pub(super) fn position(&self, item: RowItem) -> Option<f32> {
        self.0.iter().find(|(known, _)| *known == item).map(|(_, spring)| spring.position)
    }

    fn springs_mut(&mut self) -> impl Iterator<Item = &mut Spring> {
        self.0.iter_mut().map(|(_, spring)| spring)
    }
}

/// Where the idle clock and every group item sit in the collapsed pill and in the open header; the header clock
/// stays in the header's middle and needs no spring.
pub(super) struct RowMotion {
    /// The idle clock's left edge from the pill's left edge.
    pub(super) clock_left: Spring,
    /// Items of the idle or the claimed row, from the collapsed pill's right edge.
    pub(super) resting: ItemSprings,
    /// Items of the open header's group that have room, from the header's right edge.
    pub(super) header: ItemSprings,
}

impl RowMotion {
    pub(super) fn new() -> Self {
        Self { clock_left: size_spring(0.0), resting: ItemSprings::default(), header: ItemSprings::default() }
    }

    pub(super) fn springs_mut(&mut self) -> impl Iterator<Item = &mut Spring> {
        std::iter::once(&mut self.clock_left).chain(self.resting.springs_mut()).chain(self.header.springs_mut())
    }
}

/// Distances from the right edge of a row `width` wide to each item's left edge.
pub(super) fn insets(items: &[(RowItem, f32)], lefts: &[f32], width: f32) -> Vec<(RowItem, f32)> {
    items.iter().zip(lefts).map(|(&(item, _), &left)| (item, width - left)).collect()
}

impl Notch {
    /// Points every row spring at the layout just computed; true when something has to move.
    pub(super) fn retarget_rows(&mut self, expanded_width: f32) -> bool {
        let mut moved = false;
        let resting = match self.resting_index {
            Some(index) => {
                let pill_width = self.claimed_pill_width(index);
                let claim = &self.resting_row.claim;
                let mut claimed = insets(&self.claimed_items(), &claim.group_lefts(pill_width), pill_width);
                claimed.extend(claim.dots_left(pill_width).map(|left| (RowItem::Dots, pill_width - left)));
                claimed
            }
            None => insets(&self.idle_items(), &self.resting_row.layout.group_lefts, self.resting_row.layout.width),
        };
        moved |= self.row_motion.resting.follow(&resting);

        let half = expanded_width / 2.0 - chrome::PADDING_SIDE;
        let row = self.header_row_between(-half, half);
        let mut header = insets(&self.shown_header_group(&row), &row.group_lefts, half);
        header.push((RowItem::Pin, half - row.pin_left));
        header.push((RowItem::Gear, half - row.gear_left));
        moved |= self.row_motion.header.follow(&header);
        let idle_clock_left = self.resting_row.layout.clock_left;
        let spring = &mut self.row_motion.clock_left;
        if (spring.target - idle_clock_left).abs() > f32::EPSILON {
            spring.target = idle_clock_left;
            moved = true;
        }
        moved
    }

    /// The idle clock's center in DIP from the collapsed pill's left edge.
    pub(super) fn idle_clock_center(&self) -> f32 {
        self.row_motion.clock_left.position + clock::layout_width(&self.fonts) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RING: RowItem = RowItem::Source(Source::Activity(0));
    const TIMER: RowItem = RowItem::Source(Source::Page(3));

    #[test]
    fn a_new_item_starts_at_its_place_and_only_the_items_already_shown_move() {
        let mut springs = ItemSprings::default();
        assert!(!springs.follow(&[(RING, 26.0)]), "the first item has nothing to slide");
        assert!(!springs.follow(&[(TIMER, 70.0), (RING, 26.0), (RowItem::Dots, 18.0)]), "new items need no slide");
        assert_eq!(springs.position(TIMER), Some(70.0));
        assert!(springs.follow(&[(TIMER, 84.0), (RING, 40.0), (RowItem::Dots, 18.0)]));
        assert_eq!(springs.position(RING), Some(26.0), "the ring slides from where it was");
    }

    #[test]
    fn an_item_that_leaves_loses_its_spring() {
        let mut springs = ItemSprings::default();
        springs.follow(&[(TIMER, 70.0), (RING, 26.0)]);
        springs.follow(&[(RING, 26.0)]);
        assert_eq!(springs.position(TIMER), None);
    }

    #[test]
    fn insets_are_measured_from_the_right_edge_to_each_left_edge() {
        let items = [(TIMER, 40.0), (RING, 14.0)];
        assert_eq!(insets(&items, &[80.0, 124.0], 150.0), [(TIMER, 70.0), (RING, 26.0)]);
    }
}
