use std::rc::Rc;

use gpui::{AnyElement, App, SharedString, Window, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, Icon, h_flex, v_flex};

/// From one section's last block to the next section's heading.
const SECTION_GAP: f32 = 14.0;
/// Between a section's heading and its parts, and between two parts.
const SECTION_TITLE_GAP: f32 = 7.0;
/// The heading row's inset from the blocks' edges.
const SECTION_TITLE_PADDING_X: f32 = 4.0;
/// The corner radius of a section's bordered block.
const BLOCK_RADIUS: f32 = 12.0;
const ROW_PADDING_X: f32 = 14.0;
const ROW_PADDING_Y: f32 = 9.0;
const ROW_MIN_HEIGHT: f32 = 50.0;
/// Between a row's text and its control.
const ROW_GAP: f32 = 12.0;
const ROW_DESCRIPTION_LINE_HEIGHT: f32 = 17.0;
/// A heading with an icon (Files on About) is set a little larger than a plain one.
const ICON_HEADING_SIZE: f32 = 14.0;
/// The title inside a picture block, a little larger than a row title and semibold.
const BLOCK_HEADING_SIZE: f32 = 14.5;
/// The vertical padding of a row that holds pictures or a wide control instead of a title.
const PICTURE_ROW_PADDING_Y: f32 = 12.0;

/// The page's description: full title color, regular weight, `text_base`.
fn description_heading(text: &'static str, cx: &App) -> gpui::Div {
    div().text_base().text_color(crate::palette::heading(cx)).child(text)
}

/// A section's heading row: its title (with its small blue icon when it has one) at the left, its description at the right.
/// A heading with an icon reads as a group's name (the Files of About): full title color and a larger size.
pub fn section_title(
    text: impl Into<SharedString>,
    icon: Option<&'static str>,
    description: Option<SharedString>,
    cx: &App,
) -> gpui::Div {
    let muted = cx.theme().muted_foreground;
    let title = h_flex()
        .flex_none()
        .gap_2()
        .items_center()
        .when(icon.is_some(), |title| title.text_size(px(ICON_HEADING_SIZE)).text_color(crate::palette::heading(cx)))
        .children(icon.map(|path| Icon::default().path(path).size_4().text_color(crate::palette::accent(cx))))
        .child(text.into());
    h_flex()
        .w_full()
        .px(px(SECTION_TITLE_PADDING_X))
        .gap_4()
        .items_center()
        .justify_between()
        .text_sm()
        .text_color(muted)
        .child(title)
        .children(description.map(|note| div().min_w_0().text_right().child(note)))
}

/// A run of a group's visible items: the items that share one bordered block, or one item drawn outside any block.
#[derive(Debug, PartialEq, Eq)]
enum Part {
    Block(Vec<usize>),
    Loose(usize),
}

/// Splits items into parts in order: neighbours that sit in a block share one, an item outside a block stands alone.
fn parts(in_block: &[bool]) -> Vec<Part> {
    let mut parts = Vec::new();
    for (index, &boxed) in in_block.iter().enumerate() {
        match (boxed, parts.last_mut()) {
            (true, Some(Part::Block(indexes))) => indexes.push(index),
            (true, _) => parts.push(Part::Block(vec![index])),
            (false, _) => parts.push(Part::Loose(index)),
        }
    }
    parts
}

fn row_description(text: SharedString, cx: &App) -> gpui::Div {
    div().text_sm().line_height(px(ROW_DESCRIPTION_LINE_HEIGHT)).text_color(cx.theme().muted_foreground).child(text)
}

pub type Render = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

enum Body {
    /// A title and description on the left, a control on the right.
    Control { title: SharedString, description: Option<SharedString>, tag: Option<&'static str>, control: Render },
    /// A row that draws itself across the whole width.
    Custom(Render),
}

/// How a row in a block is padded and separated from the row above it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RowFrame {
    /// Pictures and wide controls: more room above and below, no minimum height.
    picture: bool,
}

const PLAIN_ROW: RowFrame = RowFrame { picture: false };

pub struct Item {
    body: Body,
    visible: Option<Rc<dyn Fn() -> bool>>,
    in_block: bool,
    frame: RowFrame,
}

impl Item {
    pub fn new(title: &'static str, control: Render) -> Self {
        Self {
            body: Body::Control { title: title.into(), description: None, tag: None, control },
            visible: None,
            in_block: true,
            frame: PLAIN_ROW,
        }
    }

