use crate::{AdbClient, AdbConfig, Result};
use std::str::{FromStr};
use crate::adb::base_type::{MappingDevice, MappingType, DeviceEvent, DeviceStatus, ShellResult};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;
use std::fmt::Display;


/// android设备抽象，每个AdbDevice都持有一份AdbClient拷贝
pub struct AdbDevice {
	#[cfg(feature = "server")]
	inner: AdbClient,
	/// 设备序列号
	serial: String,
	/// 设备状态
	status: DeviceStatus,
}

impl AdbDevice {
	fn new(inner: AdbClient, serial: String, status: DeviceStatus) -> Self {
		Self {
			inner,
			serial,
			status,
		}
	}

	/// 在Android设备中执行shell命令
	/// 如果`stream`为true，则返回 `ShellResult::Stream`,否则返回`ShellResult::Output`
	/// 其中`timeout`会同时设置写超时和读超时
	pub fn shell<S>(&mut self, command: S, stream: bool, timeout: Option<Duration>) -> Result<ShellResult> where S: Into<String> + Display {
		let cmd = format!("host:transport:{}", self.serial);
		if stream {
			let mut given_client = self.inner.clone();
			AdbDevice::send_shell(command, &cmd, &mut given_client, timeout)?;
			return Ok(ShellResult::Stream(given_client));
		}
		AdbDevice::send_shell(command, &cmd, &mut self.inner, timeout)?;
		Ok(ShellResult::Output(self.inner.rend_full_string()?))
	}

	fn send_shell<S>(command: S, cmd: &String, given_client: &mut AdbClient, timeout: Option<Duration>) -> Result<()> where S: Into<String> + Display {
		given_client.set_timeout(timeout)?;
		given_client.send(cmd.as_bytes())?;
		given_client.check_ok()?;
		given_client.send(format!("shell:{}", command).as_bytes())?;
		given_client.check_ok()?;
		Ok(())
	}

	/// 获取当前设备的所有转发列表
	pub fn forward_list(&mut self) -> Result<Vec<MappingDevice>> {
		forward_list(&mut self.inner, Some(&self.serial))
	}
	/// 获取当前设备的所有反向代理列表
	pub fn reverse_list(&mut self) -> Result<Vec<MappingDevice>> {
		reverse_list(&mut self.inner, Some(&self.serial))
	}

	/// 与设备建立一个转发，adb -s `self.serial` forward tcp:1000 tcp:2000的意思是，将PC端的1000端口收到的数据，转发给到手机中2000端口
	/// PC可以通过访问自身的2000端口来访问手机的1000端口
	pub fn forward(&mut self, local: u32, remote: MappingType, norebind: Option<bool>) -> Result<()> {
		forward(&mut self.inner, &self.serial, local, remote, norebind)
	}

	/// 与设备建立反向代理，adb -s `self.serial` reverse tcp:1000 tcp:2000的意思是，将手机端的1000端口收到的数据，反向代理到PC中2000端口
	/// 手机可以访问自身的1000端口来访问PC的2000端口
	pub fn reverse(&mut self, remote: u32, local: u32) -> Result<()> {
		reverse(&mut self.inner, &self.serial, remote, local)
	}
}

pub struct AndroidDebugBridge {
	#[cfg(feature = "server")]
	inner: AdbClient,
}

impl AndroidDebugBridge {
	#[cfg(feature = "server")]
	pub fn new(cfg: AdbConfig) -> Result<Self> {
		let client = AdbClient::new(cfg)?;
		Ok(Self {
			inner: client,
		})
	}

	#[cfg(feature = "server")]
	pub fn adb_version(&mut self) -> Result<u32> {
		let res = self.inner.read_string()?;
		let version = u32::from_str(res.as_ref())?;
		Ok(version)
	}

	#[cfg(feature = "server")]
	pub fn kill_server(&mut self) -> Result<()> {
		// todo:check server is online
		self.inner.send("host:kill".as_bytes())?;
		self.inner.check_ok()?;
		Ok(())
	}

	#[cfg(feature = "server")]
	pub fn connect(&mut self, addr: &String) -> Result<()> {
		self.inner.send(format!("host:connect:{}", addr).as_bytes())?;
		self.inner.check_ok()?;
		//todo: check connect result
		self.inner.read_string()?;
		Ok(())
	}

	#[cfg(feature = "server")]
	pub fn disconnect(&mut self, addr: &String) -> Result<String> {
		self.inner.send(format!("host:disconnect:{}", addr).as_bytes())?;
		self.inner.check_ok()?;
		self.inner.read_string()
	}

	#[cfg(feature = "server")]
	pub fn watch_device(&mut self) -> Receiver<DeviceEvent> {
		let (mut sender, receiver) = channel();
		let mut th_client = self.inner.clone();
		thread::spawn(move || {
			AndroidDebugBridge::_watch_device(&mut sender, &mut th_client)
		});
		receiver
	}

