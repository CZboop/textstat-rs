use pyo3::prelude::*;

mod counts;
mod formulas;
mod syllable;
mod tokenize;

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
    fn linsear_write_formula(text: &str) -> PyResult<f64> {
        Ok(formulas::linsear_write_formula(text))
    }
}
