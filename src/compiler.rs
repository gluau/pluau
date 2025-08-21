use pyo3::{exceptions::PyRuntimeError, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

#[gen_stub_pyclass]
#[pyclass(frozen)]
/// Represents a Luau compiler that compiles Luau chunks into bytecode.
/// 
/// Note: All pluau `Lua` instances have a default compiler set to compile Lua chunks.
/// However you can create your own compiler with custom settings using this class.
pub struct Compiler {
    pub(crate) compiler: mluau::Compiler,
}

#[gen_stub_pymethods]
#[pymethods]
impl Compiler {
    #[new]
    #[pyo3(signature = (optimization_level, debug_level, type_info_level, coverage_level), text_signature = "(optimization_level, debug_level, type_info_level, coverage_level)")]
    /// Constructs a new Compiler with the specified settings.
    /// 
    /// # Arguments:
    /// - `optimization_level`: The optimization level for the Lua chunk (0 = no optimization, 1 = basic optimization, 2 = full optimization which may impact debugging).
    /// - `debug_level`: The debug level for the Lua chunk (0 = no debugging, 1 = line info + function names only, 2 = full debug info with locals+upvalues)
    /// - `type_info` (0 = native modules only, 1 = all modules)
    pub fn new(
        optimization_level: u8,
        debug_level: u8,
        type_info_level: u8,
        coverage_level: u8,
    ) -> Self {
        Compiler {
            compiler: mluau::Compiler::new()
                .set_optimization_level(optimization_level)
                .set_debug_level(debug_level)
                .set_type_info_level(type_info_level)
                .set_coverage_level(coverage_level),
        }
    }

    /// Compiles a Lua source code string into bytecode.
    pub fn compile(&self, source: &str) -> PyResult<Vec<u8>> {
        self.compiler.compile(source)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}