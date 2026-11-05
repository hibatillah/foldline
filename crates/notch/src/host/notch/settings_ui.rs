use super::{
    HIDE_WHEN_IDLE_SETTING, KeyboardOwner, Notch, PAGE_ORDER_SETTING, PAGES_OFF_SETTING, activity_setting_key,
    read_showing,
};
use crate::page::SearchHit;
use windows::Win32::{System::SystemInformation::GetTickCount64, UI::Input::KeyboardAndMouse::SetCapture};

use crate::host::{
    chrome,
    page_editor::{self, ChipView, Entry, Outcome, PageEditor},
    settings_panel::{self, Editing, Tab},
    startup,
    visibility::{self},
};
use crate::{
    gfx::{Canvas, Point, Rect, Size},
    log,
    page::{Control, Frame, KeyInput},
    ui::{FieldOutcome, TextField},
};

/// Which tab the settings panel shows, the page whose settings its first tab holds, and the text being typed.
pub(super) struct PanelState {
    pub(super) tab: Tab,
    pub(super) page: Option<usize>,
    pub(super) editing: Option<Editing>,
    /// The matches of the search being typed; their labels show as chips.
    pub(super) hits: Vec<SearchHit>,
}

impl Default for PanelState {
    fn default() -> Self {
        Self { tab: Tab::General, page: None, editing: None, hits: Vec::new() }
    }
}

impl Notch {
    pub(super) fn editor_entries(&self) -> Vec<Entry> {
        let chip_width =
            |page: usize| page_editor::chip_width(self.fonts.measure(&page_editor::CHIP, self.pages[page].name()));
        let on = self.page_list.order().iter().copied().filter(|&page| self.page_list.is_enabled(page));
        let off = (0..self.pages.len()).filter(|&page| !self.page_list.is_enabled(page));
        on.map(|page| (page, true))
            .chain(off.map(|page| (page, false)))
            .map(|(page, is_on)| Entry { page, is_on, chip_width: chip_width(page) })
            .collect()
    }

    pub(super) fn editor_layout(&self, frame: &Frame) -> page_editor::Layout {
        page_editor::layout(
            &self.editor_entries(),
            chrome::content_rect(frame.bounds, self.scale),
            chrome::header_rect(frame.bounds, self.scale),
            self.scale,
        )
    }

    pub(super) fn editor_size(&self) -> Size {
        let width = settings_panel::PANEL_WIDTH;
        let height = if self.panel.tab == Tab::Pages {
            let bounds = Rect::new(0.0, 0.0, width, 0.0);
            page_editor::layout(&self.editor_entries(), bounds, Rect::new(0.0, 0.0, width, 16.0), 1.0).content_height
        } else {
            settings_panel::content_height(&self.panel_controls())
        };
        Size::new(width, height)
    }

    /// Opens the settings panel on the open page's own settings when it has any.
    pub(super) fn open_panel(&mut self) {
        self.panel.page = self.expanded_index.filter(|&index| !self.pages[index].settings_groups().is_empty());
        self.panel.tab = if self.panel.page.is_some() { Tab::Page(0) } else { Tab::General };
        self.editor = Some(PageEditor::default());
        self.pending.repaint = true;
    }

    pub(super) fn close_panel(&mut self) {
        self.finish_panel_editing(false);
        self.editor = None;
        self.pending.repaint = true;
    }

    pub(super) fn panel_tabs(&self) -> Vec<(Tab, &'static str)> {
        let groups = self.panel.page.map(|index| self.pages[index].settings_groups()).unwrap_or_default();
        let page_tabs = groups.iter().enumerate().map(|(position, group)| (Tab::Page(position), group.title));
        page_tabs.chain([(Tab::General, "General"), (Tab::Behavior, "Behavior"), (Tab::Pages, "Pages")]).collect()
    }