    /// Draws the item straight onto the page between its group's blocks, for content that brings its own cards.
    pub fn outside_block(mut self) -> Self {
        self.in_block = false;
        self
    }

    pub fn custom(render: impl Fn(&mut Window, &mut App) -> AnyElement + 'static) -> Self {
        Self { body: Body::Custom(Rc::new(render)), visible: None, in_block: true, frame: PLAIN_ROW }
    }

    /// A row of pictures or a wide control in a block, with more room above and below than a titled row.
    pub fn picture_row(render: impl Fn(&mut Window, &mut App) -> AnyElement + 'static) -> Self {
        let mut item = Self::custom(render);
        item.frame.picture = true;
        item
    }

    /// A small muted tag after the title, such as "After restart".
    pub fn tag(mut self, text: &'static str) -> Self {
        if let Body::Control { tag, .. } = &mut self.body {
            *tag = Some(text);
        }
        self
    }

    pub fn description(mut self, text: &'static str) -> Self {
        if let Body::Control { description, .. } = &mut self.body {
            *description = Some(text.into());
        }
        self
    }

    /// The row is left out, with its divider, while the function gives `false`.
    pub fn visible_when(mut self, visible: impl Fn() -> bool + 'static) -> Self {
        self.visible = Some(Rc::new(visible));
        self
    }

    fn is_visible(&self) -> bool {
        self.visible.as_ref().is_none_or(|visible| visible())
    }

    fn render(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        match &self.body {
            Body::Custom(render) => render(window, cx),
            Body::Control { title, description, tag, control } => {
                let control = control(window, cx);
                let title = title_with_tag(title.clone(), *tag, cx);
                control_row_element(title, description.iter().cloned(), control, false, cx)
            }
        }
    }
}

/// A row's insides: the title over its muted lines at the left, the control at the right; for custom items whose
/// title changes while the page is shown.
pub fn control_row(
    title: SharedString,
    lines: impl IntoIterator<Item = SharedString>,
    control: AnyElement,
    top_aligned: bool,
    cx: &App,
) -> AnyElement {
    control_row_element(title_with_tag(title, None, cx), lines, control, top_aligned, cx)
}

/// A row title in medium weight, with its tag after it when it has one.
pub fn title_with_tag(title: SharedString, tag: Option<&'static str>, cx: &App) -> AnyElement {
    h_flex()
        .gap(px(8.0))
        .items_center()
        .child(div().text_base().font_weight(gpui::FontWeight::MEDIUM).child(title))
        .children(tag.map(|text| crate::tag::tag(text, crate::tag::Tone::Muted, cx)))
        .into_any_element()
}

/// A row's insides with a title already built, for a title that carries a tag.
pub fn control_row_element(
    title: AnyElement,
    lines: impl IntoIterator<Item = SharedString>,
    control: AnyElement,
    top_aligned: bool,
    cx: &App,
) -> AnyElement {
    h_flex()
        .when(top_aligned, |row| row.items_start())
        .when(!top_aligned, |row| row.items_center())
        .justify_between()
        .gap(px(ROW_GAP))
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap(px(1.0))
                .child(title)
                .children(lines.into_iter().map(|text| row_description(text, cx))),
        )
        .child(div().flex_none().child(control))
        .into_any_element()
}

/// The note at the right end of a section's heading row, read again on every paint.
type Note = Rc<dyn Fn() -> SharedString>;

pub struct Group {
    title: Option<&'static str>,
    icon: Option<&'static str>,
    description: Option<Note>,
    items: Vec<Item>,
    boxed: bool,
}

impl Group {
    pub fn new(items: Vec<Item>) -> Self {
        Self { title: None, icon: None, description: None, items, boxed: true }
    }

    /// Items drawn straight onto the page, for rows that bring their own border.
    pub fn loose(items: Vec<Item>) -> Self {
        Self { title: None, icon: None, description: None, items, boxed: false }
    }

    pub fn titled(title: &'static str, items: Vec<Item>) -> Self {
        Self { title: Some(title), icon: None, description: None, items, boxed: true }
    }

    /// A small blue icon (an SVG path from `icons`) before the title.
    pub fn icon(mut self, path: &'static str) -> Self {
        self.icon = Some(path);
        self
    }

    /// A short line at the right end of the heading row that says what the section is for.
    pub fn description(mut self, text: &'static str) -> Self {
        self.description = Some(Rc::new(move || text.into()));
        self
    }

