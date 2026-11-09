//! The quick reply a toast offers: a text `<input>` and the action button tied to it by `hint-inputId`.

use super::payload::attributes;

/// What the app needs to receive a reply: the action's arguments and the text under the input's id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplyAction {
    pub input_id: String,
    pub arguments: String,
    pub placeholder: String,
}

/// The reply a toast's XML offers, or `None` when it has no text input with an action tied to it.
pub fn reply_action(payload: &str) -> Option<ReplyAction> {
    let inputs = tags(payload, "input");
    let actions = tags(payload, "action");
    inputs.iter().filter(|input| value(input, "type").is_none_or(|kind| kind.eq_ignore_ascii_case("text"))).find_map(
        |input| {
            let input_id = value(input, "id")?;
            let action = actions.iter().find(|action| value(action, "hint-inputId") == Some(input_id))?;
            Some(ReplyAction {
                input_id: input_id.to_owned(),
                arguments: value(action, "arguments").unwrap_or_default().to_owned(),
                placeholder: value(input, "placeHolderContent").unwrap_or_default().to_owned(),
            })
        },
    )
}

type Attributes = Vec<(String, String)>;

fn value<'a>(attributes: &'a Attributes, name: &str) -> Option<&'a str> {
    attributes.iter().find(|(key, _)| key == name).map(|(_, value)| value.as_str())
}

/// The attributes of every `<name ...>` element in the XML.
fn tags(payload: &str, name: &str) -> Vec<Attributes> {
    let opening = format!("<{name}");
    let mut found = Vec::new();
    let mut rest = payload;
    while let Some(start) = rest.find(&opening) {
        let after = &rest[start + opening.len()..];
        let is_the_element = after.starts_with(|character: char| character.is_whitespace() || character == '/');
        // Quoted values hold `>` rarely and never in a toast's attributes the notch reads, so the tag ends at the first.
        let end = after.find('>').unwrap_or(after.len());
        if is_the_element {
            found.push(attributes(&after[..end]));
        }
        rest = &after[end..];
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    const WHATSAPP: &str = r#"<toast launch="a"><actions><input id="quickReplyInputBoxId" type="text" placeHolderContent="Type a reply" /><action content="Send" arguments="action=Click&amp;key=msg%3A1" activationType="background" hint-inputId="quickReplyInputBoxId" /><action content="Mute" arguments="x=1" placement="contextMenu" /></actions></toast>"#;

    #[test]
    fn a_toast_with_a_text_input_and_its_send_action_offers_a_reply() {
        assert_eq!(
            reply_action(WHATSAPP),
            Some(ReplyAction {
                input_id: "quickReplyInputBoxId".to_owned(),
                arguments: "action=Click&key=msg%3A1".to_owned(),
                placeholder: "Type a reply".to_owned(),
            })
        );
    }

    #[test]
    fn a_toast_with_only_buttons_offers_no_reply() {
        let toast = r#"<toast><actions><action content="Snooze" arguments="a=1" activationType="foreground"/></actions></toast>"#;
        assert_eq!(reply_action(toast), None);
    }

    #[test]
    fn an_input_without_an_action_tied_to_it_offers_no_reply() {
        let toast = r#"<toast><actions><input id="reply" type="text"/><action content="Go" arguments="a=1"/></actions></toast>"#;
        assert_eq!(reply_action(toast), None);
    }

    #[test]
    fn a_selection_input_is_not_a_reply() {
        let toast = r#"<toast><actions><input id="pick" type="selection"/><action content="Go" arguments="a" hint-inputId="pick"/></actions></toast>"#;
        assert_eq!(reply_action(toast), None);
    }

    #[test]
    fn an_inputs_tag_is_not_taken_for_an_input() {
        assert!(tags("<inputs id=\"x\"/>", "input").is_empty());
    }
}
