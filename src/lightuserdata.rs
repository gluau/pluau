use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

#[gen_stub_pyclass]
#[pyclass(frozen)]
pub struct LightUserData {
    pub lud: mluau::LightUserData
}

#[gen_stub_pymethods]
#[pymethods]
impl LightUserData {
    #[getter]
    /// Returns the pointer/address of the light userdata.
    fn pointer(&self) -> usize {
        self.lud.0 as usize
    }
}