    /// A heading note that follows the settings, such as a count or a total, built again on every paint.
    pub fn live_description(mut self, note: impl Fn() -> SharedString + 'static) -> Self {
        self.description = Some(Rc::new(note));
        self
    }

    fn render(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        let visible: Vec<&Item> = self.items.iter().filter(|item| item.is_visible()).collect();
        let in_block: Vec<bool> = visible.iter().map(|item| self.boxed && item.in_block).collect();
        let heading =
            self.title.map(|title| section_title(title, self.icon, self.description.as_ref().map(|note| note()), cx));
        let parts = parts(&in_block).into_iter().map(|part| match part {
            Part::Loose(index) => visible[index].render(window, cx),
            Part::Block(indexes) => block(indexes.iter().map(|&index| visible[index]), window, cx),
        });
        v_flex().w_full().gap(px(SECTION_TITLE_GAP)).children(heading).children(parts).into_any_element()
    }
}

/// The bold line that opens a picture block (the calculation methods), with its muted sentence under it.
pub fn block_heading(title: &'static str, description: Option<SharedString>, cx: &App) -> gpui::Div {
    v_flex()
        .gap(px(2.0))
        .child(
            div()
                .text_size(px(BLOCK_HEADING_SIZE))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(crate::palette::heading(cx))
                .child(title),
        )
        .children(description.map(|text| row_description(text, cx)))
}

/// One bordered block of rows; the border closes the last row, so only the rows between two others carry a divider.
fn block<'a>(items: impl Iterator<Item = &'a Item>, window: &mut Window, cx: &mut App) -> AnyElement {
    let divider = crate::palette::row_divider(cx);
    let rows: Vec<AnyElement> = items
        .enumerate()
        .map(|(index, item)| {
            let (padding_y, min_height) =
                if item.frame.picture { (PICTURE_ROW_PADDING_Y, 0.0) } else { (ROW_PADDING_Y, ROW_MIN_HEIGHT) };
            div()
                .flex()
                .flex_col()
                .justify_center()
                .min_h(px(min_height))
                .px(px(ROW_PADDING_X))
                .py(px(padding_y))
                .when(index > 0, |row| row.border_t_1().border_color(divider))
                .child(item.render(window, cx))
                .into_any_element()
        })
        .collect();
    block_frame(cx).children(rows).into_any_element()
}

/// The rounded block itself, for a section that lays out its own rows (the page list).
pub fn block_frame(cx: &App) -> gpui::Div {
    v_flex()
        .w_full()
        .rounded(px(BLOCK_RADIUS))
        .border_1()
        .border_color(crate::palette::card_border(cx))
        .bg(crate::palette::block_fill(cx))
        .overflow_hidden()
}

pub struct Page {
    pub title: &'static str,
    description: Option<&'static str>,
    groups: Vec<Group>,
}

impl Page {
    /// The sentence under the breadcrumb that says what the page is for; a redesigned page opens with a section.
    pub fn description(&self) -> Option<&'static str> {
        self.description
    }

    /// The section titles in order, for tests that check what a page holds.
    #[cfg(test)]
    pub fn group_titles(&self) -> Vec<&'static str> {
        self.groups.iter().filter_map(|group| group.title).collect()
    }

    /// A page that opens straight with its first section.
    pub fn plain(title: &'static str, groups: Vec<Group>) -> Self {
        Self { title, description: None, groups }
    }

    pub fn render(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        v_flex()
            .w_full()
            .gap(px(SECTION_GAP))
            .children(self.description().map(|text| description_heading(text, cx)))
            .children(self.groups.iter().map(|group| group.render(window, cx)))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbouring_rows_share_one_block() {
        assert_eq!(parts(&[true, true, true]), [Part::Block(vec![0, 1, 2])]);
    }

    #[test]
    fn an_item_outside_a_block_splits_the_rows_around_it() {
        assert_eq!(parts(&[true, false, true, true]), [Part::Block(vec![0]), Part::Loose(1), Part::Block(vec![2, 3])]);
    }

    #[test]
    fn cards_above_a_switch_row_come_first_then_the_row_in_its_own_block() {
        assert_eq!(parts(&[false, true]), [Part::Loose(0), Part::Block(vec![1])]);
    }

    #[test]
    fn a_group_without_items_has_no_parts() {
        assert_eq!(parts(&[]), []);
    }
}
