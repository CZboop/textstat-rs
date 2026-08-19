//! The locale a text is scored in.
//!
//! textstat threads a locale string (`"en_US"`, `"en_GB"`, ...) through every
//! metric, but almost everything it reaches is keyed on the *root* of that
//! locale rather than the whole thing: `get_lang_cfg` (the Flesch coefficients
//! and the syllable threshold), `get_lang_easy_words` and `get_cmudict` all
//! call `get_lang_root` first, and `"en_GB"` and `"en_US"` share the root
//! `"en"`. The one lookup that keeps the full locale is `get_pyphen`, which
//! does `Pyphen(lang=lang)`.
//!
//! So for British vs American English the *only* thing that changes is which
//! hyphenation dictionary backs the out-of-vocabulary syllable fallback. That
//! is why this enum has exactly the granularity of "which `.dic` file", and no
//! coefficient tables hang off it.

use std::str::FromStr;

/// A locale textstat-rs can score in.
///
/// Only English is supported: pyphen ships dictionaries for ~70 languages but
/// textstat-rs vendors just the two English ones, so a locale outside `en` is
/// an error here where upstream textstat would have gone on to use French
/// patterns and French coefficients.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum Lang {
    /// American English — textstat's default (`TextStatistics.__lang`).
    #[default]
    EnUs,
    /// British English.
    EnGb,
}

impl Lang {
    /// The canonical locale tag, as textstat would have been handed it.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Lang::EnUs => "en_US",
            Lang::EnGb => "en_GB",
        }
    }
}

/// A locale that doesn't resolve to a vendored dictionary.
///
/// Upstream this surfaces as a `KeyError` out of `LANGUAGES[...]` inside
/// `Pyphen.__init__`, because `language_fallback` returns `None` when nothing
/// matches and `None` is not a key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UnsupportedLang(pub String);

impl FromStr for Lang {
    type Err = UnsupportedLang;

    /// Resolve a locale tag the way pyphen's `language_fallback` does.
    ///
    /// pyphen normalises `-` to `_`, lowercases, then walks *up* the locale by
    /// lopping off trailing subtags until something matches ("normal truncation
    /// inheritance", per UTS #35). We reproduce that loop rather than matching
    /// the three tags directly so the inheritance falls out on its own — that
    /// is what makes `"en_AU"` resolve, and it is where the `"en"` quirk below
    /// comes from.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalised = s.replace('-', "_").to_lowercase();
        let mut candidate = normalised.as_str();

        loop {
            match candidate {
                "en_gb" => return Ok(Lang::EnGb),
                "en_us" => return Ok(Lang::EnUs),
                // Bare `"en"` is *not* American English. pyphen builds its
                // `LANGUAGES` map over `sorted(dictionaries.iterdir())` and
                // registers a short name only if it is still unclaimed, so the
                // first `en_*` file to sort wins — and `hyph_en_GB.dic` sorts
                // before `hyph_en_US.dic`. Matching the quirk, not fixing it.
                "en" => return Ok(Lang::EnGb),
                _ => {}
            }

            // Drop the last subtag and try again; no `_` left means no match.
            match candidate.rsplit_once('_') {
                Some((head, _)) => candidate = head,
                None => return Err(UnsupportedLang(s.to_string())),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_tags_resolve() {
        assert_eq!("en_US".parse(), Ok(Lang::EnUs));
        assert_eq!("en_GB".parse(), Ok(Lang::EnGb));
    }

    #[test]
    fn separator_and_case_are_normalised() {
        for tag in ["en-GB", "EN_GB", "en_gb", "En-gB"] {
            assert_eq!(tag.parse(), Ok(Lang::EnGb), "{tag}");
        }
    }

    #[test]
    fn bare_en_is_british() {
        // See the note in `from_str`: pyphen's short-name registration is
        // first-sorted-wins, and `hyph_en_GB.dic` sorts first.
        assert_eq!("en".parse(), Ok(Lang::EnGb));
    }

    #[test]
    fn unknown_region_truncates_to_the_root() {
        // No `hyph_en_AU.dic` is vendored, so this falls back to `en` — which
        // is British, per the quirk above.
        assert_eq!("en_AU".parse(), Ok(Lang::EnGb));
        assert_eq!("en_US_posix".parse(), Ok(Lang::EnUs));
    }

    #[test]
    fn non_english_is_unsupported() {
        for tag in ["fr", "de_DE", "", "nonsense"] {
            assert_eq!(
                tag.parse::<Lang>(),
                Err(UnsupportedLang(tag.to_string())),
                "{tag}"
            );
        }
    }
}
