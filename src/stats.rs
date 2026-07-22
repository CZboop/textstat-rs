use crate::syllable::count_syllables;
use std::cell::OnceCell;
// lazy init - in case of stats that don't need everything computed
use crate::counts::{char_count, sentence_count, syllable_count, words_per_sentence, count_letters, word_count};
use crate::tokenize::word_list;

pub(crate) struct TextStats<'a> {
    text: &'a str,
    words: OnceCell<Vec<String>>,
    words_lower: OnceCell<Vec<String>>,
    words_no_apostrophe: OnceCell<Vec<String>>,
    tokens_with_punct: OnceCell<Vec<String>>,
    n_sentences: OnceCell<usize>,
    n_syllables: OnceCell<usize>,
    n_chars: OnceCell<usize>,
    n_letters: OnceCell<usize>,
    words_per_sent: OnceCell<f64>,
    letters_per_word: OnceCell<f64>,
}

impl<'a> TextStats<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            words: OnceCell::new(),
            words_lower: OnceCell::new(),
            words_no_apostrophe: OnceCell::new(),
            tokens_with_punct: OnceCell::new(),
            n_sentences: OnceCell::new(),
            n_syllables: OnceCell::new(),
            n_chars: OnceCell::new(), // NOTE: naming to match textstat def of chars/letters
            n_letters: OnceCell::new(),
            words_per_sent: OnceCell::new(),
            letters_per_word: OnceCell::new(),
        }
    }

    pub fn words(&self) -> &[String] {
        self.words
            .get_or_init(|| word_list(self.text, true, false, false, false, false))
    }

    pub fn words_lower(&self) -> &[String] {
        self.words_lower
            .get_or_init(|| word_list(self.text, true, false, true, false, false))
    }

    pub fn words_no_apostrophe(&self) -> &[String] {
        self.words_no_apostrophe
            .get_or_init(|| word_list(self.text, true, true, false, false, false))
    }

    pub fn tokens_with_punct(&self) -> &[String] {
        self.tokens_with_punct
            .get_or_init(|| word_list(self.text, false, false, false, false, false))
    }

    pub fn n_sentences(&self) -> &usize {
        self.n_sentences.get_or_init(|| sentence_count(self.text))
    }

    pub fn n_syllables(&self) -> &usize {
        self.n_syllables.get_or_init(|| syllable_count(self.text))
    }

    pub fn n_chars(&self) -> &usize {
        self.n_chars.get_or_init(|| char_count(self.text))
    }

    pub fn n_letters(&self) -> &usize {
        self.n_letters.get_or_init(|| count_letters(self.text))
    }
    // TODO: switch to words_per_sentence
    pub fn words_per_sent(&self) -> &f64 {
        self.words_per_sent.get_or_init(|| words_per_sentence(self.text))
    }

    pub fn letters_per_word(&self) -> f64 {
        let letters = *self.n_letters() as f64;
        let words = self.words().len() as f64;
        if words == 0.0 {
            return 0.0;
        }
        letters / words
    }

    pub fn miniword_count(&self) -> usize {
        self.words_no_apostrophe()
            .iter()
            .filter(|w| w.chars().count() <= 3)
            .count()
    }

    pub fn words_per_sentence(&self) -> f64 {
        self.words().len() as f64 / *self.n_sentences() as f64
    }

    pub fn count_difficult_words(&self, syllable_threshold: usize) -> usize {
        let easy_words = crate::data::easy_words();
        self.words()
            .iter()
            .filter(|w| {
                let lower = w.to_lowercase();
                !easy_words.contains(lower.as_str()) && count_syllables(&lower) >= syllable_threshold
            })
            .count()
    }

    pub fn polysyllable_word_count(&self) -> usize {
        self.words()
        .iter()
        // TODO: can this use internal syllable count?
        .map(|w| count_syllables(w))
        .filter(|c| *c >= 3)
        .count()
}
}
