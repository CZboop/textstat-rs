use crate::counts::{
    char_count, count_difficult_words, letter_count, miniword_count, polysyllable_word_count,
    sentence_count, syllable_count, word_count, words_per_sentence,
};
use crate::syllable::count_syllables;
use crate::tokenize::{sentence_list, word_list};
use std::collections::HashMap;

pub(crate) fn flesch_reading_ease(text: &str) -> f64 {
    let words = word_list(text, true, false, false, false, false).len() as f64;
    let sentences = sentence_count(text) as f64;
    let syllables = syllable_count(text) as f64;
    if words == 0.0 {
        return 0.0;
    }
    if sentences == 0.0 {
        return 0.0;
    }
    206.835 - 1.015 * (words / sentences) - 84.6 * (syllables / words)
}

pub(crate) fn flesch_kincaid_grade(text: &str) -> f64 {
    let words = word_list(text, true, false, false, false, false).len() as f64;
    let sentences = sentence_list(text).len() as f64;
    let syllables = syllable_count(text) as f64;
    0.39 * (words / sentences) + 11.8 * (syllables / words) - 15.59
}

pub(crate) fn automated_readability_index(text: &str) -> f64 {
    let words = word_list(text, true, false, false, false, false).len() as f64;
    let sentences = sentence_list(text).len() as f64;
    let chars = char_count(text) as f64;
    4.71 * (chars / words) + 0.5 * (words / sentences) - 21.43
}

pub(crate) fn coleman_liau_index(text: &str) -> f64 {
    let words = word_list(text, true, false, false, false, false).len() as f64;
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
    for w in word_list(truncated, true, false, false, false, false) {
        if count_syllables(&w) < 3 {
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

pub(crate) fn mcalpine_eflaw(text: &str) -> f64 {
    let n_words = word_list(text, true, false, false, false, false).len();
    let n_sentences = sentence_count(text);
    let n_miniwords = miniword_count(text);

    (n_words + n_miniwords) as f64 / n_sentences as f64
}

pub(crate) fn spache_readability(text: &str) -> f64 {
    let num_total_words = word_count(text);
    let asl = words_per_sentence(text);
    if num_total_words == 0 {
        return 0.0;
    } else {
        let pdw = 100 * count_difficult_words(text, 2) / num_total_words;
        (0.141 * asl as f64) + (0.086 * pdw as f64) + 0.839
    }
}

pub(crate) fn dale_chall_readability_score(text: &str) -> f64 {
    let word_count = word_count(text);
    let hard_count = count_difficult_words(text, 0);
    if word_count == 0 {
        return 0.0;
    } else {
        let per_difficult_words = 100 * hard_count / word_count;
        let mut score =
            (0.1579 * per_difficult_words as f64) + (0.0496 * words_per_sentence(text) as f64);
        if per_difficult_words > 5 {
            score += 3.6365
        }
        score
    }
}

pub(crate) fn gunning_fog(text: &str, syllable_threshold: usize) -> f64 {
    let difficult_words = count_difficult_words(text, syllable_threshold);
    let total_words = word_count(text);
    if total_words == 0 {
        return 0.0;
    } else {
        let per_difficult_words = 100.0 * difficult_words as f64 / total_words as f64;
        0.4 * (words_per_sentence(text) as f64 + per_difficult_words)
    }
}

pub(crate) fn smog_index(text: &str) -> f64 {
    let sentences = sentence_count(text) as f64;
    let poly_syllab = polysyllable_word_count(text) as f64;
    (1.043 * (30.0 * (poly_syllab / sentences)).sqrt()) + 3.1291
}

pub(crate) fn text_standard(text: &str) -> String {
    // TODO: potentially add float_value boolean arg
    let mut grade: Vec<i32> = Vec::new(); // TODO: add with_capacity once know num items? should be constant
    // TODO: instead of pushing as you go, just create at the end?

    // Flesch Kincaid Grade
    let score = flesch_kincaid_grade(text);
    let lower = score.floor() as i32;
    let upper = score.ceil() as i32;
    let near = score.round() as i32; // TODO: verify details (of rounding)

    grade.extend([lower, upper, near]);

    // Flesch Reading Ease
    let score = flesch_reading_ease(text);
    if score < 100.0 && score >= 90.0 {
        grade.push(5);
    } else if score < 90.0 && score >= 80.0 {
        grade.push(6);
    } else if score < 80.0 && score >= 70.0 {
        grade.push(7);
    } else if score < 70.0 && score >= 60.0 {
        grade.push(8);
        grade.push(9);
    } else if score < 60.0 && score >= 50.0 {
        grade.push(10);
    } else if score < 50.0 && score >= 40.0 {
        grade.push(11);
    } else if score < 40.0 && score >= 30.0 {
        grade.push(12);
    } else {
        grade.push(13);
    }

    // SMOG Index
    let score = smog_index(text);
    let lower = score.floor() as i32;
    let upper = score.floor() as i32;
    let near = score.round() as i32;
    grade.extend([lower, upper, near]);

    // Coleman_Liau_Index
    let score = coleman_liau_index(text);
    let lower = score.floor() as i32;
    let upper = score.ceil() as i32;
    let near = score.round() as i32;
    grade.extend([lower, upper, near]);

    // Automated_Readability_Index
    let score = automated_readability_index(text);
    let lower = score.floor() as i32;
    let upper = score.ceil() as i32;
    let near = score.round() as i32;
    grade.extend([lower, upper, near]);

    // Dale_Chall_Readability_Score
    let score = dale_chall_readability_score(text);
    let lower = score.floor() as i32;
    let upper = score.ceil() as i32;
    let near = score.round() as i32;
    grade.extend([lower, upper, near]);

    // Linsear_Write_Formula
    let score = linsear_write_formula(text);
    let lower = score.floor() as i32;
    let upper = score.ceil() as i32;
    let near = score.round() as i32;
    grade.extend([lower, upper, near]);

    // Appending Gunning Fog Index
    // TODO: reconsider defaults being py signature only?
    let score = gunning_fog(text, 3);
    let lower = score.floor() as i32;
    let upper = score.ceil() as i32;
    let near = score.round() as i32;
    grade.extend([lower, upper, near]);

    // Finding the Readability Consensus based on all the above tests
    let final_grade_numeric = grade
        .iter()
        .fold(HashMap::new(), |mut m, &g| {
            *m.entry(g).or_insert(0) += 1;
            m
        })
        .into_iter()
        .max_by_key(|&(_, c)| c)
        .map(|(g, _)| g)
        .unwrap();
    let clamped_grade = final_grade_numeric.clamp(1, 18);
    let lower_score = clamped_grade - 1;
    let upper_score = lower_score + 1;

    let lower_suffix = ordinal_suffix(lower_score);
    let upper_suffix = ordinal_suffix(upper_score);

    format!("{lower_score}{lower_suffix} and {upper_score}{upper_suffix} grade")
}

fn ordinal_suffix(number: i32) -> &'static str {
    let ordinal_value = match number % 10 {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    };
    let teen_value = match number % 100 {
        11 => "th",
        12 => "th",
        13 => "th",
        _ => "none",
    };
    if teen_value == "none" {
        return ordinal_value;
    }
    ordinal_value
}
