use crate::counts::{
    char_count, count_difficult_words, letters_per_word, miniword_count, polysyllable_word_count,
    sentence_count, sentences_per_word, syllable_count, word_count, words_per_sentence,
};
use crate::syllable::count_syllables;
use crate::tokenize::word_list;
use crate::transform::remove_punctuation;
use std::collections::HashMap;
use crate::stats::TextStats;

pub(crate) fn flesch_reading_ease(stats: &TextStats) -> f64 {
    let words = stats.words().len() as f64;
    let sentence_length = stats.words_per_sent();
    let mut syllables_per_word = *stats.n_syllables() as f64 / words;
    if words == 0.0 {
        syllables_per_word = 0.0;
    }
    206.835 - 1.015 * sentence_length - 84.6 * syllables_per_word
}

pub(crate) fn flesch_kincaid_grade(stats: &TextStats) -> f64 {
    let words = stats.words().len() as f64;
    let sentences = *stats.n_sentences() as f64;
    let syllables = *stats.n_syllables() as f64;
    0.39 * (words / sentences) + 11.8 * (syllables / words) - 15.59
}

pub(crate) fn automated_readability_index(stats: &TextStats) -> f64 {
    let words = stats.words().len() as f64;
    let sentences = *stats.n_sentences() as f64;
    let chars = *stats.n_chars() as f64;
    4.71 * (chars / words) + 0.5 * (words / sentences) - 21.43
}

pub(crate) fn coleman_liau_index(stats: &TextStats) -> f64 {
    let words = stats.words().len() as f64;
    let sentences = if words == 0.0 { 0.0 } else { *stats.n_sentences() as f64 / words } * 100.0;
    let letters = stats.letters_per_word() * 100 as f64;
    if letters == 0.0 || sentences == 0.0 {
        return 0.0;
    }
    0.058 * letters - 0.296 * sentences - 15.8
}

pub(crate) fn linsear_write_formula(text: &str, strict_lower: bool, strict_upper: bool) -> f64 {
    let text_list = word_list(text, false, false, false, false, false);

    let mut words_list = Vec::new();
    let mut i_text = 0;
    let mut word;
    if strict_upper && text_list.len() > 100 {
        while (i_text < text_list.len()) && (words_list.len() < 100) {
            word = remove_punctuation(&text_list[i_text], false);
            i_text += 1;
            if word.len() > 0 {
                words_list.push(word)
            }
        }
    } else {
        words_list = word_list(text, true, false, false, false, false);
        i_text = text_list.len();
    }
    if strict_lower && (words_list.len() < 100) {
        return 0.0;
    }

    let mut easy_word = 0;
    let mut difficult_word = 0;
    for word in words_list.iter() {
        let n_syll = count_syllables(word);
        if n_syll >= 3 {
            difficult_word += 1;
        } else {
            if n_syll > 0 {
                easy_word += 1;
            }
        }
    }
    let text = text_list[..i_text].join(" ");
    let text_sentences = sentence_count(&text);
    if text_sentences == 0 {
        return 0.0;
    }
    let mut number = (easy_word * 1 + difficult_word * 3) as f64 / text_sentences as f64;
    if number <= 20.0 {
        number -= 2.0;
    }
    number / 2.0
}

pub(crate) fn reading_time(stats: &TextStats, ms_per_char: f64) -> f64 {
    // TODO: char_count ignore_spaces boolean arg
    let time: f64 = ms_per_char * *stats.n_chars() as f64 / 1000.0;
    time
}

pub(crate) fn mcalpine_eflaw(stats: &TextStats) -> f64 {
    let n_words = stats.words().len() as f64;
    let n_sentences = *stats.n_sentences() as f64;
    let n_miniwords = stats.miniword_count() as f64;

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

pub(crate) fn text_standard(text: &str, stats: &TextStats) -> String {
    // TODO: potentially add float_value boolean arg
    let mut grade: Vec<i32> = Vec::new(); // TODO: add with_capacity once know num items? should be constant
    // TODO: instead of pushing as you go, just create at the end?

    // Flesch Kincaid Grade
    let score = flesch_kincaid_grade(stats);
    let lower = score.floor() as i32;
    let upper = score.ceil() as i32;
    let near = score.round() as i32; // TODO: verify details (of rounding)

    grade.extend([lower, upper, near]);

    // Flesch Reading Ease
    let score = flesch_reading_ease(stats);
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
    let upper = score.ceil() as i32;
    let near = score.round() as i32;
    grade.extend([lower, upper, near]);

    // Coleman_Liau_Index
    let score = coleman_liau_index(stats);
    let lower = score.floor() as i32;
    let upper = score.ceil() as i32;
    let near = score.round() as i32;
    grade.extend([lower, upper, near]);

    // Automated_Readability_Index
    let score = automated_readability_index(stats);
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
    // TODO: confirm bools
    let score = linsear_write_formula(text, false, true);
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
    let counts = grade.iter().fold(HashMap::new(), |mut m, &g| {
        *m.entry(g).or_insert(0usize) += 1;
        m
    });
    let max_count = counts.values().copied().max().unwrap();
    let final_grade_numeric = grade
        .iter()
        .copied()
        .find(|g| counts[g] == max_count)
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
