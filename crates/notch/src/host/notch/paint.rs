use windows::Win32::Foundation::RECT;

use super::{animation::ramp, resting_row::RowOpacity, *};
use crate::host::{
    activity_layout::{self, DIVIDER_WIDTH},
    chrome, clock,
};
use crate::{
    gfx::{Canvas, Color, Rect, lerp},
    log,
    page::{Frame, RESTING_HEIGHT},
};

const RESTING_RADIUS: f32 = 12.0;
const BANNER_RADIUS: f32 = 18.0;
const BANNER_RADIUS_RANGE: f32 = 20.0;
const EXPANDED_RADIUS: f32 = 22.0;
const SHADOW_LAYER_ALPHA: f32 = 0.028;
const HIDDEN_ALPHA: f32 = 0.006;
const PAGE_FADE_PER_PAGE: f32 = 0.5;

impl Notch {
    pub fn paint(&self) {
        if self.is_suppressed() {
            return;
        }
        let frame = self.frame();
        let region = self.pill_region(&frame, true);
        self.region_origin.set((region.left, region.top));
        let visibility = self.visibility();
        let resting_opacity = (1.0 - frame.expansion / RESTING_FADE_END).clamp(0.0, 1.0) * visibility;
        let fill = Color::BLACK.with_alpha(lerp(HIDDEN_ALPHA, 1.0, visibility));
        let blend = self.springs.activity_blend.position.clamp(0.0, 1.0);
        let page_opacity = resting_opacity * (1.0 - blend);
        let activity_opacity = resting_opacity * blend;
        let header_opacity = ramp(frame.expansion, HEADER_FADE);
        let expanded_opacity = ramp(frame.expansion, CONTENT_FADE);

        if !self.hover.is_hovering && frame.expansion <= 0.001 {
            crate::host::drop_target::set_resting_area(RECT {
                left: self.screen_position.x + region.left,
                top: self.screen_position.y + region.top,
                right: self.screen_position.x + region.right,
                bottom: self.screen_position.y + region.bottom,
            });
        }
        let result = self.surface.render_and_present(
            self.window,
            self.screen_position,
            region,
            &self.fonts,
            self.scale,
            |canvas| {
                let bounds = frame.bounds;
                let banner_amount = ((self.springs.resting_height.position - RESTING_PILL_HEIGHT)
                    / BANNER_RADIUS_RANGE)
                    .clamp(0.0, 1.0);
                let resting_radius = lerp(RESTING_RADIUS, BANNER_RADIUS, banner_amount);
                let radius = (lerp(resting_radius, EXPANDED_RADIUS, frame.expansion.clamp(0.0, 1.0)) * self.scale)
                    .min(bounds.height() / 2.0);
                self.draw_shadow(canvas, bounds, radius, frame.expansion.clamp(0.0, 1.0) * visibility);
                // The shape starts one radius above the screen edge so only the bottom corners show as rounded.
                canvas.fill_rounded_rect(
                    Rect::new(bounds.left, bounds.top - radius, bounds.right, bounds.bottom),
                    radius,
                    fill,
                );
                if visibility > 0.05 {
                    canvas.fill_top_flares(bounds, self.flare_size(&frame), Color::BLACK);
                }
                canvas.clipped(bounds, || {
                    if expanded_opacity > 0.01 {
                        self.draw_backdrops(canvas, &frame, radius, expanded_opacity);
                    }
                    if page_opacity > 0.01 {
                        if let Some(index) = self.resting_index {
                            let claim = &self.resting_row.claim;
                            let right_inset = self.springs.claim_right_inset.position.max(0.0) * self.scale;
                            let page_frame = Frame {
                                bounds: Rect {
                                    left: bounds.left + claim.left_inset() * self.scale,
                                    right: bounds.right - right_inset,
                                    ..bounds
                                },
                                lead_gap: self.springs.claim_lead_gap.position.max(0.0),
                                ..frame
                            };
                            self.pages[index].draw_resting(canvas, &page_frame, page_opacity);
                            self.draw_claimed_group(canvas, &frame, page_opacity);
                        } else {
                            self.draw_group(canvas, &frame, page_opacity);
                        }
                    }
                    let collapsed_visibility = visibility * (1.0 - blend);
                    let opacity = RowOpacity {
                        clock: clock::opacity(self.resting_index.is_none(), collapsed_visibility, header_opacity),
                        dots: clock::opacity(true, collapsed_visibility, header_opacity),
                        header: header_opacity,
                    };
                    if opacity.clock > 0.01 || opacity.dots > 0.01 || opacity.header > 0.01 {
                        self.draw_row(canvas, &frame, &opacity);
                    }
                    if activity_opacity > 0.01 {
                        if self.shown_banner.is_some() {
                            self.draw_banner(canvas, &frame, activity_opacity);
                        } else {
                            self.draw_transients(canvas, &frame, activity_opacity);
                        }
                    }
                    if header_opacity > 0.01 {
                        self.draw_header(canvas, &frame, header_opacity);
                    }
                    if expanded_opacity > 0.01 {
                        self.draw_open_pages(canvas, &frame, expanded_opacity);
                    }
                });
            },
        );
        if let Err(error) = result {
            log::debug(&format!("paint failed: {error}"));
        }
    }

