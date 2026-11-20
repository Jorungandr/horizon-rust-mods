//! Time Formatter: rewrite Discord-style `<t:...>` tags in incoming chat.
//! Port of m3z0id/TimeFormatter (MIT) to the official Horizon SDK.
mod local_time;
mod runtime;

use mod_api::text::TextContent;
use mod_api::{ClientChatReceivedEvent, ClientHost, Flow, Hooks, Text};
use runtime::format_literal;

#[derive(Default)]
struct Mod;

/// Rebuild literal spans that contain a valid tag, preserving each component's
/// style and children. Chat lines arrive as `chat.type.text`, whose text is the
/// `Translatable` arguments rather than a child, so those are rewritten too.
fn rewrite(node: Text) -> Text {
    let mut node = node;
    let children = std::mem::take(&mut node.children);
    node.children = children.into_iter().map(rewrite).collect();

    let content = std::mem::replace(&mut node.content, TextContent::Literal(String::new()));
    node.content = match content {
        TextContent::Literal(text) => TextContent::Literal(match format_literal(&text) {
            Some(replaced) => replaced,
            None => text,
        }),
        TextContent::Translatable {
            key,
            fallback,
            arguments,
        } => TextContent::Translatable {
            key,
            fallback,
            arguments: arguments.into_iter().map(rewrite).collect(),
        },
        TextContent::Selector { pattern, separator } => TextContent::Selector {
            pattern,
            separator: separator.map(|inner| Box::new(rewrite(*inner))),
        },
        other => other,
    };
    node
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn client_chat_received(
        &self,
        _host: &mut dyn ClientHost,
        event: &mut ClientChatReceivedEvent<'_>,
    ) -> Flow {
        if event.overlay {
            return Flow::Continue;
        }
        let original = std::mem::take(&mut *event.message);
        *event.message = rewrite(original);
        Flow::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_translatable_arguments() {
        let mut message = Text::literal("");
        message.content = TextContent::Translatable {
            key: "chat.type.text".to_owned(),
            fallback: None,
            arguments: vec![
                Text::literal("Steve"),
                Text::literal("meet at <t:1704067200:t>"),
            ],
        };

        let out = rewrite(message);
        let TextContent::Translatable { arguments, .. } = &out.content else {
            panic!("chat line lost its translation key");
        };
        let TextContent::Literal(text) = &arguments[1].content else {
            panic!("argument is not literal");
        };
        assert!(!text.contains("<t:"), "tag left unrewritten: {text}");
        assert!(text.starts_with("meet at "), "prefix lost: {text}");
    }

    #[test]
    fn leaves_tags_alone_inside_escaped_literal() {
        let out = rewrite(Text::literal("<<t:1704067200:t>"));
        let TextContent::Literal(text) = &out.content else {
            panic!("literal lost");
        };
        assert_eq!(text, "<<t:1704067200:t>");
    }
}
