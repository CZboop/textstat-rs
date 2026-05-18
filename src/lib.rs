use pyo3::prelude::*;
use rust_readability;

/// A Python module implemented in Rust.
#[pymodule]
mod textstat_rs {
    use pyo3::prelude::*;

    // map rust readability funcs to textstat i/o
    #[pyfunction]
    fn flesch_reading_ease(text: &str) -> PyResult<f64> {
        Ok(rust_readability::flesch(text))
    }

    #[pyfunction]
    fn flesch_kincaid_grade(text: &str) -> PyResult<f64> {
        Ok(rust_readability::flesch_kincaid(text))
    }

    #[pyfunction]
    fn automated_readability_index(text: &str) -> PyResult<f64> {
        Ok(rust_readability::ari(text))
    }

    #[pyfunction]
    fn coleman_liau_index(text: &str) -> PyResult<f64> {
        Ok(rust_readability::coleman_liau(text))
    }

    #[pyfunction]
    fn linsear_write_formula(text: &str) -> PyResult<f64> {
        Ok(rust_readability::linsear_write(text))
    }
}
