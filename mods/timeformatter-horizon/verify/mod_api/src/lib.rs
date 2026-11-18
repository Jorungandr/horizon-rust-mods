// Minimal stub of the ENGINE-side Text API that horizon-mixins helpers see
// when injected into `Game::native_client_chat_received` (Horizon 9ee13cd0).
// The real type is `horizon_client::text::Text` in horizon-client-core/src/text.rs:
//   Text { pub spans: Vec<Span> }, Span { text: String, style: Style },
//   Style { color: Option<u32>, hover: Option<Box<Text>>, .. }
// with .plain()/.literal()/.colored()/.push()/.append() and ::named_color().
//
// This is NOT the mod_api component tree (that was the old 85b5bcca API).
// It exists so `cargo test` can compile the REAL chat.rs helper, which is
// `include!`d byte-for-byte into the test module. The authoritative check is
// `native.py check` against the real Horizon source; this stub is for fast
// local logic tests only.

// The helper writes `use horizon_client::text::...` / refers to `Text` directly.
// Re-export this stub under that path so the real helper source compiles as-is.
pub mod horizon_client {
    pub mod text {
        pub use crate::text::*;
    }
}

pub mod text {
    /// Cargo colour constants (packed ARGB), a subset of the real palette.
    pub const GRAY: u32 = 0xFF_AAAAAA;
    pub const RED: u32 = 0xFF_FF5555;
    pub const GREEN: u32 = 0xFF_55FF55;
    pub const WHITE: u32 = 0xFF_FFFFFF;

    pub fn named_color(name: &str) -> Option<u32> {
        match name {
            "black" => Some(0xFF_000000),
            "white" => Some(WHITE),
            "gray" | "grey" => Some(GRAY),
            "dark_gray" => Some(0xFF_555555),
            "red" => Some(RED),
            "dark_red" => Some(0xFF_AA0000),
            "green" => Some(GREEN),
            "dark_green" => Some(0xFF_00AA00),
            "blue" => Some(0xFF_5555FF),
            "dark_blue" => Some(0xFF_0000AA),
            "yellow" => Some(0xFF_FFFF55),
            "gold" => Some(0xFF_FFAA00),
            "aqua" => Some(0xFF_55FFFF),
            "light_purple" => Some(0xFF_FF55FF),
            _ => None,
        }
    }

    #[derive(Clone, Debug, PartialEq, Default)]
    pub struct Style {
        pub color: Option<u32>,
        pub hover: Option<Box<Text>>,
    }

    impl Style {
        pub fn color(c: u32) -> Self {
            Style { color: Some(c), hover: None }
        }
    }

    #[derive(Clone, Debug, PartialEq, Default)]
    pub struct Span {
        pub text: String,
        pub style: Style,
    }

    #[derive(Clone, Debug, PartialEq, Default)]
    pub struct Text {
        pub spans: Vec<Span>,
    }

    impl From<&str> for Text {
        fn from(s: &str) -> Self { Text::literal(s) }
    }
    impl From<String> for Text {
        fn from(s: String) -> Self { Text::literal(s) }
    }

    impl Text {
        pub fn empty() -> Self { Text::default() }

        pub fn literal(s: impl Into<String>) -> Self {
            Text {
                spans: vec![Span { text: s.into(), style: Style::default() }],
            }
        }

        pub fn styled(s: impl Into<String>, style: Style) -> Self {
            Text {
                spans: vec![Span { text: s.into(), style }],
            }
        }

        pub fn colored(s: impl Into<String>, color: u32) -> Self {
            Self::styled(s, Style::color(color))
        }

        pub fn push(&mut self, other: impl Into<Text>) {
            let other = other.into();
            self.spans.extend(other.spans);
        }

        pub fn append(mut self, other: impl Into<Text>) -> Self {
            self.push(other);
            self
        }

        /// Concatenate all span texts (mirrors the engine's .plain()).
        pub fn plain(&self) -> String {
            let mut out = String::new();
            for span in &self.spans {
                out.push_str(&span.text);
            }
            out
        }

        pub fn is_empty(&self) -> bool {
            self.plain().is_empty()
        }
    }

    /// Test helper (NOT part of the real API): extract plain text.
    pub fn plain(text: &Text) -> String {
        text.plain()
    }
}