    /// Backdrops crossfade in place while page content slides.
    pub(super) fn draw_backdrops(&self, canvas: &Canvas, frame: &Frame, radius: f32, opacity: f32) {
        let Some(expanded_index) = self.expanded_index else { return };
        if let Some(pages) = self.visible_page_frames(frame) {
            for (index, _, distance) in pages {
                let page_frame = self.natural_page_frame(index, frame, 0.0);
                self.pages[index].draw_backdrop(canvas, &page_frame, frame.bounds, radius, opacity * (1.0 - distance));
            }
            return;
        }
        let index = if self.navigable.len() < 2 { expanded_index } else { self.page_at(self.track.position) };
        let page_frame = self.natural_page_frame(index, frame, 0.0);
        self.pages[index].draw_backdrop(canvas, &page_frame, frame.bounds, radius, opacity);
    }

    pub(super) fn draw_open_pages(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let Some(expanded_index) = self.expanded_index else { return };
        if let Some(pages) = self.visible_page_frames(frame) {
            for (index, page_frame, distance) in pages {
                let fade = (1.0 - distance * PAGE_FADE_PER_PAGE).clamp(0.0, 1.0);
                self.pages[index].draw_expanded(canvas, &page_frame, opacity * fade);
            }
            return;
        }
        let index = if self.navigable.len() < 2 { expanded_index } else { self.page_at(self.track.position) };
        let page_frame = self.natural_page_frame(index, frame, 0.0);
        self.pages[index].draw_expanded(canvas, &page_frame, opacity);
    }

    pub(super) fn draw_shadow(&self, canvas: &Canvas, bounds: Rect, radius: f32, strength: f32) {
        if strength <= 0.01 {
            return;
        }
        let color = Color::BLACK.with_alpha(SHADOW_LAYER_ALPHA * strength);
        for layer in (1..=SHADOW_LAYERS).rev() {
            let grow = layer as f32 * SHADOW_STEP * self.scale;
            let rect = Rect::new(
                bounds.left - grow,
                bounds.top - radius,
                bounds.right + grow,
                bounds.bottom + grow + SHADOW_OFFSET * self.scale,
            );
            canvas.fill_rounded_rect(rect, radius + grow, color);
        }
    }

    pub(super) fn draw_header(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        if self.navigable.len() >= 2 {
            let dots = chrome::dot_rects(self.navigable.len(), self.track.position, frame.bounds, self.scale);
            chrome::draw_dots(canvas, &dots, self.track.position, opacity);
        }
    }

    pub(super) fn draw_transients(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let scale = self.scale;
        let widths: Vec<f32> = self.shown_transients.iter().map(|&(_, width)| width).collect();
        let offsets = activity_layout::offsets(&widths);
        let total = activity_layout::total_width(&widths) * scale;
        let first_left = frame.bounds.left + (frame.bounds.width() - total) / 2.0;
        let (top, bottom) = (frame.bounds.top, frame.bounds.top + RESTING_HEIGHT * scale);
        for (position, &(source, width)) in self.shown_transients.iter().enumerate() {
            let left = first_left + offsets[position] * scale;
            let area = Rect::new(left, top, left + width * scale, bottom);
            self.draw_source(source, canvas, area, opacity);
            if position > 0 {
                let divider = Rect::new(left - DIVIDER_WIDTH * scale, top + 9.0 * scale, left, bottom - 9.0 * scale);
                canvas.fill_rounded_rect(divider, 0.0, Color::gray(0.35).faded(opacity));
            }
        }
    }

    pub(super) fn draw_banner(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let Some((source, area)) = self.banner_area(frame) else { return };
        self.draw_source(source, canvas, area, opacity);
    }

    /// The banner's owner and the rect it draws into, which is also where its pointer input is measured.
    pub(super) fn banner_area(&self, frame: &Frame) -> Option<(Source, Rect)> {
        let (source, _, _) = self.shown_banner?;
        let bounds = frame.bounds;
        Some((source, Rect { bottom: bounds.bottom - RESTING_BOTTOM_PADDING * self.scale, ..bounds }))
    }

    /// A badge of a page draws its short form when `short` is set: several badges with text share the pill.
    pub(super) fn draw_badge(&self, source: Source, canvas: &Canvas, area: Rect, short: bool, opacity: f32) {
        match source {
            Source::Page(index) if short => self.pages[index].draw_short_presence(canvas, area, self.scale, opacity),
            _ => self.draw_source(source, canvas, area, opacity),
        }
    }

    pub(super) fn draw_source(&self, source: Source, canvas: &Canvas, area: Rect, opacity: f32) {
        match source {
            Source::Activity(index) => self.activities[index].activity.draw(canvas, area, self.scale, opacity),
            Source::Page(index) => self.pages[index].draw_presence(canvas, area, self.scale, opacity),
        }
    }
}
