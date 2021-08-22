use std::{collections::HashMap, convert::TryInto, str::FromStr, time::Duration};

use pyo3::exceptions::PyValueError;
use pyo3::{exceptions::PyTypeError, prelude::*, types::PyDict};
use reqwest::multipart::{Form, Part};
use reqwest::{redirect::Policy, Client, Method, Proxy};
use reqwest::{Certificate, Version};

use crate::response::RSResponse;
use crate::val;

#[pyclass]
#[derive(Clone)]
pub struct RSCert(Certificate);

#[pymethods]
impl RSCert {
    #[staticmethod]
    pub fn from_der(der: &[u8]) -> PyResult<Self> {
        match Certificate::from_der(der) {
            Ok(res) => Ok(Self(res)),
            Err(e) => Err(PyErr::new::<PyValueError, _>(format!("{}", e))),
        }
    }

    #[staticmethod]
    pub fn from_pem(pem: &[u8]) -> PyResult<Self> {
        match Certificate::from_pem(pem) {
            Ok(res) => Ok(Self(res)),
            Err(e) => Err(PyErr::new::<PyValueError, _>(format!("{}", e))),
        }
    }
}

#[pyclass]
pub struct RSClient(pub(crate) Client);

#[pymethods]
impl RSClient {
    #[new]
    #[args(kwargs = "**")]
    pub fn new(kwargs: Option<&PyDict>) -> PyResult<Self> {
        let mut builder = Client::builder();
        if let Some(kwargs) = kwargs {
            if let Some(proxies) = kwargs.get_item("proxies") {
                let proxies: &PyDict = proxies.downcast()?;
                for (k, v) in proxies.iter() {
                    let k = k.extract::<&str>()?;
                    let v = v.extract::<&str>()?;
                    let p = match k {
                        "http" => val!(Proxy::http(v)),
                        "https" => val!(Proxy::https(v)),
                        "all" => val!(Proxy::all(v)),
                        _ => {
                            return Err(PyErr::new::<PyValueError, _>(format!(
                                "No support {}:{}",
                                k, v
                            )))
                        }
                    };
                    builder = builder.proxy(p);
                }
            }
            if let Some(headers) = kwargs.get_item("headers") {
                let headers = &headers.extract::<HashMap<String, String>>()?;
                let headers = match headers.try_into() {
                    Ok(res) => res,
                    Err(e) => return Err(PyErr::new::<PyTypeError, _>(format!("{}", e))),
                };
                builder = builder.default_headers(headers);
            }
            if let Some(timeout) = kwargs.get_item("timeout") {
                builder = builder.timeout(Duration::from_millis(timeout.extract()?));
            }
            if let Some(cookie_store) = kwargs.get_item("cookie_store") {
                builder = builder.cookie_store(cookie_store.extract()?);
            }
            if let Some(limit) = kwargs.get_item("redirect") {
                builder = builder.redirect(Policy::limited(limit.extract()?));
            } else {
                builder = builder.redirect(Policy::default());
            }
            if let Some(cert) = kwargs.get_item("cert") {
                let RSCert(cert) = cert.extract()?;
                builder = builder.add_root_certificate(cert);
            }
        }

        Ok(Self(val!(builder.build())))
    }

    #[args(kwargs = "**")]
    #[pyo3(text_signature = "(method, url, /, *, **kwargs)")]
    pub fn request<'p>(
        &self,
        py: Python<'p>,
        method: &str,
        url: &str,
        kwargs: Option<&PyDict>,
    ) -> PyResult<&'p PyAny> {
        let method = method.to_uppercase();
        let mut builder = self.0.request(val!(Method::from_str(&method)), url);
        if let Some(kwargs) = kwargs {
            if let Some(body) = kwargs.get_item("body") {
                builder = builder.body(body.extract::<Vec<u8>>()?);
            }
            if let Some(headers) = kwargs.get_item("headers") {
                let headers = &headers.extract::<HashMap<String, String>>()?;
                let headers = match headers.try_into() {
                    Ok(res) => res,
                    Err(e) => return Err(PyErr::new::<PyValueError, _>(format!("{}", e))),
                };
                builder = builder.headers(headers);
            }
            if let Some(usr) = kwargs.get_item("usr") {
                let usr = usr.extract::<&str>()?;
                let pwd = if let Some(pwd) = kwargs.get_item("pwd") {
                    Some(pwd.extract::<&str>()?)
                } else {
                    None
                };
                builder = builder.basic_auth(usr, pwd);
            }
            if let Some(token) = kwargs.get_item("token") {
                builder = builder.bearer_auth(token.extract::<&str>()?);
            }
            if let Some(timeout) = kwargs.get_item("timeout") {
                builder = builder.timeout(Duration::from_millis(timeout.extract()?));
            }
            if let Some(query) = kwargs.get_item("query") {
                let query = query.extract::<Vec<(&str, &str)>>()?;
                builder = builder.query(&query);
            }
            if let Some(data) = kwargs.get_item("multipart") {
                let data = data.downcast()?;
                let part = build_multipart(data)?;
                builder = builder.multipart(part);
            }
            if let Some(form) = kwargs.get_item("form") {
                let form = &form.extract::<HashMap<String, String>>()?;
                builder = builder.form(form);
            }
            if let Some(version) = kwargs.get_item("version") {
                let ver = version.extract::<i32>()?;
                let ver = match ver {
                    0 => Version::HTTP_09,
                    1 => Version::HTTP_10,
                    2 => Version::HTTP_11,
                    3 => Version::HTTP_2,
                    4 => Version::HTTP_3,
                    _ => {
                        return Err(PyErr::new::<PyValueError, _>(
                            "Just support HTTP/0.9,/1.0,/1.1,/2.0,/3.0",
                        ))
                    }
                };
                builder = builder.version(ver);
            }
        };

        let req = val!(builder.build());
        let fut = self.0.execute(req);
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let resp = val!(fut.await);
            let resp = RSResponse(Some(resp));
            Ok(Python::with_gil(|py| resp.into_py(py)))
        })
    }
}

fn build_multipart(data: &PyDict) -> PyResult<Form> {
    let mut form = Form::new();
    if let Some(texts) = data.get_item("text") {
        let texts: &PyDict = texts.downcast()?;
        for (name, value) in texts.iter() {
            let name: String = name.extract()?;
            let value: String = value.extract()?;
            form = form.text(name, value);
        }
    }
    if let Some(parts) = data.get_item("part") {
        let parts: &PyDict = parts.downcast()?;
        for (name, part) in parts {
            let name: String = name.extract()?;
            let part: &PyDict = part.downcast()?;
            if let Some(bytes) = part.get_item("bytes") {
                let bytes: Vec<u8> = bytes.extract()?;
                let mut data = Part::bytes(bytes);
                if let Some(mime) = part.get_item("mime") {
                    data = val!(data.mime_str(mime.extract()?));
                }
                if let Some(filename) = part.get_item("filename") {
                    data = data.file_name(filename.extract::<String>()?);
                }
                form = form.part(name, data);
            }
        }
    }
    if let Some(encode) = data.get_item("encode") {
        let encode: &PyDict = encode.downcast()?;
        if let Some(path_segment) = encode.get_item("path-segment") {
            if path_segment.is_true()? {
                form = form.percent_encode_path_segment();
            }
        }
        if let Some(attr_char) = encode.get_item("attr-char") {
            if attr_char.is_true()? {
                form = form.percent_encode_attr_chars();
            }
        }
        if let Some(noop) = encode.get_item("noop") {
            if noop.is_true()? {
                form = form.percent_encode_noop();
            }
        }
    }

    Ok(form)
}
