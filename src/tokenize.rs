use regex::Regex;
use std::sync::LazyLock;

static SENTENCE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b[^.!?]+[.!?]*").unwrap());

pub(crate) fn word_list(text: &str) -> Vec<&str> {
    let mut words = Vec::new();
    let mut start: Option<usize> = None;
    for (i, c) in text.char_indices() {
        let is_word_char = c.is_alphanumeric() || c == '\'' || c == '-';
        match (is_word_char, start) {
            (true, None) => start = Some(i),
            (false, Some(s)) => {
                words.push(&text[s..i]);
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        words.push(&text[s..]);
    }
    words
}

pub(crate) fn sentence_list(text: &str) -> Vec<&str> {
    SENTENCE_RE.find_iter(text).map(|m| m.as_str()).collect()
}
