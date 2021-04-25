use serde::{Deserialize, Serialize};


#[derive(Deserialize, Debug, Serialize)]
pub(crate) enum Status {
	Unknown = -1,
	Ok = 0,
	Error = 1,
}

/// 设备信息
#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct Device {
	/// 设备序列号
	phone_serial: String,
	/// 手机名称
	phone_name: String,
	/// 手机节点
	own_node: String,
	/// 手机生产厂商
	phone_manufacturer: String,
	/// 手机型号
	phone_mode_type: String,
	/// 屏幕尺寸
	screen_size: String,
	/// 安卓版本
	android_version: String,
	/// 分辨率
	resolution: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct Script {
	script_file_url: String,
	script_type: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct InstallResult {
	start_time: i64,
	end_time: i64,
	status: Status,
	result: i32,
}

#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct Meta {
	package_name: String,
	activity_name: String,
	install_result: InstallResult,
}

#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct TaskParam {
	task_name: String,
	task_type: String,
	devices: Vec<Device>,
	script: Script,
}

#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct CreateResult<T: Serialize> {
	status: Status,
	msg: Option<String>,
	data: Option<T>,
}

impl<T: Serialize> CreateResult<T> {
	pub fn ok() -> Self {
		Self {
			status: Status::Ok,
			msg: None,
			data: None
		}
	}

	pub fn ok_data(data: T) -> Self {
		Self {
			status: Status::Ok,
			msg: None,
			data: Some(data),
		}
	}
	pub fn with_data(status: Status, data: T) -> Self {
		Self {
			status,
			msg: None,
			data: Some(data),
		}
	}

	pub fn error(msg: String) -> Self {
		Self {
			status: Status::Ok,
			msg: Some(msg),
			data: None
		}
	}
}

