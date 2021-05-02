use std::str::FromStr;
use crate::errors::AdbError;

/// 设备映射类型，分为IPC模式和TCP模式
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MappingType {
	/// Ipc模式进行映射，例如：localabstruct:ipcName
	Ipc(String),
	/// TCP模式因为，例如：tcp:8080
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


/// 使用adb进行Forward与Reverse的类型
/// 主要有2种形式
/// 1. tcp -> tcp: 命令基本为 adb forward tcp:xxx tcp:xxx
/// 2. tcp - > ipc: 命令基本为 adb forward tcp:xxx localabstruct:ipcName
/// forward模式三元组为 <serial,local,remote>
/// reverse模式三元组为 <serial,remote,local> **z注意顺序与forward相反**
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct MappingDevice {
	/// 对于reverse模式来时serial可有可无
	/// 对于forward模式来说serial总是不为None
	serial: Option<String>,
	/// 本地映射,不论是forward还是reverse`local`总是表示本机映射
	local: MappingType,
	/// 手机设备映射（远程），不论是forward还是reverse `remote`总是表示远程映射
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

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum DeviceStatus {
	Online,
	Offline,
	Unauthorized,
	Absent,
	Unknown
}

impl FromStr for DeviceStatus {
	type Err = AdbError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"online" => Ok(Self::Online),
			"offline" => Ok(Self::Offline),
			"unauthorized" => Ok(Self::Unauthorized),
			"absent" => Ok(Self::Absent),
			_ => Ok(Self::Unknown),
		}
	}
}

pub struct DeviceEvent {
	pub(crate) present: Option<bool>,
	pub(crate) serial: String,
	pub(crate) status: DeviceStatus,
}
