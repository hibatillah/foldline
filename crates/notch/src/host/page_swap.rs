//! The open notch's page swap: the leaving page fades out with a small drift, then the arriving page fades and shifts
//! in from the side it comes from. A swap always runs to its end; nothing rests between two pages.

use super::motion::{
    PAGE_SWAP_IN_DELAY, PAGE_SWAP_IN_FADE_SHARE, PAGE_SWAP_IN_SECONDS, PAGE_SWAP_IN_SHIFT, PAGE_SWAP_OUT_SECONDS,
    PAGE_SWAP_OUT_SHIFT,
};

/// How one page shows during a swap: its opacity and its horizontal offset in DIP (positive is east).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PageLook {
    pub opacity: f32,
    pub offset: f32,
}

impl PageLook {
    pub const AT_REST: PageLook = PageLook { opacity: 1.0, offset: 0.0 };
}

/// Seconds from the start of a swap to its end, when the arriving page is in place.
pub(super) const PAGE_SWAP_SECONDS: f32 = PAGE_SWAP_IN_DELAY + PAGE_SWAP_IN_SECONDS;

#[derive(Clone, Copy, Debug)]
struct Leaving {
    page: usize,
    from: PageLook,
}

pub(super) struct PageSwap {
    leaving: Option<Leaving>,
    /// 1 when the arriving page comes from the east (the next page), -1 from the west.
    direction: f32,
    elapsed_seconds: f32,
    is_running: bool,
}

impl Default for PageSwap {
    fn default() -> Self {
        Self { leaving: None, direction: 1.0, elapsed_seconds: PAGE_SWAP_SECONDS, is_running: false }
    }
}

/// Fast start, gentle stop.
fn ease_out(progress: f32) -> f32 {
    let remaining = 1.0 - progress.clamp(0.0, 1.0);
    1.0 - remaining * remaining * remaining
}

impl PageSwap {
    /// Starts a swap away from `leaving`; a swap still running hands its arriving page's look to the leaving one.
    pub fn begin(&mut self, leaving: usize, direction: f32) {
        let from = if self.is_running { self.arriving() } else { PageLook::AT_REST };
        *self = Self {
            leaving: Some(Leaving { page: leaving, from }),
            direction: if direction < 0.0 { -1.0 } else { 1.0 },
            elapsed_seconds: 0.0,
            is_running: true,
        };
    }

    /// Moves the swap on by one frame; returns whether it still runs.
    pub fn advance(&mut self, elapsed_seconds: f32) -> bool {
        if self.is_running {
            self.elapsed_seconds += elapsed_seconds;
            if self.elapsed_seconds >= PAGE_SWAP_SECONDS {
                self.finish();
            }
        }
        self.is_running
    }

    pub fn finish(&mut self) {
        *self = Self::default();
    }

    #[cfg(test)]
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// The page that is leaving and how it shows; `None` once it has faded out.
    pub fn leaving(&self) -> Option<(usize, PageLook)> {
        let leaving = self.leaving.filter(|_| self.is_running)?;
        let eased = ease_out(self.elapsed_seconds / PAGE_SWAP_OUT_SECONDS);
        let opacity = leaving.from.opacity * (1.0 - eased);
        if opacity <= 0.0 {
            return None;
        }
        let offset = leaving.from.offset - self.direction * PAGE_SWAP_OUT_SHIFT * eased;
        Some((leaving.page, PageLook { opacity, offset }))
    }

