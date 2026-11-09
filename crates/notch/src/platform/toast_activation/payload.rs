//! What a toast's XML says about opening it, and the ways to try in order (no Windows calls).

/// How the toast asks to be activated (`activationType` on `<toast>`); a missing value means foreground.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivationKind {
    Foreground,
    Background,
    Protocol,
}

/// The body-click activation of a toast: its `launch` attribute and its kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToastLaunch {
    pub arguments: String,
    pub kind: ActivationKind,
}

/// One way to open the app, tried in order until one works.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    /// The app's toast activator gets the arguments exactly as a click on Windows' banner gives them.
    Activator(String),
    OpenUri(String),
    /// A packaged app launched with the arguments, which it reads as launch arguments.
    LaunchWithArguments(String),
    /// Starts the app, or brings it forward when it runs, with no target inside it.
    StartApp,
}

/// Reads the `<toast>` element's `launch` and `activationType`; a payload that cannot be read gives no arguments.
pub fn toast_launch(payload: &str) -> ToastLaunch {
    let attributes = toast_tag(payload).map(attributes).unwrap_or_default();
    let value = |name: &str| attributes.iter().find(|(key, _)| key == name).map(|(_, value)| value.clone());
    let kind = match value("activationType").as_deref() {
        Some("protocol") => ActivationKind::Protocol,
        Some("background") => ActivationKind::Background,
        _ => ActivationKind::Foreground,
    };
    ToastLaunch { arguments: value("launch").unwrap_or_default(), kind }
}

/// A protocol toast opens its URI; a background one has nothing to show, so it only starts the app. A toast without
/// arguments (Discord's) skips the activator: it can name no target, and Electron's answers it with success and nothing.
pub fn routes(launch: &ToastLaunch) -> Vec<Route> {
    let arguments = launch.arguments.trim();
    match launch.kind {
        ActivationKind::Protocol if uri_scheme(arguments).is_some() => {
            vec![Route::OpenUri(arguments.to_owned()), Route::StartApp]
        }
        ActivationKind::Protocol | ActivationKind::Background => vec![Route::StartApp],
        ActivationKind::Foreground if arguments.is_empty() => vec![Route::StartApp],
        ActivationKind::Foreground => {
            let mut routes = vec![Route::Activator(arguments.to_owned())];
            if uri_scheme(arguments).is_some() {
                routes.push(Route::OpenUri(arguments.to_owned()));
            }
            routes.push(Route::LaunchWithArguments(arguments.to_owned()));
            routes.push(Route::StartApp);
            routes
        }
    }
}

/// A route that reports success proves it only when another window than the notch's, or the one in front at the
/// click, comes to the front; window handles are compared as integers.
pub fn came_forward(window_at_click: isize, window_now: isize, window_now_is_the_notch: bool) -> bool {
    window_now != 0 && window_now != window_at_click && !window_now_is_the_notch
}

/// The scheme of a URI such as `ms-phone:messages?...`; two letters at least, so a drive letter is no scheme.
pub fn uri_scheme(text: &str) -> Option<&str> {
    let (scheme, rest) = text.split_once(':')?;
    let mut characters = scheme.chars();
    let starts_with_letter = characters.next().is_some_and(|character| character.is_ascii_alphabetic());
    let is_scheme = starts_with_letter
        && scheme.len() >= 2
        && characters.all(|character| character.is_ascii_alphanumeric() || "+-.".contains(character));
    (is_scheme && !rest.is_empty()).then_some(scheme)
}

/// The part of an app identifier after `!`, which names the application inside a package.
pub fn application_id(app_user_model_id: &str) -> Option<&str> {
    app_user_model_id.split_once('!').map(|(_, application)| application)
}

/// The `ToastActivatorCLSID` a package manifest declares for the application with `application_id`.
pub fn manifest_activator(manifest: &str, application_id: &str) -> Option<String> {
    let mut rest = manifest;
    while let Some(start) = rest.find("<Application ") {
        let element = &rest[start..];
        let end = element.find("</Application>").map_or(element.len(), |end| end + "</Application>".len());
        let element = &element[..end];
        let opening = element.split('>').next().unwrap_or_default();
        let is_wanted =
            attributes(opening).iter().any(|(key, value)| key == "Id" && value.eq_ignore_ascii_case(application_id));
        if is_wanted {
            let marker = "ToastActivatorCLSID=";
            let at = element.find(marker)?;
            let tag_start = element[..at].rfind('<')?;
            let tag = element[tag_start..].split('>').next()?;
            return attributes(tag).into_iter().find(|(key, _)| key == "ToastActivatorCLSID").map(|(_, value)| value);
        }
        rest = &rest[start + end..];
    }
    None
}

