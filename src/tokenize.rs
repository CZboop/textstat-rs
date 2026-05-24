use crate::transform::remove_punctuation;
use fancy_regex::Regex as FancyRegex;
use regex::Regex;
use std::sync::LazyLock;

static SENTENCE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b[^.!?]+[.!?]*").unwrap());
static RE_CONTRACTION_APOSTROPHE: LazyLock<FancyRegex> =
    LazyLock::new(|| FancyRegex::new(r"\'(?=[tsd]|ve|ll|re)").unwrap());

pub(crate) fn word_list(
    text: &str,
    rm_punctuation: bool,
    rm_apostrophe: bool,
    lowercase: bool,
    split_contractions: bool,
    split_hyphens: bool,
) -> Vec<String> {
    let mut new_text = text.to_owned();
    if split_hyphens {
        new_text = new_text.replace("-", " ");
    }
    if rm_punctuation {
        new_text = remove_punctuation(&new_text, rm_apostrophe);
    }
    if lowercase {
        new_text = new_text.to_lowercase();
    }
    if split_contractions {
        new_text = RE_CONTRACTION_APOSTROPHE
            .replace_all(&new_text, " ")
            .into_owned();
    }
    new_text.split_whitespace().map(str::to_owned).collect()
}

pub(crate) fn sentence_list(text: &str) -> Vec<&str> {
    SENTENCE_RE.find_iter(text).map(|m| m.as_str()).collect()
}
