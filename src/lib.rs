use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyo3::wrap_pyfunction;
use std::fs::File;
use std::io::{BufReader, BufRead};
use walkdir::WalkDir;
use rayon::prelude::*;

#[derive(Clone)]
enum MatcherType {
    Regex(regex::Regex),
    #[cfg(feature = "pcre")]
    Pcre2(pcre2::bytes::Regex),
}

impl MatcherType {
    fn is_match(&self, text: &str) -> bool {
        match self {
            MatcherType::Regex(re) => re.is_match(text),
            #[cfg(feature = "pcre")]
            MatcherType::Pcre2(re) => re.is_match(text.as_bytes()).unwrap_or(false),
        }
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

#[pyclass]
pub struct MatcherWrapper {
    matcher: MatcherType,
}

#[pymethods]
impl MatcherWrapper {
    #[new]
    fn new(pattern: &str, ignore_case: Option<bool>, engine: Option<&str>) -> PyResult<Self> {
        let ignore_case = ignore_case.unwrap_or(false);
        let use_pcre = engine.unwrap_or("regex") == "pcre2";

        let matcher = if use_pcre {
            #[cfg(feature = "pcre")]
            {
                let re = pcre2::bytes::RegexBuilder::new()
                    .caseless(ignore_case)
                    .build(pattern)
                    .map_err(|e| PyValueError::new_err(format!("Invalid PCRE pattern: {}", e)))?;
                MatcherType::Pcre2(re)
            }
            #[cfg(not(feature = "pcre"))]
            {
                return Err(PyValueError::new_err("pcre2 engine not enabled"));
            }
        } else {
            let re = regex::RegexBuilder::new(pattern)
                .case_insensitive(ignore_case)
                .build()
                .map_err(|e| PyValueError::new_err(format!("Invalid regex: {}", e)))?;
            MatcherType::Regex(re)
        };

        Ok(MatcherWrapper { matcher })
    }

    #[pyo3(name = "is_match")]
    pub fn is_match_py(
        &self, 
        text: &str
    ) -> bool {
        self.matcher.is_match(text)
    }

    fn search_file(
        &self,
        path: &str,
        count: Option<bool>,
        invert_match: Option<bool>,
        py: Python,
    ) -> PyResult<PyObject> {
        let file = File::open(path).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let reader = BufReader::new(file);
        let invert = invert_match.unwrap_or(false);

        let results: Vec<MatchEntry> = py.allow_threads(|| {
            reader
                .lines()
                .enumerate()
                .par_bridge()
                .filter_map(|(i, line)| {
                    line.ok().and_then(|text| {
                        let matched = self.matcher.is_match(&text);
                        if matched ^ invert {
                            Some(MatchEntry {
                                path: path.to_string(),
                                line_number: i + 1,
                                text,
                            })
                        } else {
                            None
                        }
                    })
                })
                .collect()
        });

        if count.unwrap_or(false) {
            Ok((results.len() as u64).into_py(py))
        } else {
            Ok(results.into_py(py))
        }
    }

    fn search_dir(
        &self,
        dir: &str,
        count: Option<bool>,
        invert_match: Option<bool>,
        py: Python,
    ) -> PyResult<PyObject> {
        let invert = invert_match.unwrap_or(false);
        let matcher = self.matcher.clone();

        let entries: Vec<_> = WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().display().to_string())
            .collect();

        let results: Vec<MatchEntry> = py.allow_threads(|| {
            entries
                .par_iter()
                .flat_map(|path_str| {
                    let file = File::open(path_str);
                    if let Ok(file) = file {
                        let reader = BufReader::new(file);
                        reader
                            .lines()
                            .enumerate()
                            .filter_map(|(i, line)| {
                                line.ok().and_then(|text| {
                                    let matched = matcher.is_match(&text);
                                    if matched ^ invert {
                                        Some(MatchEntry {
                                            path: path_str.clone(),
                                            line_number: i + 1,
                                            text,
                                        })
                                    } else {
                                        None
                                    }
                                })
                            })
                            .collect()
                    } else {
                        vec![]
                    }
                })
                .collect()
        });

        if count.unwrap_or(false) {
            Ok((results.len() as u64).into_py(py))
        } else {
            Ok(results.into_py(py))
        }
    }
}

#[pyfunction]
fn compile(pattern: &str, ignore_case: Option<bool>, engine: Option<&str>) -> PyResult<MatcherWrapper> {
    MatcherWrapper::new(pattern, ignore_case, engine)
}

#[pyfunction]
fn search_file(
    pattern: &str,
    path: &str,
    ignore_case: Option<bool>,
    engine: Option<&str>,
    count: Option<bool>,
    invert_match: Option<bool>,
    py: Python,
) -> PyResult<PyObject> {
    compile(pattern, ignore_case, engine)?.search_file(path, count, invert_match, py)
}

#[pyfunction]
fn search_dir(
    pattern: &str,
    dir: &str,
    ignore_case: Option<bool>,
    engine: Option<&str>,
    count: Option<bool>,
    invert_match: Option<bool>,
    py: Python,
) -> PyResult<PyObject> {
    compile(pattern, ignore_case, engine)?.search_dir(dir, count, invert_match, py)
}

#[pymodule]
fn rgpy(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<MatchEntry>()?;
    m.add_class::<MatcherWrapper>()?;
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_function(wrap_pyfunction!(search_file, m)?)?;
    m.add_function(wrap_pyfunction!(search_dir, m)?)?;
    Ok(())
}
