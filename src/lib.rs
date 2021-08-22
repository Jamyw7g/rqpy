use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::client::{RSCert, RSClient};

mod client;
mod macros;
mod response;

#[pymodule]
#[pyo3(name = "rqpy")]
fn main_module(_py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, kwargs = "**")]
    #[pyo3(text_signature = "(url, /, *, **kwargs)")]
    fn get<'p>(py: Python<'p>, url: &str, kwargs: Option<&PyDict>) -> PyResult<&'p PyAny> {
        let client = RSClient::new(kwargs)?;
        client.request(py, "GET", url, kwargs)
    }

    #[pyfn(m, kwargs = "**")]
    #[pyo3(text_signature = "(url, /, *, **kwargs)")]
    fn post<'p>(py: Python<'p>, url: &str, kwargs: Option<&PyDict>) -> PyResult<&'p PyAny> {
        let client = RSClient::new(kwargs)?;
        client.request(py, "POST", url, kwargs)
    }

    #[pyfn(m, kwargs = "**")]
    #[pyo3(text_signature = "(url, /, *, **kwargs)")]
    fn put<'p>(py: Python<'p>, url: &str, kwargs: Option<&PyDict>) -> PyResult<&'p PyAny> {
        let client = RSClient::new(kwargs)?;
        client.request(py, "PUT", url, kwargs)
    }

    #[pyfn(m, kwargs = "**")]
    #[pyo3(text_signature = "(url, /, *, **kwargs)")]
    fn head<'p>(py: Python<'p>, url: &str, kwargs: Option<&PyDict>) -> PyResult<&'p PyAny> {
        let client = RSClient::new(kwargs)?;
        client.request(py, "HEAD", url, kwargs)
    }

    #[pyfn(m, kwargs = "**")]
    #[pyo3(text_signature = "(url, /, *, **kwargs)")]
    fn options<'p>(py: Python<'p>, url: &str, kwargs: Option<&PyDict>) -> PyResult<&'p PyAny> {
        let client = RSClient::new(kwargs)?;
        client.request(py, "OPTIONS", url, kwargs)
    }

    #[pyfn(m, kwargs = "**")]
    #[pyo3(text_signature = "(url, /, *, **kwargs)")]
    fn delete<'p>(py: Python<'p>, url: &str, kwargs: Option<&PyDict>) -> PyResult<&'p PyAny> {
        let client = RSClient::new(kwargs)?;
        client.request(py, "DELETE", url, kwargs)
    }

    #[pyfn(m, kwargs = "**")]
    #[pyo3(text_signature = "(url, /, *, **kwargs)")]
    fn trace<'p>(py: Python<'p>, url: &str, kwargs: Option<&PyDict>) -> PyResult<&'p PyAny> {
        let client = RSClient::new(kwargs)?;
        client.request(py, "TRACE", url, kwargs)
    }

    m.add("H09", 0)?;
    m.add("H10", 1)?;
    m.add("H11", 2)?;
    m.add("H2", 3)?;
    m.add("H3", 4)?;

    m.add_class::<RSClient>()?;
    m.add_class::<RSCert>()?;
    Ok(())
}
