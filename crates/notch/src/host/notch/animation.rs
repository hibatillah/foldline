use super::{
    ANIMATION_TIMER_ID, HEADER_STAGE, Notch, OPEN_RESPONSE, RESTING_FLARE, RESTING_PILL_HEIGHT, SHADOW_LAYERS,
    SHADOW_OFFSET, SHADOW_STEP, Spring, WINDOW_WIDTH,
};
use windows::Win32::{
    Foundation::RECT,
    Media::{timeBeginPeriod, timeEndPeriod},
    System::Performance::QueryPerformanceCounter,
    UI::WindowsAndMessaging::{KillTimer, SetTimer},
};

use crate::host::chrome;
use crate::{
    gfx::{Rect, Size, lerp},
    page::Frame,
};

/// The springs that size and fade the pill and its content.
pub(super) struct Springs {
    pub(super) expansion: Spring,
    pub(super) resting_width: Spring,
    pub(super) resting_height: Spring,
    pub(super) expanded_width: Spring,
    pub(super) expanded_height: Spring,
    pub(super) activity_blend: Spring,
    pub(super) badge_width: Spring,
}

impl Springs {
    pub(super) fn new() -> Self {
        Self {
            expansion: Spring::new(0.0, OPEN_RESPONSE.0, OPEN_RESPONSE.1).with_rebound_damping(1.4),
            resting_width: Spring::new(0.0, 220.0, 26.0),
            resting_height: Spring::new(RESTING_PILL_HEIGHT, 260.0, 26.0),
            expanded_width: Spring::new(0.0, 200.0, 26.0),
            expanded_height: Spring::new(0.0, 220.0, 26.0),
            activity_blend: Spring::new(0.0, 300.0, 28.0),
            badge_width: Spring::new(0.0, 220.0, 26.0),
        }
    }
}

const EXPANDED_FLARE: f32 = 12.0;
const HEADER_STAGE_WIDTH: f32 = 240.0;
const HEADER_STAGE_HEIGHT: f32 = 52.0;

/// Interpolate through the header stage and extrapolate past the endpoints for spring overshoot.
pub(super) fn staged(resting: f32, header: f32, full: f32, expansion: f32) -> f32 {
    if expansion <= HEADER_STAGE {
        lerp(resting, header, expansion / HEADER_STAGE)
    } else {
        lerp(header, full, (expansion - HEADER_STAGE) / (1.0 - HEADER_STAGE))
    }
}

pub(super) fn ramp(expansion: f32, range: (f32, f32)) -> f32 {
    ((expansion - range.0) / (range.1 - range.0)).clamp(0.0, 1.0)
}

impl Notch {
    pub(super) fn start_animation(&mut self) {
        if self.is_animating {
            return;
        }
        self.is_animating = true;
        unsafe {
            timeBeginPeriod(1);
            let _ = QueryPerformanceCounter(&mut self.last_tick);
            SetTimer(Some(self.window), ANIMATION_TIMER_ID, self.frame_interval_milliseconds, None);
        }
    }

    pub(super) fn stop_animation(&mut self) {
        self.is_animating = false;
        unsafe {
            let _ = KillTimer(Some(self.window), ANIMATION_TIMER_ID);
            timeEndPeriod(1);
        }
    }

    pub(super) fn advance_animation(&mut self) {
        let mut now = 0;
        unsafe {
            let _ = QueryPerformanceCounter(&mut now);
        }
        let elapsed_seconds = ((now - self.last_tick) as f32 / self.ticks_per_second as f32).clamp(0.0, 0.05);
        self.last_tick = now;

        let follows_page_width = self.holds_page_position || !self.track.is_settled();
        self.springs.expansion.step(elapsed_seconds);
        for spring in self.layout_springs() {
            spring.step(elapsed_seconds);
        }
        if follows_page_width {
            self.follow_page_width();
        }
        let is_settled =
            self.springs.expansion.is_settled() && self.layout_springs().iter().all(|spring| spring.is_settled());
        let mut pages_busy = false;
        for page in &mut self.pages {
            pages_busy |= page.animate(elapsed_seconds);
        }

        if is_settled && !pages_busy {
            self.springs.expansion.snap_to_target();
            for spring in self.layout_springs() {
                spring.snap_to_target();
            }
            if follows_page_width {
                self.follow_page_width();
            }
            self.stop_animation();
        }
        self.paint();
    }

