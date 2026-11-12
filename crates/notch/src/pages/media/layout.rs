use super::controls::Control;
use crate::gfx::{Point, Rect, lerp};

// Layout is in units; the host adds padding and the header.
pub const PLAIN_HEIGHT: f32 = crate::page::DEFAULT_CONTENT_HEIGHT;
pub const TEXT_MAX_WIDTH: f32 = 170.0;
/// The narrowest text column, so a short title keeps the page 240 wide: the width the earlier, larger control row set
/// (9 + 3 x 38 + 1 + 10), kept when the controls shrank so the open page does not change size.
pub const TEXT_MIN_WIDTH: f32 = 134.0;
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
const CONTROLS_GAP: f32 = 12.0;
/// The lyrics icon is the tallest control, so it sets the height of the control row.
const CONTROLS_HEIGHT: f32 = 18.0;
/// Center of the first control from the text column edge: half of its 16 unit icon, so the icon lines up with the text.
const CONTROL_FIRST_CENTER: f32 = 8.0;
/// Distance between control centers: a 16 unit icon and a 14 unit gap.
const CONTROL_PITCH: f32 = 30.0;
/// The 18 unit lyrics icon is 2 wider than the others, so its center sits 1 further along.
const LYRICS_NUDGE: f32 = 1.0;
/// Width of the control row from the text column edge to the right edge of the lyrics icon.
const CONTROLS_WIDTH: f32 = CONTROL_FIRST_CENTER + 3.0 * CONTROL_PITCH + LYRICS_NUDGE + 9.0;
const _: () = assert!(CONTROLS_WIDTH <= TEXT_MIN_WIDTH, "the control row fits the narrowest text column");
/// Half the height of a control's click area; its width is the pitch, so neighboring areas meet without overlapping.
const CONTROL_HIT_HALF_HEIGHT: f32 = 15.0;
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
    pub bounds: Rect,
    blend: f32,
    title_lines: usize,
}

impl Layout {
    /// `expanded_width` is the width the pill is opening towards; it blends the text column into the lyric layout.
    pub fn new(bounds: Rect, expanded_width: f32, title_lines: usize, text_width: f32) -> Self {
        let plain = plain_width(text_width);
        let blend = ((expanded_width - plain) / (LYRICS_WIDTH - plain)).clamp(0.0, 1.0);
        Self { bounds, blend, title_lines: title_lines.clamp(1, TITLE_MAX_LINES) }
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
        Rect::new(self.bounds.left, self.bounds.top, self.bounds.left + ARTWORK_SIZE, self.bounds.top + ARTWORK_SIZE)
    }

    pub fn artwork_radius(&self) -> f32 {
        ARTWORK_RADIUS
    }

    pub fn app_badge(&self) -> Rect {
        let artwork = self.artwork();
        let right = artwork.right + BADGE_OVERHANG;
        let bottom = artwork.bottom + BADGE_OVERHANG;
        Rect::new(right - BADGE_SIZE, bottom - BADGE_SIZE, right, bottom)
    }

    pub fn app_badge_radius(&self) -> f32 {
        BADGE_RADIUS
    }

    pub fn app_badge_ring(&self) -> (Rect, f32) {
        let badge = self.app_badge();
        let ring = BADGE_RING;
        let rect = Rect::new(badge.left - ring, badge.top - ring, badge.right + ring, badge.bottom + ring);
        (rect, self.app_badge_radius() + ring)
    }

    fn text_left(&self) -> f32 {
        self.bounds.left + ARTWORK_SIZE + TEXT_GAP
    }

    fn text_right(&self) -> f32 {
        let with_lyrics = self.text_left() + COLUMN_WIDTH_WITH_LYRICS;
        lerp(self.bounds.right, with_lyrics, self.blend).min(self.bounds.right)
    }

    fn row(&self, top: f32, height: f32) -> Rect {
        let top = self.bounds.top + self.stack_top() + top;
        Rect::new(self.text_left(), top, self.text_right(), top + height)
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
        let x = self.text_left() + CONTROL_FIRST_CENTER + control.slot() * CONTROL_PITCH + nudge;
        Point::new(x, self.bounds.top + top)
    }

    /// Where a press counts for `control`: one pitch wide around its center, so the areas tile the row.
    pub fn control_hit_area(&self, control: Control) -> Rect {
        Rect::around(self.control_center(control), CONTROL_PITCH / 2.0, CONTROL_HIT_HALF_HEIGHT)
    }

    /// The app icon inside the badge's dark tile.
    pub fn app_badge_icon(&self) -> Rect {
        let badge = self.app_badge();
        let inset = BADGE_ICON_INSET;
        Rect::new(badge.left + inset, badge.top + inset, badge.right - inset, badge.bottom - inset)
    }

    pub fn app_badge_icon_radius(&self) -> f32 {
        BADGE_RADIUS - 1.0
    }

