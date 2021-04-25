use crate::resource::phone::adb::{Device, AdbClient};
use std::ffi::OsStr;
use crate::errors::error::ExecuteResult;
use std::path::Path;

pub struct Phone<'a, S> {
	pub device: Device,
	adb: &'a AdbClient<S>,
}

impl<'a, S> Phone<'a, S> {
	pub fn new(device: Device, client: &'a AdbClient<S>) -> Self {
		Self {
			device,
			adb: client,
		}
	}
}

impl<'a, S: AsRef<OsStr>> Phone<'a, S> {
	pub fn from_serial(serial: &str, client: &'a AdbClient<S>) -> ExecuteResult<Option<Self>> {
		let device = client.find_device(serial)??;
		Ok(Some(Self::new(device, client)))
	}

	pub fn forward(&self, local: &str, remote: &str) -> ExecuteResult<()> {
		self.adb.forward(self.device.serial.as_str(), local, remote)?;
		Ok(())
	}

	pub fn remove_forward(&self, local: &str) -> ExecuteResult<()> {
		self.adb.remove_forward(self.device.serial.as_str(), local)?;
		Ok(())
	}

	pub fn push_file<P: AsRef<Path>>(&self, local: P, remote: P) -> ExecuteResult<()> {
		self.adb.push_file(self.device.serial.as_str(), local, remote)?;
		Ok(())
	}
}