use hyphenation::{Hyphenator, Language, Load, Standard};
use std::collections::HashMap;
use std::sync::LazyLock;

static EN_US: LazyLock<Standard> =
    LazyLock::new(|| Standard::from_embedded(Language::EnglishUS).unwrap());

static CMU_RAW: &str = include_str!("data/cmudict.txt");

static CMU: LazyLock<HashMap<String, usize>> = LazyLock::new(|| {
    CMU_RAW
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            // single space separator, lowercase words already
            let (word, phones) = line.split_once(' ')?;
            if word.contains('(') {
                return None;
            }
            let syllables = phones
                .split_whitespace()
                .filter(|p| p.chars().last().is_some_and(|c| c.is_ascii_digit()))
                .count();
            Some((word.to_string(), syllables))
        })
        .collect()
});

pub(crate) fn count_syllables(word: &str) -> usize {
    let lower: String = word
        .chars()
        .filter(|c| c.is_alphabetic())
        .collect::<String>()
        .to_lowercase();

    if lower.is_empty() {
        return 0;
    }
    if let Some(&n) = CMU.get(&lower) {
        return n.max(1);
    }
    
    // out of vocab fallback, typographic breaks + 1, to match pyphen
    (EN_US.hyphenate(&lower).breaks.len() + 1).max(1)
}
