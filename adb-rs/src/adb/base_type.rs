use std::str::FromStr;
use crate::errors::AdbError;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MappingType {
	Ipc(String),
	Port(u32),
}

impl FromStr for MappingType {
	type Err = AdbError;

	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		if let Ok(port) = u32::from_str(s) {
			return Ok(Self::Port(port));
		}
		Ok(Self::Ipc(s.to_string()))
	}
}


#[derive(Debug, Eq, PartialEq, Clone)]
pub struct MappingDevice {
	serial: Option<String>,
	local: MappingType,
	remote: MappingType,
}

impl MappingDevice {
	pub fn new(serial: Option<String>, local: &str, remote: &str) -> Self {
		// cause we return nothing err use unwrap is safe
		let remote_type = MappingType::from_str(remote).unwrap();
		let local_type = MappingType::from_str(local).unwrap();
		Self {
			serial,
			local: local_type,
			remote: remote_type,
		}
	}
}