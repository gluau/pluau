use mluau::ffi::lua_Integer;
use pyo3::IntoPyObjectExt;
use pyo3::{exceptions::PyRuntimeError, prelude::*};

use mluau::prelude::*;
use pyo3_stub_gen::{impl_stub_type, PyStubType};
use crate::lightuserdata::LightUserData as PyLightUserData;
use crate::vector::Vector as PyLuaVector;
use crate::string::String as PyLuaString;
use crate::table::Table as PyLuaTable;
use crate::function::Function as PyLuaFunction;
use crate::thread::Thread as PyLuaThread;
use crate::userdata::UserData as PyLuaAnyUserData;
use crate::buffer::Buffer as PyLuaBuffer;

/// Intermediate structure that can be extracted to
/// 
/// Needed as we dont have a Lua
pub enum ValueLike {
    Nil,
    Boolean(bool),
    LightUserData(LuaLightUserData),
    Integer(LuaInteger),
    Number(LuaNumber),
    Vector(LuaVector),
    String(LuaString),
    PyString(String),
    Table(LuaTable),
    Function(LuaFunction),
    Thread(LuaThread),
    UserData(LuaAnyUserData),
    Buffer(mluau::Buffer),
}

impl FromLua for ValueLike {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Nil => Ok(ValueLike::Nil),
            LuaValue::Boolean(val) => Ok(ValueLike::Boolean(val)),
            LuaValue::LightUserData(lud) => Ok(ValueLike::LightUserData(lud)),
            LuaValue::Integer(val) => Ok(ValueLike::Integer(val)),
            LuaValue::Number(val) => Ok(ValueLike::Number(val)),
            LuaValue::Vector(vec) => Ok(ValueLike::Vector(vec)),
            LuaValue::String(s) => Ok(ValueLike::String(s)),
            LuaValue::Table(tab) => Ok(ValueLike::Table(tab)),
            LuaValue::Function(func) => Ok(ValueLike::Function(func)),
            LuaValue::Thread(thread) => Ok(ValueLike::Thread(thread)),
            LuaValue::UserData(ud) => Ok(ValueLike::UserData(ud)),
            LuaValue::Buffer(buf) => Ok(ValueLike::Buffer(buf)),
            _ => Ok(ValueLike::Nil), // Default case, should not happen and not implemented yet either
        }
    }
}

impl IntoLua for ValueLike {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        match self {
            Self::Nil => Ok(LuaValue::Nil),
            Self::Boolean(val) => Ok(LuaValue::Boolean(val)),
            Self::LightUserData(lud) => Ok(LuaValue::LightUserData(lud)),
            Self::Integer(val) => Ok(LuaValue::Integer(val)),
            Self::Number(val) => Ok(LuaValue::Number(val)),
            Self::Vector(vec) => Ok(LuaValue::Vector(vec)),
            Self::String(s) => Ok(LuaValue::String(s)),
            Self::PyString(s) => Ok(LuaValue::String(lua.create_string(s)?)),
            Self::Table(tab) => Ok(LuaValue::Table(tab)),
            Self::Function(func) => Ok(LuaValue::Function(func)),
            Self::Thread(thread) => Ok(LuaValue::Thread(thread)),
            Self::UserData(ud) => Ok(LuaValue::UserData(ud)),
            Self::Buffer(buf) => Ok(LuaValue::Buffer(buf)),
        }
    }
}

impl<'py> FromPyObject<'py> for ValueLike {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        if ob.is_none() {
            return Ok(ValueLike::Nil)
        }

        if let Ok(val) = ob.extract::<bool>() {
            return Ok(ValueLike::Boolean(val))
        }

        if let Ok(data) = ob.downcast::<PyLightUserData>() {
            let lud = data.get();
            return Ok(ValueLike::LightUserData(lud.lud.clone()))
        }

        if let Ok(val) = ob.extract::<lua_Integer>() {
            return Ok(ValueLike::Integer(val));
        }
        if let Ok(val) = ob.extract::<f64>() {
            return Ok(ValueLike::Number(val));
        }

        if let Ok(val) = ob.extract::<f64>() {
            return Ok(ValueLike::Number(val));
        }