    pub(super) fn panel_controls(&self) -> Vec<Control> {
        match self.panel.tab {
            Tab::Page(position) => self
                .panel
                .page
                .and_then(|index| self.pages[index].settings_groups().into_iter().nth(position))
                .map(|group| group.controls)
                .unwrap_or_default(),
            Tab::General => settings_panel::general_controls(),
            Tab::Behavior => settings_panel::behavior_controls(),
            Tab::Pages => Vec::new(),
        }
    }

    pub(super) fn saved_value(&self, key: &str) -> Option<String> {
        if key == settings_panel::START_WITH_WINDOWS_KEY {
            return Some(if startup::is_enabled() { "on" } else { "off" }.to_owned());
        }
        self.settings.get_string(key).map(str::to_owned)
    }

    pub(super) fn save_setting(&mut self, key: &str, value: &str) {
        log::debug(&format!("setting {key} changed in the notch"));
        if key == settings_panel::START_WITH_WINDOWS_KEY {
            startup::set_enabled(value == "on");
        } else {
            self.settings.set_string(key, value);
            self.reload_settings();
        }
        self.pending.repaint = true;
    }

    pub(super) fn panel_layout(&self, frame: &Frame) -> (settings_panel::TabsLayout, Vec<settings_panel::Placed>) {
        let measure = |label: &str| self.fonts.measure(&page_editor::CHIP, label);
        let tabs = settings_panel::tabs_layout(
            &self.panel_tabs(),
            chrome::header_rect(frame.bounds, self.scale),
            self.scale,
            &measure,
        );
        let controls = self.panel_controls();
        let content = chrome::content_rect(frame.bounds, self.scale);
        let results = self.hit_labels();
        (tabs, settings_panel::layout(&controls, content, self.scale, &measure, &results))
    }

    pub(super) fn press_panel(&mut self, point: Point) -> bool {
        let frame = self.frame();
        let (tabs, placed) = self.panel_layout(&frame);
        if tabs.done.contains(point) {
            self.close_panel();
            return true;
        }
        if let Some(&(tab, _)) = tabs.tabs.iter().find(|(_, rect)| rect.contains(point)) {
            self.finish_panel_editing(false);
            self.panel.tab = tab;
            self.pending.repaint = true;
            return true;
        }
        if self.panel.tab == Tab::Pages {
            return false;
        }
        let controls = self.panel_controls();
        let Some((index, part)) = settings_panel::hit(&placed, point) else {
            self.finish_panel_editing(true);
            return true;
        };
        let control = &controls[index];
        if let settings_panel::Part::Result(position) = part {
            self.pick_hit(position);
        } else if part == settings_panel::Part::Field {
            if self.panel.editing.as_ref().is_some_and(|editing| editing.control == index) {
                return true;
            }
            self.finish_panel_editing(true);
            let mut field = TextField::new(settings_panel::TEXT_LIMIT);
            if !matches!(control, Control::Search { .. }) {
                field.apply(&KeyInput::Text(self.saved_value(control.key()).unwrap_or_default()));
            }
            self.panel.editing = Some(Editing { control: index, field });
            self.take_keyboard_for(KeyboardOwner::Panel);
        } else {
            self.finish_panel_editing(true);
            let saved = self.saved_value(control.key());
            if let Some(value) = settings_panel::pressed_value(control, part, saved.as_deref()) {
                self.save_setting(control.key(), &value);
            }
        }
        self.pending.repaint = true;
        true
    }

    pub(super) fn panel_key(&mut self, key: &KeyInput) {
        let Some(editing) = self.panel.editing.as_mut() else { return };
        let outcome = editing.field.apply(key);
        let (control, query) = (editing.control, editing.field.text().to_owned());
        let is_search = matches!(self.panel_controls().get(control), Some(Control::Search { .. }));
        match outcome {
            FieldOutcome::Submitted if is_search => {
                self.pick_hit(0);
                self.finish_panel_editing(false);
            }
            FieldOutcome::Submitted => self.finish_panel_editing(true),
            FieldOutcome::Cancelled => self.finish_panel_editing(false),
            FieldOutcome::Changed | FieldOutcome::Unchanged => {
                if is_search {
                    self.refresh_hits(control, &query);
                }
            }
        }
        self.pending.repaint = true;
    }

