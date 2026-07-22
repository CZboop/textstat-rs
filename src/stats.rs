use crate::syllable::count_syllables; // TODO: standardise, two versions of syllable count still exist!
use std::cell::OnceCell;
// lazy init - in case of stats that don't need everything computed
use crate::counts::{char_count, sentence_count};
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
            n_chars: OnceCell::new(),
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
            .get_or_init(|| word_list(self.text, true, true, true, false, false))
    }

    pub fn tokens_with_punct(&self) -> &[String] {
        self.tokens_with_punct
            .get_or_init(|| word_list(self.text, false, false, false, false, false))
    }

    pub fn n_sentences(&self) -> &usize {
        self.n_sentences.get_or_init(|| sentence_count(self.text))
    }

    pub fn n_syllables(&self) -> &usize {
        self.n_syllables.get_or_init(|| count_syllables(self.text))
    }

    pub fn n_chars(&self) -> &usize {
        self.n_chars.get_or_init(|| char_count(self.text))
    }
}
