//! A chat keyword highlighter: colour chosen words in incoming chat.
//!
//! The matching and segmentation below are pure, so they can be tested
//! without a game. The engine adapter is appended at injection time (see
//! `tools/inject_all_ports.py`).

pub const ID: &str = "chathighlight";
pub const NAME: &str = "Chat Highlight";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Highlights configured keywords in chat with a colour of your choice.";

const KEY_ENABLE: &str = "enable_mod";
const KEY_KEYWORDS: &str = "keywords";
const KEY_COLOR: &str = "color";
const KEY_MATCH_CASE: &str = "match_case";

/// Keyword packs the settings button cycles. Anyone can also set an arbitrary
/// comma-separated list by editing the mod's `keywords` value in the game's
/// config directly; the button is just the quick, in-game way.
pub const KEYWORD_PACKS: [&str; 4] = [
    "server, look here, important",
    "hello, hi, hey",
    "help, question, answer",
    "warning, danger, alert",
];

/// Preset colours the settings button cycles, as `(label, name)` pairs.
/// `name` is resolved by the engine's `named_color`.
pub const COLORS: [(&str, &str); 6] = [
    ("Gold", "gold"),
    ("Red", "red"),
    ("Aqua", "aqua"),
    ("Green", "green"),
    ("Yellow", "yellow"),
    ("Light Purple", "light_purple"),
];

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub enable_mod: bool,
    /// Comma-separated keywords.
    pub keywords: String,
    /// A name from `COLORS` (index kept separately for the cycling button).
    pub color: usize,
    pub match_case: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enable_mod: true,
            keywords: "server, look here, important".to_owned(),
            color: 0,
            match_case: false,
        }
    }
}

impl Settings {
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let defaults = Self::default();
        let on_off = |key: &str, fallback: bool| match lookup(key).as_deref() {
            Some("on") | Some("true") => true,
            Some("off") | Some("false") => false,
            _ => fallback,
        };
        let color = lookup(KEY_COLOR)
            .and_then(|v| v.trim().parse::<usize>().ok())
            .filter(|i| *i < COLORS.len())
            .unwrap_or(defaults.color);
        Self {
            enable_mod: on_off(KEY_ENABLE, defaults.enable_mod),
            keywords: lookup(KEY_KEYWORDS).unwrap_or(defaults.keywords),
            match_case: on_off(KEY_MATCH_CASE, defaults.match_case),
            color,
        }
    }

    pub fn stored(&self) -> Vec<(&'static str, String)> {
        vec![
            (KEY_ENABLE, on_off(self.enable_mod).to_owned()),
            (KEY_KEYWORDS, self.keywords.clone()),
            (KEY_COLOR, self.color.to_string()),
            (KEY_MATCH_CASE, on_off(self.match_case).to_owned()),
        ]
    }

    pub fn labels(&self) -> Vec<String> {
        vec![
            format!("Highlight: {}", on_off(self.enable_mod).to_uppercase()),
            format!("Keywords: {}", self.keywords),
            format!("Colour: {}", COLORS[self.color.min(COLORS.len() - 1)].0),
            format!("Match case: {}", on_off(self.match_case).to_uppercase()),
        ]
    }

    pub fn click(&mut self, button: usize) {
        match button {
            0 => self.enable_mod = !self.enable_mod,
            1 => {
                let current = KEYWORD_PACKS
                    .iter()
                    .position(|p| *p == self.keywords)
                    .unwrap_or(0);
                self.keywords = KEYWORD_PACKS[(current + 1) % KEYWORD_PACKS.len()].to_owned();
            }
            2 => self.color = (self.color + 1) % COLORS.len(),
            3 => self.match_case = !self.match_case,
            _ => {}
        }
    }

    /// The colour name to resolve through `named_color`.
    pub fn color_name(&self) -> &'static str {
        COLORS[self.color.min(COLORS.len() - 1)].1
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

/// Parse the comma-separated keyword list, trimming each entry and dropping
/// empty ones. Comparable pieces are produced by [`normalise`].
pub fn keywords(settings: &Settings) -> Vec<String> {
    settings
        .keywords
        .split(',')
        .map(|k| k.trim().to_owned())
        .filter(|k| !k.is_empty())
        .collect()
}

fn normalise(s: &str, match_case: bool) -> String {
    if match_case {
        s.to_owned()
    } else {
        s.to_lowercase()
    }
}

/// A run of text and whether it is a highlighted match.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    pub text: String,
    pub hit: bool,
}

