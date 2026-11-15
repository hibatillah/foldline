use crate::catalog::{CATALOG, is_known};
use crate::status::Status;
use std::collections::HashMap;

/// Status keys of a server table are the four stored statuses; `Launch` entries are ignored.
pub type ServerGrants = Vec<(Status, Vec<(String, Vec<String>)>)>;

#[derive(Debug, PartialEq, Eq)]
pub enum GrantError {
    GuestHasNoPage,
    ActionWithoutBase { resource: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grants {
    actions: HashMap<(Status, String), Vec<String>>,
}

impl Grants {
    pub fn built_in() -> Grants {
        let mut actions = HashMap::new();
        let basic_activities =
            ["volume", "brightness", "battery", "caps_lock", "microphone", "camera", "downloads", "account_notice"];
        for status in [Status::Guest, Status::Member] {
            insert(&mut actions, status, "media", &["page"]);
            insert(&mut actions, status, "calendar", &["page"]);
            insert(&mut actions, status, "shelf", &["page"]);
            insert(&mut actions, status, "activity", &basic_activities);
        }
        for status in [Status::Trial, Status::Paid] {
            for entry in CATALOG {
                insert(&mut actions, status, entry.name, entry.actions);
            }
        }
        Grants { actions }
    }

    /// `Launch` reads the `Paid` grants.
    pub fn allows(&self, status: Status, resource: &str, action: &str) -> bool {
        let status = if status == Status::Launch { Status::Paid } else { status };
        self.actions
            .get(&(status, resource.to_string()))
            .is_some_and(|granted| granted.iter().any(|name| name == action))
    }

    /// A resource the server names replaces the built-in actions; unnamed and unknown ones change nothing.
    /// A list naming only actions this build does not know keeps the built-in ones; an empty list removes them.
    pub fn merged(&self, server: &ServerGrants) -> Grants {
        let mut merged = self.clone();
        for (status, resources) in server {
            if *status == Status::Launch {
                continue;
            }
            for (resource, actions) in resources {
                if !CATALOG.iter().any(|entry| entry.name == resource) {
                    continue;
                }
                let known: Vec<String> = actions.iter().filter(|action| is_known(resource, action)).cloned().collect();
                if known.is_empty() && !actions.is_empty() {
                    continue;
                }
                merged.actions.insert((*status, resource.clone()), known);
            }
        }
        merged
    }
}

fn insert(actions: &mut HashMap<(Status, String), Vec<String>>, status: Status, resource: &str, granted: &[&str]) {
    let granted = granted.iter().map(|action| action.to_string()).collect();
    actions.insert((status, resource.to_string()), granted);
}

/// A table is refused when an action lacks its page or when it leaves guests without any page.
pub fn validate(server: &ServerGrants) -> Result<(), GrantError> {
    for (_, resources) in server {
        for (resource, actions) in resources {
            let has_base = CATALOG.iter().any(|entry| entry.name == resource && entry.has_base);
            let names_action = actions.iter().any(|action| is_known(resource, action));
            if has_base && names_action && !actions.iter().any(|action| action == "page") {
                return Err(GrantError::ActionWithoutBase { resource: resource.clone() });
            }
        }
    }
    let merged = Grants::built_in().merged(server);
    let guest_has_page = CATALOG.iter().any(|entry| entry.has_base && merged.allows(Status::Guest, entry.name, "page"));
    if guest_has_page { Ok(()) } else { Err(GrantError::GuestHasNoPage) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(status: Status, resource: &str, actions: &[&str]) -> ServerGrants {
        let actions = actions.iter().map(|action| action.to_string()).collect();
        vec![(status, vec![(resource.to_string(), actions)])]
    }

    #[test]
    fn guests_get_media_calendar_shelf_and_eight_activities() {
        let grants = Grants::built_in();
        for (resource, action) in [
            ("media", "page"),
            ("calendar", "page"),
            ("shelf", "page"),
            ("activity", "volume"),
            ("activity", "brightness"),
            ("activity", "battery"),
            ("activity", "caps_lock"),
            ("activity", "microphone"),
            ("activity", "camera"),
            ("activity", "downloads"),
            ("activity", "account_notice"),
        ] {
            assert!(grants.allows(Status::Guest, resource, action), "{resource}/{action}");
        }
        assert!(!grants.allows(Status::Guest, "weather", "page"));
        assert!(!grants.allows(Status::Guest, "media", "lyrics"));
        assert!(!grants.allows(Status::Guest, "activity", "notifications"));
    }

    #[test]
    fn the_launch_uses_the_paid_grants() {
        let grants = Grants::built_in();
        assert!(grants.allows(Status::Launch, "weather", "rain_alert"));
        assert!(grants.allows(Status::Launch, "calendar", "google"));
    }

    #[test]
    fn a_server_table_replaces_only_the_resources_it_names() {
        let base = Grants::built_in();
        let replaced = base.merged(&table(Status::Guest, "weather", &["page"]));
        assert!(replaced.allows(Status::Guest, "weather", "page"));
        assert!(replaced.allows(Status::Guest, "media", "page"));
        let emptied = base.merged(&table(Status::Guest, "media", &[]));
        assert!(!emptied.allows(Status::Guest, "media", "page"));
    }

    #[test]
    fn a_server_table_without_a_guest_page_is_refused() {
        let mut server = table(Status::Guest, "media", &[]);
        server.push((Status::Guest, vec![("calendar".to_string(), Vec::new()), ("shelf".to_string(), Vec::new())]));
        assert_eq!(validate(&server), Err(GrantError::GuestHasNoPage));
        assert_eq!(validate(&table(Status::Guest, "weather", &["page"])), Ok(()));
    }

    #[test]
    fn a_server_table_without_a_guest_entry_is_accepted() {
        assert_eq!(validate(&table(Status::Member, "weather", &["page"])), Ok(()));
        assert_eq!(validate(&Vec::new()), Ok(()));
    }

    #[test]
    fn an_action_without_its_page_is_refused() {
        let server = table(Status::Paid, "weather", &["rain_alert"]);
        assert_eq!(validate(&server), Err(GrantError::ActionWithoutBase { resource: "weather".to_string() }));
    }

    #[test]
    fn unknown_resources_and_actions_in_a_server_table_are_ignored() {
        let base = Grants::built_in();
        let mut server = table(Status::Guest, "media", &["page", "karaoke"]);
        server.push((Status::Guest, vec![("ghost".to_string(), vec!["page".to_string()])]));
        let merged = base.merged(&server);
        assert!(merged.allows(Status::Guest, "media", "page"));
        assert!(!merged.allows(Status::Guest, "media", "karaoke"));
        assert!(!merged.allows(Status::Guest, "ghost", "page"));
        assert_eq!(validate(&server), Ok(()));
    }

    #[test]
    fn a_resource_named_with_only_unknown_actions_keeps_its_built_in_grants() {
        let server = table(Status::Paid, "media", &["karaoke"]);
        let merged = Grants::built_in().merged(&server);
        assert!(merged.allows(Status::Paid, "media", "page"));
        assert!(merged.allows(Status::Paid, "media", "lyrics"));
        assert_eq!(validate(&server), Ok(()));
    }

    #[test]
    fn an_empty_server_list_still_removes_the_resource() {
        let merged = Grants::built_in().merged(&table(Status::Paid, "media", &[]));
        assert!(!merged.allows(Status::Paid, "media", "page"));
    }
}
