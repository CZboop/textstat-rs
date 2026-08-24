use pyo3::prelude::*;

mod counts;
mod formulas;
mod lang;
mod pyphen;
mod syllable;
mod tokenize;
mod transform;
mod stats;

/// A Python module implemented in Rust.
#[pymodule]
mod textstat_rs {
    use crate::counts;
    use crate::formulas;
    use crate::lang::{Lang, UnsupportedLang};
    use crate::stats::TextStats;
    use pyo3::exceptions::PyValueError;
    use pyo3::prelude::*;
    use std::sync::RwLock;

    // --- locale plumbing ---------------------------------------------------
    //
    // Two ways in, mirroring the two things callers want:
    //
    //   * `set_lang("en_GB")` once, then score as usual. This is textstat's
    //     surface — there it writes `self.__lang` on a module-level
    //     `TextStatistics` singleton — so a script ported from textstat keeps
    //     working unchanged.
    //   * `flesch_reading_ease(text, lang="en_GB")` per call, which textstat
    //     has no equivalent for. Explicit, and unaffected by whatever another
    //     thread has done to the default.
    //
    // Only functions whose answer actually depends on the locale take `lang=`.
    // In textstat as here, the locale reaches exactly one decision — which
    // hyphenation dictionary spells out out-of-vocabulary syllables — so
    // `char_count` and friends are locale-inert, and say so by not offering
    // the argument at all.

    /// The locale used by any call that doesn't name one.
    ///
    /// Process-wide mutable state, because that is what `set_lang` is. Python
    /// keeps it as a plain attribute since the GIL serialises the write for
    /// free; in Rust the lock has to be written down. Contention is a
    /// non-issue: the guarded region is a single one-byte `Copy`.
    static DEFAULT_LANG: RwLock<Lang> = RwLock::new(Lang::DEFAULT);

    /// Read the module default, ignoring lock poisoning.
    ///
    /// A lock is poisoned when a thread panics while holding it, so the value
    /// behind it may be half-written. Here the guarded region is one `Copy`
    /// read or write that cannot panic part-way, so there is no torn state to
    /// protect anyone from and refusing to serve the value would be noise.
    fn default_lang() -> Lang {
        *DEFAULT_LANG.read().unwrap_or_else(|e| e.into_inner())
    }

    /// Parse a locale tag, turning an unsupported one into a Python exception.
    ///
    /// textstat is laxer: its `set_lang` is a bare assignment, so a bad tag is
    /// accepted and only blows up later — as a `KeyError` from inside
    /// `Pyphen.__init__`, and only once some word actually misses CMUdict. We
    /// reject at the boundary instead; see PARITY.md.
    fn parse_lang(tag: &str) -> PyResult<Lang> {
        tag.parse().map_err(|UnsupportedLang(tag)| {
            // Built from `Lang::ALL`, so a new locale
            // shows up in the error automatically
            let supported = Lang::ALL
                .iter()
                .map(|l| l.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            PyValueError::new_err(format!(
                "unsupported language {tag:?}: textstat-rs supports {supported}"
            ))
        })
    }

    /// Resolve one call's locale. An explicit `lang=` wins over the default.
    fn resolve(lang: Option<&str>) -> PyResult<Lang> {
        match lang {
            Some(tag) => parse_lang(tag),
            None => Ok(default_lang()),
        }
    }

    /// Set the locale used by calls that don't pass `lang=` explicitly.
    #[pyfunction]
    fn set_lang(lang: &str) -> PyResult<()> {
        let parsed = parse_lang(lang)?;
        *DEFAULT_LANG.write().unwrap_or_else(|e| e.into_inner()) = parsed;
        Ok(())
    }

    /// The locale currently used by calls that don't pass `lang=`.
    ///
    /// No textstat equivalent — there the field is name-mangled private — but
    /// without it a test fixture cannot restore the default it clobbered.
    #[pyfunction]
    fn get_lang() -> &'static str {
        default_lang().as_str()
    }