        if let Ok(data) = ob.downcast::<PyLuaVector>() {
            let value = data.get();
            return Ok(ValueLike::Vector(value.value))
        }

        if let Ok(data) = ob.downcast::<PyLuaString>() {
            let value = data.get();
            return Ok(ValueLike::String(value.value.clone()))
        }

        if let Ok(data) = ob.extract::<String>() {
            return Ok(ValueLike::PyString(data))
        }

        if let Ok(data) = ob.downcast::<PyLuaTable>() {
            let tab = data.get();
            return Ok(ValueLike::Table(tab.table.clone()))
        }

        if let Ok(data) = ob.downcast::<PyLuaFunction>() {
            let func = data.get();
            return Ok(ValueLike::Function(func.function.clone()))
        }

        if let Ok(data) = ob.downcast::<PyLuaThread>() {
            let thread = data.get();
            return Ok(ValueLike::Thread(thread.thread.clone()))
        }

        if let Ok(data) = ob.downcast::<PyLuaAnyUserData>() {
            let ud = data.get();
            return Ok(ValueLike::UserData(ud.ud.clone()))
        }

        if let Ok(data) = ob.downcast::<PyLuaBuffer>() {
            let buf = data.get();
            return Ok(ValueLike::Buffer(buf.buf.clone()))
        }

        let type_name = ob.get_type().name()?;
        Err(PyRuntimeError::new_err(format!(
            "Could not convert Python object of type '{type_name}' to a Luau Value. Must be one of None, bool, LightUserData, i64, f64, Vector, String, str (python string), Table, Function, Thread, UserData or Buffer.",
        )))
    }
}

impl<'py> IntoPyObject<'py> for ValueLike {
    type Error = PyErr;
    type Target = PyAny; // the Python type
    type Output = Bound<'py, Self::Target>; // in most cases this will be `Bound`

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            ValueLike::Nil => Ok(py.None().into_bound(py)),
            ValueLike::Boolean(val) => {
                val.into_pyobject(py)?.into_bound_py_any(py)
            },
            ValueLike::LightUserData(lud) => {
                let py_lud = PyLightUserData { lud };
                py_lud.into_bound_py_any(py)
            },
            ValueLike::Integer(val) => {
                val.into_pyobject(py)?.into_bound_py_any(py)
            },
            ValueLike::Number(val) => {
                val.into_pyobject(py)?.into_bound_py_any(py)
            },
            ValueLike::Vector(vec) => {
                let py_vec = PyLuaVector { value: vec };
                py_vec.into_bound_py_any(py)
            },
            ValueLike::String(s) => {
                let py_string = PyLuaString { value: s };
                py_string.into_bound_py_any(py)
            },
            ValueLike::PyString(s) => {
                s.into_pyobject(py)?.into_bound_py_any(py)
            },
            ValueLike::Table(tab) => {
                let py_table = PyLuaTable { table: tab };
                py_table.into_bound_py_any(py)
            },
            ValueLike::Function(func) => {
                let py_func = PyLuaFunction { function: func };
                py_func.into_bound_py_any(py)
            },
            ValueLike::Thread(thread) => {
                let py_thread = PyLuaThread { thread };
                py_thread.into_bound_py_any(py)
            },
            ValueLike::UserData(ud) => {
                let py_ud = PyLuaAnyUserData { ud };
                py_ud.into_bound_py_any(py)
            },
            ValueLike::Buffer(buf) => {
                let py_buf = PyLuaBuffer { buf };
                py_buf.into_bound_py_any(py)
            },
        }
    }
}

struct DocNone {}
impl PyStubType for DocNone {
    fn type_input() -> pyo3_stub_gen::TypeInfo {
        pyo3_stub_gen::TypeInfo::none()
    }

    fn type_output() -> pyo3_stub_gen::TypeInfo {
        pyo3_stub_gen::TypeInfo::none()
    }
}

impl_stub_type!(ValueLike = DocNone | bool | PyLightUserData | i64 | f64 | PyLuaVector | String | PyLuaString | PyLuaTable | PyLuaFunction | PyLuaThread | PyLuaAnyUserData | PyLuaBuffer);
