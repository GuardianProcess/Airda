mod launcher;
mod task;
mod python;
mod task_env;


mod test {
	use crate::execution::task::{Task, ArgsProtocol};
	use crate::errors::error::{ExecuteResult, ExecuteError};
	use std::ffi::OsStr;
	use std::path::Path;
	use std::hash::Hash;
	use std::fmt::Debug;
	use crate::execution::launcher::Launcher;
	use std::thread::sleep;
	use std::time::Duration;

	struct PythonArgs {
		module: String,
	}

	impl PythonArgs {
		pub fn new(data: String) -> Self {
			Self {
				module: data,
			}
		}
	}

	impl ArgsProtocol for PythonArgs {
		fn to_command_args(&self) -> Vec<String> {
			vec![
				"-m".to_string(),
				self.module.clone(),
				"--help".to_string(),
			]
		}
	}

	fn done<P>(_t: &Task<P>) -> ExecuteResult<()>
		where P: AsRef<Path> + AsRef<OsStr> + Hash + Eq + Clone + Debug {
		println!("done");
		Ok(())
	}

	fn error<P>(_t: &Task<P>) -> ExecuteResult<()>
		where P: AsRef<Path> + AsRef<OsStr> + Hash + Eq + Clone + Debug {
		Err(ExecuteError::TaskError { why: "test error".to_string() })
	}

	#[test]
	fn test_task() -> ExecuteResult<()> {
		log4rs::init_file("config.yaml", Default::default()).unwrap();
		let mut task = Task::from_program("python3");
		task.redirect_to_file("example.txt")?;
		let args = PythonArgs::new("pipenv".to_string());
		task.add_callback(done);
		task.add_callback(error);
		task.use_args(args);
		assert!(!task.still_alive());
		task.run()?;
		task.wait_for_end()?;
		Ok(())
	}

	#[test]
	fn test_launcher() -> ExecuteResult<()> {
		let system = actix::System::new();
		log4rs::init_file("config.yaml", Default::default()).unwrap();
		let mut runner = Launcher::new();
		let mut task = Task::from_program("python3");
		task.redirect_to_file("example.txt")?;
		let args = PythonArgs::new("pipenv".to_string());
		task.add_callback(done);
		task.use_args(args);
		runner.submit_task(task);
		sleep(Duration::from_secs(2));
		system.run();
		Ok(())
	}
}