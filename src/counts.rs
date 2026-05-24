use crate::syllable::count_syllables;
use crate::tokenize::sentence_list;
use crate::tokenize::word_list;

pub(crate) fn char_count(text: &str) -> usize {
    text.chars().filter(|c| !c.is_whitespace()).count()
}

pub(crate) fn letter_count(text: &str) -> usize {
    text.chars().filter(|c| c.is_alphabetic()).count()
}

pub(crate) fn syllable_count(text: &str) -> usize {
    word_list(text, true, false, false, false, false)
        .iter()
        .map(|w| count_syllables(w))
        .sum()
}

pub(crate) fn miniword_count(text: &str) -> usize {
    word_list(text, true, false, false, false, false)
        .iter()
        .filter(|w| w.chars().count() <= 3)
        .count()
}

pub(crate) fn sentence_count(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    // get as a str vec from sentence_list
    // count if less than 2 words based on word_count
    sentence_list(text)
        .into_iter()
        .filter(|s| word_count(s) > 2)
        .count()
        .max(1)
}

pub(crate) fn word_count(text: &str) -> usize {
    word_list(text, true, false, false, false, false)
        .len()
        .max(1)
}

pub(crate) fn words_per_sentence(text: &str) -> f64 {
    word_list(text, true, false, false, false, false).len() as f64 / sentence_count(text) as f64
}

pub(crate) fn count_difficult_words(text: &str, syllable_threshold: usize) -> usize {
    let easy_words = crate::data::easy_words();
    word_list(text, true, false, false, false, false)
        .iter()
        .filter(|w| {
            let lower = w.to_lowercase();
            !easy_words.contains(lower.as_str()) && count_syllables(&lower) >= syllable_threshold
        })
        .count()
}

pub(crate) fn polysyllable_word_count(text: &str) -> usize {
    word_list(text, true, false, false, false, false)
        .iter()
        .map(|w| count_syllables(w))
        .filter(|c| *c >= 3)
        .count()
}
