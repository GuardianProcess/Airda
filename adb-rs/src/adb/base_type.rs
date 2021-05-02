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
pub struct ForwardDevice {
	serial: String,
	local: u32,
	remote: MappingType,
}

impl ForwardDevice {
	pub fn new(serial: String, local: u32, remote: &str) -> Self {
		// cause we return nothing err use unwrap is safe
		let remote_type = MappingType::from_str(remote).unwrap();
		Self {
			serial,
			local,
			remote: remote_type,
		}
	}
}