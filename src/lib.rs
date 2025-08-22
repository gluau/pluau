mod compiler; 
mod table;
mod value;
mod vector;
mod lightuserdata;
mod string;
mod function;
mod thread;
mod userdata;
mod buffer;
use pyo3_stub_gen::{create_exception, define_stub_info_gatherer, derive::{gen_stub_pyclass, gen_stub_pyclass_enum, gen_stub_pymethods}};
use pyo3::{exceptions::{PyBaseException, PyRuntimeError}, prelude::*, types::{PySequence, PyTuple}};
use mluau::{FromLua, IntoLua};

use crate::{string::StringLike, value::ValueLike};

// Represents the different standard libraries that can be loaded into the Luau VM.
bitflags::bitflags! {
    pub struct StdLib: u32 {
        const COROUTINE = 1 << 0;
        const TABLE = 1 << 1;
        const OS = 1 << 2;
        const STRING = 1 << 3;
        const UTF8 = 1 << 4;
        const BIT = 1 << 5;
        const MATH = 1 << 6;
        const BUFFER = 1 << 7;
        const VECTOR = 1 << 8;
        const DEBUG = 1 << 9;
        const ALL = 1 << 31;
    }
}

impl StdLib {
    pub fn to_mluau(self) -> mluau::StdLib {
        if self.contains(StdLib::ALL) {
            return mluau::StdLib::ALL_SAFE; // Return all safe libraries
        }

        let mut libs = mluau::StdLib::NONE;

        if self.contains(StdLib::COROUTINE) {
            libs |= mluau::StdLib::COROUTINE;
        }
        if self.contains(StdLib::TABLE) {
            libs |= mluau::StdLib::TABLE;
        }
        if self.contains(StdLib::OS) {
            libs |= mluau::StdLib::OS;
        }
        if self.contains(StdLib::STRING) {
            libs |= mluau::StdLib::STRING;
        }
        if self.contains(StdLib::UTF8) {
            libs |= mluau::StdLib::UTF8;
        }
        if self.contains(StdLib::BIT) {
            libs |= mluau::StdLib::BIT;
        }
        if self.contains(StdLib::MATH) {
            libs |= mluau::StdLib::MATH;
        }
        if self.contains(StdLib::BUFFER) {
            libs |= mluau::StdLib::BUFFER;
        }
        if self.contains(StdLib::VECTOR) {
            libs |= mluau::StdLib::VECTOR;
        }
        if self.contains(StdLib::DEBUG) {
            libs |= mluau::StdLib::DEBUG;
        }
        libs
    }
}

#[gen_stub_pyclass_enum]
#[pyclass(eq)]
#[derive(PartialEq, Clone, Copy)]
pub enum VmState {
    Continue,
    Yield,
}

#[gen_stub_pyclass_enum]
#[pyclass(eq)]
#[derive(PartialEq, Clone, Copy)]
pub enum LuaType {
    Boolean,
    LightUserData,
    Number,
    Vector,
    String,
    Function,
    Thread,
    Buffer,
}

/// Represents a Luau VM handle
#[gen_stub_pyclass]
#[pyclass(frozen)]
pub struct Lua {
    vm: mluau::Lua,
}

fn convert_error(py: Python<'_>, e: PyErr) -> mluau::Error {
    // Check if RawError
    if e.is_instance_of::<RawError>(py) {
        let raw_error = e.value(py).to_string();
        return mluau::Error::external(raw_error);
    }

    return mluau::Error::external(e.to_string());
}

#[gen_stub_pymethods]
#[pymethods]
impl Lua {
    #[new]
    #[pyo3(signature = (stdlib=None), text_signature = "(stdlib=None)")]
    /// Constructs a new Lua VM with an optional standard library set
    pub fn new(stdlib: Option<u32>) -> PyResult<Self> {
        let stdlib = match stdlib {
            Some(stdlib) => StdLib::from_bits_truncate(stdlib).to_mluau(),
            None => mluau::StdLib::ALL_SAFE,
        };

        let vm = mluau::Lua::new_with(
            stdlib,
            mluau::LuaOptions::new()
            .catch_rust_panics(false)
            .disable_error_userdata(true)
        )
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        .map(|x| Lua { vm: x })?;

        vm.vm.set_on_close(|| {
            println!("Lua VM is being closed");
        });

        Ok(vm)
    }

    // Define string repr
    fn __repr__(&self) -> String {
        format!("{:?}", self.vm)
    }

    /// Returns the amount of memory used by the Lua VM
    fn used_memory(&self) -> usize {
        self.vm.used_memory()
    }

