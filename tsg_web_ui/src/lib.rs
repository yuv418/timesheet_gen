use std::path::PathBuf;

use pyo3::{exceptions::PyValueError, prelude::*};
use timesheet_gen::timesheet_generator::{self, TimesheetOutputFormat};

// Info is JSON. Great signature, I know
#[pyfunction]
fn gen_ts(filename: String, info: String, output_fmt: String, output_path: String) -> PyResult<()> {
    let output_fmt = if output_fmt == "png" {
        TimesheetOutputFormat::Png(PathBuf::from(output_path))
    } else if output_fmt == "pdf" {
        TimesheetOutputFormat::Pdf(PathBuf::from(output_path))
    } else {
        return Err(PyValueError::new_err("output_fmt is not 'png' or 'pdf'"));
    };

    timesheet_generator::generate_timesheet(
        PathBuf::from(filename),
        serde_json::from_str(&info).map_err(|v| PyValueError::new_err(v.to_string()))?,
        output_fmt,
    )
    .map_err(|v| PyValueError::new_err(v.to_string()))?;

    Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
fn tsg_web_ui(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(gen_ts, m)?)?;
    Ok(())
}
