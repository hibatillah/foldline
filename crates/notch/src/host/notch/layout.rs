use super::{HIDDEN_WIDTH, Notch, RESTING_BOTTOM_PADDING, RESTING_PILL_HEIGHT, units};
use crate::host::{activity_layout, chrome, navigation};
use crate::{
    gfx::{Point, Size, Spring},
    page::{DEFAULT_CONTENT_HEIGHT, Frame, KeyboardOwner, MAX_CONTENT_HEIGHT, Mode, Priority, RESTING_MIN_WIDTH},
};

const DRAG_GROWTH: Size = Size::new(28.0, 14.0);
/// The page area of the open notch while no page has anything to show: only the header remains.
const EMPTY_PAGE_SIZE: Size = Size::new(220.0, 0.0);
/// The page whose open width every page opens at least as wide as, while it is switched on.
pub(super) const WIDTH_FLOOR_PAGE_ID: &str = "timer";

/// A page's own content height, capped to what the window holds; nothing reported means the shared default.
pub(super) fn content_height(reported: f32) -> f32 {
    if reported > 0.0 { reported.min(MAX_CONTENT_HEIGHT) } else { DEFAULT_CONTENT_HEIGHT }
}

impl Notch {
    pub(super) fn page_frame(&self, frame: &Frame) -> Frame {
        Frame { bounds: chrome::content_rect(frame.bounds, self.scale), ..*frame }
    }

    /// Whether the open page has its own use for a right-click at `point`.
    pub(super) fn page_uses_right_click(&self, point: Point) -> bool {
        let Some(index) = self.expanded_index else { return false };
        let frame = self.natural_page_frame(index, &self.frame(), 0.0);
        units::page_uses_right_click(self.pages[index].as_ref(), point, &frame, self.scale)
    }

    /// Whether the open page scrolls something at `point` with the vertical wheel.
    pub(super) fn page_claims_scroll(&self, point: Point) -> bool {
        let Some(index) = self.expanded_index else { return false };
        let viewport = self.frame();
        if !viewport.is_open() || !chrome::content_rect(viewport.bounds, self.scale).contains(point) {
            return false;
        }
        let frame = self.natural_page_frame(index, &viewport, 0.0);
        units::page_claims_scroll(self.pages[index].as_ref(), point, &frame, self.scale)
    }

    /// A page's own frame: its requested width and height from the top of the content area, shifted `center_offset` DIP.
    /// It never follows the pill's size spring, so a page entering a pill that is still resizing moves only sideways.
    pub(super) fn natural_page_frame(&self, index: usize, viewport: &Frame, center_offset: f32) -> Frame {
        let mut frame = self.page_frame(viewport);
        let size = self.pages[index].sizes().expanded;
        let center = frame.bounds.center_x() + center_offset * self.scale;
        let left = if self.pages[index].is_left_aligned() {
            frame.bounds.left + center_offset * self.scale
        } else {
            center - size.width * self.scale / 2.0
        };
        let growth = if self.hover.is_dragging_files { DRAG_GROWTH.height } else { 0.0 };
        let height = content_height(size.height) + growth;
        frame.bounds.left = left;
        frame.bounds.right = left + size.width * self.scale;
        frame.bounds.bottom = frame.bounds.top + height * self.scale;
        frame.expanded = Size::new(size.width, height);
        frame
    }

    /// The open page and, during a swap, the page leaving it, each with its frame and opacity; the leaving one first.
    pub(super) fn swap_page_frames(&self, viewport: &Frame) -> [Option<(usize, Frame, f32)>; 2] {
        let leaving = self.page_swap.leaving().filter(|&(index, _)| self.page_list.is_enabled(index));
        let leaving =
            leaving.map(|(index, look)| (index, self.natural_page_frame(index, viewport, look.offset), look.opacity));
        let arriving = self.expanded_index.map(|index| {
            let look = self.page_swap.arriving();
            (index, self.natural_page_frame(index, viewport, look.offset), look.opacity)
        });
        [leaving, arriving]
    }

    /// Starts a swap when the open page changes while the notch shows it; closed, the new page is simply there.
    fn begin_page_swap(&mut self, previous: Option<usize>) {
        let (Some(leaving), Some(arriving)) = (previous, self.expanded_index) else { return };
        if leaving == arriving || !self.hover.is_hovering || self.springs.expansion.position <= 0.0 {
            return;
        }
        let direction = self.track.target - self.track.position;
        self.page_swap.begin(leaving, if direction < 0.0 { -1.0 } else { 1.0 });
        self.start_animation();
    }

    pub(super) fn sync_shown_page(&mut self) -> bool {
        let is_typing_in_a_page = matches!(self.keyboard.owner, Some(KeyboardOwner::Page(_)));
        let shown = (self.hover.is_hovering || is_typing_in_a_page).then_some(self.expanded_index).flatten();
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
        let presences_changed = self.refresh_presences();
        let selection_changed =
            resting_index != self.resting_index || expanded_index != self.expanded_index || presences_changed;
        self.resting_index = resting_index;
        let previous_expanded_index = std::mem::replace(&mut self.expanded_index, expanded_index);
        self.follow_open_page();
        self.begin_page_swap(previous_expanded_index);

        self.refresh_header_marks();
        self.refresh_resting_row();
        let expanded = expanded_index.map_or(EMPTY_PAGE_SIZE, |index| self.pages[index].sizes().expanded);
        let expanded_width = (expanded.width + chrome::WIDTH_EXTRA).max(self.open_min_width());
        let is_claimed = resting_index.is_some();
        let claim_right_inset = if is_claimed { self.resting_row.claim.right_inset() } else { 0.0 };
        let claim_lead_gap = self.resting_row.claim.lead_gap();
        let banner = self.presences.banner;
        // Only the empty idle pill may be narrower than the minimum: it hugs its centered clock.
        let resting_target = if let Some((_, width, _)) = banner {
            width.max(RESTING_MIN_WIDTH)
        } else if !self.presences.transients.is_empty() {
            activity_layout::total_width(self.presences.transients.iter().map(|&(_, width)| width))
                .max(RESTING_MIN_WIDTH)
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
            (
                &mut self.springs.activity_blend,
                if self.presences.transients.is_empty() && banner.is_none() { 0.0 } else { 1.0 },
            ),
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

    /// Moves the dots' long mark to the selected page; jumps while closed.
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
        navigation::highest(self.pages.iter().map(|page| page.priority(mode)), Priority::BACKGROUND)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_page_sets_its_own_open_height_up_to_what_the_window_holds() {
        assert_eq!(content_height(86.0), 86.0);
        assert_eq!(content_height(150.0), 150.0);
        assert_eq!(content_height(400.0), MAX_CONTENT_HEIGHT);
        assert_eq!(content_height(0.0), DEFAULT_CONTENT_HEIGHT);
    }
}
