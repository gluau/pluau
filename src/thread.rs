use mluau::{FromLua, IntoLua};
use pyo3::{exceptions::PyRuntimeError, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyclass_enum, gen_stub_pymethods};
use crate::value::ValueLike;

#[gen_stub_pyclass_enum]
#[pyclass(eq)]
#[derive(PartialEq, Clone, Copy)]
pub enum ThreadState {
    /// The thread was just created or is suspended (yielded).
    ///
    /// If a thread is in this state, it can be resumed by calling Thread.resume.
    Resumable,

    /// The thread is currently running.
    Running,

    /// The thread has finished executing.
    Finished,

    /// The thread has raised a Lua error during execution.
    Error,
}

#[gen_stub_pyclass]
#[pyclass(frozen)]
pub struct Thread {
    pub(crate) thread: mluau::Thread,
}

#[gen_stub_pymethods]
#[pymethods]
impl Thread {
    #[getter]
    /// Returns the pointer to the Lua thread value.
    /// 
    /// This pointer cannot be converted back to a Lua thread
    /// and is only useful for hashing and debugging.
    fn pointer(&self) -> usize {
        self.thread.to_pointer() as usize
    }

    fn __str__(&self) -> String {
        format!("{:#?}", self.thread)
    }

    // Define string repr
    fn __repr__(&self) -> String {
        format!("Thread({:?})", self.thread.to_pointer())
    }

    /// Returns if two threads are equal by (Luau) reference
    fn __eq__(&self, other: &Thread) -> bool {
        self.thread == other.thread
    }

    #[getter]
    /// Returns the current status of the LuaThread
    fn status(&self) -> ThreadState {
        match self.thread.status() {
            mluau::ThreadStatus::Resumable => ThreadState::Resumable,
            mluau::ThreadStatus::Running => ThreadState::Running,
            mluau::ThreadStatus::Finished => ThreadState::Finished,
            mluau::ThreadStatus::Error => ThreadState::Error,
        }
    }


    /// Sandboxes a Luau thread
    ///
    /// Under the hood replaces the global environment table with a new table, that performs writes locally and proxies reads to caller's global environment.
    ///
    /// This mode ideally should be used together with the global sandbox mode (Lua.sandbox).
    ///
    /// Please note that Luau links environment table with chunk when loading it into Lua state. Therefore you need to load chunks into a thread to link with the thread environment.
    fn sandbox(&self) -> PyResult<()> {
        self.thread.sandbox().map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    // Resets the Thread to the initial state of a newly created Luau thread regardless
    // of its current state and sets its function afterwards
    fn reset(&self, func: &crate::function::Function) -> PyResult<()> {
        self.thread.reset(func.function.clone()).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Resume resumes a thread `th`
    ///
    /// Passes args as arguments to the thread. If the coroutine has called coroutine.yield, it will return these arguments. Otherwise, the coroutine wasn’t yet started, so the arguments are passed to its main function.
    ///
    /// If the thread is no longer resumable (meaning it has finished execution or encountered an error), this will return a coroutine unresumable error, otherwise will return as follows:
    /// If the thread is yielded via coroutine.yield or CallbackLua.YieldWith, returns the values passed to yield. If the thread returns values from its main function, returns those.
    #[pyo3(signature = (*args))]
    fn resume(&self, args: Vec<ValueLike>) -> PyResult<Vec<ValueLike>> {
        let weak_lua = self.thread.weak_lua();
        let Some(lua) = weak_lua.try_upgrade() else {
            return Err(PyRuntimeError::new_err("Lua state has been garbage collected"));
        };

        let mut values: mluau::MultiValue = mluau::MultiValue::with_capacity(args.len());
        for arg in args {
            values.push_back(arg.into_lua(&lua).map_err(|e| PyRuntimeError::new_err(e.to_string()))?);
        }

        let result = self.thread.resume::<mluau::MultiValue>(
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

    // resume_error resumes a thread `th` with an error
    //
    // Similar to resume, but allows the resume to throw an error into the thread.
    fn resume_error(&self, arg: ValueLike) -> PyResult<Vec<ValueLike>> {
        let weak_lua = self.thread.weak_lua();
        let Some(lua) = weak_lua.try_upgrade() else {
            return Err(PyRuntimeError::new_err("Lua state has been garbage collected"));
        };

        let result = self.thread.resume_error::<mluau::MultiValue>(arg)
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

    /// Closes a thread and marks it as finished, resetting it to the initial state of a newly created Lua thread regardless of current thread state. 
    fn close(&self) -> PyResult<()> {
        self.thread.close().map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Creates a traceback of the thread
    fn traceback(&self) -> PyResult<String> {
        self.thread.traceback()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}