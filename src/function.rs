use pyo3::{exceptions::PyRuntimeError, prelude::*};

use crate::value::ValueLike;
use mluau::{FromLua, IntoLua};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

#[gen_stub_pyclass]
#[pyclass(frozen)]
pub struct Function {
    pub(crate) function: mluau::Function, // to be implemented: coverage, info (more complex to implement),
}

#[gen_stub_pymethods]
#[pymethods]
impl Function {
    /// Calls the Lua function with the provided arguments.
    #[pyo3(signature = (*args))]
    fn call(&self, args: Vec<ValueLike>) -> PyResult<Vec<ValueLike>> {
        let weak_lua = self.function.weak_lua();
        let Some(lua) = weak_lua.try_upgrade() else {
            return Err(PyRuntimeError::new_err("Lua state has been garbage collected"));
        };

        let mut values: mluau::MultiValue = mluau::MultiValue::with_capacity(args.len());
        for arg in args {
            values.push_back(arg.into_lua(&lua).map_err(|e| PyRuntimeError::new_err(e.to_string()))?);
        }

        let result = self.function.call::<mluau::MultiValue>(
            values
        )
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let mut value_likes: Vec<ValueLike> = Vec::with_capacity(result.len());
        for val in result {
            value_likes.push(
                ValueLike::from_lua(val, &lua)
                    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?
            );
        }

        Ok(value_likes)
    }

    /// Syntactic sugar for Function.call(args)
    #[pyo3(signature = (*args))]
    fn __call__(&self, args: Vec<ValueLike>) -> PyResult<Vec<ValueLike>> {
        self.call(args)
    }

    /// Returns a deep clone to a Lua-owned function
    ///
    /// If called on a Luau function, this method copies the function prototype and all its upvalues to the newly created function
    ///
    /// If called on a Python function, this method merely clones the function's handle
    fn deep_clone(&self) -> PyResult<Function> {
        let cloned_fn = self.function.deep_clone()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        
        Ok(Function { function: cloned_fn })
    }

    /// Returns the environment table of the Lua function
    /// 
    /// Returns None if the function does not have an environment
    /// 
    /// Python functions do not have an environment, so this will always return None
    fn environment(&self) -> Option<crate::table::Table> {
        let env = self.function.environment();
        env.map(|table| crate::table::Table { table })
    }

    /// Sets the environment table of the Lua function
    ///
    /// This uses setfenv and will hence deoptimize your function. Consider using load_chunk's environment argument instead
    fn set_environment(&self, table: &crate::table::Table) -> PyResult<()> {
        self.function.set_environment(table.table.clone())
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(())
    }

    fn __eq__(&self, other: &Function) -> bool {
        self.function == other.function
    }

    #[getter]
    /// Returns the pointer to the Lua function value.
    /// 
    /// This pointer cannot be converted back to a Lua function
    /// and is only useful for hashing and debugging.
    fn pointer(&self) -> usize {
        self.function.to_pointer() as usize
    }

    // Define string repr
    fn __repr__(&self) -> String {
        format!("LuaFunction({:?})", self.function.to_pointer())
    }
}