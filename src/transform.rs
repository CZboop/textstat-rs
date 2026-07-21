use fancy_regex::Regex as FancyRegex;
use regex::Regex;
use std::sync::LazyLock;

static RE_NONCONTRACTION_APOSTROPHE: LazyLock<FancyRegex> =
    LazyLock::new(|| FancyRegex::new(r"\'(?![tsd]|ve|ll|re)").unwrap());
static RE_PUNCTUATION_RM_APOSTROPHE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^\w\s]").unwrap());
static RE_PUNCTUATION_KEEP_APOSTROPHE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^\w\s\']").unwrap());

pub(crate) fn remove_punctuation(text: &str, rm_apostrophe: bool) -> String {
    let mut new_text = text.to_owned();
    let punctuation_regex = if rm_apostrophe {
        &RE_PUNCTUATION_RM_APOSTROPHE
    } else {
        &RE_PUNCTUATION_KEEP_APOSTROPHE
    };
    if !rm_apostrophe {
        new_text = RE_NONCONTRACTION_APOSTROPHE
            .replace_all(&new_text, "")
            .into_owned();
    }
    punctuation_regex.replace_all(&new_text, "").into_owned()
}
