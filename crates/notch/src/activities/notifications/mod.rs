//! A Windows notification opens the pill into a card for a few seconds. The pointer on the card keeps it and shows
//! Dismiss and Open, and Reply when the toast has a quick reply; notifications.show_text=off hides the message text.

mod card;
mod items;
mod listener;
mod reply;
mod rounds;

use std::{sync::Arc, thread};

use self::{
    card::{Card, DrawLook, WIDTH},
    items::Item,
    listener::{Access, Shared},
    reply::{ReplyState, Typed},
};
use crate::{
    activity::{Activity, FLASH_TIMER, Flash, Presence},
    gfx::{Canvas, Rect},
    log,
    page::{Context, KeyInput, PointerEvent, TimerId},
    platform::toast_activation,
    ui::BannerButton,
};

/// As long as Windows shows its own banner by default.
const SHOW_MILLISECONDS: u32 = 5000;
/// After the pointer leaves the card, so a pointer that slips off the edge on the way to a button does not lose it.
const LINGER_MILLISECONDS: u32 = 1500;
const SHOW_TEXT_SETTING: &str = "notifications.show_text";

pub struct NotificationsActivity {
    shared: Option<Arc<Shared>>,
    shows_text: bool,
    flash: Flash,
    item: Option<Item>,
    card: Option<Card>,
    is_pointer_over: bool,
    reply: ReplyState,
}

impl NotificationsActivity {
    pub fn new() -> Self {
        Self {
            shared: None,
            shows_text: true,
            flash: Flash::default(),
            item: None,
            card: None,
            is_pointer_over: false,
            reply: ReplyState::default(),
        }
    }

    #[cfg(test)]
    pub fn sample(fonts: &crate::gfx::Fonts) -> Self {
        Self::sample_with_body(
            fonts,
            "Deploy is green. Can you check the hover state on the 1440p monitor before we tag it?",
        )
    }

    #[cfg(test)]
    pub fn sample_with_body(fonts: &crate::gfx::Fonts, body: &str) -> Self {
        let mut activity = Self::new();
        let item = Item {
            id: 1,
            app: "Discord".to_owned(),
            app_id: String::new(),
            icon: None,
            title: "Rafi".to_owned(),
            body: body.to_owned(),
            read_at: std::time::Instant::now(),
        };
        activity.card = Some(Card::new(&item, true, fonts));
        activity.item = Some(item);
        activity.flash = Flash::visible();
        activity
    }

    #[cfg(test)]
    pub fn hovered(mut self) -> Self {
        self.is_pointer_over = true;
        self
    }

    fn show(&mut self, item: Item, context: &mut Context) {
        log::debug(&format!(
            "notifications: card for {} shown {} ms after the listener read it",
            item.id,
            item.read_at.elapsed().as_millis()
        ));
        self.card = Some(Card::new(&item, self.shows_text, context.fonts()));
        self.item = Some(item);
        self.reply = ReplyState::default();
        self.flash.show(context, SHOW_MILLISECONDS);
        if self.is_pointer_over {
            context.kill_timer(FLASH_TIMER);
        }
    }

    fn hide(&mut self, context: &mut Context) {
        self.is_pointer_over = false;
        self.stop_typing(context);
        if self.flash.is_visible() {
            self.flash.on_timer(FLASH_TIMER, context);
        }
    }

    /// Leaves the reply field and gives the keyboard back; returns whether one was open.
    fn stop_typing(&mut self, context: &mut Context) -> bool {
        let was_typing = self.reply.is_typing();
        if was_typing {
            self.reply.stop_typing();
            context.release_keyboard();
        }
        was_typing
    }

    /// Reads the toast for a reply the first time the pointer is on the card, so the buttons are known by then.
    fn look_for_reply(&mut self, context: &mut Context) {
        let (Some(item), Some(card)) = (&self.item, &mut self.card) else { return };
        if self.reply.check(item.id) {
            card.offer_reply(context.fonts());
        }
    }

    fn pointer_left(&mut self, context: &mut Context) {
        if !self.is_pointer_over {
            return;
        }
        self.is_pointer_over = false;
        if self.flash.is_visible() && !self.reply.is_typing() {
            self.flash.show(context, LINGER_MILLISECONDS);
        }
    }

    fn shows_buttons(&self) -> bool {
        self.is_pointer_over || self.reply.is_typing()
    }

    fn send_reply(&mut self, text: String, context: &mut Context) {
        let (Some(item), Some(shared), Some(reply)) = (&self.item, &self.shared, self.reply.offer().cloned()) else {
            return;
        };
        let (shared, app_id, id) = (shared.clone(), item.app_id.clone(), item.id);
        thread::spawn(move || {
            toast_activation::send_reply(&app_id, &reply, &text);
            shared.remove(id);
        });
        self.hide(context);
    }

