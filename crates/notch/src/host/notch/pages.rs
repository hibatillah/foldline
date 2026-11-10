use super::{LAST_PAGE_SETTING, Notch, PAGE_ORDER_SETTING, PAGES_OFF_SETTING};
use crate::host::{chrome, navigation, navigation::PageList};
use crate::{
    activity::{Activity, SystemMessage},
    gfx::Point,
    log,
    page::{Context, Page},
};

const KNOWN_PAGES_SETTING: &str = "pages.known";

impl Notch {
    pub fn page_notified(&mut self, index: usize) {
        if index < self.pages.len() {
            self.with_page(index, |page, context| page.on_notify(context));
            self.flush();
        }
    }

    pub fn activity_notified(&mut self, index: usize) {
        if index < self.activities.len() && self.activities[index].is_started {
            self.with_activity(index, |activity, context| activity.on_notify(context));
            self.flush();
        }
    }

    pub fn system_message(&mut self, message: SystemMessage) {
        for index in 0..self.pages.len() {
            if self.page_list.is_enabled(index) {
                self.with_page(index, |page, context| page.on_system_message(&message, context));
            }
        }
        for index in 0..self.activities.len() {
            if self.activities[index].is_started {
                self.with_activity(index, |activity, context| activity.on_system_message(&message, context));
            }
        }
        self.flush();
    }

    pub(super) fn start_activity(&mut self, index: usize) {
        if self.activities[index].is_started {
            return;
        }
        self.activities[index].is_started = true;
        log::debug(&format!("activity {index}: {}", self.activities[index].activity.id()));
        self.with_activity(index, |activity, context| activity.on_start(context));
    }

    pub(super) fn with_activity<R>(
        &mut self,
        index: usize,
        hook: impl FnOnce(&mut dyn Activity, &mut Context) -> R,
    ) -> R {
        let mut context = Context::for_activity(
            self.window,
            index,
            &self.surface,
            &self.fonts,
            &mut self.pending,
            &mut self.settings,
        );
        hook(self.activities[index].activity.as_mut(), &mut context)
    }

    pub(super) fn with_page<R>(&mut self, index: usize, hook: impl FnOnce(&mut dyn Page, &mut Context) -> R) -> R {
        let mut context =
            Context::new(self.window, index, &self.surface, &self.fonts, &mut self.pending, &mut self.settings);
        hook(self.pages[index].as_mut(), &mut context)
    }

    /// Moves one page forward or back and loops from the last page to the first and the other way round.
    pub(super) fn navigate(&mut self, step: i32) {
        if self.navigable.len() < 2 {
            return;
        }
        self.wrap_track();
        let count = self.navigable.len() as i64;
        let origin = self.track.target.round();
        let current = (origin as i64).rem_euclid(count);
        let next = (current + i64::from(step)).rem_euclid(count);
        self.track.target = origin + step as f32;
        self.select_page(self.navigable[next as usize]);
        self.start_animation();
    }

    pub(super) fn wrap_track(&mut self) {
        let count = self.navigable.len() as f32;
        if self.track.is_settled() && count > 0.0 {
            self.track.jump_to(self.track.target.rem_euclid(count));
        }
    }

    pub(super) fn go_to_page(&mut self, target: usize) {
        if Some(target) != self.expanded_index {
            self.select_page(target);
            self.start_animation();
        }
    }

    /// A page asked to be shown by a click on its banner, which opens the notch even where hovering would not.
    pub(super) fn open_on_page(&mut self, index: usize) {
        // Only a pointer on the notch can close it again by leaving.
        if !self.page_list.is_enabled(index) || !self.hover.is_hovering {
            return;
        }
        self.go_to_page(index);
        self.hiding.is_revealed = true;
        self.begin_opening();
        self.start_animation();
    }

    pub(super) fn select_page(&mut self, target: usize) {
        self.selected_page = Some(target);
        let id = self.pages[target].id();
        self.settings.set_string(LAST_PAGE_SETTING, id);
    }

    pub(super) fn press_header(&mut self, point: Point) -> bool {
        let frame = self.frame();
        if !frame.is_open() {
            return false;
        }
        let dots = chrome::dot_rects(self.navigable.len(), self.track.position, frame.bounds, self.scale);
        if let Some(position) = chrome::dot_at(&dots, point, self.scale) {
            self.go_to_page(self.navigable[position]);
            return true;
        }
        self.press_header_marks(point)
    }

    /// Lets the settings window list the pages; the file is written only when the list changed.
    pub(super) fn publish_known_pages(&mut self, ids: &[&str], opt_in: &[bool]) {
        let names: Vec<String> = self.pages.iter().map(|page| page.name()).collect();
        let labels: Vec<&str> = names.iter().map(String::as_str).collect();
        let known = navigation::known_pages_text(ids, &labels, opt_in);
        if self.settings.get_string(KNOWN_PAGES_SETTING) != Some(known.as_str()) {
            self.settings.set_string(KNOWN_PAGES_SETTING, &known);
        }
    }

    pub(super) fn page_ids(&self) -> Vec<&'static str> {
        self.pages.iter().map(|page| page.id()).collect()
    }

    pub(super) fn flush(&mut self) {
        let requests = std::mem::take(&mut self.pending);
        if let Some((owner, wants_keyboard)) = requests.keyboard {
            self.set_keyboard(owner, wants_keyboard);
        }
        if requests.close {
            self.set_hovering(false);
        }
        if let Some(index) = requests.open_page {
            self.open_on_page(index);
        }
        let layout_changed = self.refresh_layout();
        self.sync_page_hotkeys();
        if requests.animate {
            self.start_animation();
        }
        if requests.repaint || layout_changed {
            self.paint();
        }
        // Telling pages what is on screen can make them ask for more, which the next flush applies.
        if self.sync_shown_page() {
            self.flush();
        }
    }

    /// A list that switches every page off is ignored.
    pub(super) fn reload_page_list(&mut self) {
        let was_enabled: Vec<bool> = (0..self.pages.len()).map(|index| self.page_list.is_enabled(index)).collect();
        let ids = self.page_ids();
        let opt_in: Vec<bool> = self.pages.iter().map(|page| page.is_opt_in()).collect();
        let (current_order, current_disabled) = (self.page_list.order_text(&ids), self.page_list.disabled_text(&ids));
        let page_list = PageList::load(
            &ids,
            &opt_in,
            Some(self.settings.get_string(PAGE_ORDER_SETTING).unwrap_or(&current_order)),
            Some(self.settings.get_string(PAGES_OFF_SETTING).unwrap_or(&current_disabled)),
        );
        if (0..self.pages.len()).any(|index| page_list.is_enabled(index)) {
            self.page_list = page_list;
        }
        for (index, was_enabled) in was_enabled.into_iter().enumerate() {
            let is_enabled = self.page_list.is_enabled(index);
            if is_enabled != was_enabled {
                self.with_page(index, |page, context| page.on_enabled_changed(is_enabled, context));
            }
        }
    }

    /// A reordered or switched page list moves the open page to another row; jump there instead of scrolling.
    pub(super) fn keep_shown_page_in_place(&mut self, navigable_before: &[usize]) {
        self.refresh_layout();
        if self.navigable == navigable_before {
            return;
        }
        if let Some(position) = self.navigable.iter().position(|&index| Some(index) == self.expanded_index) {
            self.track.jump_to(position as f32);
        }
    }
}

pub(super) fn activity_setting_key(id: &str) -> String {
    format!("activity.{id}")
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn an_activity_is_switched_off_by_its_own_settings_key() {
        assert_eq!(activity_setting_key("volume"), "activity.volume");
    }
}
