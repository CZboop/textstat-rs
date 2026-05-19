use std::collections::HashSet;
use std::sync::OnceLock;

const EASY_WORDS_TXT: &str = include_str!("data/easy_words.txt");

static EASY_WORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();

pub(crate) fn easy_words() -> &'static HashSet<&'static str> {
    EASY_WORDS.get_or_init(|| EASY_WORDS_TXT.lines().filter(|l| !l.is_empty()).collect())
}
