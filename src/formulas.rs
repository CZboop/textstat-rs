use crate::counts::{char_count, letter_count, syllable_count};
use crate::syllable::count_syllables;
use crate::tokenize::{sentence_list, word_list};

pub(crate) fn flesch_reading_ease(text: &str) -> f64 {
    let words = word_list(text).len() as f64;
    let sentences = sentence_list(text).len() as f64;
    let syllables = syllable_count(text) as f64;
    206.835 - 1.015 * (words / sentences) - 84.6 * (syllables / words)
}

pub(crate) fn flesch_kincaid_grade(text: &str) -> f64 {
    let words = word_list(text).len() as f64;
    let sentences = sentence_list(text).len() as f64;
    let syllables = syllable_count(text) as f64;
    0.39 * (words / sentences) + 11.8 * (syllables / words) - 15.59
}

pub(crate) fn automated_readability_index(text: &str) -> f64 {
    let words = word_list(text).len() as f64;
    let sentences = sentence_list(text).len() as f64;
    let chars = char_count(text) as f64;
    4.71 * (chars / words) + 0.5 * (words / sentences) - 21.43
}

pub(crate) fn coleman_liau_index(text: &str) -> f64 {
    let words = word_list(text).len() as f64;
    let sentences = sentence_list(text).len() as f64;
    let letters = letter_count(text) as f64;
    let l = letters / words * 100.0;
    let s = sentences / words * 100.0;
    0.0588 * l - 0.296 * s - 15.8
}

pub(crate) fn linsear_write_formula(text: &str) -> f64 {
    // textstat scores only the first 100 whitespace-separated tokens — truncate
    // the original text to that prefix so sentence boundaries are preserved.
    let mut token_count = 0usize;
    let mut in_token = false;
    let mut end_byte = text.len();
    for (i, c) in text.char_indices() {
        if c.is_whitespace() {
            if in_token {
                token_count += 1;
                in_token = false;
                if token_count >= 100 {
                    end_byte = i;
                    break;
                }
            }
        } else {
            in_token = true;
        }
    }
    let truncated = &text[..end_byte];

    let mut easy = 0usize;
    let mut hard = 0usize;
    for w in word_list(truncated) {
        if count_syllables(w) < 3 {
            easy += 1;
        } else {
            hard += 1;
        }
    }

    let sentences = sentence_list(truncated).len() as f64;
    let mut number = (easy + hard * 3) as f64 / sentences;
    if number <= 20.0 {
        number -= 2.0;
    }
    number / 2.0
}

pub(crate) fn reading_time(text: &str, ms_per_char: f64) -> f64 {
    // TODO: char_count ignore_spaces boolean arg
    let time: f64 = ms_per_char * char_count(text) as f64 / 1000.0;
    time
}

// TODO: relies on other metrics not yet implemented
// pub(crate) fn text_standard(text: &str) -> f64 {
//     0.0
// }
