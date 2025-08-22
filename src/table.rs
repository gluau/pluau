use mluau::ffi::lua_Integer;
use pyo3::{exceptions::PyRuntimeError, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::{convert_error, value::ValueLike};

#[gen_stub_pyclass]
#[pyclass(frozen)]
pub struct Table {
    pub(crate) table: mluau::Table,
}

#[gen_stub_pymethods]
#[pymethods]
impl Table {
    /// Clears the table, removing all keys and values from array and hash parts, without invoking metamethods.
    ///
    /// This method is useful to clear the table while keeping its capacity
    /// 
    /// Internally invokes ``lua_cleartable``
    fn clear(&self) -> PyResult<()> {
        self.table.clear().map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Returns if the table contains a non-nil value for the specified key.
    /// 
    /// Might invoke ``__index`` metamethod. Use raw_get if this is not desired
    fn contains_key(&self, key: ValueLike) -> PyResult<bool> {
        self.table.contains_key(key).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Compares two tables for equality.
    ///
    /// Tables are compared by reference first. If they are not referentially equal, then pluau will try to invoke the __eq metamethod on `self`` first and then `other`` if not found.
    fn equals(&self, other: &Table) -> PyResult<bool> {
        self.table.equals(&other.table).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    // Same as equals but without __eq metatable
    fn __eq__(&self, other: &Table) -> bool {
        self.table == other.table
    }

    /// Iterates over the pairs of the table, invoking the given callback on each key-value pair.
    fn for_each(&self, py: Python<'_>, callback: PyObject) -> PyResult<()> {
        // Ensure the provided Python object is actually a callable
        if callback.getattr(py, "__call__").is_err() {
            return Err(PyRuntimeError::new_err("Provided object is not callable"));
        }

        self.table.for_each::<ValueLike, ValueLike>(move |k, v| {
            Python::with_gil(|py| {
                let values = (k,v)
                    .into_pyobject(py)
                    .map_err(|e| mluau::Error::external(e.to_string()))?;

                let cb = callback.call1(py, values);
                match cb {
                    Ok(_) => Ok(()),
                    Err(err) => Err(mluau::Error::external(convert_error(py, err))),
                }
            })
        })
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(())
    }

    /// Gets the value associated to key from the table. Might invoke ``__index`` metamethod
    ///
    /// Also see ``raw_get`` which does the same thing as ``get`` without invoking metamethods
    fn get(&self, key: ValueLike) -> PyResult<ValueLike> {
        self.table.get(key).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Gets the value associated to key from the table without invoking metamethods
    ///
    /// Also see ``get`` which does the same thing as ``raw_get`` while invoking ``__index`` metamethods
    fn raw_get(&self, key: ValueLike) -> PyResult<ValueLike> {
        self.table.raw_get(key).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Returns true if the table is empty, without invoking any metamethods
    #[getter]
    fn empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Returns the result of the Lua # operator. Might invoke the __len metamethod
    /// 
    /// Use ``raw_len`` to get the length without invoking any metamethods
    fn len(&self) -> PyResult<lua_Integer> {
        self.table.len().map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Returns the result of the Lua # operator, without invoking the __len metamethod.
    fn raw_len(&self) -> usize {
        self.table.raw_len()
    }

    /// Same as ``Table.raw_len()``
    /// 
    /// To invoke the ``__len`` metamethod. Use ``Table.len()`` instead
    fn __len__(&self) -> usize {
        self.raw_len()
    }

    /// Returns if the table is readonly
    #[getter]
    fn get_readonly(&self) -> bool {
        self.table.is_readonly()
    }
    
    /// Sets the table to be readonly (or not)
    #[setter]
    fn set_readonly(&self, enabled: bool) {
        self.table.set_readonly(enabled);
    }

    // Sets whether or not the LuaTable is 'safeenv'.
    //
    // Safeenv provides special performance optimizations for Lua tables
    // used as the environment of a Luau chunk such as optimizing table
    // accesses, fastpaths for iteration and fastpaths for fastcall optimization
    // at the expense of breaking some metamethods and making the table de-facto
    // readonly.
    //
    // This is a Luau-specific feature.
    fn set_safeenv(&self, enabled: bool) {
        self.table.set_safeenv(enabled);
    }

    /// Returns the table's metatable (if there is any present)
    /// 
    /// Ignores __metatable and other protections
    fn metatable(&self) -> Option<Table> {
        self.table.metatable().map(|table| Table { table })
    }

    /// Sets or removes a metable on this table
    fn set_metatable(&self, mt: Option<&Table>) -> PyResult<()> {
        self.table.set_metatable(mt.map(|x| x.table.clone())).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Removes the last element from the table and returns it. Might invoke the __len and __newindex metamethods.
    /// 
    /// Also see ``raw_pop`` which does the same thing as ``pop`` without invoking metamethods
    fn pop(&self) -> PyResult<ValueLike> {
        self.table.pop().map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Removes the last element from the table and returns it without invoking metamethods
    /// 
    /// Also see ``pop`` which does the same thing as ``raw_pop`` while invoking metamethods
    fn raw_pop(&self) -> PyResult<ValueLike> {
        self.table.raw_pop().map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Appends a value to the back of the table. Might invoke the __len and __newindex metamethods.
    /// 
    /// Also see ``raw_push`` which does the same thing as ``push`` without invoking metamethods
    fn push(&self, value: ValueLike) -> PyResult<()> {
        self.table.push(value).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Appends a value to the back of the table without invoking metamethods
    /// 
    /// Also see ``push`` which does the same thing as ``raw_push`` while invoking metamethods
    fn raw_push(&self, value: ValueLike) -> PyResult<()> {
        self.table.raw_push(value).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Inserts element value at position idx to the table (without invoking metamethods), shifting up the elements from table[idx].
    ///
    /// The worst case complexity is O(n), where n is the table length.
    fn insert(&self, index: lua_Integer, value: ValueLike) -> PyResult<()> {
        self.table.raw_insert(index, value).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Removes a key from the table without invoking metamethods.
    ///
    /// If the key is an integer, all elements from table[key+1] will be shifted down.
    /// and table[key] will be removed with a worst case complexity of O(n),
    ///
    /// For non-integer keys, this is equivalent to a table[key] = nil operation,
    fn remove(&self, key: ValueLike) -> PyResult<()> {
        self.table.raw_remove(key).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Sets a key-value pair (effectively removing the pair if value is nil). Might invoke ``__newindex`` metamethod
    ///
    /// Also see ``raw_set`` which does the same thing as ``set`` without invoking metamethods
    fn set(&self, key: ValueLike, value: ValueLike) -> PyResult<()> {
        self.table.set(key, value).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Sets a key-value pair (effectively removing the pair if value is nil) without invoking metamethods
    ///
    /// Also see ``set`` which does the same thing as ``raw_set`` while invoking ``__newindex`` metamethods
    fn raw_set(&self, key: ValueLike, value: ValueLike) -> PyResult<()> {
        self.table.raw_set(key, value).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    #[getter]
    /// Returns the pointer to the Lua table value.
    /// 
    /// This pointer cannot be converted back to a Lua table
    /// and is only useful for hashing and debugging.
    fn pointer(&self) -> usize {
        self.table.to_pointer() as usize
    }

    fn __str__(&self) -> String {
        format!("{:#?}", self.table)
    }

    // Define string repr
    fn __repr__(&self) -> String {
        format!("Table({:?})", self.table.to_pointer())
    }

    fn __iter__(&self) -> TableIterOwned {
        TableIterOwned {
            iter: self.table.pairs_owned(),
        }
    }
}

/// Non thread-safe iterator over table key-value pairs.
/// 
/// Attempting to use this iterator from multiple threads will result in a panic.
#[gen_stub_pyclass]
#[pyclass(unsendable)]
pub struct TableIterOwned {
    iter: mluau::TablePairsOwned<ValueLike, ValueLike>,
}

#[gen_stub_pymethods]
#[pymethods]
impl TableIterOwned {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    // TODO: Remember to change this if the Luau primitives list changes
    #[gen_stub(override_return_type(type_repr="typing.Tuple[None | builtins.bool | LightUserData | builtins.int | builtins.float | Vector | String | Table | Function | Thread | UserData | Buffer, None | builtins.bool | LightUserData | builtins.int | builtins.float | Vector | String | Table | Function | Thread | UserData | Buffer]", imports=("typing", "builtins")))]
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyResult<(ValueLike, ValueLike)>> {
        match slf.iter.next() {
            Some(x) => {
                Some(x.map_err(|e| PyRuntimeError::new_err(e.to_string())))
            }
            None => None,
        }
    }
}
