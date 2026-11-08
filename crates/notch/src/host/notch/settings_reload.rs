use super::{HIDE_WHEN_IDLE_SETTING, Notch, activity_setting_key, read_showing};
use crate::host::{notification_banners, visibility, volume_flyout};

impl Notch {
    /// Re-reads settings.txt after `WM_SETTINGS_CHANGED`; a missing or unreadable value keeps the current one.
    /// Hover, the open page and the page shown stay as they are.
    pub fn reload_settings(&mut self) {
        self.settings.reload();
        // A hand-edited old key still migrates while the notch runs.
        visibility::migrate(&mut self.settings);
        let was_suppressed = self.is_suppressed();
        self.hiding.hides_when_idle = self.settings.get_bool(HIDE_WHEN_IDLE_SETTING, self.hiding.hides_when_idle);
        volume_flyout::set_hidden(self.settings.get_bool(volume_flyout::SETTING_KEY, volume_flyout::is_hidden()));
        let hides_banners = notification_banners::is_hidden();
        let wants_banners_hidden = self.settings.get_bool(notification_banners::SETTING_KEY, hides_banners);
        // Unchanged stays put: switching off restores, which every unrelated settings change must not repeat.
        if wants_banners_hidden != hides_banners {
            notification_banners::set_hidden(wants_banners_hidden);
        }
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
