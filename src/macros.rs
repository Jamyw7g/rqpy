#[macro_export]
macro_rules! val {
    ($res:expr) => {
        match $res {
            Ok(res) => res,
            Err(e) => {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("{}", e),
                ))
            }
        }
    };
}
