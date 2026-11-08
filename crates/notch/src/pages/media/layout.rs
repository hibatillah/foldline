use super::controls::Control;
use crate::{
    gfx::{Point, Rect, lerp},
    page::Frame,
};

// Layout converts content DIP to device pixels; the host adds padding and the header.
pub const PLAIN_HEIGHT: f32 = crate::page::DEFAULT_CONTENT_HEIGHT;
pub const TEXT_MAX_WIDTH: f32 = 170.0;
/// A short title never narrows the column below the control row, so the content block has no empty strip on its right.
pub const TEXT_MIN_WIDTH: f32 = CONTROLS_WIDTH;
const ARTWORK_SIZE: f32 = 86.0;
const ARTWORK_RADIUS: f32 = 12.0;
const TEXT_GAP: f32 = 20.0;
pub fn plain_width(text_width: f32) -> f32 {
    ARTWORK_SIZE + TEXT_GAP + text_width
}
pub const LYRICS_WIDTH: f32 = 582.0;
const COLUMN_WIDTH_WITH_LYRICS: f32 = 170.0;
pub const TITLE_MAX_LINES: usize = 2;
const TITLE_LINE_HEIGHT: f32 = 18.0;
const DETAIL_LINE_HEIGHT: f32 = 15.0;
const DETAIL_GAP: f32 = 2.0;
const CONTROLS_GAP: f32 = 14.0;
/// The lyrics icon is the tallest control, so it sets the height of the control row.
const CONTROLS_HEIGHT: f32 = 16.0;
/// Center of the first control from the text column edge: half of its 14 px icon.
const CONTROL_FIRST_CENTER: f32 = 7.0;
/// Distance between control centers: a 14 px icon and a 20 px gap.
const CONTROL_PITCH: f32 = 34.0;
/// The 16 px lyrics icon is 2 px wider than the others, so its center sits 1 px further along.
const LYRICS_NUDGE: f32 = 1.0;
/// Width of the control row from the text column edge to the right edge of the lyrics icon.
const CONTROLS_WIDTH: f32 = CONTROL_FIRST_CENTER + 3.0 * CONTROL_PITCH + LYRICS_NUDGE + 8.0;
const LYRIC_GAP: f32 = 20.0;
pub const LYRIC_MAX_LINES: usize = 2;
const BADGE_SIZE: f32 = 19.0;
const BADGE_RADIUS: f32 = 5.0;
/// How far the badge's right and bottom edges hang past the artwork's: 1 px, and the black rim reaches 3 px further.
const BADGE_OVERHANG: f32 = 1.0;
/// The app icon sits inside the dark tile of the badge, 2 px in from each edge.
const BADGE_ICON_INSET: f32 = 2.0;
const BADGE_RING: f32 = 3.0;

pub fn lyric_panel_width() -> f32 {
    LYRICS_WIDTH - (ARTWORK_SIZE + TEXT_GAP + COLUMN_WIDTH_WITH_LYRICS + LYRIC_GAP)
}

pub fn title_width(has_lyrics: bool) -> f32 {
    if has_lyrics { COLUMN_WIDTH_WITH_LYRICS } else { TEXT_MAX_WIDTH }
}

pub struct Layout {
    pub scale: f32,
    pub bounds: Rect,
    blend: f32,
    title_lines: usize,
}

impl Layout {
    pub fn new(frame: &Frame, title_lines: usize, text_width: f32) -> Self {
        let plain = plain_width(text_width);
        let blend = ((frame.expanded.width - plain) / (LYRICS_WIDTH - plain)).clamp(0.0, 1.0);
        Self { scale: frame.scale, bounds: frame.bounds, blend, title_lines: title_lines.clamp(1, TITLE_MAX_LINES) }
    }

    pub fn lyric_blend(&self) -> f32 {
        self.blend
    }

    fn stack_top(&self) -> f32 {
        let height = self.title_lines as f32 * TITLE_LINE_HEIGHT
            + DETAIL_GAP
            + DETAIL_LINE_HEIGHT
            + CONTROLS_GAP
            + CONTROLS_HEIGHT;
        (PLAIN_HEIGHT - height) / 2.0
    }

    pub fn artwork(&self) -> Rect {
        Rect::new(
            self.bounds.left,
            self.bounds.top,
            self.bounds.left + ARTWORK_SIZE * self.scale,
            self.bounds.top + ARTWORK_SIZE * self.scale,
        )
    }

    pub fn artwork_radius(&self) -> f32 {
        ARTWORK_RADIUS * self.scale
    }

    pub fn app_badge(&self) -> Rect {
        let artwork = self.artwork();
        let right = artwork.right + BADGE_OVERHANG * self.scale;
        let bottom = artwork.bottom + BADGE_OVERHANG * self.scale;
        Rect::new(right - BADGE_SIZE * self.scale, bottom - BADGE_SIZE * self.scale, right, bottom)
    }

    pub fn app_badge_radius(&self) -> f32 {
        BADGE_RADIUS * self.scale
    }

