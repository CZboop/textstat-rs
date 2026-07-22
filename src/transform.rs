use regex::Regex;
use std::sync::LazyLock;

static RE_APOSTROPHE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"'((?:ve|ll|re|[tsd])?)").unwrap());
static RE_PUNCTUATION_RM_APOSTROPHE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^\p{L}\p{N}_\s]").unwrap());
static RE_PUNCTUATION_KEEP_APOSTROPHE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^\p{L}\p{N}_\s']").unwrap());

pub(crate) fn remove_punctuation(text: &str, rm_apostrophe: bool) -> String {
    let mut new_text = text.to_owned();
    let punctuation_regex = if rm_apostrophe {
        &RE_PUNCTUATION_RM_APOSTROPHE
    } else {
        &RE_PUNCTUATION_KEEP_APOSTROPHE
    };
    if !rm_apostrophe {
        new_text = RE_APOSTROPHE
            .replace_all(&new_text, |caps: &regex::Captures| {
                if caps[1].is_empty() {
                    String::new()
                } else {
                    format!("'{}", &caps[1])
                }
            })
            .into_owned();
    }
    punctuation_regex.replace_all(&new_text, "").into_owned()
}
