use pyo3::{exceptions::PyRuntimeError, prelude::*};
use std::string::String as StdString;
use pyo3::types::PyBytes;
use pyo3_stub_gen::{derive::{gen_stub_pyclass, gen_stub_pymethods}, impl_stub_type};

#[gen_stub_pyclass]
#[pyclass(frozen)]
/// A handle to a Lua-owned string value.
/// 
/// Can be created using Lua.create_string()
/// 
/// Note: using a string once the Lua state has been closed will result in a exception.
pub struct String {
    pub(crate) value: mluau::String,
}

#[gen_stub_pymethods]
#[pymethods]
impl String {
    /// Returns the string value as a bytes
    fn as_bytes(&self, py: Python<'_>) -> PyObject {
        let value = self.value.as_bytes();
        PyBytes::new(py, &value).into()
    }

    /// Returns the string value as a bytes with the null terminator
    fn as_bytes_with_nul(&self, py: Python<'_>) -> PyObject {
        let value = self.value.as_bytes_with_nul();
        PyBytes::new(py, &value).into()
    }

    /// Returns the string's length in bytes.
    fn __len__(&self) -> usize {
        self.value.as_bytes().len()
    }

    // Define string repr
    fn __str__(&self) -> StdString {
        self.value.to_string_lossy() 
    }

    // Define string repr
    fn __repr__(&self) -> StdString {
        format!("LuaString({:?})", self.value.to_pointer())
    }

    // Equals method for string comparison
    //
    // Currently only supports comparison with Python strings
    // and Lua strings
    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        // Lua string
        if let Ok(data) = other.downcast::<String>() {
            let data = data.get();
            return self.value == data.value;
        }

        // Std string
        if let Ok(data) = other.extract::<StdString>() {
            return self.value.as_bytes() == data.as_bytes();
        }

        // bytes
        if let Ok(data) = other.downcast::<PyBytes>() {
            return self.value.as_bytes() == data.as_bytes();
        }

        return false;
    }

    #[getter]
    /// Returns the pointer to the Lua string value.
    /// 
    /// This pointer cannot be converted back to a Lua string
    /// and is only useful for hashing and debugging.
    fn pointer(&self) -> usize {
        self.value.to_pointer() as usize
    }
}

#[derive(Clone)]
pub enum StringLike<'py> {
    Bytes(Bound<'py, PyBytes>),
    StdString(StdString),
    LuaString(mluau::String),
}

impl<'py> FromPyObject<'py> for StringLike<'py> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        if ob.is_none() {
            return Err(PyRuntimeError::new_err("Cannot convert None to StringLike"));
        }

        if let Ok(data) = ob.downcast::<String>() {
            let value = data.get();
            return Ok(StringLike::LuaString(value.value.clone()))
        }

        if let Ok(data) = ob.extract::<StdString>() {
            return Ok(StringLike::StdString(data))
        }

        if let Ok(data) = ob.downcast::<PyBytes>() {
            return Ok(StringLike::Bytes(data.clone()))
        }

        let type_name = ob.get_type().name()?;
        Err(PyRuntimeError::new_err(format!(
            "Could not convert Python object of type '{type_name}' to a StringLike. Must be one of String, str (python string), bytes.",
        )))
    }
}

impl_stub_type!(StringLike<'_> = String | StdString | PyBytes);