    pub(super) fn frame(&self) -> Frame {
        let amount = self.springs.expansion.position.clamp(-0.08, 1.1);
        let (resting_width, full_width) = (self.springs.resting_width.position, self.springs.expanded_width.position);
        let (resting_height, full_height) =
            (self.springs.resting_height.position, self.springs.expanded_height.position);
        let header_width = HEADER_STAGE_WIDTH.min(full_width).max(resting_width);
        let header_height = HEADER_STAGE_HEIGHT.min(full_height).max(resting_height);
        let width = staged(resting_width, header_width, full_width, amount) * self.scale;
        let height = staged(resting_height, header_height, full_height, amount) * self.scale;
        let left = (WINDOW_WIDTH * self.scale - width) / 2.0;
        Frame {
            scale: self.scale,
            bounds: Rect::new(left, 0.0, left + width, height),
            expansion: self.springs.expansion.position,
            expanded: Size::new(
                self.springs.expanded_width.position - chrome::WIDTH_EXTRA,
                self.springs.expanded_height.position - chrome::HEIGHT_EXTRA,
            ),
        }
    }

    pub(super) fn flare_size(&self, frame: &Frame) -> f32 {
        lerp(RESTING_FLARE, EXPANDED_FLARE, frame.expansion.clamp(0.0, 1.0)) * self.scale * self.visibility()
    }

    pub(super) fn visibility(&self) -> f32 {
        ((self.springs.resting_height.position - self.hidden_height()) / (RESTING_PILL_HEIGHT - self.hidden_height()))
            .clamp(0.0, 1.0)
            .max(self.springs.expansion.position.clamp(0.0, 1.0))
    }

    pub(super) fn pill_region(&self, frame: &Frame, include_shadow: bool) -> RECT {
        const MARGIN: i32 = 2;
        let (surface_width, surface_height) = (self.surface.width, self.surface.height);
        let shadow = if include_shadow { frame.expansion.clamp(0.0, 1.0) * self.scale } else { 0.0 };
        let shadow_sides = (SHADOW_LAYERS as f32 * SHADOW_STEP * shadow).ceil() as i32;
        let shadow_below = ((SHADOW_LAYERS as f32 * SHADOW_STEP + SHADOW_OFFSET) * shadow).ceil() as i32;
        let side_extent = MARGIN + self.flare_size(frame).ceil() as i32 + shadow_sides;
        let left = (frame.bounds.left.floor() as i32 - side_extent).clamp(0, surface_width - 1);
        let right = (frame.bounds.right.ceil() as i32 + side_extent).clamp(left + 1, surface_width);
        let bottom = (frame.bounds.bottom.ceil() as i32 + MARGIN + shadow_below).clamp(1, surface_height);
        RECT { left, top: 0, right, bottom }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::*, *};

    #[test]
    pub(super) fn a_staged_size_passes_through_the_header_pill_on_the_way_to_the_open_notch() {
        assert_eq!(staged(96.0, 240.0, 280.0, 0.0), 96.0);
        assert_eq!(staged(96.0, 240.0, 280.0, HEADER_STAGE), 240.0);
        assert_eq!(staged(96.0, 240.0, 280.0, 1.0), 280.0);
        assert_eq!(staged(96.0, 240.0, 280.0, 0.25), 168.0);
        assert_eq!(staged(96.0, 240.0, 280.0, 0.75), 260.0);
    }
    #[test]
    pub(super) fn a_staged_size_goes_past_both_ends_when_the_spring_overshoots() {
        assert!(staged(96.0, 240.0, 280.0, 1.05) > 280.0);
        assert!(staged(96.0, 240.0, 280.0, -0.05) < 96.0);
    }
    #[test]
    pub(super) fn the_header_shows_before_the_page_content_and_the_collapsed_content_is_gone_first() {
        assert!(RESTING_FADE_END <= HEADER_FADE.0 + 0.1);
        assert!(HEADER_FADE.1 <= CONTENT_FADE.0);
        assert_eq!(ramp(HEADER_STAGE, HEADER_FADE), 1.0);
        assert_eq!(ramp(HEADER_STAGE, CONTENT_FADE), 0.0);
        assert_eq!(ramp(1.0, CONTENT_FADE), 1.0);
    }
}
