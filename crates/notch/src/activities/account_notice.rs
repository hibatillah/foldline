//! The one pill message about the account, shown when the daily check finds this PC signed out elsewhere.

use crate::{
    activity::{Activity, Flash, Presence},
    gfx::{Canvas, Color, Rect},
    i18n::tr,
    page::{Context, TimerId},
    ui::{self, Icon},
};

/// The sentence is longer than the other messages, so it stays up longer.
const SHOW_MILLISECONDS: u32 = 5000;

/// In the account catalog and granted to every status, so it is never locked.
#[derive(Default)]
pub struct AccountNotice {
    flash: Flash,
    width: f32,
}

impl AccountNotice {
    fn measure(&mut self, context: &Context) {
        self.width = ui::message_width(context.fonts(), &tr!("account.signed_out_elsewhere"), None);
    }
}

impl Activity for AccountNotice {
    fn id(&self) -> &'static str {
        "account_notice"
    }

    fn presence(&self) -> Option<Presence> {
        self.flash.is_visible().then_some(Presence::Transient { width: self.width })
    }

    fn draw(&self, canvas: &Canvas, area: Rect, opacity: f32) {
        let message = tr!("account.signed_out_elsewhere");
        ui::draw_message(canvas, area, opacity, Icon::Bell, Color::WHITE, &message, None);
    }

    fn on_start(&mut self, context: &mut Context) {
        self.measure(context);
    }

    fn on_language_changed(&mut self, context: &mut Context) {
        self.measure(context);
    }

    fn on_account_signed_out_elsewhere(&mut self, context: &mut Context) {
        self.flash.show(context, SHOW_MILLISECONDS);
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        self.flash.on_timer(timer, context);
    }
}
