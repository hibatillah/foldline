use super::{Notch, activity_setting_key};
use store::language::resolve_setting;

use crate::host::{notification_banners, visibility, volume_flyout};

impl Notch {
    /// Re-reads settings.txt after `WM_SETTINGS_CHANGED`; a missing or unreadable value keeps the current one.
    /// Hover, the open page and the page shown stay as they are.
    pub fn reload_settings(&mut self) {
        self.settings.reload();
        let language_before = self.language;
        self.language = resolve_setting(&self.settings, self.system_language);
        store::language::set_current(self.language);
        // A hand-edited old key still migrates while the notch runs.
        visibility::migrate(&mut self.settings);
        let was_suppressed = self.is_suppressed();
        self.hiding.reload(&self.settings);
        volume_flyout::set_hidden(self.settings.get_bool(volume_flyout::SETTING_KEY, volume_flyout::is_hidden()));
        let hides_banners = notification_banners::is_hidden();
        let wants_banners_hidden = self.settings.get_bool(notification_banners::SETTING_KEY, hides_banners);
        // Unchanged stays put: switching off restores, which every unrelated settings change must not repeat.
        if wants_banners_hidden != hides_banners {
            notification_banners::set_hidden(wants_banners_hidden);
        }
        self.sync_location_tracking();
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
        if self.language != language_before {
            self.language_changed();
        }
        self.apply_suppression(was_suppressed);
        self.keep_shown_page_in_place(&navigable_before);
        self.flush();
    }

    /// Tells every page and started activity to rebuild cached text, and lists the pages again under their new names.
    fn language_changed(&mut self) {
        for index in 0..self.pages.len() {
            self.with_page(index, |page, context| page.on_language_changed(context));
        }
        for index in 0..self.activities.len() {
            if self.activities[index].is_started {
                self.with_activity(index, |activity, context| activity.on_language_changed(context));
            }
        }
        let ids = self.page_ids();
        let opt_in: Vec<bool> = self.pages.iter().map(|page| page.is_opt_in()).collect();
        self.publish_known_pages(&ids, &opt_in);
        self.pending.repaint = true;
    }
}
