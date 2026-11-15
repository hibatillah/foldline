use crate::grants::Grants;
use crate::status::Status;

/// One status with the grants that apply to it; the host asks it before showing a page or activity.
pub struct Access {
    pub status: Status,
    pub grants: Grants,
}

impl Access {
    pub fn guest() -> Access {
        Access { status: Status::Guest, grants: Grants::built_in() }
    }

    pub fn allows(&self, resource: &str, action: &str) -> bool {
        self.grants.allows(self.status, resource, action)
    }

    pub fn allows_page(&self, page_id: &str) -> bool {
        self.allows(page_id, "page")
    }

    pub fn allows_activity(&self, activity_id: &str) -> bool {
        self.allows("activity", activity_id)
    }

    /// The notch replaces Windows' volume and brightness flyout only while it may show both of them.
    pub fn may_hide_volume_flyout(&self) -> bool {
        self.allows_activity("volume") && self.allows_activity("brightness")
    }

    pub fn may_hide_notification_banners(&self) -> bool {
        self.allows_activity("notifications")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_guest_sees_its_pages_and_activities_only() {
        let access = Access::guest();
        assert!(access.allows_page("media"));
        assert!(!access.allows_page("weather"));
        assert!(access.allows_activity("volume"));
        assert!(!access.allows_activity("notifications"));
        assert!(!access.allows("media", "lyrics"));
    }

    #[test]
    fn a_paid_status_allows_the_sub_features() {
        let access = Access { status: Status::Paid, grants: Grants::built_in() };
        assert!(access.allows("media", "lyrics"));
        assert!(access.allows_page("weather"));
    }

    fn guest_with_activities(actions: &[&str]) -> Access {
        let allowed = actions.iter().map(|action| action.to_string()).collect();
        let table = vec![(Status::Guest, vec![("activity".to_owned(), allowed)])];
        Access { status: Status::Guest, grants: Grants::built_in().merged(&table) }
    }

    #[test]
    fn the_volume_flyout_may_be_hidden_only_while_volume_and_brightness_are_both_allowed() {
        assert!(guest_with_activities(&["volume", "brightness"]).may_hide_volume_flyout());
        assert!(!guest_with_activities(&["volume"]).may_hide_volume_flyout());
        assert!(!guest_with_activities(&["brightness"]).may_hide_volume_flyout());
    }

    #[test]
    fn the_notification_banners_may_be_hidden_only_while_notifications_are_allowed() {
        assert!(Access { status: Status::Paid, grants: Grants::built_in() }.may_hide_notification_banners());
        assert!(!Access::guest().may_hide_notification_banners());
    }
}
