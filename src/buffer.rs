use pyo3::{exceptions::PyRuntimeError, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

#[gen_stub_pyclass]
#[pyclass(frozen)]
pub struct Buffer {
    pub(crate) buf: mluau::Buffer,
}

// TODO: Implement methods for Buffer