    /// How the open page shows: still away and transparent at the start of a swap, at rest at its end.
    pub fn arriving(&self) -> PageLook {
        if !self.is_running {
            return PageLook::AT_REST;
        }
        let progress = ((self.elapsed_seconds - PAGE_SWAP_IN_DELAY) / PAGE_SWAP_IN_SECONDS).clamp(0.0, 1.0);
        let opacity = ease_out(progress / PAGE_SWAP_IN_FADE_SHARE);
        let offset = self.direction * PAGE_SWAP_IN_SHIFT * (1.0 - ease_out(progress));
        PageLook { opacity, offset }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::Spring;
    use crate::host::motion::{SIZE_LEAD_IN, SIZE_RESPONSE};

    impl PageSwap {
        fn jump_to(&mut self, elapsed_seconds: f32) {
            self.elapsed_seconds = elapsed_seconds.min(PAGE_SWAP_SECONDS);
        }
    }

    const FRAME: f32 = 1.0 / 240.0;

    fn running(direction: f32) -> PageSwap {
        let mut swap = PageSwap::default();
        swap.begin(3, direction);
        swap
    }

    /// Each frame of a whole swap with the leaving and the arriving page's look.
    fn frames(direction: f32) -> Vec<(Option<PageLook>, PageLook)> {
        let mut swap = running(direction);
        let mut looks = vec![(swap.leaving().map(|(_, look)| look), swap.arriving())];
        while swap.advance(FRAME) {
            looks.push((swap.leaving().map(|(_, look)| look), swap.arriving()));
        }
        looks
    }

    #[test]
    fn a_swap_starts_on_the_leaving_page_and_the_arriving_page_unseen() {
        let swap = running(1.0);
        assert_eq!(swap.leaving(), Some((3, PageLook::AT_REST)));
        assert_eq!(swap.arriving().opacity, 0.0);
    }

    #[test]
    fn a_swap_always_runs_to_its_end_and_leaves_the_open_page_at_rest() {
        let mut swap = running(1.0);
        let mut seconds = 0.0;
        while swap.advance(1.0 / 60.0) {
            seconds += 1.0 / 60.0;
            assert!(seconds < 1.0, "the swap never ended");
        }
        assert_eq!(swap.arriving(), PageLook::AT_REST);
        assert!(swap.leaving().is_none());
    }

    #[test]
    fn the_next_page_comes_in_from_the_east_and_the_leaving_one_drifts_west() {
        let mut swap = running(1.0);
        swap.jump_to(PAGE_SWAP_IN_DELAY + 0.02);
        assert!(swap.arriving().offset > 0.0);
        swap.jump_to(0.05);
        assert!(swap.leaving().expect("still fading").1.offset < 0.0);
    }

    #[test]
    fn the_previous_page_comes_in_from_the_west() {
        let mut swap = running(-1.0);
        swap.jump_to(PAGE_SWAP_IN_DELAY + 0.02);
        assert!(swap.arriving().offset < 0.0);
    }

    #[test]
    fn the_arriving_page_only_moves_toward_its_place_and_never_passes_it() {
        let offsets: Vec<f32> = frames(1.0).iter().map(|(_, arriving)| arriving.offset).collect();
        assert!(offsets.windows(2).all(|pair| pair[1] <= pair[0]));
        assert!(offsets.iter().all(|&offset| (0.0..=PAGE_SWAP_IN_SHIFT).contains(&offset)));
    }

    #[test]
    fn both_pages_are_barely_visible_at_the_same_time() {
        for (leaving, arriving) in frames(1.0) {
            let leaving_opacity = leaving.map_or(0.0, |look| look.opacity);
            assert!(leaving_opacity.min(arriving.opacity) < 0.05, "{leaving_opacity} and {}", arriving.opacity);
        }
    }

    #[test]
    fn a_swap_is_quick() {
        const { assert!(PAGE_SWAP_SECONDS <= 0.45) };
        let mut swap = running(1.0);
        swap.jump_to(0.25);
        assert!(swap.arriving().opacity > 0.95, "the page reads after a quarter of a second");
    }

    #[test]
    fn a_swap_started_during_another_fades_the_half_arrived_page_out_from_where_it_was() {
        let mut swap = running(1.0);
        swap.jump_to(PAGE_SWAP_IN_DELAY + 0.05);
        let half_arrived = swap.arriving();
        swap.begin(5, 1.0);
        assert_eq!(swap.leaving(), Some((5, half_arrived)));
        assert_eq!(swap.arriving().opacity, 0.0);
    }

    /// Seconds until a size spring stays within `band` of a unit move for good.
    fn size_spring_arrival(band: f32) -> f32 {
        let mut spring = Spring::new(0.0, SIZE_RESPONSE.0, SIZE_RESPONSE.1).with_lead_in(SIZE_LEAD_IN);
        spring.target = 1.0;
        let mut last_outside = 0.0;
        for frame in 1..=960 {
            spring.step(FRAME);
            if (spring.position - 1.0).abs() > band {
                last_outside = frame as f32 * FRAME;
            }
        }
        last_outside
    }

    #[test]
    fn the_content_settles_just_after_the_pill_reaches_its_new_size() {
        let arrival = size_spring_arrival(0.03);
        let gap = PAGE_SWAP_SECONDS - arrival;
        assert!((0.0..=0.06).contains(&gap), "content ends {gap} s after the size ({arrival} s)");
    }

    #[test]
    fn the_arriving_page_starts_only_after_the_size_has_begun_to_change() {
        let mut spring = Spring::new(0.0, SIZE_RESPONSE.0, SIZE_RESPONSE.1).with_lead_in(SIZE_LEAD_IN);
        spring.target = 1.0;
        for _ in 0..(PAGE_SWAP_IN_DELAY / FRAME) as usize {
            spring.step(FRAME);
        }
        assert!(spring.position > 0.0 && spring.position < 0.5, "size at {}", spring.position);
    }
}
