use pyo3::prelude::*;

mod counts;
mod formulas;
mod syllable;
mod tokenize;
mod transform;

/// A Python module implemented in Rust.
#[pymodule]
mod textstat_rs {
    use crate::formulas;
    use pyo3::prelude::*;

    #[pyfunction]
    fn flesch_reading_ease(text: &str) -> PyResult<f64> {
        Ok(formulas::flesch_reading_ease(text))
    }

    #[pyfunction]
    fn flesch_kincaid_grade(text: &str) -> PyResult<f64> {
        Ok(formulas::flesch_kincaid_grade(text))
    }

    #[pyfunction]
    fn automated_readability_index(text: &str) -> PyResult<f64> {
        Ok(formulas::automated_readability_index(text))
    }

    #[pyfunction]
    fn coleman_liau_index(text: &str) -> PyResult<f64> {
        Ok(formulas::coleman_liau_index(text))
    }

    #[pyfunction]
    #[pyo3(signature = (text, strict_lower=false, strict_upper=true))]
    fn linsear_write_formula(text: &str, strict_lower: bool, strict_upper: bool) -> PyResult<f64> {
        Ok(formulas::linsear_write_formula(text, strict_lower, strict_upper))
    }

    #[pyfunction]
    #[pyo3(signature = (text, ms_per_char=14.69))]
    fn reading_time(text: &str, ms_per_char: f64) -> PyResult<f64> {
        Ok(formulas::reading_time(text, ms_per_char))
    }

    #[pyfunction]
    fn mcalpine_eflaw(text: &str) -> PyResult<f64> {
        Ok(formulas::mcalpine_eflaw(text))
    }

    #[pyfunction]
    fn spache_readability(text: &str) -> PyResult<f64> {
        Ok(formulas::spache_readability(text))
    }

    #[pyfunction]
    fn dale_chall_readability_score(text: &str) -> PyResult<f64> {
        Ok(formulas::dale_chall_readability_score(text))
    }

    #[pyfunction]
    #[pyo3(signature = (text, syllable_threshold=3))]
    fn gunning_fog(text: &str, syllable_threshold: usize) -> PyResult<f64> {
        Ok(formulas::gunning_fog(text, syllable_threshold))
    }

    #[pyfunction]
    fn smog_index(text: &str) -> PyResult<f64> {
        Ok(formulas::smog_index(text))
    }

    #[pyfunction]
    fn text_standard(text: &str) -> PyResult<String> {
        Ok(formulas::text_standard(text))
    }
}

mod data;
