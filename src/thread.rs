use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

#[gen_stub_pyclass]
#[pyclass(frozen)]
pub struct Thread {
    pub(crate) thread: mluau::Thread,
}

// TODO: Implement methods for Thread