use super::{MediaPage, layout::Layout};
use crate::ui::palette::TEXT_DIM;
use crate::{
    gfx::{Canvas, Color, Point},
    ui::{Icon, IconButton},
};

const ACCENT: Color = Color::rgb(0.118, 0.843, 0.376);
const INACTIVE_HOVERED: Color = Color::gray(0.85);
/// Icon edge in DIP; the lyrics icon is drawn 2 px larger than the transport icons.
const ICON_SIZE: f32 = 18.0;
const LYRICS_ICON_SIZE: f32 = 20.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Control {
    Previous,
    PlayPause,
    Next,
    Lyrics,
}

impl Control {
    pub const ALL: [Control; 4] = [Control::Previous, Control::PlayPause, Control::Next, Control::Lyrics];

    pub fn slot(self) -> f32 {
        match self {
            Control::Previous => 0.0,
            Control::PlayPause => 1.0,
            Control::Next => 2.0,
            Control::Lyrics => 3.0,
        }
    }
}

fn lyrics_color(is_on: bool, has_lyrics: bool, is_hovered: bool) -> Color {
    match (is_on, has_lyrics) {
        (true, _) => ACCENT,
        (false, true) => Color::WHITE,
        (false, false) if is_hovered => INACTIVE_HOVERED,
        (false, false) => TEXT_DIM,
    }
}

pub fn hit_test(layout: &Layout, point: Point) -> Option<Control> {
    Control::ALL.into_iter().find(|&control| IconButton::is_hit(layout.control_center(control), 1.0, point))
}

pub fn draw(page: &MediaPage, canvas: &Canvas, layout: &Layout, opacity: f32) {
    for control in Control::ALL {
        let (icon, is_on) = match control {
            Control::Previous => (Icon::Previous, None),
            Control::PlayPause => (if page.info.is_playing { Icon::Pause } else { Icon::Play }, None),
            Control::Next => (Icon::Next, None),
            Control::Lyrics => (Icon::Lyrics, Some(page.is_lyric_panel_visible())),
        };
        let is_hovered = page.hovered_control.is(control);
        let color = match is_on {
            _ if !page.info.available => TEXT_DIM,
            Some(is_on) => lyrics_color(is_on, page.has_lyrics(), is_hovered),
            None => Color::WHITE,
        };
        let size = if control == Control::Lyrics { LYRICS_ICON_SIZE } else { ICON_SIZE };
        IconButton { icon, center: layout.control_center(control), color, is_hovered }.draw(canvas, opacity, size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{gfx::Rect, pages::media::layout};

    #[test]
    fn each_control_center_in_units_hits_its_own_control_and_the_artwork_hits_none() {
        let width = layout::plain_width(layout::TEXT_MAX_WIDTH);
        let bounds = Rect::new(0.0, 0.0, width, layout::PLAIN_HEIGHT);
        let layout = Layout::new(bounds, width, 1, layout::TEXT_MAX_WIDTH);
        for control in Control::ALL {
            assert_eq!(hit_test(&layout, layout.control_center(control)), Some(control));
        }
        assert_eq!(hit_test(&layout, Point::new(10.0, 10.0)), None);
    }

    #[test]
    fn the_lyrics_button_is_green_when_on() {
        assert_eq!(lyrics_color(true, true, false), ACCENT);
    }

    #[test]
    fn the_lyrics_button_is_white_when_off_but_lyrics_exist() {
        assert_eq!(lyrics_color(false, true, false), Color::WHITE);
        assert_eq!(lyrics_color(false, true, true), Color::WHITE);
    }

    #[test]
    fn the_lyrics_button_is_gray_when_the_song_has_no_lyrics() {
        assert_eq!(lyrics_color(false, false, false), TEXT_DIM);
        assert_eq!(lyrics_color(false, false, true), INACTIVE_HOVERED);
    }
}
