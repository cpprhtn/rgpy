use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyo3::wrap_pyfunction;
use std::fs::File;
use std::io::{BufReader, BufRead};
use walkdir::WalkDir;

trait Matcher: Send + Sync {
    fn is_match(&self, text: &str) -> bool;
}

struct RustRegexMatcher {
    re: regex::Regex,
}

impl Matcher for RustRegexMatcher {
    fn is_match(&self, text: &str) -> bool {
        self.re.is_match(text)
    }
}

#[cfg(feature = "pcre")]
struct Pcre2Matcher {
    re: pcre2::bytes::Regex,
}

#[cfg(feature = "pcre")]
impl Matcher for Pcre2Matcher {
    fn is_match(&self, text: &str) -> bool {
        self.re.is_match(text.as_bytes()).unwrap_or(false)
    }
}

#[pyclass]
pub struct MatchEntry {
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub line_number: usize,
    #[pyo3(get)]
    pub text: String,
}

#[pyfunction]
fn search_file(
    pattern: &str,
    path: &str,
    ignore_case: Option<bool>,
    engine: Option<&str>,
    count: Option<bool>,
    invert_match: Option<bool>,
) -> PyResult<PyObject> {
    let use_pcre = engine.unwrap_or("regex") == "pcre2";
    let pattern = if ignore_case.unwrap_or(false) {
        format!("(?i){}", pattern)
    } else {
        pattern.to_string()
    };
    let invert = invert_match.unwrap_or(false);

    let matcher: Box<dyn Matcher> = if use_pcre {
        #[cfg(feature = "pcre")]
        {
            let re = pcre2::bytes::Regex::new(&pattern)
                .map_err(|e| PyValueError::new_err(format!("Invalid PCRE pattern: {}", e)))?;
            Box::new(Pcre2Matcher { re })
        }
        #[cfg(not(feature = "pcre"))]
        {
            return Err(PyValueError::new_err("pcre2 engine not enabled"));
        }
    } else {
        let re = regex::Regex::new(&pattern)
            .map_err(|e| PyValueError::new_err(format!("Invalid regex: {}", e)))?;
        Box::new(RustRegexMatcher { re })
    };

    let file = File::open(path)
        .map_err(|e| PyValueError::new_err(format!("Failed to open file: {}", e)))?;
    let reader = BufReader::new(file);

    let mut results = vec![];
    for (i, line) in reader.lines().enumerate() {
        if let Ok(text) = line {
            let matched = matcher.is_match(&text);
            if matched ^ invert {
                results.push(MatchEntry {
                    path: path.to_string(),
                    line_number: i + 1,
                    text,
                });
            }
        }
    }

    Python::with_gil(|py| {
        if count.unwrap_or(false) {
            Ok((results.len() as u64).into_py(py))
        } else {
            Ok(results.into_py(py))
        }
    })
}

#[pyfunction]
fn search_dir(
    pattern: &str,
    dir: &str,
    ignore_case: Option<bool>,
    engine: Option<&str>,
    count: Option<bool>,
    invert_match: Option<bool>,
) -> PyResult<PyObject> {
    let use_pcre = engine.unwrap_or("regex") == "pcre2";
    let pattern = if ignore_case.unwrap_or(false) {
        format!("(?i){}", pattern)
    } else {
        pattern.to_string()
    };
    let invert = invert_match.unwrap_or(false);

    let matcher: Box<dyn Matcher> = if use_pcre {
        #[cfg(feature = "pcre")]
        {
            let re = pcre2::bytes::Regex::new(&pattern)
                .map_err(|e| PyValueError::new_err(format!("Invalid PCRE pattern: {}", e)))?;
            Box::new(Pcre2Matcher { re })
        }
        #[cfg(not(feature = "pcre"))]
        {
            return Err(PyValueError::new_err("pcre2 engine not enabled"));
        }
    } else {
        let re = regex::Regex::new(&pattern)
            .map_err(|e| PyValueError::new_err(format!("Invalid regex: {}", e)))?;
        Box::new(RustRegexMatcher { re })
    };

    let mut results = vec![];
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path_str = entry.path().display().to_string();
        if let Ok(file) = File::open(entry.path()) {
            let reader = BufReader::new(file);
            for (i, line) in reader.lines().enumerate() {
                if let Ok(text) = line {
                    let matched = matcher.is_match(&text);
                    if matched ^ invert {
                        results.push(MatchEntry {
                            path: path_str.clone(),
                            line_number: i + 1,
                            text,
                        });
                    }
                }
            }
        }
    }

    Python::with_gil(|py| {
        if count.unwrap_or(false) {
            Ok((results.len() as u64).into_py(py))
        } else {
            Ok(results.into_py(py))
        }
    })
}

#[pymodule]
fn rgpy(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<MatchEntry>()?;
    m.add_function(wrap_pyfunction!(search_file, m)?)?;
    m.add_function(wrap_pyfunction!(search_dir, m)?)?;
    Ok(())
}