    pub fn lyric_panel(&self) -> Rect {
        let left = self.text_left() + COLUMN_WIDTH_WITH_LYRICS + LYRIC_GAP;
        Rect::new(left, self.bounds.top, self.bounds.right, self.bounds.top + PLAIN_HEIGHT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT_WIDTH: f32 = 170.0;
    const PLAIN_WIDTH: f32 = ARTWORK_SIZE + TEXT_GAP + TEXT_WIDTH;

    fn bounds(width: f32) -> Rect {
        Rect::new(100.0, 0.0, 100.0 + width, PLAIN_HEIGHT)
    }

    fn plain(title_lines: usize) -> Layout {
        Layout::new(bounds(PLAIN_WIDTH), PLAIN_WIDTH, title_lines, TEXT_WIDTH)
    }

    #[test]
    fn the_artwork_is_a_square_as_high_as_the_page_at_its_left_edge() {
        let artwork = plain(2).artwork();
        assert_eq!(artwork.left, 100.0);
        assert_eq!(artwork.width(), artwork.height());
        assert_eq!(artwork.height(), PLAIN_HEIGHT);
    }

    #[test]
    fn the_app_icon_hangs_slightly_past_the_artwork_bottom_right_corner() {
        let layout = plain(2);
        let (artwork, badge) = (layout.artwork(), layout.app_badge());
        let overhang = badge.right - artwork.right;
        assert_eq!(overhang, BADGE_OVERHANG);
        assert_eq!(badge.bottom - artwork.bottom, BADGE_OVERHANG);
        assert!(overhang < badge.width() / 2.0, "most of the icon stays over the artwork");
        assert!(badge.left > artwork.left && badge.top > artwork.top);
        let (ring, _) = layout.app_badge_ring();
        assert!(ring.left < badge.left && ring.bottom > badge.bottom);
    }

    #[test]
    fn the_app_icon_sits_inside_the_dark_tile_of_the_badge() {
        let layout = plain(2);
        let (badge, icon) = (layout.app_badge(), layout.app_badge_icon());
        assert_eq!(icon.width(), BADGE_SIZE - 2.0 * BADGE_ICON_INSET);
        assert!(
            icon.left > badge.left && icon.right < badge.right && icon.top > badge.top && icon.bottom < badge.bottom
        );
        assert!(layout.app_badge_icon_radius() < layout.app_badge_radius());
    }

    #[test]
    fn the_controls_are_16_unit_icons_30_apart_with_a_slightly_wider_last_one() {
        let layout = plain(2);
        let x = |control| layout.control_center(control).x - layout.title_line(0).left;
        assert_eq!(x(Control::Previous), 8.0, "the first icon's left edge lines up with the text");
        assert_eq!(x(Control::PlayPause) - x(Control::Previous), 30.0);
        assert_eq!(x(Control::Next) - x(Control::PlayPause), 30.0);
        assert_eq!(x(Control::Lyrics) - x(Control::Next), 31.0);
    }

    #[test]
    fn the_click_areas_of_neighboring_controls_meet_without_overlapping() {
        let layout = plain(1);
        let areas: Vec<Rect> = Control::ALL.iter().map(|&control| layout.control_hit_area(control)).collect();
        for pair in areas.windows(2) {
            assert!(pair[0].right <= pair[1].left + 0.01, "{:?} overlaps {:?}", pair[0], pair[1]);
            assert!(pair[1].left - pair[0].right < 1.01, "no dead strip between controls");
        }
        assert!(areas[0].left >= layout.bounds.left + ARTWORK_SIZE, "the first area stays clear of the artwork");
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
        assert!(layout.title_line(0).top - layout.bounds.top <= 2.0);
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
        let buttons_bottom = layout.control_center(Control::PlayPause).y + CONTROLS_HEIGHT / 2.0;
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
        let with = Layout::new(bounds(PLAIN_WIDTH), LYRICS_WIDTH, 2, TEXT_WIDTH);
        assert_eq!(without.title_line(0).width(), TEXT_WIDTH);
        assert!(with.title_line(0).right < with.lyric_panel().left);
        assert_eq!(with.title_line(0).width(), COLUMN_WIDTH_WITH_LYRICS);
    }

    #[test]
    fn a_short_title_keeps_the_open_page_240_wide_with_the_controls_inside_it() {
        let width = plain_width(TEXT_MIN_WIDTH);
        assert_eq!(width, 240.0);
        let layout = Layout::new(bounds(width), width, 1, TEXT_MIN_WIDTH);
        let lyrics = layout.control_center(Control::Lyrics);
        assert!(lyrics.x + 9.0 < layout.bounds.right, "the lyrics icon ends inside the page");
        assert_eq!(layout.title_line(0).right, layout.bounds.right);
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
        let layout = Layout::new(bounds(LYRICS_WIDTH), LYRICS_WIDTH, 2, TEXT_WIDTH);
        assert_eq!(layout.lyric_panel().left - layout.title_line(0).right, LYRIC_GAP);
        assert_eq!(layout.lyric_panel().right, layout.bounds.right);
        assert_eq!(layout.lyric_panel().width(), lyric_panel_width());
    }

    #[test]
    fn the_layout_changes_gradually_while_the_pill_widens() {
        let halfway = (PLAIN_WIDTH + LYRICS_WIDTH) / 2.0;
        let (narrow, middle, wide) = (
            Layout::new(bounds(PLAIN_WIDTH), PLAIN_WIDTH, 2, TEXT_WIDTH).lyric_blend(),
            Layout::new(bounds(PLAIN_WIDTH), halfway, 2, TEXT_WIDTH).lyric_blend(),
            Layout::new(bounds(PLAIN_WIDTH), LYRICS_WIDTH, 2, TEXT_WIDTH).lyric_blend(),
        );
        assert_eq!((narrow, wide), (0.0, 1.0));
        assert!(middle > 0.0 && middle < 1.0);
    }
}
