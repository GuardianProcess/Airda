use std::process::{Command, Stdio, Child, ExitStatus};
use std::{fs, env};
use crate::errors::error::{ExecuteResult, ExecuteError};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::hash::Hash;
use uuid::Uuid;

use log::{info, error};
use std::fmt::Debug;
use nix::sys::signal::{SIGTERM, kill};
use nix::unistd::Pid;


pub trait ArgsProtocol {
	fn to_command_args(&self) -> Vec<String>;
}

type Callback<P> = dyn FnOnce(dyn AsRef<Task<P>>) -> ();

pub struct Task<P>
	where P: AsRef<Path> + AsRef<OsStr> + Hash + Eq + Clone + Debug,
{
	id: Uuid,
	envs: HashMap<P, P>,
	program_name: Option<P>,
	proc: Option<Command>,
	work_space: Option<P>,
	child: Option<Child>,
	metadata: HashMap<P, Vec<P>>,
	status: Option<ExitStatus>,
	callbacks: Vec<fn(&Task<P>) -> ExecuteResult<()>>,
}

impl<P> Task<P>
	where P: AsRef<Path> + AsRef<OsStr> + Hash + Eq + Clone + Debug,
{
	pub fn new() -> Self {
		Self {
			id: Uuid::new_v4(),
			envs: Default::default(),
			proc: None,
			program_name: None,
			work_space: None,
			child: None,
			metadata: Default::default(),
			status: None,
			callbacks: Default::default(),
		}
	}


	pub fn from_program(program: P) -> Self {
		let name = program.clone();
		Self {
			id: Uuid::new_v4(),
			envs: Default::default(),
			proc: Some(Command::new(program)),
			program_name: Some(name),
			work_space: None,
			child: None,
			metadata: Default::default(),
			status: None,
			callbacks: Default::default(),
		}
	}

	pub fn use_task_env(&self){
	}

	pub fn stop(&mut self) -> ExecuteResult<()> {
		if let Some(ref mut child) = self.child {
			kill(Pid::from_raw(child.id() as i32), SIGTERM)?;
			self.status = child.try_wait()?;
		}
		Ok(())
	}

	pub fn kill(&mut self) -> ExecuteResult<()> {
		if let Some(ref mut child) = self.child {
			child.kill()?;
			self.status = Some(child.wait()?);
		}
		Ok(())
	}

	pub fn task_status(&mut self) -> Option<&ExitStatus> {
		self.status.as_ref()
	}

	pub fn add_callback(&mut self, f: fn(&Task<P>) -> ExecuteResult<()>) {
		self.callbacks.push(f);
	}

	pub fn add_callbacks(&mut self, f: Vec<fn(&Task<P>) -> ExecuteResult<()>>) {
		self.callbacks.extend(f)
	}

	pub fn task_id(&self) -> String {
		self.id.to_string()
	}

	pub fn use_args<ARG>(&mut self, args: ARG)
		where ARG: ArgsProtocol,
	{
		let _args = args.to_command_args();
		info!("task {} -> running with args {:?} {:?}", self.task_id(), self.program_name, _args);
		if let Some(ref mut proc) = self.proc {
			proc.args(_args);
		}
	}

	pub fn redirect_to_path<PH: AsRef<OsStr>>(&mut self, path: PH) -> ExecuteResult<()> {
		let p = Path::new(&path);
		self.redirect_to_file(p.join(format!("{}.log", self.task_id())))?;
		Ok(())
	}

	pub fn redirect_to_file<Ps: AsRef<Path>>(&mut self, file: Ps) -> ExecuteResult<()> {
		if let Some(ref mut proc) = self.proc {
			let handle = fs::File::create(file)?;
			let handle_err = handle.try_clone()?;
			proc.stderr(Stdio::from(handle));
			proc.stderr(Stdio::from(handle_err));
		}
		Ok(())
	}

	pub fn set_metadata(&mut self, key: P, val: P) {
		let mut list = self.metadata.entry(key).or_insert(vec![]);
		list.push(val);
	}


	pub fn work_on(&mut self, path: P) {
		if let Some(ref mut proc) = self.proc {
			proc.current_dir(&path);
		}
		self.work_space = Some(path);
	}

	pub fn set_input(&mut self, input: Stdio) {
		if let Some(ref mut proc) = self.proc {
			proc.stdin(input);
		}
	}

	pub(crate) fn try_wait(&mut self) -> ExecuteResult<Option<()>> {
		if let Some(ref mut child) = self.child {
			self.status = child.try_wait()?;
			return Ok(Some(()))
		}
		Ok(None)
	}

	pub fn wait_for_end(&mut self) -> ExecuteResult<()> {
		if !self.still_alive() {
			return Ok(());
		}
		if let Some(ref mut ch) = self.child {
			self.status = Some(ch.wait()?);
			return Ok(());
		}
		Err(ExecuteError::TaskError { why: "program not started".to_string() })
	}

	pub fn still_alive(&mut self) -> bool {
		if let Some(ref mut child) = self.child {
			return match child.try_wait() {
				Ok(res) => res.is_none(),
				Err(_) => false
			}
		}
		false
	}

	pub fn run(&mut self) -> ExecuteResult<()> {
		let proc = self.proc
			.as_mut()
			.ok_or(ExecuteError::TaskError { why: "process not initialized".to_string() })?
			.envs(env::vars())
			.envs(&self.envs);
		let child = proc.spawn()?;
		self.child = Some(child);
		Ok(())
	}
}

impl<P> Drop for Task<P>
	where P: AsRef<Path> + AsRef<OsStr> + Hash + Eq + Clone + Debug,
{
	fn drop(&mut self) {
		for callback in &self.callbacks {
			let res: ExecuteResult<()> = callback(&self);
			if let Err(ref e) = res {
				error!("task {} ->  callback error: {}", self.task_id(), e);
			}
		}
	}
}

