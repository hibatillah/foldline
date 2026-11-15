//! The window's switch, 42 by 25: the component library's switch is fixed at 36 by 20.

use std::{rc::Rc, time::Duration};

use gpui::{Animation, AnimationExt as _, App, ElementId, IntoElement, RenderOnce, Window, div, prelude::*, px};
use gpui_component::ActiveTheme as _;

const WIDTH: f32 = 42.0;
const HEIGHT: f32 = 25.0;
const THUMB_SIZE: f32 = 21.0;
/// The space between the thumb and the track's edge.
const INSET: f32 = 2.0;
const SLIDE: Duration = Duration::from_millis(150);

type ClickHandler = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// The thumb's distance from the track's left edge.
fn thumb_left(checked: bool) -> f32 {
    if checked { WIDTH - THUMB_SIZE - INSET } else { INSET }
}

#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    checked: bool,
    on_click: Option<ClickHandler>,
}

impl Switch {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), checked: false, on_click: None }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Called with the new state when the switch is clicked.
    pub fn on_click(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let checked = self.checked;
        // The state last drawn, so the thumb slides only when the state changes.
        let drawn_state = window.use_keyed_state(self.id.clone(), cx, |_, _| checked);
        let was_checked = *drawn_state.read(cx);
        let theme = cx.theme();
        let track = if checked { theme.primary } else { theme.switch };
        let thumb =
            div().absolute().top(px(INSET)).size(px(THUMB_SIZE)).rounded_full().bg(theme.switch_thumb).shadow_sm();
        let thumb = if was_checked == checked {
            thumb.left(px(thumb_left(checked))).into_any_element()
        } else {
            let drawn_state = drawn_state.clone();
            cx.spawn(async move |cx| {
                cx.background_executor().timer(SLIDE).await;
                let _ = drawn_state.update(cx, |drawn, _| *drawn = checked);
            })
            .detach();
            let (from, to) = (thumb_left(was_checked), thumb_left(checked));
            thumb
                .with_animation(
                    ElementId::NamedInteger("slide".into(), u64::from(checked)),
                    Animation::new(SLIDE),
                    move |thumb, delta| thumb.left(px(from + (to - from) * delta)),
                )
                .into_any_element()
        };
        div()
            .id(self.id)
            .relative()
            .flex_none()
            .w(px(WIDTH))
            .h(px(HEIGHT))
            .rounded_full()
            .bg(track)
            .cursor_pointer()
            .child(thumb)
            .when_some(self.on_click, |switch, on_click| {
                switch.on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                    cx.stop_propagation();
                    on_click(&!checked, window, cx);
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_off_thumb_sits_at_the_left_inset() {
        assert_eq!(thumb_left(false), 2.0);
    }

    #[test]
    fn an_on_thumb_sits_at_the_right_inset() {
        assert_eq!(thumb_left(true) + THUMB_SIZE + INSET, WIDTH);
        assert_eq!(thumb_left(true), 19.0);
    }
}
