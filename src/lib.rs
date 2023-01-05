use pyo3::exceptions;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use numpy::{ToPyArray, PyArray, IntoPyArray, PyArray2, PyReadonlyArray2};
use crate::data::{DrawType, EntryError, PlaceEntry, Rect};

mod data;
mod parser;

#[pymodule]
fn r_place_data(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(data_head, m)?)?;
    m.add_function(wrap_pyfunction!(pixel_frequencies, m)?)?;
    m.add_function(wrap_pyfunction!(pixel_frequencies_nom, m)?)?;
    m.add_function(wrap_pyfunction!(find_rects, m)?)?;
    m.add_function(wrap_pyfunction!(find_before_rects, m)?)?;
    m.add_class::<PyPlaceData>()?;
    m.add_class::<PyPlaceEntry>()?;
    Ok(())
}

#[pyfunction]
fn data_head(n: usize) -> PyResult<Vec<String>> {
    Ok(data::PlaceData::new().into_iter().take(n)
        .map(|x| x.unwrap().uid_hash)
        .collect()
    )
}

#[pyfunction]
fn pixel_frequencies(print_status: usize) -> Py<PyArray2<usize>> {
    let frequencies = data::ParallelPlaceData::pixel_frequencies(print_status);
    Python::with_gil(|py| frequencies.into_pyarray(py).to_owned())
}

#[pyfunction]
fn pixel_frequencies_nom(print_status: usize) -> Py<PyArray2<usize>> {
    let frequencies = data::ParallelPlaceData::pixel_frequencies_nom(print_status);
    Python::with_gil(|py| frequencies.into_pyarray(py).to_owned())
}

#[pyfunction]
fn find_rects() -> Vec<PyPlaceEntry> {
    data::ParallelPlaceData::find_rects().into_iter().map(|x| x.into()).collect()
}

#[pyfunction]
fn find_before_rects(rects: Vec<(Rect, i64)>) -> Vec<(Rect, Vec<Vec<Option<PyPlaceEntry>>>)> {
    data::ParallelPlaceData::find_before_rects(&rects).into_iter()
        .map(|(rect, pixels)| (
            rect,
            pixels.axis_iter(ndarray::Axis(1))
                .map(|x| x.into_iter().map(|y| y.clone().map(PyPlaceEntry::from)).collect())
                .collect()
        ))
        .collect()
}

#[pyclass(name="PlaceData")]
struct PyPlaceData {
    inner: data::PlaceData,
}

#[pymethods]
impl PyPlaceData {
    #[new]
    fn __new__() -> Self {
        Self { inner: data::PlaceData::new() }
    }

    fn __iter__(&self) -> Self {
        Self::__new__()
    }

    fn __next__(&mut self) -> PyResult<Option<PyPlaceEntry>> {
        self.inner.next().map(|x| match x {
            Ok(y) => Ok(y.into()),
            Err(err) => Err(err.into()),
        })
            .transpose()
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(data::PlaceData::new().into_iter()
            .map(|x| x.map_err::<PyErr, _>(|y| y.into()))
            .count()
        )
    }
}

#[pyclass]
struct PyPlaceEntry {
    #[pyo3(get)]
    timestamp: i64,
    #[pyo3(get)]
    uid_hash: String,
    #[pyo3(get)]
    color: String,
    inner_loc: data::DrawType,
    #[pyo3(get)]
    is_rect: bool,
}

#[pymethods]
impl PyPlaceEntry {
    #[getter]
    fn location(&self) -> PyObject {
        Python::with_gil(|py| match self.inner_loc {
            DrawType::Pixel(x, y) => PyTuple::new(py, vec![x, y]).into(),
            DrawType::Rect(x1, y1, x2, y2) => PyTuple::new(py, vec![x1, y1, x2, y2]).into(),
        })
    }
}

impl From<data::PlaceEntry> for PyPlaceEntry{
    fn from(entry: PlaceEntry) -> Self {
        let is_rect = matches!(entry.location, data::DrawType::Rect(_, _, _, _));
        Self {
            timestamp: entry.timestamp.timestamp_millis(),
            uid_hash: entry.uid_hash,
            color: entry.color,
            inner_loc: entry.location,
            is_rect
        }
    }
}

impl Into<PyErr> for data::EntryError {
    fn into(self) -> PyErr {
        match self {
            EntryError::ParseError { entry } => exceptions::PyValueError::new_err(
                format!("failed to parse line: '{}'", entry)
            ),
            EntryError::DateTimeParseError(err) => exceptions::PyValueError::new_err(
                format!("error parsing datetime: {}", err)
            ),
            EntryError::IoError(err) => exceptions::PyIOError::new_err(
                format!("io error reading line: {}", err)
            ),
        }
    }
}