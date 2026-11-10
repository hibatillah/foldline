/// How far a list is scrolled up, in the page's own units; a limit below zero counts as zero.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollOffset {
    value: f32,
}

impl ScrollOffset {
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Moves by `distance` within `0..=limit`; true when the value changed.
    pub fn scroll_by(&mut self, distance: f32, limit: f32) -> bool {
        let moved = (self.value + distance).clamp(0.0, limit.max(0.0));
        let changed = moved != self.value;
        self.value = moved;
        changed
    }

    pub fn clamp_to(&mut self, limit: f32) {
        self.value = self.value.clamp(0.0, limit.max(0.0));
    }

    pub fn reset(&mut self) {
        self.value = 0.0;
    }

    pub fn jump_to_end(&mut self, limit: f32) {
        self.value = limit.max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrolling_stops_at_both_ends() {
        let mut offset = ScrollOffset::default();
        offset.scroll_by(500.0, 90.0);
        assert_eq!(offset.value(), 90.0);
        offset.scroll_by(-500.0, 90.0);
        assert_eq!(offset.value(), 0.0);
    }

    #[test]
    fn a_scroll_that_changes_nothing_reports_no_change() {
        let mut offset = ScrollOffset::default();
        assert!(!offset.scroll_by(-10.0, 90.0), "already at the top");
        assert!(offset.scroll_by(40.0, 90.0));
        offset.jump_to_end(90.0);
        assert!(!offset.scroll_by(10.0, 90.0), "already at the end");
    }

    #[test]
    fn a_shorter_list_pulls_the_offset_back_inside() {
        let mut offset = ScrollOffset::default();
        offset.jump_to_end(120.0);
        offset.clamp_to(50.0);
        assert_eq!(offset.value(), 50.0);
        offset.reset();
        assert_eq!(offset.value(), 0.0);
    }

    #[test]
    fn a_negative_or_zero_limit_keeps_the_offset_at_zero() {
        let mut offset = ScrollOffset::default();
        assert!(!offset.scroll_by(30.0, 0.0));
        assert!(!offset.scroll_by(30.0, -5.0));
        offset.jump_to_end(-5.0);
        offset.clamp_to(-1.0);
        assert_eq!(offset.value(), 0.0);
    }
}
