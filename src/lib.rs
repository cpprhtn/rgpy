use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyo3::wrap_pyfunction;
use std::fs::File;
use std::io::{BufReader, BufRead};
use rayon::prelude::*;
use regex::Regex;

#[pyfunction]
fn search_file(
    pattern: &str,
    path: &str,
    ignore_case: Option<bool>,
    engine: Option<&str>,
) -> PyResult<Vec<String>> {
    let use_pcre = engine.unwrap_or("regex") == "pcre2";

    if use_pcre {
        #[cfg(feature = "pcre")]
        {
            use pcre2::bytes::Regex;
            let pattern = if ignore_case.unwrap_or(false) {
                format!("(?i){}", pattern)
            } else {
                pattern.to_string()
            };
            let re = Regex::new(&pattern)
                .map_err(|e| PyValueError::new_err(format!("Invalid PCRE pattern: {}", e)))?;
            let file = File::open(path)
                .map_err(|e| PyValueError::new_err(format!("Failed to open file: {}", e)))?;
            let reader = BufReader::new(file);

            let results: Vec<String> = reader
                .lines()
                .par_bridge()
                .filter_map(Result::ok)
                .filter(|line| re.is_match(line.as_bytes()).unwrap_or(false))
                .collect();

            Ok(results)
        }
        #[cfg(not(feature = "pcre"))]
        {
            Err(PyValueError::new_err(
                "pcre2 engine not enabled. Rebuild with --features pcre",
            ))
        }
    } else {
        let pattern = if ignore_case.unwrap_or(false) {
            format!("(?i){}", pattern)
        } else {
            pattern.to_string()
        };
        let re = Regex::new(&pattern)
            .map_err(|e| PyValueError::new_err(format!("Invalid regex: {}", e)))?;

        let file = File::open(path)
            .map_err(|e| PyValueError::new_err(format!("Failed to open file: {}", e)))?;
        let reader = BufReader::new(file);

        let results: Vec<String> = reader
            .lines()
            .par_bridge()
            .filter_map(Result::ok)
            .filter(|line| re.is_match(line))
            .collect();

        Ok(results)
    }
}

#[pymodule]
fn rgpy(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(search_file, m)?)?;
    Ok(())
}