    /// Returns the memory limit of the Lua VM
    fn memory_limit(&self) -> PyResult<usize> {
        self.vm.memory_limit()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    #[pyo3(signature = (limit), text_signature = "(limit)")]
    /// Sets the memory limit for the Lua VM.
    fn set_memory_limit(&self, limit: usize) -> PyResult<()> {
        self.vm.set_memory_limit(limit)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(())
    }

    /// Enables or disables the sandbox mode for the Luau VM.
    ///
    /// This method, in particular:
    ///
    /// - Set all libraries to read-only
    /// - Set all builtin metatables to read-only
    /// - Set globals to read-only (and activates safeenv)
    /// - Setup local environment table that performs writes locally and proxies reads to the global environment.
    /// - Allow only count mode in collectgarbage function.
    ///
    /// Note that this is a Luau-specific feature.
    #[pyo3(signature = (enabled), text_signature = "(enabled)")]
    fn sandbox(&self, enabled: bool) -> PyResult<()> {
        self.vm.sandbox(enabled)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Sets the compiler to use
    fn set_compiler(&self, compiler: &compiler::Compiler) {
        self.vm.set_compiler(compiler.compiler.clone());
    }

    /// Creates a new Lua string
    fn create_string(&self, value: string::StringLike) -> PyResult<string::String> {
        let lua_string = match value {
            string::StringLike::Bytes(bytes) => self.vm.create_string(bytes.as_bytes())
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
            string::StringLike::StdString(std_string) => self.vm.create_string(std_string)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
            string::StringLike::LuaString(lua_string) => lua_string,
        };
        Ok(string::String { value: lua_string })
    }

    /// Creates a new Lua function
    fn create_function(&self, py: Python<'_>, callback: PyObject) -> PyResult<function::Function> {
        // Ensure the provided Python object is actually a callable
        if callback.getattr(py, "__call__").is_err() {
            return Err(PyRuntimeError::new_err("Provided object is not callable"));
        }

        let func = self.vm.create_function(move |lua, args: mluau::MultiValue| {
            let mut values: Vec<value::ValueLike> = Vec::with_capacity(args.len());
            for arg in args {
                values.push(value::ValueLike::from_lua(arg, lua)?);
            }

            let result = Python::with_gil(|py| {
                let callback_vm = Lua { vm: lua.clone() };
                let args_tuple = PyTuple::new(py, values)
                    .map_err(|e| mluau::Error::external(e.to_string()))?;

                let values = (callback_vm, args_tuple)
                    .into_pyobject(py)
                    .map_err(|e| mluau::Error::external(e.to_string()))?;

                let cb = callback.call1(py, values);
                match cb {
                    Ok(result) => {
                        // Check if the result is a tuple or list
                        let bound = result.bind(py);
                        if let Ok(x) = bound.downcast::<PySequence>() {
                            let value: Vec<value::ValueLike> = x.extract().map_err(|e| mluau::Error::external(e.to_string()))?;
                            return Ok(value)
                        } 

                        let value: value::ValueLike = result.extract(py).map_err(|e| mluau::Error::external(e.to_string()))?;
                        Ok(vec![value])
                    }
                    Err(err) => Err(mluau::Error::external(convert_error(py, err))),
                }
            });

            match result {
                Ok(value) => {
                    let mut mv = mluau::MultiValue::with_capacity(value.len());
                    for val in value {
                        mv.push_back(val.into_lua(lua)?);
                    }
                    Ok(mv)
                }
                Err(err) => Err(err),
            }
        })
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(function::Function { function: func })
    }

    /// Returns a handle to the active Thread.
    ///
    /// When using a Lua handle originating outside a callback. this will be the main Lua thread
    /// 
    /// Within a callback, this will return a reference to the Lua thread that called the callback.
    fn current_thread(&self) -> thread::Thread {
        let thread = self.vm.current_thread();
        thread::Thread { thread }
    }

    /// Sets the metatable for a type
    /// 
    /// # Arguments
    /// - `type`: The type to set the metatable for
    /// - `metatable`: The metatable to set
    fn set_type_metatable(&self, type_: LuaType, metatable: Option<&table::Table>) {
        let metatable = metatable.map(|x| x.table.clone());

        match type_ {
            LuaType::Boolean => self.vm.set_type_metatable::<bool>(metatable),
            LuaType::LightUserData => self.vm.set_type_metatable::<mluau::LightUserData>(metatable),
            LuaType::Number => self.vm.set_type_metatable::<f64>(metatable),
            LuaType::Vector => self.vm.set_type_metatable::<mluau::Vector>(metatable),
            LuaType::String => self.vm.set_type_metatable::<mluau::String>(metatable),
            LuaType::Function => self.vm.set_type_metatable::<mluau::Function>(metatable),
            LuaType::Thread => self.vm.set_type_metatable::<mluau::Thread>(metatable),
            LuaType::Buffer => self.vm.set_type_metatable::<mluau::Buffer>(metatable),
        };
    }

    /// Sets a value on the Lua registry with a given key name
    fn set_registry_value(&self, key: &str, value: value::ValueLike) -> PyResult<()> {
        self.vm.set_named_registry_value(
            key, 
            value.into_lua(&self.vm)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        )
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Gets a value from the Lua registry by key name
    fn get_registry_value(&self, key: &str) -> PyResult<value::ValueLike> {
        let value = self.vm.named_registry_value(key)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let value = ValueLike::from_lua(value, &self.vm)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(value)
    }

    /// Returns the global environment table of the Lua VM
    fn globals(&self) -> table::Table {
        let globals = self.vm.globals();
        table::Table { table: globals }
    }

    /// Sets the global environment table of the Lua VM
    /// Note that any existing Lua functions have cached global environment and will not see the changes made by this method.
    ///
    /// To update the environment for existing Lua functions, use Function.set_environment instead.
    fn set_globals(&self, table: &table::Table) -> PyResult<()> {
        self.vm.set_globals(table.table.clone())
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(())
    }

    /// Sets an interrupt function that will periodically be called by Luau VM. The interrupt function will provide the callback VM as arguments and expects a VmState to be returned.
    ///
    /// Any Luau code is guaranteed to call this handler “eventually” (in practice this can happen at any function call or at any loop iteration).
    ///
    /// The provided interrupt function can error, and this error will be propagated through the Luau code that was executing at the time the interrupt was triggered.
    ///
    /// Also this can be used to implement continuous execution limits by instructing Luau VM to yield by returning VmState.Yield.
    fn set_interrupt(&self, py: Python<'_>, callback: PyObject) -> PyResult<()> {
        // Ensure the provided Python object is actually a callable
        if callback.getattr(py, "__call__").is_err() {
            return Err(PyRuntimeError::new_err("Provided object is not callable"));
        }

        self.vm.set_interrupt(move |lua| {
            let result = Python::with_gil(|py| {
                let callback_vm = Lua { vm: lua.clone() };

                let values = (callback_vm,)
                    .into_pyobject(py)
                    .map_err(|e| mluau::Error::external(e.to_string()))?;

                let cb = callback.call1(py, values);
                match cb {
                    Ok(result) => {
                        let vm_state: VmState = result.extract(py).map_err(|e| mluau::Error::external(e.to_string()))?;
                        Ok(vm_state)
                    }
                    Err(err) => Err(mluau::Error::external(convert_error(py, err))),
                }
            });

            result.map(|vm_state| {
                match vm_state {
                    VmState::Continue => mluau::VmState::Continue,
                    VmState::Yield => mluau::VmState::Yield,
                }
            })
        });

        Ok(())
    }

    /// Removes the currently set interrupt function
    /// 
    /// This is a no-op if no interrupt function was set.
    fn remove_interrupt(&self) {
        self.vm.remove_interrupt();
    }

    /// Loads a luau chunk into a function
    #[pyo3(signature = (contents, name=None, env=None, is_binary_chunk=None, compiler=None), text_signature = "(contents, name=None, env=None, is_binary_chunk=None, compiler=None)")]
    fn load_chunk(
        &self, 
        contents: StringLike,
        name: Option<&str>,
        env: Option<&table::Table>,
        is_binary_chunk: Option<bool>,
        compiler: Option<&compiler::Compiler>
    ) -> PyResult<function::Function> {
        let mut chunk = match contents {
            StringLike::Bytes(bytes) => {
                let as_bytes = bytes.as_bytes().to_vec();
                self.vm.load(as_bytes)
            },
            StringLike::StdString(std_string) => {
                self.vm.load(std_string)
            },
            StringLike::LuaString(lua_string) => {
                let as_bytes = lua_string.as_bytes().to_vec();
                self.vm.load(as_bytes)
            },
        };
        if let Some(name) = name {
            chunk = chunk.set_name(name);
        }
        if let Some(env) = env {
            chunk = chunk.set_environment(env.table.clone());
        }
        if let Some(is_binary_chunk) = is_binary_chunk {
            if is_binary_chunk {
                chunk = chunk.set_mode(mluau::ChunkMode::Binary);
            } else {
                chunk = chunk.set_mode(mluau::ChunkMode::Text);
            }
        } else {
            chunk = chunk.set_mode(mluau::ChunkMode::Text);
        }

        if let Some(compiler) = compiler {
            chunk = chunk.set_compiler(compiler.compiler.clone());
        }

        match chunk.into_function() {  
            Ok(func) => Ok(function::Function { function: func }),
            Err(e) => Err(PyRuntimeError::new_err(e.to_string())),
        }
    }

    /// Creates a table
    fn create_table(&self) -> PyResult<table::Table> {
        let table = self.vm.create_table()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(table::Table { table })
    }

    /// Creates a table with `narr` being the capacity of the array part and `nrec` being the capacity of the hash part.
    #[pyo3(signature = (narr, nrec), text_signature = "(narr, nrec)")]
    fn create_table_with_capacity(&self, narr: usize, nrec: usize) -> PyResult<table::Table> {
        let table = self.vm.create_table_with_capacity(narr, nrec)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(table::Table { table })  
    }

    /// Creates a new Lua thread given its base Lua function
    ///
    /// Equivalent to ``coroutine.create(func)`` in Luau
    fn create_thread(&self, func: &function::Function) -> PyResult<thread::Thread> {
        let thread = self.vm.create_thread(func.function.clone())
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(thread::Thread { thread })
    }

    /// Creates a new Luau buffer
    fn create_buffer(&self, contents: StringLike) -> PyResult<buffer::Buffer> {
        let buf = match contents {
            StringLike::Bytes(bytes) => self.vm.create_buffer(bytes.as_bytes())
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
            StringLike::StdString(std_string) => self.vm.create_buffer(std_string)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
            StringLike::LuaString(lua_string) => self.vm.create_buffer(lua_string.as_bytes())
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        };
        Ok(buffer::Buffer { buf })
    }

    /// Creates a new Userdata given associated data and a metatable
    fn create_userdata(
        &self, 
        data: PyObject, 
        metatable: &table::Table
    ) -> PyResult<userdata::UserData> {
        let userdata = self.vm.create_dynamic_userdata(data, &metatable.table)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(userdata::UserData { ud: userdata })
    }

    /// Tells Lua to the currently running Lua thread once the ongoing callback returns.
    /// 
    /// Any results from the ongoing callback will be ignored and the args passed to yield_with will instead be yielded
    // TODO: Consider making a CallbackLua instead and putting this there
    #[pyo3(signature = (*args))]
    fn yield_with(&self, args: Vec<ValueLike>) -> PyResult<()> {
        let mut mv = mluau::MultiValue::with_capacity(args.len());
        for value in args {
            mv.push_back(value.into_lua(&self.vm).map_err(|x| PyRuntimeError::new_err(x.to_string()))?);
        }
        self.vm.yield_with(mv).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(())
    }

    /// Returns a weak reference to the Lua VM
    fn weak(&self) -> WeakLua {
        let weak_lua = self.vm.weak();
        WeakLua { weak_lua }
    }

    /// Returns the strong count of the Lua VM.
    /// 
    /// Can be useful for debugging
    fn strong_count(&self) -> usize {
        self.vm.strong_count()
    }

    /// Returns the weak count of the Lua VM.
    /// 
    /// Can be useful for debugging
    fn weak_count(&self) -> usize {
        self.vm.weak_count()
    }
}

#[gen_stub_pyclass]
#[pyclass(frozen)]
/// Provides a weak reference to a Lua VM
pub struct WeakLua {
    weak_lua: mluau::WeakLua,
}

#[gen_stub_pymethods]
#[pymethods]
impl WeakLua {
    /// Attempts to upgrade the weak reference to a strong reference.
    /// 
    /// Returns None if the Lua VM has been garbage collected.
    fn upgrade(&self) -> Option<Lua> {
        self.weak_lua.try_upgrade().map(|vm| Lua { vm })
    }

    /// Returns the strong count of the Lua VM.
    ///
    /// Mostly useful for debugging
    fn strong_count(&self) -> usize {
        self.weak_lua.strong_count()
    }

    /// Returns the weak count of the Lua VM.
    /// 
    /// Mostly useful for debugging
    fn weak_count(&self) -> usize {
        self.weak_lua.weak_count()  
    }

    /// Returns if the Lua instance is destroyed.
    ///
    /// This is equivalent to checking if the strong count is `0`.
    fn is_destroyed(&self) -> bool {
        self.weak_lua.is_destroyed()
    }

    fn __eq__(&self, other: &WeakLua) -> bool {
        self.weak_lua == other.weak_lua
    }
}

create_exception!(pluau._pluau, RawError, PyBaseException, "RawError allows for directly passing through a error message to Luau without any error type information.");

/// Pluau provides Luau bindings for Python using PyO3.
#[pymodule(name="_pluau")]
fn pluau(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("RawError", m.py().get_type::<RawError>())?;
    m.add_class::<VmState>()?;
    m.add_class::<LuaType>()?;
    m.add_class::<Lua>()?;
    m.add_class::<WeakLua>()?;
    m.add_class::<compiler::Compiler>()?;
    m.add_class::<table::Table>()?;
    m.add_class::<table::TableIterOwned>()?;
    m.add_class::<vector::Vector>()?;
    m.add_class::<lightuserdata::LightUserData>()?;
    m.add_class::<string::String>()?;
    m.add_class::<function::Function>()?;
    m.add_class::<thread::Thread>()?;
    m.add_class::<thread::ThreadState>()?;
    m.add_class::<userdata::UserData>()?;
    m.add_class::<buffer::Buffer>()?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);