    pub fn app_badge_ring(&self) -> (Rect, f32) {
        let badge = self.app_badge();
        let ring = BADGE_RING * self.scale;
        let rect = Rect::new(badge.left - ring, badge.top - ring, badge.right + ring, badge.bottom + ring);
        (rect, self.app_badge_radius() + ring)
    }

    fn text_left(&self) -> f32 {
        self.bounds.left + (ARTWORK_SIZE + TEXT_GAP) * self.scale
    }

    fn text_right(&self) -> f32 {
        let with_lyrics = self.text_left() + COLUMN_WIDTH_WITH_LYRICS * self.scale;
        lerp(self.bounds.right, with_lyrics, self.blend).min(self.bounds.right)
    }

    fn row(&self, top: f32, height: f32) -> Rect {
        let top = self.bounds.top + (self.stack_top() + top) * self.scale;
        Rect::new(self.text_left(), top, self.text_right(), top + height * self.scale)
    }

    pub fn title_line(&self, index: usize) -> Rect {
        self.row(index as f32 * TITLE_LINE_HEIGHT, TITLE_LINE_HEIGHT)
    }

    pub fn detail(&self) -> Rect {
        self.row(self.title_lines as f32 * TITLE_LINE_HEIGHT + DETAIL_GAP, DETAIL_LINE_HEIGHT)
    }

    pub fn control_center(&self, control: Control) -> Point {
        let top = self.stack_top()
            + self.title_lines as f32 * TITLE_LINE_HEIGHT
            + DETAIL_GAP
            + DETAIL_LINE_HEIGHT
            + CONTROLS_GAP
            + CONTROLS_HEIGHT / 2.0;
        let nudge = if control == Control::Lyrics { LYRICS_NUDGE } else { 0.0 };
        let x = self.text_left() + (CONTROL_FIRST_CENTER + control.slot() * CONTROL_PITCH + nudge) * self.scale;
        Point::new(x, self.bounds.top + top * self.scale)
    }

    /// The app icon inside the badge's dark tile.
    pub fn app_badge_icon(&self) -> Rect {
        let badge = self.app_badge();
        let inset = BADGE_ICON_INSET * self.scale;
        Rect::new(badge.left + inset, badge.top + inset, badge.right - inset, badge.bottom - inset)
    }

    pub fn app_badge_icon_radius(&self) -> f32 {
        (BADGE_RADIUS - 1.0) * self.scale
    }

