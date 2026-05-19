use crate::syllable::count_syllables;
use crate::tokenize::word_list;
use crate::tokenize::sentence_list;

pub(crate) fn char_count(text: &str) -> usize {
    text.chars().filter(|c| !c.is_whitespace()).count()
}

pub(crate) fn letter_count(text: &str) -> usize {
    text.chars().filter(|c| c.is_alphabetic()).count()
}

pub(crate) fn syllable_count(text: &str) -> usize {
    word_list(text).iter().map(|w| count_syllables(w)).sum()
}

pub(crate) fn miniword_count(text: &str) -> usize {
    word_list(text).iter().filter(|w| w.chars().count() <= 3).count()
}

pub(crate) fn sentence_count(text: &str) -> usize {
    sentence_list(text).len().max(1)
}

pub(crate) fn word_count(text: &str) -> usize {
    word_list(text).len().max(1)
}

pub(crate) fn words_per_sentence(text: &str) -> usize {
    word_list(text).len() / sentence_count(text)
}

pub(crate) fn count_difficult_words(text: &str, syllable_threshold: usize) -> usize {
    let easy_words = crate::data::easy_words();
    word_list(text).iter().filter(|w| {let lower = w.to_lowercase(); !easy_words.contains(lower.as_str()) && count_syllables(&lower) > syllable_threshold}).count()
}
