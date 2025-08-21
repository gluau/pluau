use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

#[gen_stub_pyclass]
#[pyclass(frozen)]
pub struct Vector {
    pub(crate) value: mluau::Vector,
}

#[gen_stub_pymethods]
#[pymethods]
impl Vector {
    #[new]
    #[pyo3(signature = (x, y, z), text_signature = "(x, y, z)")]
    /// Constructs a new vector
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vector {
            value: mluau::Vector::new(x, y, z)
        }
    }

    #[staticmethod]
    fn zero() -> Self {
        Vector {
            value: mluau::Vector::zero()
        }
    }

    #[getter]
    fn x(&self) -> f32 {
        self.value.x()
    }

    #[getter]
    fn y(&self) -> f32 {
        self.value.y()
    }

    #[getter]
    fn z(&self) -> f32 {
        self.value.z()
    }

    fn __repr__(&self) -> String {
        format!("Vector({}, {}, {})", self.x(), self.y(), self.z())
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}