    /// Every locale `set_lang` and `lang=` accept, as canonical tags.
    ///
    /// No textstat equivalent -- there the supported set is whatever pyphen
    /// happens to ship, discoverable only by trying one. Exposing it lets a
    /// caller enumerate locales instead of hardcoding them, which is what the
    /// benchmark harness does. Other tags may still resolve to one of these by
    /// truncation (`en_AU` -> `en_GB`); this is the canonical list, not the
    /// accepted one.
    #[pyfunction]
    fn supported_langs() -> Vec<&'static str> {
        Lang::ALL.iter().map(|l| l.as_str()).collect()
    }

    // Metrics
    #[pyfunction]
    #[pyo3(signature = (text, *, lang=None))]
    fn flesch_reading_ease(text: &str, lang: Option<&str>) -> PyResult<f64> {
        let stats = TextStats::new(text, resolve(lang)?);
        Ok(formulas::flesch_reading_ease(&stats))
    }

    #[pyfunction]
    #[pyo3(signature = (text, *, lang=None))]
    fn flesch_kincaid_grade(text: &str, lang: Option<&str>) -> PyResult<f64> {
        let stats = TextStats::new(text, resolve(lang)?);
        Ok(formulas::flesch_kincaid_grade(&stats))
    }

    // No `lang=` on the next four: characters, words and sentences only. They
    // still read the default rather than hardcoding one, so a locale-sensitive
    // term added here later is right by construction instead of silently
    // American.
    #[pyfunction]
    fn automated_readability_index(text: &str) -> PyResult<f64> {
        let stats = TextStats::new(text, default_lang());
        Ok(formulas::automated_readability_index(&stats))
    }

    #[pyfunction]
    fn coleman_liau_index(text: &str) -> PyResult<f64> {
        let stats = TextStats::new(text, default_lang());
        Ok(formulas::coleman_liau_index(&stats))
    }

    #[pyfunction]
    #[pyo3(signature = (text, ms_per_char=14.69))]
    fn reading_time(text: &str, ms_per_char: f64) -> PyResult<f64> {
        let stats = TextStats::new(text, default_lang());
        Ok(formulas::reading_time(&stats, ms_per_char))
    }

    #[pyfunction]
    fn mcalpine_eflaw(text: &str) -> PyResult<f64> {
        let stats = TextStats::new(text, default_lang());
        Ok(formulas::mcalpine_eflaw(&stats))
    }

    #[pyfunction]
    #[pyo3(signature = (text, strict_lower=false, strict_upper=true, *, lang=None))]
    fn linsear_write_formula(
        text: &str,
        strict_lower: bool,
        strict_upper: bool,
        lang: Option<&str>,
    ) -> PyResult<f64> {
        let stats = TextStats::new(text, resolve(lang)?);
        Ok(formulas::linsear_write_formula(
            &stats,
            strict_lower,
            strict_upper,
        ))
    }

    #[pyfunction]
    #[pyo3(signature = (text, *, lang=None))]
    fn spache_readability(text: &str, lang: Option<&str>) -> PyResult<f64> {
        let stats = TextStats::new(text, resolve(lang)?);
        Ok(formulas::spache_readability(&stats))
    }

    #[pyfunction]
    #[pyo3(signature = (text, *, lang=None))]
    fn dale_chall_readability_score(text: &str, lang: Option<&str>) -> PyResult<f64> {
        let stats = TextStats::new(text, resolve(lang)?);
        Ok(formulas::dale_chall_readability_score(&stats))
    }

    #[pyfunction]
    #[pyo3(signature = (text, syllable_threshold=3, *, lang=None))]
    fn gunning_fog(text: &str, syllable_threshold: usize, lang: Option<&str>) -> PyResult<f64> {
        let stats = TextStats::new(text, resolve(lang)?);
        Ok(formulas::gunning_fog(&stats, syllable_threshold))
    }

    #[pyfunction]
    #[pyo3(signature = (text, *, lang=None))]
    fn smog_index(text: &str, lang: Option<&str>) -> PyResult<f64> {
        let stats = TextStats::new(text, resolve(lang)?);
        Ok(formulas::smog_index(&stats))
    }

    #[pyfunction]
    #[pyo3(signature = (text, *, lang=None))]
    fn text_standard(text: &str, lang: Option<&str>) -> PyResult<String> {
        let stats = TextStats::new(text, resolve(lang)?);
        Ok(formulas::text_standard(text, &stats))
    }

    // Internal counts
    // (TODO: potentially remove as exposed mod funcs post-dev)

    // textstat's `syllable_count` still carries a `lang` argument, but it was
    // deprecated in favour of `set_lang` and now only raises a
    // DeprecationWarning before being thrown away. Same spelling here, except
    // we honour it — which is what anyone who wrote it meant.
    #[pyfunction]
    #[pyo3(signature = (text, *, lang=None))]
    fn syllable_count(text: &str, lang: Option<&str>) -> PyResult<usize> {
        Ok(counts::syllable_count(text, resolve(lang)?))
    }

    #[pyfunction]
    fn sentence_count(text: &str) -> PyResult<usize> {
        Ok(counts::sentence_count(text))
    }

    #[pyfunction]
    #[pyo3(signature = (text, removepunct=true, split_contractions=false, split_hyphens=false))]
    fn lexicon_count(
        text: &str,
        removepunct: bool,
        split_contractions: bool,
        split_hyphens: bool,
    ) -> PyResult<usize> {
        Ok(crate::tokenize::word_list(
            text,
            removepunct,
            false,
            false,
            split_contractions,
            split_hyphens,
        )
        .len())
    }

    #[pyfunction]
    #[pyo3(signature = (text, ignore_spaces=true))]
    fn char_count(text: &str, ignore_spaces: bool) -> PyResult<usize> {
        if ignore_spaces {
            Ok(counts::char_count(text))
        } else {
            Ok(text.chars().count())
        }
    }

    #[pyfunction]
    fn letter_count(text: &str) -> PyResult<usize> {
        Ok(counts::count_letters(text))
    }

    #[pyfunction]
    #[pyo3(signature = (text, *, lang=None))]
    fn polysyllabcount(text: &str, lang: Option<&str>) -> PyResult<usize> {
        Ok(counts::polysyllable_word_count(text, resolve(lang)?))
    }

    #[pyfunction]
    #[pyo3(signature = (text, max_size=3))]
    fn miniword_count(text: &str, max_size: usize) -> PyResult<usize> {
        Ok(counts::miniword_count(text, max_size))
    }

    #[pyfunction]
    #[pyo3(signature = (text, syllable_threshold=2, unique=true, *, lang=None))]
    fn difficult_words(
        text: &str,
        syllable_threshold: usize,
        unique: bool,
        lang: Option<&str>,
    ) -> PyResult<usize> {
        Ok(counts::count_difficult_words(
            text,
            syllable_threshold,
            unique,
            resolve(lang)?,
        ))
    }
}

mod data;
