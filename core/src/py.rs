use pyo3::prelude::*;
/// Accept inch/degree contract JSON and return structured output or an error status.
#[pyfunction]
fn generate_json(input_json: &str) -> String {
    crate::generate_json(input_json)
}
#[pymodule]
fn vstone_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate_json, m)?)?;
    Ok(())
}
