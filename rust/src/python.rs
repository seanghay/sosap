//! PyO3 bindings. Built only when the `python` cargo feature is on
//! (which maturin enables for `pip install`).
//!
//! Public surface mirrors the upstream `PhonetisaurusScript` C++ class:
//!   - `Model(path, delim="")` constructor
//!   - `model.phoneticize(word)` -> `list[str]`  (backward-compat with old Cython binding)
//!   - `model.phoneticize_paths(word, nbest=1, beam=10000, threshold=99.0,
//!         write_fsts=False, accumulate=False, pmass=99.0)` -> `list[PathData]`
//!   - `model.find_isym(name|id)`, `model.find_osym(name|id)`

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyInt, PyString};

use crate::{Model as InnerModel, PathData as InnerPathData, PhoneticizeOptions};

#[pyclass(name = "PathData", module = "sosap._sosap", frozen, skip_from_py_object)]
#[derive(Clone)]
struct PyPathData {
    #[pyo3(get)]
    path_weight: f32,
    #[pyo3(get)]
    path_weights: Vec<f32>,
    #[pyo3(get)]
    ilabels: Vec<i32>,
    #[pyo3(get)]
    olabels: Vec<i32>,
    #[pyo3(get)]
    uniques: Vec<i32>,
}

impl From<InnerPathData> for PyPathData {
    fn from(p: InnerPathData) -> Self {
        Self {
            path_weight: p.path_weight,
            path_weights: p.path_weights,
            ilabels: p.ilabels,
            olabels: p.olabels,
            uniques: p.uniques,
        }
    }
}

#[pymethods]
impl PyPathData {
    fn __repr__(&self) -> String {
        format!(
            "PathData(path_weight={:.4}, uniques={:?})",
            self.path_weight, self.uniques
        )
    }
}

#[pyclass(name = "Model", module = "sosap._sosap", unsendable)]
struct PyModel {
    inner: InnerModel,
}

#[pymethods]
impl PyModel {
    #[new]
    #[pyo3(signature = (path, delim=""))]
    fn new(path: &str, delim: &str) -> PyResult<Self> {
        let inner = InnerModel::open(path, delim)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Backward-compatible API matching the original Cython binding:
    /// returns the top-1 path's phoneme strings.
    fn phoneticize(&self, word: &str) -> Vec<String> {
        self.inner.phoneticize_simple(word)
    }

    /// Full upstream Phoneticize() surface. Returns a list of `PathData`,
    /// each with `path_weight`, `path_weights`, `ilabels`, `olabels`, `uniques`.
    #[pyo3(signature = (
        word,
        nbest = 1,
        beam = 10_000,
        threshold = 99.0,
        write_fsts = false,
        accumulate = false,
        pmass = 99.0,
    ))]
    fn phoneticize_paths(
        &self,
        word: &str,
        nbest: usize,
        beam: usize,
        threshold: f32,
        write_fsts: bool,
        accumulate: bool,
        pmass: f64,
    ) -> Vec<PyPathData> {
        let opts = PhoneticizeOptions {
            nbest,
            beam,
            threshold,
            write_fsts,
            accumulate,
            pmass,
        };
        self.inner
            .phoneticize(word, &opts)
            .into_iter()
            .map(PyPathData::from)
            .collect()
    }

    /// Convenience over `phoneticize_paths`: returns each path's phoneme
    /// strings (already cluster-decomposed and veto-filtered) instead of
    /// raw `PathData`. Backward-compatible with the previous Cython binding.
    #[pyo3(signature = (
        word,
        nbest = 1,
        beam = 10_000,
        threshold = 99.0,
        pmass = 99.0,
    ))]
    fn phoneticize_sampling(
        &self,
        word: &str,
        nbest: usize,
        beam: usize,
        threshold: f32,
        pmass: f64,
    ) -> Vec<Vec<String>> {
        let opts = PhoneticizeOptions {
            nbest,
            beam,
            threshold,
            pmass,
            ..Default::default()
        };
        self.inner
            .phoneticize(word, &opts)
            .into_iter()
            .map(|p| {
                p.uniques
                    .iter()
                    .filter_map(|&id| {
                        if id < 0 {
                            None
                        } else {
                            self.inner
                                .osyms()
                                .get_symbol(id as u32)
                                .map(str::to_string)
                        }
                    })
                    .collect()
            })
            .collect()
    }

    /// Look up an input symbol by name or ID. Mirrors `FindIsym` overloads.
    fn find_isym(&self, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        find_sym(key, true, &self.inner)
    }

    /// Look up an output symbol by name or ID. Mirrors `FindOsym` overloads.
    fn find_osym(&self, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        find_sym(key, false, &self.inner)
    }
}

fn find_sym(
    key: &Bound<'_, PyAny>,
    is_input: bool,
    model: &InnerModel,
) -> PyResult<Py<PyAny>> {
    let py = key.py();
    if let Ok(s) = key.cast::<PyString>() {
        let name = s.str()?.to_string();
        let name = name.as_str();
        let id = if is_input {
            model.find_isym_id(name)
        } else {
            model.find_osym_id(name)
        };
        return Ok(id.map_or_else(|| py.None(), |v| v.into_pyobject(py).unwrap().into_any().unbind()));
    }
    if let Ok(i) = key.cast::<PyInt>() {
        let id: i32 = i.extract()?;
        let name = if is_input {
            model.find_isym_name(id)
        } else {
            model.find_osym_name(id)
        };
        return Ok(name.map_or_else(
            || py.None(),
            |s| s.into_pyobject(py).unwrap().into_any().unbind(),
        ));
    }
    Err(PyTypeError::new_err("expected str or int"))
}

#[pymodule]
fn _sosap(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyModel>()?;
    m.add_class::<PyPathData>()?;
    Ok(())
}
