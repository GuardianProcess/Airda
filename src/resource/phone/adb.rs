use std::ffi::OsStr;
use crate::errors::error::ExecuteResult;
use std::path::Path;
use std::process::Command;

pub struct AdbClient<S> {
	path: S,
}

#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct Device {
	pub serial: String,
	pub state: String,
}

impl Device {
	fn is_space(ch: char) -> bool {
		ch == ' ' || ch == '\r'
	}

	pub fn new(line: &str) -> Option<Self> {
		let mut vec = line.split(Self::is_space);
		let serial = vec.next()?.to_string();
		let state = vec.next()?.to_string();
		Some(Self {
			serial,
			state,
		})
	}
}


impl<S: AsRef<OsStr>> AdbClient<S> {
	pub fn new(path: S) -> Self {
		AdbClient {
			path,
		}
	}

	pub fn command<I, T>(&self, args: I) -> ExecuteResult<String>
		where I: IntoIterator<Item = T>,
		      T: AsRef<OsStr>, {
		let child = Command::new(&self.path).args(args).spawn()?;
		let output = child.wait_with_output()?;
		Ok(String::from_utf8(output.stdout)?)
	}

	pub fn devices(&self) -> ExecuteResult<Vec<Option<Device>>> {
		let output = self.command(&["devices"])?;
		let res = output.lines().skip(1).map(|line| {
			Device::new(line)
		}).collect::<Vec<Option<Device>>>();
		Ok(res)
	}

	pub fn forward_ipc(&self, serial: &str, remote: &str) -> ExecuteResult<i32> {
		self.forward_random_port(serial, format!("localabstuct:{}", remote).as_str())
	}

	pub fn forward_random_port(&self, serial: &str, remote: &str) -> ExecuteResult<i32> {
		let res = self.command(&["-s", serial, "forward", "tcp:0", remote])?
			.lines()
			.filter(|x| x.trim().is_empty())
			.map(|x| x.parse())
			.next()??;
		Ok(res)
	}

	pub fn forward(&self, serial: &str, local: &str, remote: &str) -> ExecuteResult<()> {
		self.command(&["-s", serial, "forward", local, remote])?;
		Ok(())
	}

	pub fn remove_forward(&self, serial: &str, local: &str) -> ExecuteResult<()> {
		self.command(&["-s", serial, "forward", format!("tcp:{}", local).as_str()])?;
		Ok(())
	}

	pub fn find_device(&self, serial: &str) -> ExecuteResult<Option<Device>> {
		let iter = self.devices()?;
		let dev = iter.iter().filter(|x|
			match x {
				Some(dev) => dev.serial == serial,
				None => false,
			}).next()?;
		Ok(dev.clone())
	}

	pub fn push_file<P: AsRef<Path>>(&self, serial: &str, local: P, remote: P) -> ExecuteResult<()> {
		self.command(&["-s", serial, "push", local.as_ref().to_str()?, remote.as_ref().to_str()?])?;
		Ok(())
	}
}
