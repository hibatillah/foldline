use super::{
    ANIMATION_TIMER_ID, Notch, RESTING_FLARE, RESTING_PILL_HEIGHT, SHADOW_LAYERS, SHADOW_OFFSET, SHADOW_STEP, Spring,
    WINDOW_WIDTH,
};
use windows::Win32::{
    Foundation::RECT,
    Media::{timeBeginPeriod, timeEndPeriod},
    System::Performance::QueryPerformanceCounter,
    UI::WindowsAndMessaging::{KillTimer, SetTimer},
};

use crate::host::{chrome, motion};
use crate::{
    gfx::{Rect, Size, lerp},
    log,
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
    /// Distance of a claimed page's content from the pill's right edge: the group, the dot column and the edge.
    pub(super) claim_right_inset: Spring,
    /// Room a claimed page keeps between its lead and its line, so the line slides when a dot joins the lead.
    pub(super) claim_lead_gap: Spring,
}

impl Springs {
    pub(super) fn new() -> Self {
        Self {
            expansion: size_spring(0.0),
            resting_width: size_spring(0.0),
            resting_height: size_spring(RESTING_PILL_HEIGHT),
            expanded_width: size_spring(0.0),
            expanded_height: size_spring(0.0),
            activity_blend: Spring::new(0.0, 300.0, 28.0),
            claim_right_inset: size_spring(0.0),
            claim_lead_gap: size_spring(0.0),
        }
    }
}

/// The Windows timer that steps the springs while anything moves.
pub(super) struct FrameTimer {
    pub(super) is_running: bool,
    last_tick: i64,
    ticks_per_second: i64,
    interval_milliseconds: u32,
}

impl FrameTimer {
    pub(super) fn new(ticks_per_second: i64, interval_milliseconds: u32) -> Self {
        Self { is_running: false, last_tick: 0, ticks_per_second, interval_milliseconds }
    }
}

/// Every size spring shares one response and lead-in, so width and height of a move start and settle together.
pub(super) fn size_spring(position: f32) -> Spring {
    Spring::new(position, motion::SIZE_RESPONSE.0, motion::SIZE_RESPONSE.1).with_lead_in(motion::SIZE_LEAD_IN)
}

const EXPANDED_FLARE: f32 = 12.0;

/// Width and height go through the same open amount; it extrapolates past 0 and 1 while the spring overshoots.
pub(super) fn sized(resting: f32, full: f32, expansion: f32) -> f32 {
    lerp(resting, full, expansion)
}

/// Keeps an overshooting size inside the window (`room`) and above a fraction of the collapsed size.
pub(super) fn bounded(size: f32, resting: f32, room: f32) -> f32 {
    size.min(room).max((resting * motion::MIN_SIZE_FRACTION).min(room))
}

pub(super) fn ramp(expansion: f32, range: (f32, f32)) -> f32 {
    ((expansion - range.0) / (range.1 - range.0)).clamp(0.0, 1.0)
}

impl Notch {
    pub(super) fn start_animation(&mut self) {
        if self.frame_timer.is_running {
            return;
        }
        self.frame_timer.is_running = true;
        log::debug("frame timer started");
        unsafe { timeBeginPeriod(1) };
        let _ = unsafe { QueryPerformanceCounter(&mut self.frame_timer.last_tick) };
        unsafe { SetTimer(Some(self.window), ANIMATION_TIMER_ID, self.frame_timer.interval_milliseconds, None) };
    }

    pub(super) fn stop_animation(&mut self) {
        self.frame_timer.is_running = false;
        log::debug("frame timer stopped");
        let _ = unsafe { KillTimer(Some(self.window), ANIMATION_TIMER_ID) };
        unsafe { timeEndPeriod(1) };
    }

    pub(super) fn advance_animation(&mut self) {
        let mut now = 0;
        let _ = unsafe { QueryPerformanceCounter(&mut now) };
        let elapsed_seconds =
            ((now - self.frame_timer.last_tick) as f32 / self.frame_timer.ticks_per_second as f32).clamp(0.0, 0.05);
        self.frame_timer.last_tick = now;
        self.advance_by(elapsed_seconds);
    }

    /// One frame of `elapsed_seconds`; stops the frame timer when everything has settled.
    pub(super) fn advance_by(&mut self, elapsed_seconds: f32) {
        self.springs.expansion.step(elapsed_seconds);
        for spring in self.layout_springs() {
            spring.step(elapsed_seconds);
        }
        let is_settled = self.springs.expansion.is_settled() && self.layout_springs().all(|spring| spring.is_settled());
        let mut pages_busy = self.page_swap.advance(elapsed_seconds);
        for page in &mut self.pages {
            pages_busy |= page.animate(elapsed_seconds);
        }

        if is_settled && !pages_busy {
            self.springs.expansion.snap_to_target();
            for spring in self.layout_springs() {
                spring.snap_to_target();
            }
            self.page_swap.finish();
            self.stop_animation();
        }
        self.sync_page_hotkeys();
        self.paint();
    }