/// Split `line` around every occurrence of any keyword, longest match first,
/// so overlapping keywords (e.g. "look" and "look here") still win the more
/// specific one. Non-matching runs are kept verbatim.
pub fn highlight(line: &str, words: &[String], match_case: bool) -> Vec<Piece> {
    if words.is_empty() {
        return vec![Piece { text: line.to_owned(), hit: false }];
    }
    // Longest first so a more specific keyword takes priority.
    let mut sorted: Vec<&String> = words.iter().collect();
    sorted.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()));
    let needles: Vec<(String, usize)> = sorted
        .iter()
        .map(|w| (normalise(w, match_case), w.chars().count()))
        .filter(|(n, _)| !n.is_empty())
        .collect();
    if needles.is_empty() {
        return vec![Piece { text: line.to_owned(), hit: false }];
    }

    let haystack = normalise(line, match_case);
    let lower: Vec<char> = haystack.chars().collect();
    let chars: Vec<char> = line.chars().collect();
    let mut out = Vec::new();
    let mut cursor = 0usize; // index into `chars`

    while cursor < chars.len() {
        let mut best: Option<(usize, usize)> = None; // (start, byte-char len)
        for (needle, width) in &needles {
            let target: Vec<char> = needle.chars().collect();
            if target.is_empty() || cursor + target.len() > lower.len() {
                continue;
            }
            if lower[cursor..cursor + target.len()] == target[..] {
                if best.map_or(true, |(_, bw)| *width > bw) {
                    best = Some((cursor, *width));
                }
            }
        }
        match best {
            Some((_start, width)) if width > 0 => {
                let end = (cursor + width).min(chars.len());
                out.push(Piece {
                    text: chars[cursor..end].iter().collect(),
                    hit: true,
                });
                cursor = end;
            }
            _ => {
                // Grow the plain run until the next match begins.
                let mut end = cursor + 1;
                'outer: while end < chars.len() {
                    for (needle, _) in &needles {
                        let target: Vec<char> = needle.chars().collect();
                        if target.is_empty() || end + target.len() > lower.len() {
                            continue;
                        }
                        if lower[end..end + target.len()] == target[..] {
                            break 'outer;
                        }
                    }
                    end += 1;
                }
                out.push(Piece {
                    text: chars[cursor..end].iter().collect(),
                    hit: false,
                });
                cursor = end;
            }
        }
    }
    out
}

/// True when the line contains at least one keyword.
pub fn has_match(line: &str, words: &[String], match_case: bool) -> bool {
    highlight(line, words, match_case).iter().any(|p| p.hit)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(ws: &[&str]) -> Vec<String> {
        ws.iter().map(|w| w.to_string()).collect()
    }

    #[test]
    fn keywords_are_trimmed_and_empty_dropped() {
        let s = Settings { keywords: " foo ,  , bar ".to_owned(), ..Settings::default() };
        assert_eq!(keywords(&s), words(&["foo", "bar"]));
    }

    #[test]
    fn highlights_a_middle_keyword() {
        let pieces = highlight("meet me at the SERVER now", &words(&["server"]), false);
        assert_eq!(pieces[0], Piece { text: "meet me at the ".to_owned(), hit: false });
        assert_eq!(pieces[1], Piece { text: "SERVER".to_owned(), hit: true });
        assert_eq!(pieces[2], Piece { text: " now".to_owned(), hit: false });
    }

    #[test]
    fn case_insensitive_by_default() {
        assert!(has_match("An IMPORTANT note", &words(&["important"]), false));
        assert!(!has_match("An IMPORTANT note", &words(&["important"]), true));
    }

    #[test]
    fn multiple_and_repeated_keywords() {
        let pieces = highlight("red and gold and red", &words(&["red", "gold"]), false);
        let hits: Vec<_> = pieces.iter().filter(|p| p.hit).map(|p| p.text.as_str()).collect();
        assert_eq!(hits, vec!["red", "gold", "red"]);
    }

    #[test]
    fn longest_keyword_wins_overlap() {
        let pieces = highlight("look here now", &words(&["look", "look here"]), false);
        assert_eq!(pieces[0], Piece { text: "look here".to_owned(), hit: true });
    }

    #[test]
    fn adjacent_keywords_produce_two_hits() {
        let pieces = highlight("ab", &words(&["a", "b"]), false);
        assert_eq!(pieces.len(), 2);
        assert!(pieces.iter().all(|p| p.hit));
    }

    #[test]
    fn no_keywords_keeps_line_verbatim() {
        let pieces = highlight("hello", &[], false);
        assert_eq!(pieces, vec![Piece { text: "hello".to_owned(), hit: false }]);
    }

    #[test]
    fn empty_line_and_missing_keyword() {
        assert!(highlight("", &words(&["x"]), false).is_empty());
        assert_eq!(highlight("hello", &words(&["zzz"]), false).len(), 1);
        assert!(!highlight("hello", &words(&["zzz"]), false)[0].hit);
    }

    #[test]
    fn unicode_keywords_are_char_safe() {
        let pieces = highlight("こんにちは世界", &words(&["世界"]), false);
        assert_eq!(pieces[0].text, "こんにちは");
        assert_eq!(pieces[1].text, "世界");
        assert!(pieces[1].hit);
    }

    #[test]
    fn settings_round_trip_and_clicks() {
        let mut s = Settings::default();
        s.keywords = "one,two".to_owned();
        s.color = 3;
        s.match_case = true;
        let stored = s.stored();
        let read = Settings::from_lookup(|k| {
            stored.iter().find(|(key, _)| *key == k).map(|(_, v)| v.clone())
        });
        assert_eq!(read, s);
        let n = s.labels().len();
        for i in 0..n {
            let before = s.clone();
            s.click(i);
            assert_ne!(before, s, "button {i} did nothing");
        }
    }

    #[test]
    fn color_cycles_within_range() {
        let mut s = Settings::default();
        for _ in 0..COLORS.len() * 2 {
            assert!(s.color < COLORS.len());
            s.click(2);
        }
        assert!(s.color < COLORS.len());
    }
}
