//! Chat Highlight: colour configured keywords in incoming chat.
//! Native mod for the official Horizon SDK (api 0.2).
mod logic;

use logic::{COLORS, KEYWORD_PACKS, Piece, Settings, highlight, keywords};
use mod_api::text::TextContent;
use mod_api::{
    ClientChatReceivedEvent, ClientHost, Flow, Hooks, Registrar, RegistrationContext,
    SettingSpec, Text,
};

#[derive(Default)]
struct Mod;

/// Split literal components around keyword hits, keeping the original style
/// on every piece and overriding only the colour of matched pieces. Chat lines
/// carry their text in `Translatable` arguments, so those are walked as well.
fn recolor(node: Text, words: &[String], match_case: bool, color: &str) -> Text {
    let mut node = node;
    let children = std::mem::take(&mut node.children);
    node.children = children
        .into_iter()
        .map(|child| recolor(child, words, match_case, color))
        .collect();

    let content = std::mem::replace(&mut node.content, TextContent::Literal(String::new()));
    match content {
        TextContent::Literal(text) => {
            let pieces: Vec<Piece> = highlight(&text, words, match_case);
            if !pieces.iter().any(|piece| piece.hit) {
                node.content = TextContent::Literal(text);
                return node;
            }

            let base_style = node.style.clone();
            let children = std::mem::take(&mut node.children);
            let mut parts = Vec::with_capacity(pieces.len());
            for piece in pieces {
                let mut part = Text::literal(piece.text);
                part.style = base_style.clone();
                if piece.hit {
                    part.style.color = Some(color.to_owned());
                }
                parts.push(part);
            }
            if let Some(last) = parts.last_mut() {
                last.children = children;
            }
            let mut root = Text::literal("");
            root.children = parts;
            root
        }
        TextContent::Translatable {
            key,
            fallback,
            arguments,
        } => {
            node.content = TextContent::Translatable {
                key,
                fallback,
                arguments: arguments
                    .into_iter()
                    .map(|arg| recolor(arg, words, match_case, color))
                    .collect(),
            };
            node
        }
        TextContent::Selector { pattern, separator } => {
            node.content = TextContent::Selector {
                pattern,
                separator: separator.map(|inner| Box::new(recolor(*inner, words, match_case, color))),
            };
            node
        }
        other => {
            node.content = other;
            node
        }
    }
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        let _ = r.register_setting(SettingSpec::toggle(
            "enable_mod",
            "chathighlight.setting.enable",
            true,
        ));
        let _ = r.register_setting(SettingSpec::choice(
            "keywords",
            "chathighlight.setting.keywords",
            ["chathighlight.pack.server", "chathighlight.pack.greetings",
             "chathighlight.pack.help", "chathighlight.pack.warning"],
            0,
        ));
        let _ = r.register_setting(SettingSpec::choice(
            "color",
            "chathighlight.setting.color",
            ["chathighlight.color.gold", "chathighlight.color.red",
             "chathighlight.color.aqua", "chathighlight.color.green",
             "chathighlight.color.yellow", "chathighlight.color.light_purple"],
            0,
        ));
        let _ = r.register_setting(SettingSpec::toggle(
            "match_case",
            "chathighlight.setting.match_case",
            false,
        ));
    }

    fn client_chat_received(
        &self,
        host: &mut dyn ClientHost,
        event: &mut ClientChatReceivedEvent<'_>,
    ) -> Flow {
        if event.overlay {
            return Flow::Continue;
        }
        let pack = (host.setting_choice("keywords").unwrap_or(0) as usize)
            .min(KEYWORD_PACKS.len() - 1);
        let color_index =
            (host.setting_choice("color").unwrap_or(0) as usize).min(COLORS.len() - 1);
        let settings = Settings {
            enable_mod: host.setting_bool("enable_mod").unwrap_or(true),
            keywords: KEYWORD_PACKS[pack].to_owned(),
            color: color_index,
            match_case: host.setting_bool("match_case").unwrap_or(false),
        };
        if !settings.enable_mod {
            return Flow::Continue;
        }
        let words = keywords(&settings);
        if words.is_empty() {
            return Flow::Continue;
        }
        let color = settings.color_name();
        let original = std::mem::take(&mut *event.message);
        *event.message = recolor(original, &words, settings.match_case, color);
        Flow::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recolor_keeps_plain_text_and_children() {
        let original = Text::literal("")
            .append(Text::literal("hello "))
            .append(Text::literal("world"));
        let words = vec!["server".to_owned()];
        let out = recolor(original, &words, false, "gold");
        assert_eq!(out.children.len(), 2);
    }

    #[test]
    fn recolor_reaches_translatable_arguments() {
        let mut message = Text::literal("");
        message.content = TextContent::Translatable {
            key: "chat.type.text".to_owned(),
            fallback: None,
            arguments: vec![
                Text::literal("Steve"),
                Text::literal("join the SERVER now"),
            ],
        };
        let words = vec!["server".to_owned()];
        let out = recolor(message, &words, false, "gold");
        let TextContent::Translatable { arguments, .. } = &out.content else {
            panic!("chat line lost its translation key");
        };
        let rebuilt = &arguments[1];
        let hit = rebuilt
            .children
            .iter()
            .any(|part| part.style.color.as_deref() == Some("gold")
                && matches!(&part.content, TextContent::Literal(t) if t.eq_ignore_ascii_case("server")));
        assert!(hit, "keyword inside the argument was not highlighted");
    }

    #[test]
    fn recolor_splits_hits_and_keeps_style() {
        let original = Text::literal("meet SERVER");
        let words = vec!["server".to_owned()];
        let out = recolor(original, &words, false, "gold");
        let pieces = &out.children;
        assert_eq!(pieces.len(), 2);
        assert_eq!(pieces[0].style.color, None);
        assert_eq!(pieces[1].style.color.as_deref(), Some("gold"));
    }
}