    pub(super) fn frame(&self) -> Frame {
        let (lowest, highest) = motion::EXPANSION_OVERSHOOT_LIMIT;
        let amount = self.springs.expansion.position.clamp(lowest, highest);
        let (resting_width, full_width) = (self.springs.resting_width.position, self.springs.expanded_width.position);
        let (resting_height, full_height) =
            (self.springs.resting_height.position, self.springs.expanded_height.position);
        let (room_width, room_height) = self.size_room();
        let width =
            bounded(sized(resting_width, full_width, amount) * self.scale, resting_width * self.scale, room_width);
        let height =
            bounded(sized(resting_height, full_height, amount) * self.scale, resting_height * self.scale, room_height);
        let left = (WINDOW_WIDTH * self.scale - width) / 2.0;
        Frame {
            scale: self.scale,
            bounds: Rect::new(left, 0.0, left + width, height),
            expansion: self.springs.expansion.position,
            expanded: Size::new(
                self.springs.expanded_width.position - chrome::WIDTH_EXTRA,
                self.springs.expanded_height.position - chrome::HEIGHT_EXTRA,
            ),
            lead_gap: 0.0,
        }
    }

    /// The widest and tallest the pill may be, keeping its flares and shadow on the surface.
    fn size_room(&self) -> (f32, f32) {
        let side = (EXPANDED_FLARE + SHADOW_LAYERS as f32 * SHADOW_STEP) * self.scale;
        let below = (SHADOW_LAYERS as f32 * SHADOW_STEP + SHADOW_OFFSET) * self.scale;
        (self.surface.width as f32 - 2.0 * side, self.surface.height as f32 - below)
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
    fn width_and_height_share_one_open_amount_so_they_arrive_together() {
        for amount in [0.0, 0.3, 0.5, 1.0, 1.1] {
            let width_progress = (sized(100.0, 500.0, amount) - 100.0) / 400.0;
            let height_progress = (sized(30.0, 210.0, amount) - 30.0) / 180.0;
            assert!((width_progress - height_progress).abs() < 1e-5, "amount {amount}");
        }
    }
    #[test]
    fn a_size_goes_past_both_ends_when_the_spring_overshoots() {
        assert!(sized(96.0, 280.0, 1.05) > 280.0);
        assert!(sized(96.0, 280.0, -0.05) < 96.0);
    }
    #[test]
    fn an_overshooting_size_stays_inside_the_window_and_above_the_collapsed_floor() {
        assert_eq!(bounded(900.0, 100.0, 800.0), 800.0);
        assert_eq!(bounded(10.0, 100.0, 800.0), 75.0);
        assert_eq!(bounded(300.0, 100.0, 800.0), 300.0);
    }
    #[test]
    fn the_tallest_open_page_overshoots_without_leaving_the_window() {
        let notch = crate::host::notch::tests::rig(crate::pages::SampleOptions::default());
        let (room_width, room_height) = notch.size_room();
        assert!(room_width < notch.surface.width as f32 && room_height < notch.surface.height as f32);
        let tallest = (crate::page::MAX_CONTENT_HEIGHT + chrome::HEIGHT_EXTRA) * notch.scale;
        let overshot = sized(RESTING_PILL_HEIGHT * notch.scale, tallest, motion::EXPANSION_OVERSHOOT_LIMIT.1);
        assert!(bounded(overshot, RESTING_PILL_HEIGHT * notch.scale, room_height) <= room_height);
    }
    /// Peak overshoot (fraction of the move) and seconds until settled for a unit move at 240 steps per second.
    fn measure(mut spring: Spring) -> (f32, f32) {
        spring.target = 1.0;
        let (mut highest, mut settled_at) = (0.0_f32, None);
        for step in 0..960 {
            spring.step(1.0 / 240.0);
            highest = highest.max(spring.position);
            if settled_at.is_none() && spring.is_settled() {
                settled_at = Some(step as f32 / 240.0);
            }
        }
        (highest - 1.0, settled_at.expect("the spring settles"))
    }
    #[test]
    fn the_size_spring_overshoots_about_ten_percent_and_settles_so_the_frame_timer_can_stop() {
        let (overshoot, seconds_to_settle) = measure(size_spring(0.0));
        assert!((0.08..0.12).contains(&overshoot), "overshoot {overshoot}");
        assert!(seconds_to_settle < 1.6, "settled after {seconds_to_settle}");
    }
    #[test]
    fn the_size_spring_settles_one_and_a_third_to_one_and_a_half_times_slower_than_the_old_one() {
        let (_, old_seconds) = measure(Spring::new(0.0, 420.0, 21.0));
        let (_, new_seconds) = measure(size_spring(0.0));
        let ratio = new_seconds / old_seconds;
        assert!((1.3..1.5).contains(&ratio), "ratio {ratio} ({old_seconds} -> {new_seconds})");
    }
    #[test]
    fn a_size_move_starts_gently_instead_of_at_full_speed() {
        let (mut eased, mut plain) =
            (size_spring(0.0), Spring::new(0.0, motion::SIZE_RESPONSE.0, motion::SIZE_RESPONSE.1));
        (eased.target, plain.target) = (1.0, 1.0);
        for _ in 0..4 {
            eased.step(1.0 / 240.0);
            plain.step(1.0 / 240.0);
        }
        assert!(eased.position < plain.position * 0.1);
        for _ in 0..20 {
            eased.step(1.0 / 60.0);
        }
        assert!(eased.position > 0.5, "the move is under way after a third of a second");
    }
    #[test]
    fn width_and_height_springs_of_different_distances_reach_their_targets_on_the_same_frame() {
        let (mut width, mut height) = (size_spring(300.0), size_spring(100.0));
        (width.target, height.target) = (500.0, 180.0);
        let mut arrival_frames = (None, None);
        for frame in 0..600 {
            width.step(1.0 / 120.0);
            height.step(1.0 / 120.0);
            if arrival_frames.0.is_none() && (500.0 - width.position).abs() / 200.0 < 0.01 {
                arrival_frames.0 = Some(frame);
            }
            if arrival_frames.1.is_none() && (180.0 - height.position).abs() / 80.0 < 0.01 {
                arrival_frames.1 = Some(frame);
            }
        }
        assert_eq!(arrival_frames.0, arrival_frames.1);
        assert!(width.is_settled() && height.is_settled());
    }
    /// Frames until the frame timer stops, or `None` when it still runs after `limit_seconds`.
    fn frames_until_the_timer_stops(notch: &mut Notch, frame_seconds: f32, limit_seconds: f32) -> Option<usize> {
        notch.start_animation();
        for frame in 0..(limit_seconds / frame_seconds) as usize {
            notch.advance_by(frame_seconds);
            if !notch.frame_timer.is_running {
                return Some(frame + 1);
            }
        }
        None
    }
    #[test]
    fn the_frame_timer_stops_after_every_open_and_close_at_any_frame_rate() {
        for frame_seconds in [1.0 / 60.0, 1.0 / 144.0] {
            let mut notch = crate::host::notch::tests::rig(crate::pages::SampleOptions::default());
            crate::host::notch::tests::settle(&mut notch);
            for cycle in 0..3 {
                notch.begin_opening();
                notch.refresh_layout();
                let opened = frames_until_the_timer_stops(&mut notch, frame_seconds, 4.0);
                assert!(opened.is_some(), "open {cycle} at {frame_seconds}: {}", describe_springs(&mut notch));
                assert_eq!(notch.springs.expansion.position, 1.0);
                notch.begin_closing();
                notch.refresh_layout();
                let closed = frames_until_the_timer_stops(&mut notch, frame_seconds, 4.0);
                assert!(closed.is_some(), "close {cycle} at {frame_seconds}: {}", describe_springs(&mut notch));
                assert_eq!(notch.springs.expansion.position, 0.0);
            }
        }
    }
    #[test]
    fn the_frame_timer_stops_after_hover_cycles_that_reverse_a_move_before_it_settles() {
        let mut notch = crate::host::notch::tests::rig(crate::pages::SampleOptions::default());
        crate::host::notch::tests::settle(&mut notch);
        let mut jitter = 0x2545_f491_u32;
        let mut next_frame_seconds = || {
            jitter ^= jitter << 13;
            jitter ^= jitter >> 17;
            jitter ^= jitter << 5;
            (8 + jitter % 14) as f32 / 1000.0
        };
        notch.start_animation();
        for cycle in 0..10 {
            for (opening, hold_seconds) in [(true, 0.15 + 0.1 * cycle as f32), (false, 0.4 + 0.05 * cycle as f32)] {
                if opening {
                    notch.begin_opening();
                } else {
                    notch.begin_closing();
                }
                notch.refresh_layout();
                notch.start_animation();
                let mut held = 0.0;
                while held < hold_seconds && notch.frame_timer.is_running {
                    let frame_seconds = next_frame_seconds();
                    notch.advance_by(frame_seconds);
                    held += frame_seconds;
                }
            }
        }
        let mut waited = 0.0;
        while notch.frame_timer.is_running && waited < 4.0 {
            let frame_seconds = next_frame_seconds();
            notch.advance_by(frame_seconds);
            waited += frame_seconds;
        }
        assert!(!notch.frame_timer.is_running, "still running after {waited} s: {}", describe_springs(&mut notch));
    }
    fn describe_springs(notch: &mut Notch) -> String {
        let expansion = (notch.springs.expansion.position, notch.springs.expansion.target);
        let layout: Vec<(f32, f32, bool)> =
            notch.layout_springs().map(|spring| (spring.position, spring.target, spring.is_settled())).collect();
        format!("expansion {expansion:?}, layout {layout:?}")
    }
    #[test]
    fn the_header_shows_before_the_page_content_and_the_collapsed_content_is_gone_first() {
        const { assert!(RESTING_FADE_END <= HEADER_FADE.0 + 0.1 && HEADER_FADE.1 <= CONTENT_FADE.0) };
        assert_eq!(ramp(HEADER_STAGE, HEADER_FADE), 1.0);
        assert_eq!(ramp(HEADER_STAGE, CONTENT_FADE), 0.0);
        assert_eq!(ramp(1.0, CONTENT_FADE), 1.0);
    }
}
