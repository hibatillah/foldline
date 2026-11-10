//! The open header's icons and gear, and what a click on them does.

use super::{Notch, Source, row_motion::RowItem, units};
use crate::host::{
    chrome, clock,
    pill_row::{self, HeaderInput, HeaderRow},
    settings_launcher,
};
use crate::{
    gfx::{Canvas, Color, Point, Rect},
    ui::Icon,
};

/// `#8c8c94`.
const GEAR_COLOR: Color = Color::rgb(0.549, 0.549, 0.580);

/// What a click on one item of the header does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HeaderTarget {
    Page(&'static str),
    Settings,
    Nothing,
}

/// The header's icons, lowest rank first, with the width each asks for; rebuilt by `refresh_layout` so a paint
/// only borrows them.
#[derive(Default)]
pub(super) struct HeaderMarks {
    pub(super) icons: Vec<(f32, Source)>,
}

impl Notch {
    pub(super) fn refresh_header_marks(&mut self) {
        let mut entries: Vec<(u8, f32, Source)> = Vec::new();
        for (index, slot) in self.activities.iter().enumerate() {
            if !slot.is_enabled || !slot.is_started {
                continue;
            }
            if let Some(width) = slot.activity.header_status_width() {
                entries.push((slot.activity.header_status_rank(), width, Source::Activity(index)));
            }
        }
        for (index, page) in self.pages.iter().enumerate() {
            if !self.page_list.is_enabled(index) {
                continue;
            }
            if let Some(width) = page.header_status_width() {
                entries.push((page.header_status_rank(), width, Source::Page(index)));
            }
        }
        entries.sort_by_key(|&(rank, _, _)| rank);
        self.header_marks.icons.clear();
        self.header_marks.icons.extend(entries.into_iter().map(|(_, width, source)| (width, source)));
    }

    /// The header's whole group left to right: the icons by rank, the battery ring, the dot column; the gear follows.
    pub(super) fn header_group(&self) -> Vec<(RowItem, f32)> {
        let icons = self.header_marks.icons.iter().map(|&(width, source)| (RowItem::Source(source), width));
        icons.chain(self.resting_row.right_end()).collect()
    }

    /// The part of the group `row` has room for, in the order of `row.group_lefts`.
    pub(super) fn shown_header_group(&self, row: &HeaderRow) -> Vec<(RowItem, f32)> {
        self.header_group().into_iter().skip(row.dropped_icons).collect()
    }

    /// The width the header's content needs with its gaps and the side padding.
    pub(super) fn header_min_width(&self) -> f32 {
        let end_widths: Vec<f32> = self.resting_row.right_end().iter().map(|&(_, width)| width).collect();
        let content = pill_row::header_content_width(
            chrome::dots_width(self.navigable.len()),
            clock::layout_width(&self.fonts),
            &end_widths,
        );
        content + chrome::WIDTH_EXTRA
    }

    pub(super) fn header_row_between(&self, left: f32, right: f32) -> HeaderRow {
        let icon_widths: Vec<f32> = self.header_marks.icons.iter().map(|&(width, _)| width).collect();
        let end_widths: Vec<f32> = self.resting_row.right_end().iter().map(|&(_, width)| width).collect();
        pill_row::header_row(&HeaderInput {
            left,
            right,
            clock_width: clock::layout_width(&self.fonts),
            icon_widths: &icon_widths,
            end_widths: &end_widths,
        })
    }

    /// Settled positions of the header's items for a pill at `bounds`, in DIP from the surface origin.
    pub(super) fn header_layout(&self, bounds: Rect) -> HeaderRow {
        let header = chrome::header_rect(bounds, self.scale);
        self.header_row_between(header.left / self.scale, header.right / self.scale)
    }

    /// The gear and the icons; the clock, the ring and the dots are drawn with the idle row because they move there.
    pub(super) fn draw_header_marks(&self, canvas: &Canvas, bounds: Rect, opacity: f32) {
        let scale = self.scale;
        let header = chrome::header_rect(bounds, scale);
        let center_y = header.center_y();
        let half_height = chrome::HEADER_HEIGHT * scale / 2.0;
        for &(width, source) in &self.header_marks.icons {
            let Some(inset) = self.row_motion.header.position(RowItem::Source(source)) else { continue };
            let left = header.right - inset * scale;
            let area = Rect::new(left, center_y - half_height, left + width * scale, center_y + half_height);
            match source {
                Source::Page(index) => {
                    units::draw_page_header_status(self.pages[index].as_ref(), canvas, area, opacity)
                }
                Source::Activity(index) => {
                    units::draw_activity_header_status(self.activities[index].activity.as_ref(), canvas, area, opacity)
                }
            }
        }
        if let Some(inset) = self.row_motion.header.position(RowItem::Gear) {
            let gear_center = Point::new(header.right - (inset - pill_row::GEAR_SIZE / 2.0) * scale, center_y);
            Icon::Gear.draw(canvas, gear_center, pill_row::GEAR_SIZE * scale, GEAR_COLOR, opacity);
        }
    }

    /// Every item a click can land on, left to right: the shown group, then the gear.
    fn header_targets(&self, row: &HeaderRow) -> Vec<(f32, f32, HeaderTarget)> {
        let items = self.shown_header_group(row).into_iter().chain([(RowItem::Gear, pill_row::GEAR_SIZE)]);
        let lefts = row.group_lefts.iter().copied().chain([row.gear_left]);
        items
            .zip(lefts)
            .map(|((item, width), left)| {
                let page_id = match item {
                    RowItem::Source(Source::Page(index)) => self.pages[index].header_status_page(),
                    RowItem::Source(Source::Activity(index)) => self.activities[index].activity.header_status_page(),
                    RowItem::Dots => None,
                    RowItem::Gear => return (left, width, HeaderTarget::Settings),
                };
                (left, width, page_id.map_or(HeaderTarget::Nothing, HeaderTarget::Page))
            })
            .collect()
    }

    /// What a press at `point` (device pixels) on the open header does; `None` when it hits no item.
    pub(super) fn header_target_at(&self, point: Point) -> Option<HeaderTarget> {
        let bounds = self.frame().bounds;
        let header = chrome::header_rect(bounds, self.scale);
        if (point.y - header.center_y()).abs() > pill_row::HIT_SIZE * self.scale / 2.0 {
            return None;
        }
        let targets = self.header_targets(&self.header_layout(bounds));
        let items: Vec<(f32, f32)> = targets.iter().map(|&(left, width, _)| (left, width)).collect();
        pill_row::hit_item(&items, point.x / self.scale).map(|position| targets[position].2)
    }

    /// Runs the press; true when it hit an item (even one that does nothing), so the page below never sees it.
    pub(super) fn press_header_marks(&mut self, point: Point) -> bool {
        match self.header_target_at(point) {
            None => false,
            Some(HeaderTarget::Page(id)) => {
                if let Some(index) = self.pages.iter().position(|page| page.id() == id)
                    && self.navigable.contains(&index)
                {
                    self.go_to_page(index);
                }
                true
            }
            Some(HeaderTarget::Settings) => {
                settings_launcher::open_settings_window();
                true
            }
            Some(HeaderTarget::Nothing) => true,
        }
    }
}