    fn clicked(&mut self, button: BannerButton, context: &mut Context) {
        let (Some(item), Some(shared)) = (&self.item, &self.shared) else { return };
        match button {
            BannerButton::Reply => {
                self.reply.start_typing();
                context.take_keyboard();
                context.kill_timer(FLASH_TIMER);
                context.request_repaint();
                return;
            }
            BannerButton::Dismiss => shared.remove(item.id),
            BannerButton::Open => {
                // Only now, inside the click, may the notch let the app come to the front.
                toast_activation::allow_foreground_handoff();
                let (shared, app_id, id) = (shared.clone(), item.app_id.clone(), item.id);
                thread::spawn(move || {
                    // The stored toast is read before it is removed, which deletes it.
                    if !toast_activation::open(&app_id, id) {
                        log::debug(&format!("notifications: nothing could open {app_id}"));
                    }
                    shared.remove(id);
                });
            }
        }
        self.hide(context);
    }
}

impl Activity for NotificationsActivity {
    fn id(&self) -> &'static str {
        "notifications"
    }

    fn is_optional(&self) -> bool {
        true
    }

    fn presence(&self) -> Option<Presence> {
        let card = self.card.as_ref()?;
        self.flash.is_visible().then_some(Presence::Banner { width: WIDTH, height: card.height(self.shows_buttons()) })
    }

    fn draw(&self, canvas: &Canvas, area: Rect, scale: f32, opacity: f32) {
        let (Some(item), Some(card)) = (&self.item, &self.card) else { return };
        let look = DrawLook { area, scale, opacity, shows_buttons: self.shows_buttons() };
        card.draw(canvas, item, look, &self.reply);
    }

    fn on_start(&mut self, context: &mut Context) {
        self.shows_text = context.settings.get_bool(SHOW_TEXT_SETTING, true);
        self.shared = Some(listener::start(context.notifier()));
    }

    fn on_settings_changed(&mut self, context: &mut Context) {
        self.shows_text = context.settings.get_bool(SHOW_TEXT_SETTING, self.shows_text);
        if let Some(item) = &self.item {
            let mut card = Card::new(item, self.shows_text, context.fonts());
            if self.reply.offer().is_some() {
                card.offer_reply(context.fonts());
            }
            self.card = Some(card);
        }
        context.request_repaint();
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        if timer == FLASH_TIMER {
            self.is_pointer_over = false;
            self.flash.on_timer(timer, context);
        }
    }

    /// Leaving the notch is leaving the card, also when the notch had opened meanwhile and the card got no `Left`.
    fn on_hover_changed(&mut self, is_hovering: bool, context: &mut Context) {
        if !is_hovering {
            self.pointer_left(context);
        }
    }

    fn on_pointer(&mut self, event: PointerEvent, area: Rect, scale: f32, context: &mut Context) {
        if !self.flash.is_visible() {
            return;
        }
        match event {
            PointerEvent::Moved(_) | PointerEvent::Pressed(_) if !self.is_pointer_over => {
                self.is_pointer_over = true;
                self.look_for_reply(context);
                context.kill_timer(FLASH_TIMER);
                context.request_repaint();
            }
            PointerEvent::Left | PointerEvent::CaptureLost => self.pointer_left(context),
            _ => {}
        }
        let Some(card) = &mut self.card else { return };
        let (clicked, needs_repaint) = card.on_pointer(event, area, scale);
        if needs_repaint {
            context.request_repaint();
        }
        if let Some(button) = clicked {
            self.clicked(button, context);
        }
    }

    fn on_key(&mut self, key: KeyInput, context: &mut Context) {
        let (typed, needs_repaint) = self.reply.apply(&key);
        if needs_repaint {
            context.request_repaint();
        }
        match typed {
            Typed::Editing => {}
            Typed::Send(text) => self.send_reply(text, context),
            Typed::Cancelled => {
                self.stop_typing(context);
                if self.flash.is_visible() {
                    self.flash.show(context, LINGER_MILLISECONDS);
                }
                context.request_repaint();
            }
        }
    }

    /// Typing ended because another window took the keyboard.
    fn on_keyboard_lost(&mut self, context: &mut Context) {
        self.reply.stop_typing();
        if self.flash.is_visible() {
            self.flash.show(context, LINGER_MILLISECONDS);
        }
        context.request_repaint();
    }

    fn on_notify(&mut self, context: &mut Context) {
        // A newer notification waits until the reply being typed is sent or given up.
        if self.reply.is_typing() {
            return;
        }
        let Some(shared) = self.shared.clone() else { return };
        if shared.access() == Access::Denied {
            log::debug("notifications: Windows does not let the notch read notifications");
        }
        if let Some(newest) = shared.take_arrived().pop() {
            self.show(newest, context);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_shows_until_a_notification_arrives() {
        assert_eq!(NotificationsActivity::new().presence(), None);
    }

    #[test]
    fn the_pointer_adds_the_buttons_under_the_same_two_lines_and_keeps_the_width() {
        let fonts = crate::gfx::Fonts::new(1.0).unwrap();
        let activity = NotificationsActivity::sample_with_body(&fonts, &"A long message that wraps. ".repeat(8));
        let Some(Presence::Banner { width: calm_width, height: calm_height }) = activity.presence() else { panic!() };
        let lines_before = activity.card.as_ref().unwrap().words.body.len();
        let activity = activity.hovered();
        let Some(Presence::Banner { width, height }) = activity.presence() else { panic!() };
        assert_eq!(width, calm_width);
        assert_eq!(height - calm_height, crate::ui::BANNER_BUTTONS_HEIGHT);
        assert_eq!(activity.card.as_ref().unwrap().words.body.len(), lines_before);
    }
}