    pub(super) fn hit_labels(&self) -> Vec<&str> {
        self.panel.hits.iter().map(|hit| hit.label.as_str()).collect()
    }

    /// Asks the open page for matches once at least two characters are typed.
    pub(super) fn refresh_hits(&mut self, control: usize, query: &str) {
        let key = self.panel_controls().get(control).map(Control::key);
        self.panel.hits = match (self.panel.page, key) {
            (Some(index), Some(key)) if query.trim().chars().count() >= 2 => {
                self.pages[index].search(key, query.trim())
            }
            _ => Vec::new(),
        };
    }

    /// Saves every value of one match, then re-reads the settings once.
    pub(super) fn pick_hit(&mut self, position: usize) {
        let Some(hit) = self.panel.hits.get(position).cloned() else { return };
        for (key, value) in &hit.values {
            self.settings.set_string(key, value);
        }
        log::debug(&format!("search match {} picked in the notch", hit.label));
        self.panel.hits.clear();
        self.finish_panel_editing(false);
        self.reload_settings();
        self.pending.repaint = true;
    }

    /// Ends typing in the panel; `save` keeps what was typed.
    pub(super) fn finish_panel_editing(&mut self, save: bool) {
        let Some(editing) = self.panel.editing.take() else { return };
        self.panel.hits.clear();
        let is_search = |control: &Control| matches!(control, Control::Search { .. });
        if save && let Some(control) = self.panel_controls().get(editing.control).filter(|control| !is_search(control))
        {
            let typed = editing.field.text().trim().to_owned();
            if self.saved_value(control.key()).unwrap_or_default() != typed {
                self.save_setting(control.key(), &typed);
            }
        }
        if self.keyboard.owner == Some(KeyboardOwner::Panel) {
            self.give_keyboard_back();
        }
        self.pending.repaint = true;
    }

    pub(super) fn press_editor(&mut self, point: Point) {
        let frame = self.frame();
        if !frame.is_open() || self.press_panel(point) {
            return;
        }
        let layout = self.editor_layout(&frame);
        let Some(editor) = self.editor.as_mut() else { return };
        let outcome = editor.press(&layout, point, unsafe { GetTickCount64() });
        if editor.is_holding() {
            self.hover.is_captured = true;
            unsafe { SetCapture(self.window) };
        }
        self.apply_edit(outcome);
        self.pending.repaint = true;
    }

    pub(super) fn release_editor(&mut self, point: Point) {
        let frame = self.frame();
        let layout = self.editor_layout(&frame);
        let Some(editor) = self.editor.as_mut() else { return };
        let outcome = editor.release(&layout, point);
        self.apply_edit(outcome);
        self.pending.repaint = true;
    }

    pub(super) fn apply_edit(&mut self, outcome: Outcome) {
        match outcome {
            Outcome::Nothing => return,
            Outcome::Done => {
                self.close_panel();
            }
            Outcome::Move { page, position } => {
                let on_pages: Vec<usize> =
                    self.page_list.order().iter().copied().filter(|&index| self.page_list.is_enabled(index)).collect();
                if let Some(&target) = on_pages.get(position)
                    && let Some(order_position) = self.page_list.order().iter().position(|&index| index == target)
                {
                    self.page_list.move_to(page, order_position);
                }
            }
            Outcome::Remove(page) => {
                let on_count = (0..self.pages.len()).filter(|&index| self.page_list.is_enabled(index)).count();
                if on_count > 1 {
                    self.page_list.set_enabled(page, false);
                }
            }
            Outcome::Add(page) => self.page_list.set_enabled(page, true),
        }
        let ids = self.page_ids();
        let (order, disabled) = (self.page_list.order_text(&ids), self.page_list.disabled_text(&ids));
        self.settings.set_string(PAGE_ORDER_SETTING, &order);
        self.settings.set_string(PAGES_OFF_SETTING, &disabled);
    }