/// The opening `<toast ...>` tag, without its angle brackets' closing part.
fn toast_tag(payload: &str) -> Option<&str> {
    let mut search = payload;
    loop {
        let start = search.find("<toast")?;
        let after = &search[start + "<toast".len()..];
        if after.starts_with(|character: char| character.is_whitespace() || character == '>' || character == '/') {
            return after.split('>').next();
        }
        search = after;
    }
}

/// `name="value"` pairs of one tag, with single or double quotes and XML escapes decoded.
fn attributes(tag: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let mut rest = tag;
    while let Some(equals) = rest.find('=') {
        let name = rest[..equals].split_whitespace().last().unwrap_or_default().to_owned();
        let after = rest[equals + 1..].trim_start();
        let Some(quote) = after.chars().next().filter(|character| *character == '"' || *character == '\'') else {
            rest = &rest[equals + 1..];
            continue;
        };
        let Some(close) = after[1..].find(quote) else { break };
        found.push((name, unescape(&after[1..1 + close])));
        rest = &after[close + 2..];
    }
    found
}

fn unescape(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('&') {
        result.push_str(&rest[..start]);
        let entity_and_more = &rest[start + 1..];
        let Some(end) = entity_and_more.find(';') else {
            result.push_str(&rest[start..]);
            return result;
        };
        let entity = &entity_and_more[..end];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ => entity
                .strip_prefix("#x")
                .map(|hex| u32::from_str_radix(hex, 16).ok())
                .unwrap_or_else(|| entity.strip_prefix('#').and_then(|decimal| decimal.parse().ok()))
                .and_then(char::from_u32),
        };
        match decoded {
            Some(character) => result.push(character),
            None => result.push_str(&rest[start..start + 1 + end + 1]),
        }
        rest = &entity_and_more[end + 1..];
    }
    result.push_str(rest);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_whatsapp_toast_gives_its_launch_arguments_for_a_foreground_activation() {
        let payload = r#"<toast launch="action=LaunchApp&amp;tag=mayHaveNewMessagesNotification"><visual/></toast>"#;
        let launch = toast_launch(payload);
        assert_eq!(launch.arguments, "action=LaunchApp&tag=mayHaveNewMessagesNotification");
        assert_eq!(launch.kind, ActivationKind::Foreground);
    }

    #[test]
    fn a_protocol_toast_is_read_in_any_attribute_order() {
        let payload =
            r#"<?xml version="1.0"?><toast activationType='protocol' launch='ms-screensketch:edit?a=1&amp;b=2'>"#;
        assert_eq!(
            toast_launch(payload),
            ToastLaunch { arguments: "ms-screensketch:edit?a=1&b=2".to_owned(), kind: ActivationKind::Protocol }
        );
    }

    #[test]
    fn a_toast_without_launch_has_no_arguments() {
        assert_eq!(toast_launch("<toast><visual/></toast>").arguments, "");
        assert_eq!(toast_launch("not xml").arguments, "");
    }

    #[test]
    fn a_toaster_element_is_not_taken_for_the_toast() {
        assert_eq!(toast_launch(r#"<toaster launch="no"/><toast launch="yes">"#).arguments, "yes");
    }

    #[test]
    fn numeric_escapes_are_decoded() {
        assert_eq!(toast_launch(r#"<toast launch="a&#38;b&#x3D;c">"#).arguments, "a&b=c");
    }

    #[test]
    fn a_foreground_toast_goes_to_its_activator_first_and_starts_the_app_last() {
        let launch = ToastLaunch { arguments: "chat=42".to_owned(), kind: ActivationKind::Foreground };
        assert_eq!(
            routes(&launch),
            vec![
                Route::Activator("chat=42".to_owned()),
                Route::LaunchWithArguments("chat=42".to_owned()),
                Route::StartApp
            ]
        );
    }

    #[test]
    fn a_foreground_toast_with_a_uri_tries_the_uri_after_the_activator() {
        let launch =
            ToastLaunch { arguments: "ms-phone:messages?threadid=82".to_owned(), kind: ActivationKind::Foreground };
        assert_eq!(routes(&launch)[1], Route::OpenUri("ms-phone:messages?threadid=82".to_owned()));
    }

    #[test]
    fn a_protocol_toast_opens_its_uri() {
        let launch = ToastLaunch { arguments: "whatsapp://send?phone=1".to_owned(), kind: ActivationKind::Protocol };
        assert_eq!(routes(&launch), vec![Route::OpenUri("whatsapp://send?phone=1".to_owned()), Route::StartApp]);
    }

    #[test]
    fn a_background_toast_only_starts_the_app() {
        let launch = ToastLaunch { arguments: "snooze".to_owned(), kind: ActivationKind::Background };
        assert_eq!(routes(&launch), vec![Route::StartApp]);
    }

    #[test]
    fn a_discord_toast_without_arguments_starts_the_app_instead_of_asking_its_activator() {
        let launch = toast_launch(
            r#"<toast><visual><binding template="ToastGeneric"><text>raul</text></binding></visual></toast>"#,
        );
        assert_eq!(routes(&launch), vec![Route::StartApp]);
    }

    #[test]
    fn every_route_list_ends_by_starting_the_app() {
        let kinds = [ActivationKind::Foreground, ActivationKind::Background, ActivationKind::Protocol];
        for kind in kinds {
            for arguments in ["", "chat=42", "ms-phone:messages?threadid=82"] {
                let launch = ToastLaunch { arguments: arguments.to_owned(), kind };
                assert_eq!(routes(&launch).last(), Some(&Route::StartApp), "{kind:?} with {arguments:?}");
            }
        }
    }

    #[test]
    fn an_app_came_forward_only_when_another_window_than_the_one_at_the_click_is_in_front() {
        assert!(came_forward(10, 20, false));
        assert!(!came_forward(10, 10, false));
        assert!(!came_forward(10, 0, false));
    }

    #[test]
    fn the_notch_itself_in_front_is_not_the_app_coming_forward() {
        assert!(!came_forward(10, 30, true));
    }

    #[test]
    fn query_strings_and_drive_letters_are_not_uris() {
        assert_eq!(uri_scheme("action=LaunchApp&tag=x"), None);
        assert_eq!(uri_scheme(r"C:\Users\file.png"), None);
        assert_eq!(uri_scheme("ms-phone:messages"), Some("ms-phone"));
        assert_eq!(uri_scheme("tg:"), None);
    }

    #[test]
    fn the_application_id_is_the_part_after_the_bang() {
        assert_eq!(application_id("5319275A.WhatsAppDesktop_cv1g1gvanyjgm!App"), Some("App"));
        assert_eq!(application_id("com.squirrel.Discord.Discord"), None);
    }

    const MANIFEST: &str = r#"<Package><Applications>
        <Application Id="Other" Executable="other.exe"><Extensions>
          <desktop:Extension Category="windows.toastNotificationActivation">
            <desktop:ToastNotificationActivation ToastActivatorCLSID="11111111-1111-1111-1111-111111111111" />
          </desktop:Extension></Extensions></Application>
        <Application Id="App" Executable="WhatsApp.Root.exe" EntryPoint="Windows.FullTrustApplication"><Extensions>
          <desktop:Extension Category="windows.toastNotificationActivation">
            <desktop:ToastNotificationActivation ToastActivatorCLSID="bebd6a5e-1b14-4a57-a195-c772d28b6d15" />
          </desktop:Extension></Extensions></Application>
        </Applications></Package>"#;

    #[test]
    fn the_manifest_gives_the_activator_of_the_named_application() {
        assert_eq!(manifest_activator(MANIFEST, "App").as_deref(), Some("bebd6a5e-1b14-4a57-a195-c772d28b6d15"));
        assert_eq!(manifest_activator(MANIFEST, "Other").as_deref(), Some("11111111-1111-1111-1111-111111111111"));
    }

    #[test]
    fn an_application_without_an_activator_gives_none() {
        let manifest = r#"<Application Id="App" Executable="a.exe"><Extensions/></Application>"#;
        assert_eq!(manifest_activator(manifest, "App"), None);
        assert_eq!(manifest_activator(MANIFEST, "Missing"), None);
    }
}
