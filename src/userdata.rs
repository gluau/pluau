use pyo3::{exceptions::PyRuntimeError, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

#[gen_stub_pyclass]
#[pyclass(frozen)]
pub struct UserData {
    pub(crate) ud: mluau::AnyUserData,
}

// TODO: Implement methods for UserData
#[gen_stub_pymethods]
#[pymethods]
impl UserData {
    #[getter]
    /// Returns the pointer/address of the userdata.
    fn pointer(&self) -> usize {
        self.ud.to_pointer() as usize
    }

    /// Returns the underlying associated data of the userdata.
    ///
    /// Errors if the userdata does not contain associated data (did not originate from Python)
    fn data(&self, py: Python<'_>) -> PyResult<PyObject> {
        match self.ud.dynamic_data::<PyObject>() {
            Ok(data) => Ok(data.clone_ref(py)),
            Err(e) => Err(PyRuntimeError::new_err(format!(
                "Failed to get associated data: {}",
                e
            ))),
        }
    }

    /// Returns the underlying metatable of the userdata.
    fn metatable(&self) -> PyResult<crate::table::Table> {
        // SAFETY: Luau does not have __gc metamethod, so this is safe to call here.
        let mt = unsafe { self.ud.underlying_metatable() };
        match mt {
            Ok(mt) => Ok(crate::table::Table { table: mt }),
            Err(e) => Err(PyRuntimeError::new_err(format!(
                "Failed to get metatable: {}",
                e
            ))),
        }
    }

    // Define string repr
    fn __repr__(&self) -> String {
        format!("UserData({:?})", self.ud.to_pointer())
    }

    fn __eq__(&self, other: &UserData) -> bool {
        self.ud == other.ud
    }
}