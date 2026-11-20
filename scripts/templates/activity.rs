//! The {{Title}} activity. TODO: say in one line what it shows in the collapsed pill and what it listens to.

use crate::{
    activity::{Activity, Presence},
    gfx::{Canvas, Rect},
};

pub struct {{Name}}Activity;

impl {{Name}}Activity {
    pub fn new() -> Self {
        Self
    }
}

impl Activity for {{Name}}Activity {
    fn id(&self) -> &'static str {
        "{{id}}"
    }

    /// Nothing to show yet; answer a `Presence` while something happens (see `docs/reference/activities.md`).
    fn presence(&self) -> Option<Presence> {
        None
    }

    fn draw(&self, _canvas: &Canvas, _area: Rect, _opacity: f32) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_activity_shows_nothing_until_something_happens() {
        let activity = {{Name}}Activity::new();
        assert_eq!(activity.id(), "{{id}}");
        assert!(activity.presence().is_none());
    }
}
