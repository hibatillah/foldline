use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use gpui::{AnyElement, App, SharedString, Window, div, prelude::*, px};
use gpui_component::{ActiveTheme as _, accordion::Accordion, h_flex, v_flex};

/// From a card's bottom to the next section's title.
const SECTION_GAP: f32 = 26.0;
/// From a section title to its card.
const SECTION_TITLE_GAP: f32 = 9.0;

/// A section title above its card, in full foreground and regular weight.
fn heading(title: &'static str, cx: &App) -> gpui::Div {
    div().text_base().font_weight(gpui::FontWeight::NORMAL).text_color(crate::section_title(cx)).child(title)
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
                .child(v_flex().flex_1().min_w_0().gap_0p5().child(div().text_sm().child(title.clone())).children(
                    description.clone().map(|text| div().text_sm().text_color(cx.theme().muted_foreground).child(text)),
                ))
                .child(div().flex_none().child(control(window, cx)))
                .into_any_element(),
        }
    }
}

pub struct Group {
    title: Option<&'static str>,
    items: Vec<Item>,
    boxed: bool,
}

impl Group {
    pub fn new(items: Vec<Item>) -> Self {
        Self { title: None, items, boxed: true }
    }

    /// Items drawn straight onto the page, for rows that bring their own border.
    pub fn loose(items: Vec<Item>) -> Self {
        Self { title: None, items, boxed: false }
    }

    pub fn titled(title: &'static str, items: Vec<Item>) -> Self {
        Self { title: Some(title), items, boxed: true }
    }

    fn render(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        if !self.boxed {
            let heading = self.title.map(|title| heading(title, cx));
            return v_flex()
                .gap(px(SECTION_TITLE_GAP))
                .children(heading)
                .children(self.items.iter().map(|item| item.render(window, cx)))
                .into_any_element();
        }
        let divider = crate::row_divider(cx);
        let rows: Vec<AnyElement> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                div()
                    .px_4()
                    .py_3()
                    .when(index > 0, |row| row.border_t_1().border_color(divider))
                    .child(item.render(window, cx))
                    .into_any_element()
            })
            .collect();
        let heading = self.title.map(|title| heading(title, cx));
        let card = v_flex()
            .w_full()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().group_box)
            .overflow_hidden()
            .children(rows);
        v_flex().w_full().gap(px(SECTION_TITLE_GAP)).children(heading).child(card).into_any_element()
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
    pub fn new(title: &'static str, description: &'static str, groups: Vec<Group>) -> Self {
        Self { title, description, groups }
    }

    pub fn render(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        v_flex()
            .w_full()
            .gap(px(SECTION_GAP))
            .child(
                v_flex()
                    .gap_1()
                    .child(div().text_xl().font_weight(gpui::FontWeight::SEMIBOLD).child(self.title))
                    .child(div().text_sm().text_color(cx.theme().muted_foreground).child(self.description)),
            )
            .children(self.groups.iter().map(|group| group.render(window, cx)))
            .into_any_element()
    }
}
