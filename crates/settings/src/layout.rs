use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use gpui::{AnyElement, App, SharedString, Window, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, Icon, accordion::Accordion, h_flex, v_flex};

/// From a card's bottom to the next section's title.
const SECTION_GAP: f32 = 22.0;
/// From a section title to its block.
const SECTION_TITLE_GAP: f32 = 8.0;
/// The corner radius of a section's bordered block.
const BLOCK_RADIUS: f32 = 8.0;

/// The page's description: full title color, regular weight, `text_base`.
fn description_heading(text: &'static str, cx: &App) -> gpui::Div {
    div().text_base().text_color(crate::palette::heading(cx)).child(text)
}

/// A section's title above its block, with its small blue icon when it has one.
fn section_title(text: &'static str, icon: Option<&'static str>, cx: &App) -> gpui::Div {
    h_flex()
        .gap_2()
        .items_center()
        .text_base()
        .text_color(crate::palette::heading(cx))
        .children(icon.map(|path| Icon::default().path(path).size_4().text_color(crate::palette::accent(cx))))
        .child(text)
}

pub type Render = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

enum Body {
    /// A title and description on the left, a control on the right.
    Control { title: SharedString, description: Option<SharedString>, control: Render, top_aligned: bool },
    /// A row that draws itself across the whole width.
    Custom(Render),
}

pub struct Item(Body);

impl Item {
    pub fn new(title: &'static str, control: Render) -> Self {
        Self(Body::Control { title: title.into(), description: None, control, top_aligned: false })
    }

    /// Aligns the text with the top of a control that grows downward, like a search field with its results.
    pub fn top_aligned(mut self) -> Self {
        if let Body::Control { top_aligned, .. } = &mut self.0 {
            *top_aligned = true;
        }
        self
    }

    pub fn custom(render: impl Fn(&mut Window, &mut App) -> AnyElement + 'static) -> Self {
        Self(Body::Custom(Rc::new(render)))
    }

    pub fn description(mut self, text: &'static str) -> Self {
        if let Body::Control { description, .. } = &mut self.0 {
            *description = Some(text.into());
        }
        self
    }

    fn render(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        match &self.0 {
            Body::Custom(render) => render(window, cx),
            Body::Control { title, description, control, top_aligned } => h_flex()
                .when(*top_aligned, |row| row.items_start())
                .when(!*top_aligned, |row| row.items_center())
                .justify_between()
                .gap_4()
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .gap_0p5()
                        .child(div().text_sm().font_weight(gpui::FontWeight::MEDIUM).child(title.clone()))
                        .children(
                            description
                                .clone()
                                .map(|text| div().text_sm().text_color(cx.theme().muted_foreground).child(text)),
                        ),
                )
                .child(div().flex_none().child(control(window, cx)))
                .into_any_element(),
        }
    }
}

pub struct Group {
    title: Option<&'static str>,
    icon: Option<&'static str>,
    items: Vec<Item>,
    boxed: bool,
}

impl Group {
    pub fn new(items: Vec<Item>) -> Self {
        Self { title: None, icon: None, items, boxed: true }
    }

    /// Items drawn straight onto the page, for rows that bring their own border.
    pub fn loose(items: Vec<Item>) -> Self {
        Self { title: None, icon: None, items, boxed: false }
    }

    pub fn titled(title: &'static str, items: Vec<Item>) -> Self {
        Self { title: Some(title), icon: None, items, boxed: true }
    }

    /// A small blue icon (an SVG path from `icons`) before the title.
    pub fn icon(mut self, path: &'static str) -> Self {
        self.icon = Some(path);
        self
    }

    fn render(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        if !self.boxed {
            let heading = self.title.map(|title| section_title(title, self.icon, cx));
            return v_flex()
                .gap(px(SECTION_TITLE_GAP))
                .children(heading)
                .children(self.items.iter().map(|item| item.render(window, cx)))
                .into_any_element();
        }
        let divider = crate::palette::row_divider(cx);
        let rows: Vec<AnyElement> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                div()
                    .px_4()
                    .py_3p5()
                    .when(index > 0, |row| row.border_t_1().border_color(divider))
                    .child(item.render(window, cx))
                    .into_any_element()
            })
            .collect();
        let heading = self.title.map(|title| section_title(title, self.icon, cx));
        // The border closes the last row, so only the rows between two others carry a divider.
        let block = v_flex()
            .w_full()
            .rounded(px(BLOCK_RADIUS))
            .border_1()
            .border_color(crate::palette::card_border(cx))
            .bg(crate::palette::block_fill(cx))
            .overflow_hidden()
            .children(rows);
        v_flex().w_full().gap(px(SECTION_TITLE_GAP)).children(heading).child(block).into_any_element()
    }
}

/// Rows that open and close; `multiple` lets several stay open. The open rows are kept per `id` across repaints.
pub fn accordion(
    id: &'static str,
    multiple: bool,
    entries: Vec<(SharedString, AnyElement)>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let open_rows = window.use_keyed_state(id, cx, |_window, _cx| Arc::new(Mutex::new(Vec::<usize>::new())));
    let open_rows = open_rows.read(cx).clone();
    let open_now = open_rows.lock().map(|rows| rows.clone()).unwrap_or_default();
    let mut accordion = Accordion::new(id).multiple(multiple).on_toggle_click({
        let open_rows = open_rows.clone();
        move |rows, window, _cx| {
            if let Ok(mut stored) = open_rows.lock() {
                *stored = rows.to_vec();
            }
            window.refresh();
        }
    });
    for (index, (title, body)) in entries.into_iter().enumerate() {
        let is_open = open_now.contains(&index);
        accordion = accordion.item(move |item| item.title(title).open(is_open).child(body));
    }
    div().w_full().child(accordion).into_any_element()
}

pub struct Page {
    pub title: &'static str,
    description: &'static str,
    groups: Vec<Group>,
}

impl Page {
    /// The sentence under the breadcrumb that says what the page is for.
    pub fn description(&self) -> &'static str {
        self.description
    }

    pub fn new(title: &'static str, description: &'static str, groups: Vec<Group>) -> Self {
        Self { title, description, groups }
    }

    pub fn render(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        v_flex()
            .w_full()
            .gap(px(SECTION_GAP))
            .child(description_heading(self.description(), cx))
            .children(self.groups.iter().map(|group| group.render(window, cx)))
            .into_any_element()
    }
}
