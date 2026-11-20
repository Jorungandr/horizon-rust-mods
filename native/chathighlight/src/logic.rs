//! Pure logic for Chat Highlight: keyword parsing and text splitting.
//! The Unicode-safe normalise keeps a strict 1:1 char mapping with the source.

pub const ID: &str = "chathighlight";

pub const KEYWORD_PACKS: [&str; 4] = [
    "server, look here, important",
    "hello, hi, hey",
    "help, question, answer",
    "warning, danger, alert",
];

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
    pub keywords: String,
    pub color: usize,
    pub match_case: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enable_mod: true,
            keywords: String::from("server, look here, important"),
            color: 0,
            match_case: false,
        }
    }
}

impl Settings {
    pub fn color_name(&self) -> &'static str {
        COLORS[self.color.min(COLORS.len() - 1)].1
    }
}

pub fn keywords(settings: &Settings) -> Vec<String> {
    settings
        .keywords
        .split(',')
        .map(|k| k.trim().to_owned())
        .filter(|k| !k.is_empty())
        .collect()
}

/// Lowercase for matching while keeping a strict 1:1 char mapping with the
/// source. Per-character lowercasing that expands to multiple chars is left
/// as-is so char indices used for slicing stay valid.
fn normalise(s: &str, match_case: bool) -> String {
    if match_case {
        return s.to_owned();
    }
    s.chars()
        .map(|c| {
            let mut lower = c.to_lowercase();
            let first = lower.next().unwrap_or(c);
            if lower.next().is_some() { c } else { first }
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    pub text: String,
    pub hit: bool,
}

pub fn highlight(line: &str, words: &[String], match_case: bool) -> Vec<Piece> {
    if words.is_empty() {
        return vec![Piece { text: line.to_owned(), hit: false }];
    }
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
    let mut cursor = 0usize;

    while cursor < chars.len() {
        let mut best: Option<(usize, usize)> = None;
        for (needle, width) in &needles {
            let target: Vec<char> = needle.chars().collect();
            if target.is_empty() || cursor + target.len() > lower.len() {
                continue;
            }
            if lower[cursor..cursor + target.len()] == target[..]
                && best.map_or(true, |(_, bw)| *width > bw)
            {
                best = Some((cursor, *width));
            }
        }
        match best {
            Some((_start, width)) if width > 0 => {
                let end = (cursor + width).min(chars.len());
                out.push(Piece { text: chars[cursor..end].iter().collect(), hit: true });
                cursor = end;
            }
            _ => {
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
                out.push(Piece { text: chars[cursor..end].iter().collect(), hit: false });
                cursor = end;
            }
        }
    }
    out
}

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
        let s = Settings { keywords: String::from(" foo ,  , bar "), ..Settings::default() };
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
    fn longest_keyword_wins_overlap() {
        let pieces = highlight("look here now", &words(&["look", "look here"]), false);
        assert_eq!(pieces[0], Piece { text: "look here".to_owned(), hit: true });
    }

    #[test]
    fn unicode_keywords_are_char_safe() {
        let pieces = highlight("こんにちは世界", &words(&["世界"]), false);
        assert_eq!(pieces[0].text, "こんにちは");
        assert_eq!(pieces[1].text, "世界");
        assert!(pieces[1].hit);
    }

    #[test]
    fn case_folding_that_expands_chars_keeps_indices_aligned() {
        let pieces = highlight("İSTANBUL", &words(&["istanbul"]), false);
        assert_eq!(pieces.iter().map(|p| p.text.as_str()).collect::<String>(), "İSTANBUL");
        let pieces = highlight("İ go SERVER now", &words(&["server"]), false);
        assert_eq!(pieces[1], Piece { text: "SERVER".to_owned(), hit: true });
    }
}
