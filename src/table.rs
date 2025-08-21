use pyo3::{exceptions::PyRuntimeError, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

#[gen_stub_pyclass]
#[pyclass(frozen)]
pub struct Table {
    pub(crate) table: mluau::Table,
}

// TODO: Implement methods for Table