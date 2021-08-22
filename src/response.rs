use pyo3::exceptions::*;
use pyo3::prelude::*;
use pyo3::types::PyString;
use reqwest::Response;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::val;

#[pyclass]
pub struct RSResponse(pub(crate) Option<Response>);

#[pymethods]
impl RSResponse {
    pub fn status_code(&self) -> PyResult<u16> {
        if let Some(resp) = self.0.as_ref() {
            Ok(resp.status().as_u16())
        } else {
            Err(PyValueError::new_err("No response."))
        }
    }

    pub fn version(&self) -> PyResult<String> {
        if let Some(resp) = self.0.as_ref() {
            Ok(format!("{:?}", resp.version()))
        } else {
            Err(PyValueError::new_err("No response."))
        }
    }

    pub fn headers(&self) -> PyResult<HashMap<String, String>> {
        if let Some(resp) = self.0.as_ref() {
            let headers = resp.headers();
            Ok(headers
                .iter()
                .map(|(name, val)| {
                    let name = String::from(name.as_str());
                    let val = String::from_utf8_lossy(val.as_bytes()).to_string();
                    (name, val)
                })
                .collect::<HashMap<_, _>>())
        } else {
            Err(PyValueError::new_err("No response."))
        }
    }

    pub fn cookies(&self) -> PyResult<String> {
        if let Some(resp) = self.0.as_ref() {
            let mut cookies = String::new();
            for c in resp.cookies() {
                cookies.push_str(&format!("{:?}\n", c));
            }
            Ok(cookies)
        } else {
            Err(PyValueError::new_err("No response."))
        }
    }

    pub fn content_length(&self) -> PyResult<Option<u64>> {
        if let Some(resp) = self.0.as_ref() {
            Ok(resp.content_length())
        } else {
            Err(PyValueError::new_err("No response."))
        }
    }

    #[args(encoding = "\"utf-8\"")]
    pub fn text_with_charset<'p>(&mut self, py: Python<'p>, encoding: &str) -> PyResult<&'p PyAny> {
        if let Some(resp) = self.0.take() {
            let encoding = String::from(encoding);
            pyo3_asyncio::tokio::future_into_py(py, async move {
                if let Ok(txt) = resp.text_with_charset(&encoding).await {
                    Ok(Python::with_gil(|py| txt.into_py(py)))
                } else {
                    Err(PyValueError::new_err("No response."))
                }
            })
        } else {
            Err(PyValueError::new_err("No response."))
        }
    }

    pub fn bytes<'p>(&mut self, py: Python<'p>) -> PyResult<&'p PyAny> {
        if let Some(resp) = self.0.take() {
            pyo3_asyncio::tokio::future_into_py(py, async move {
                if let Ok(txt) = resp.bytes().await {
                    Ok(Python::with_gil(|py| txt.into_py(py)))
                } else {
                    Err(PyValueError::new_err("No response."))
                }
            })
        } else {
            Err(PyValueError::new_err("No response."))
        }
    }

    pub fn write_bytes<'p>(&mut self, py: Python<'p>, dst: &PyAny) -> PyResult<&'p PyAny> {
        if dst.is_instance::<PyString>()? || dst.hasattr("write")? {
            if let Some(mut resp) = self.0.take() {
                if dst.hasattr("write")? {
                    let writer = dst.into_py(py.clone());
                    pyo3_asyncio::tokio::future_into_py(py, async move {
                        let mut total = 0;
                        while let Some(chunk) = val!(resp.chunk().await) {
                            Python::with_gil(|py| writer.call_method1(py, "write", (&*chunk,)))?;
                            total += chunk.len();
                        }
                        Ok(Python::with_gil(|py| total.into_py(py)))
                    })
                } else {
                    let path = dst.extract::<PathBuf>()?;
                    pyo3_asyncio::tokio::future_into_py(py, async move {
                        let mut buf_writer = BufWriter::new(File::create(path).await?);
                        let mut total = 0;
                        while let Some(chunk) = val!(resp.chunk().await) {
                            buf_writer.write_all(&chunk).await?;
                            total += chunk.len();
                        }
                        buf_writer.flush().await?;
                        Ok(Python::with_gil(|py| total.into_py(py)))
                    })
                }
            } else {
                Err(PyValueError::new_err("No response."))
            }
        } else {
            Err(PyTypeError::new_err("Input an IO instance or a path"))
        }
    }

    pub fn write_with_callback<'p>(&mut self, py: Python<'p>, cb: &PyAny) -> PyResult<&'p PyAny> {
        if cb.is_callable() {
            if let Some(mut resp) = self.0.take() {
                let cb = cb.into_py(py);
                pyo3_asyncio::tokio::future_into_py(py, async move {
                    let total = resp.content_length().unwrap_or_default();
                    while let Some(chunk) = val!(resp.chunk().await) {
                        Python::with_gil(|py| cb.call(py, (&*chunk, total), None))?;
                    }
                    Ok(Python::with_gil(|py| py.None()))
                })
            } else {
                Err(PyValueError::new_err("No response."))
            }
        } else {
            Err(PyTypeError::new_err("Input a callable function."))
        }
    }
}
