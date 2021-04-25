use pyo3::prelude::*;
use pyo3::ffi::PyObject_Call;
use pyo3::types::{PyTuple, PyString, IntoPyDict};


fn run_script() -> PyResult<()> {
	let gil = Python::acquire_gil();
	let py = gil.python();
	let sys = py.import("sys")?;
	let path = sys.get("path")?;
	let arr = path.getattr("append")?;
	let args = PyTuple::new(py,
	&[PyString::new(py, "/Users/venmosnake/.local/share/virtualenvs/adb-uGPfhvtw/lib/python3.9/site-packages")]
	);
	arr.call(args, None)?;
	py.run("print(sys.path)",
	       Some([("sys", sys)].into_py_dict(py)),
	       None)?;

	Ok(())
}

#[test]
fn test() -> PyResult<()> {
	run_script()
}