	fn _watch_device(receiver: &mut Sender<DeviceEvent>, th_client: &mut AdbClient) -> Result<()> {
		th_client.send("host:track-devices".as_bytes())?;
		th_client.check_ok()?;
		loop {
			let raw_info = th_client.read_string()?;
			let events = raw_info
				.split("\n")
				.map(|line| line.split_ascii_whitespace())
				.fold(Vec::new(), |mut acc, mut item| {
					let serial = item.next().map(|x| x.to_string());
					let status = item.next();
					if serial.is_some() && status.is_some() {
						acc.push(DeviceEvent {
							present: None,
							serial: serial.unwrap(),
							status: DeviceStatus::from_str(status.unwrap()).unwrap(),
						})
					}
					acc
				});
			//todo: check different?
			for event in events {
				if let Err(_) = receiver.send(event) {
					break;
				}
			}
		}
	}

	/// 获取当所有反向代理列表
	pub fn reverse_list(&mut self) -> Result<Vec<MappingDevice>> {
		reverse_list(&mut self.inner, None)
	}

	/// 根据serial获取设备
	pub fn find_devices(&mut self, serial: String) -> Result<Option<AdbDevice>> {
		let mut find_result = self.available_devices().and_then(|d| {
			let res = d.into_iter().filter(|dev| dev.serial == serial).collect::<Vec<AdbDevice>>();
			Ok(res)
		})?;
		Ok(find_result.pop())
	}

	/// 获取当所有可用的设备
	pub fn available_devices(&mut self) -> Result<Vec<AdbDevice>> {
		self.devices().and_then(|x| {
			let res = x
				.into_iter()
				.filter(|dev| dev.status == DeviceStatus::Online)
				.collect::<Vec<AdbDevice>>();
			Ok(res)
		})
	}

	/// 获取当所有设备，包括可用和不可用
	pub fn devices(&mut self) -> Result<Vec<AdbDevice>> {
		self.inner.send("host:devices".as_bytes())?;
		self.inner.check_ok()?;
		let device_str = self.inner.read_string()?;
		let devices = device_str
			.split("\n")
			.map(|line| line.split_ascii_whitespace())
			.fold(Vec::new(), |mut acc, mut item| {
				let serial = item.next();
				let status = item.next();
				if serial.is_some() && status.is_some() {
					let dev = AdbDevice::new(
						self.inner.clone(),
						serial.unwrap().to_string(),
						DeviceStatus::from_str(status.unwrap()).unwrap(),
					);
					acc.push(dev);
				}
				acc
			});
		Ok(devices)
	}
}


fn forward_list(client: &mut AdbClient, serial: Option<&String>) -> Result<Vec<MappingDevice>> {
	let cmd = if let Some(s) = serial {
		format!("host-serial:{}:list-forward", s)
	} else {
		String::from("host:list-forward")
	};
	client.send(cmd.as_bytes())?;
	client.check_ok()?;
	// parse result
	let list = client.read_string()?;
	let res = list
		.split("\n")
		.map(|line| line.split_ascii_whitespace())
		.fold(Vec::new(), |mut acc, mut item| {
			// <serial local remote>
			let serial = item.next().map(|data| data.to_string());
			let local = item.next();
			let remote = item.next();
			check_and_push(&mut acc, serial, local, remote);
			acc
		});
	Ok(res)
}

fn check_and_push(acc: &mut Vec<MappingDevice>, serial: Option<String>, local: Option<&str>, remote: Option<&str>) {
	if serial.is_some() && local.is_some() && remote.is_some() {
		acc.push(MappingDevice::new(serial, local.unwrap(), remote.unwrap()));
	}
}

fn forward(client: &mut AdbClient, serial: &String, local: u32, remote: MappingType, norebind: Option<bool>) -> Result<()> {
	let mut cmds = vec!["host-serial", serial.as_ref(), "forward"];
	if norebind.is_some() && norebind.unwrap() {
		cmds.push("norebind")
	}
	// it cloud be better?
	let forward_cmd = match remote {
		MappingType::Ipc(ref ipc) => format!("{};{}", local, ipc),
		MappingType::Port(port) => format!("{};{}", local, port)
	};
	cmds.push(forward_cmd.as_ref());
	let res: String = cmds.join(";");
	client.send(res.as_bytes())?;
	Ok(())
}


fn reverse(client: &mut AdbClient, serial: &String, remote: u32, local: u32) -> Result<()> {
	client.send(format!("{},{}", "host:transport:", serial).as_bytes())?;
	client.check_ok()?;
	let sub_cmd: String = format!("{};{}", remote, local);
	let cmds = vec!["reverse:forward", sub_cmd.as_str()];
	client.send(cmds.join(":").as_bytes())?;
	Ok(())
}

fn reverse_list(client: &mut AdbClient, serial: Option<&String>) -> Result<Vec<MappingDevice>> {
	let cmd = if let Some(s) = serial {
		format!("host:transport:{}", s)
	} else {
		String::from("host:transport")
	};
	client.send(cmd.as_bytes())?;
	client.check_ok()?;
	client.send("reverse:list-forward".as_bytes())?;
	client.check_ok()?;
	let result = client.read_string()?;
	// it possible to reuse `forward_list`?
	let devices = result.split("\n")
		.map(|line| line.split_ascii_whitespace())
		.fold(Vec::new(), |mut acc, mut item| {
			let serial = item.next().map(|data| data.to_string());
			let remote = item.next();
			let local = item.next();
			check_and_push(&mut acc, serial, local, remote);
			acc
		});
	Ok(devices)
}