    pub(super) fn draw_editor(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let (tabs, placed) = self.panel_layout(frame);
        settings_panel::draw_tabs(canvas, &tabs, &self.panel_tabs(), self.panel.tab, opacity);
        if self.panel.tab != Tab::Pages {
            let controls = self.panel_controls();
            for item in &placed {
                let control = &controls[item.control];
                let saved = self.saved_value(control.key());
                let editing = self.panel.editing.as_ref().filter(|editing| editing.control == item.control);
                let field = editing.map(|editing| &editing.field);
                let results = self.hit_labels();
                settings_panel::draw_control(canvas, control, item, saved.as_deref(), field, &results, opacity);
            }
            return;
        }
        let layout = self.editor_layout(frame);
        page_editor::draw_labels(canvas, &layout, opacity);
        let dragged = self.editor.as_ref().and_then(PageEditor::dragged);
        for (is_on, chips) in [(true, &layout.on_chips), (false, &layout.off_chips)] {
            for &(page, rect) in chips {
                let is_dragged = is_on && dragged.is_some_and(|(dragged_page, _)| dragged_page == page);
                let rect = match dragged {
                    Some((_, offset)) if is_dragged => Rect::new(
                        rect.left + offset.x,
                        rect.top + offset.y,
                        rect.right + offset.x,
                        rect.bottom + offset.y,
                    ),
                    _ => rect,
                };
                let view = ChipView { name: self.pages[page].name(), icon: self.pages[page].icon() };
                page_editor::draw_chip(canvas, rect, &view, is_on, is_dragged, opacity);
            }
        }
    }

    /// Re-reads settings.txt after `WM_SETTINGS_CHANGED`; a missing or unreadable value keeps the current one.
    /// Hover, the open page and the page shown stay as they are.
    pub fn reload_settings(&mut self) {
        self.settings.reload();
        // A hand-edited old key still migrates while the notch runs.
        visibility::migrate(&mut self.settings);
        let was_suppressed = self.is_suppressed();
        self.hiding.hides_when_idle = self.settings.get_bool(HIDE_WHEN_IDLE_SETTING, self.hiding.hides_when_idle);
        self.hiding.maximized_showing =
            read_showing(&self.settings, visibility::MAXIMIZED_KEY, self.hiding.maximized_showing);
        self.hiding.fullscreen_showing =
            read_showing(&self.settings, visibility::FULLSCREEN_KEY, self.hiding.fullscreen_showing);
        self.hiding.opens_on_hover_over_maximized =
            self.settings.get_bool(visibility::HOVER_MAXIMIZED_KEY, self.hiding.opens_on_hover_over_maximized);
        self.hiding.opens_on_hover_over_fullscreen =
            self.settings.get_bool(visibility::HOVER_FULLSCREEN_KEY, self.hiding.opens_on_hover_over_fullscreen);
        let navigable_before = self.navigable.clone();
        self.reload_page_list();
        for index in 0..self.pages.len() {
            self.with_page(index, |page, context| page.on_settings_changed(context));
        }
        for index in 0..self.activities.len() {
            let slot = &mut self.activities[index];
            if slot.activity.is_optional() {
                slot.is_enabled = self.settings.get_bool(&activity_setting_key(slot.activity.id()), slot.is_enabled);
            }
            // A started listener thread cannot be stopped; switching off only hides the activity's presence.
            if self.activities[index].is_enabled {
                self.start_activity(index);
            }
            if self.activities[index].is_started {
                self.with_activity(index, |activity, context| activity.on_settings_changed(context));
            }
        }
        self.apply_suppression(was_suppressed);
        self.keep_shown_page_in_place(&navigable_before);
        self.flush();
    }
}
