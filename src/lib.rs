use pyo3::prelude::*;

mod counts;
mod formulas;
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
    use pyo3::prelude::*;
    use crate::stats::TextStats;

    // Metrics
    #[pyfunction]
    fn flesch_reading_ease(text: &str) -> PyResult<f64> {
        let stats = TextStats::new(text);
        Ok(formulas::flesch_reading_ease(&stats))
    }

    #[pyfunction]
    fn flesch_kincaid_grade(text: &str) -> PyResult<f64> {
        let stats = TextStats::new(text);
        Ok(formulas::flesch_kincaid_grade(&stats))
    }

    #[pyfunction]
    fn automated_readability_index(text: &str) -> PyResult<f64> {
        let stats = TextStats::new(text);
        Ok(formulas::automated_readability_index(&stats))
    }

    #[pyfunction]
    fn coleman_liau_index(text: &str) -> PyResult<f64> {
        let stats = TextStats::new(text);
        Ok(formulas::coleman_liau_index(&stats))
    }

    #[pyfunction]
    #[pyo3(signature = (text, strict_lower=false, strict_upper=true))]
    fn linsear_write_formula(text: &str, strict_lower: bool, strict_upper: bool) -> PyResult<f64> {
        Ok(formulas::linsear_write_formula(
            text,
            strict_lower,
            strict_upper,
        ))
    }

    #[pyfunction]
    #[pyo3(signature = (text, ms_per_char=14.69))]
    fn reading_time(text: &str, ms_per_char: f64) -> PyResult<f64> {
        let stats = TextStats::new(text);
        Ok(formulas::reading_time(&stats, ms_per_char))
    }

    #[pyfunction]
    fn mcalpine_eflaw(text: &str) -> PyResult<f64> {
        let stats = TextStats::new(text);
        Ok(formulas::mcalpine_eflaw(&stats))
    }

    #[pyfunction]
    fn spache_readability(text: &str) -> PyResult<f64> {
        let stats = TextStats::new(text);
        Ok(formulas::spache_readability(&stats))
    }

    #[pyfunction]
    fn dale_chall_readability_score(text: &str) -> PyResult<f64> {
        let stats = TextStats::new(text);
        Ok(formulas::dale_chall_readability_score(&stats))
    }

    #[pyfunction]
    #[pyo3(signature = (text, syllable_threshold=3))]
    fn gunning_fog(text: &str, syllable_threshold: usize) -> PyResult<f64> {
        let stats = TextStats::new(text);
        Ok(formulas::gunning_fog(&stats, syllable_threshold))
    }

    #[pyfunction]
    fn smog_index(text: &str) -> PyResult<f64> {
        let stats = TextStats::new(text);
        Ok(formulas::smog_index(&stats))
    }

    #[pyfunction]
    fn text_standard(text: &str) -> PyResult<String> {
        let stats = TextStats::new(text);
        Ok(formulas::text_standard(text, &stats))
    }

    // Internal counts 
    // (TODO: potentially remove as exposed mod funcs post-dev)
    #[pyfunction]
    fn syllable_count(text: &str) -> PyResult<usize> {
        Ok(counts::syllable_count(text))
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
    fn polysyllabcount(text: &str) -> PyResult<usize> {
        Ok(counts::polysyllable_word_count(text))
    }
}

mod data;
