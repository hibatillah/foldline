//! The lock a row shows when the account does not unlock what it controls, with the button that lifts it. Display
//! only: the control beside it stays, and the notch enforces access.

use std::rc::Rc;

use gpui::{AnyElement, App, IntoElement, div, prelude::*};
use gpui_component::{Icon, IconName, h_flex};

use super::about::widgets::{Handler, pill};
use crate::{
    i18n::tr,
    icons,
    layout::Render,
    standing::{self, Lock},
};

/// **Sign in** and **Sign in again** open the Account page; **Upgrade** opens the site's pricing page.
fn lock_button(lock: Lock) -> (&'static str, Option<IconName>, Handler) {
    let open_account: Handler = Rc::new(|_window, cx| crate::select_page(icons::ACCOUNT, cx));
    match lock {
        Lock::SignIn => (tr!("pages.locked_sign_in"), None, open_account),
        Lock::SignInAgain => (tr!("pages.locked_sign_in_again"), None, open_account),
        Lock::Upgrade => (
            tr!("pages.locked_upgrade"),
            Some(IconName::ExternalLink),
            Rc::new(|_window, cx| super::account::open_pricing(cx)),
        ),
    }
}

/// A lock in the accent blue and the button that lifts it; `id` names the row it sits in.
pub fn lock_badge(id: &str, lock: Lock, cx: &App) -> AnyElement {
    let (label, icon, handler) = lock_button(lock);
    h_flex()
        .flex_none()
        .gap_2()
        .items_center()
        .child(Icon::default().path(icons::LOCK).size_4().text_color(crate::palette::accent(cx)))
        .child(pill(&format!("lock-{id}"), label, icon, Some(handler), cx))
        .into_any_element()
}

/// A row's control with the lock of `resource`/`action` before it while the account does not allow it.
pub fn locked(resource: &'static str, action: &'static str, control: Render) -> Render {
    Rc::new(move |window, cx| {
        let lock = standing::current().lock_for_feature(resource, action);
        let control = control(window, cx);
        match lock {
            Some(lock) => h_flex()
                .gap_3()
                .items_center()
                .child(lock_badge(&format!("{resource}-{action}"), lock, cx))
                .child(control)
                .into_any_element(),
            None => div().child(control).into_any_element(),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_locked_row_asks_a_guest_to_sign_in_and_a_member_to_upgrade() {
        assert_eq!(lock_button(Lock::SignIn).0, "Sign in");
        assert_eq!(lock_button(Lock::Upgrade).0, "Upgrade");
        assert_eq!(lock_button(Lock::SignInAgain).0, "Sign in again");
        assert!(lock_button(Lock::Upgrade).1.is_some(), "upgrading opens the site, so it shows the external mark");
    }
}