    pub fn lyric_panel(&self) -> Rect {
        let left = self.text_left() + (COLUMN_WIDTH_WITH_LYRICS + LYRIC_GAP) * self.scale;
        Rect::new(left, self.bounds.top, self.bounds.right, self.bounds.top + PLAIN_HEIGHT * self.scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::Size;

    const TEXT_WIDTH: f32 = 170.0;
    const PLAIN_WIDTH: f32 = ARTWORK_SIZE + TEXT_GAP + TEXT_WIDTH;

    fn frame(expanded_width: f32) -> Frame {
        Frame {
            scale: 2.0,
            bounds: Rect::new(100.0, 0.0, 100.0 + PLAIN_WIDTH * 2.0, PLAIN_HEIGHT * 2.0),
            expansion: 1.0,
            expanded: Size::new(expanded_width, PLAIN_HEIGHT),
            lead_gap: 0.0,
        }
    }

    fn plain(title_lines: usize) -> Layout {
        Layout::new(&frame(PLAIN_WIDTH), title_lines, TEXT_WIDTH)
    }

    #[test]
    fn the_artwork_is_a_square_as_high_as_the_page_at_its_left_edge() {
        let artwork = plain(2).artwork();
        assert_eq!(artwork.left, 100.0);
        assert_eq!(artwork.width(), artwork.height());
        assert_eq!(artwork.height(), PLAIN_HEIGHT * 2.0);
    }

    #[test]
    fn the_app_icon_hangs_slightly_past_the_artwork_bottom_right_corner() {
        let layout = plain(2);
        let (artwork, badge) = (layout.artwork(), layout.app_badge());
        let overhang = BADGE_OVERHANG * 2.0;
        assert_eq!(badge.right - artwork.right, overhang);
        assert_eq!(badge.bottom - artwork.bottom, overhang);
        assert!(overhang < badge.width() / 2.0, "most of the icon stays over the artwork");
        assert!(badge.left > artwork.left && badge.top > artwork.top);
        let (ring, _) = layout.app_badge_ring();
        assert!(ring.left < badge.left && ring.bottom > badge.bottom);
    }

    #[test]
    fn the_app_icon_sits_inside_the_dark_tile_of_the_badge() {
        let layout = plain(2);
        let (badge, icon) = (layout.app_badge(), layout.app_badge_icon());
        assert_eq!(icon.width(), (BADGE_SIZE - 2.0 * BADGE_ICON_INSET) * 2.0);
        assert!(
            icon.left > badge.left && icon.right < badge.right && icon.top > badge.top && icon.bottom < badge.bottom
        );
        assert!(layout.app_badge_icon_radius() < layout.app_badge_radius());
    }

    #[test]
    fn the_controls_are_14_px_icons_with_a_20_px_gap_and_a_slightly_wider_last_one() {
        let layout = plain(2);
        let x = |control| (layout.control_center(control).x - layout.title_line(0).left) / layout.scale;
        assert_eq!(x(Control::Previous), 7.0);
        assert_eq!(x(Control::PlayPause) - x(Control::Previous), 34.0);
        assert_eq!(x(Control::Next) - x(Control::PlayPause), 34.0);
        assert_eq!(x(Control::Lyrics) - x(Control::Next), 35.0);
    }

    #[test]
    fn the_badge_and_its_rim_stay_clear_of_the_text() {
        let layout = plain(2);
        let (ring, _) = layout.app_badge_ring();
        assert!(ring.right < layout.title_line(0).left);
    }

    #[test]
    fn two_title_lines_nearly_fill_the_page_and_stack_above_the_detail_and_the_buttons() {
        let layout = plain(2);
        assert!(layout.title_line(0).top - layout.bounds.top <= 2.0 * layout.scale);
        assert!(layout.title_line(0).bottom <= layout.title_line(1).top);
        assert!(layout.title_line(1).bottom <= layout.detail().top);
        let button = layout.control_center(Control::PlayPause);
        assert!(button.y > layout.detail().bottom);
        assert!(button.y < layout.bounds.bottom);
    }

    #[test]
    fn a_one_line_title_leaves_equal_space_above_and_below_the_text() {
        let layout = plain(1);
        let above = layout.title_line(0).top - layout.bounds.top;
        let buttons_bottom = layout.control_center(Control::PlayPause).y + CONTROLS_HEIGHT * layout.scale / 2.0;
        let below = layout.bounds.bottom - buttons_bottom;
        assert!((above - below).abs() < 0.01);
        assert!(above > 0.0);
    }

    #[test]
    fn a_title_of_any_line_count_stays_inside_the_page() {
        for lines in [0, 1, 2, 5] {
            let layout = plain(lines);
            assert!(layout.title_line(0).top >= layout.bounds.top);
            assert!(layout.control_center(Control::Lyrics).y < layout.bounds.bottom);
        }
    }

    #[test]
    fn the_text_column_is_at_most_the_text_width_without_lyrics_and_narrows_with_them() {
        let without = plain(2);
        let with = Layout::new(&frame(LYRICS_WIDTH), 2, TEXT_WIDTH);
        assert_eq!(without.title_line(0).width(), TEXT_WIDTH * 2.0);
        assert!(with.title_line(0).right < with.lyric_panel().left);
        assert_eq!(with.title_line(0).width(), COLUMN_WIDTH_WITH_LYRICS * 2.0);
    }

    #[test]
    fn a_short_title_leaves_equal_space_left_of_the_artwork_and_right_of_the_controls() {
        let width = plain_width(TEXT_MIN_WIDTH);
        let narrow = Frame { bounds: Rect::new(100.0, 0.0, 100.0 + width * 2.0, PLAIN_HEIGHT * 2.0), ..frame(width) };
        let layout = Layout::new(&narrow, 1, TEXT_MIN_WIDTH);
        let controls_right = layout.control_center(Control::Lyrics).x + 8.0 * layout.scale;
        assert_eq!(controls_right, layout.bounds.right);
    }

    #[test]
    fn the_page_is_exactly_as_wide_as_the_artwork_the_gap_and_the_text() {
        let layout = plain(2);
        assert_eq!(layout.title_line(0).right, layout.bounds.right);
    }

    #[test]
    fn the_controls_follow_each_other_left_to_right_under_the_text() {
        let layout = plain(2);
        let centers: Vec<Point> = Control::ALL.iter().map(|&control| layout.control_center(control)).collect();
        assert!(centers.windows(2).all(|pair| pair[0].x < pair[1].x));
        assert!(centers.iter().all(|center| center.y > layout.detail().bottom));
    }

    #[test]
    fn the_lyric_panel_starts_a_gap_after_the_text_and_ends_at_the_page_edge() {
        let wide = Frame {
            bounds: Rect::new(100.0, 0.0, 100.0 + LYRICS_WIDTH * 2.0, PLAIN_HEIGHT * 2.0),
            ..frame(LYRICS_WIDTH)
        };
        let layout = Layout::new(&wide, 2, TEXT_WIDTH);
        assert_eq!(layout.lyric_panel().left - layout.title_line(0).right, LYRIC_GAP * 2.0);
        assert_eq!(layout.lyric_panel().right, layout.bounds.right);
        assert_eq!(layout.lyric_panel().width() / 2.0, lyric_panel_width());
    }

    #[test]
    fn the_layout_changes_gradually_while_the_pill_widens() {
        let halfway = (PLAIN_WIDTH + LYRICS_WIDTH) / 2.0;
        let (narrow, middle, wide) = (
            Layout::new(&frame(PLAIN_WIDTH), 2, TEXT_WIDTH).lyric_blend(),
            Layout::new(&frame(halfway), 2, TEXT_WIDTH).lyric_blend(),
            Layout::new(&frame(LYRICS_WIDTH), 2, TEXT_WIDTH).lyric_blend(),
        );
        assert_eq!((narrow, wide), (0.0, 1.0));
        assert!(middle > 0.0 && middle < 1.0);
    }
}
