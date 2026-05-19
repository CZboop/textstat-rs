use crate::syllable::count_syllables;
use crate::tokenize::word_list;

pub(crate) fn char_count(text: &str) -> usize {
    text.chars().filter(|c| !c.is_whitespace()).count()
}

pub(crate) fn letter_count(text: &str) -> usize {
    text.chars().filter(|c| c.is_alphabetic()).count()
}

pub(crate) fn syllable_count(text: &str) -> usize {
    word_list(text).iter().map(|w| count_syllables(w)).sum()
}
