use crate::{AdbClient, AdbConfig, Result};
use std::str::{FromStr, SplitAsciiWhitespace};
use crate::adb::base_type::{MappingDevice, MappingType};


pub struct AdbDevice {
	#[cfg(feature = "server")]
	inner: AdbClient,
	serial: String,
}

impl AdbDevice {
	fn new(inner: AdbClient, serial: String) -> Self {
		Self {
			inner,
			serial
		}
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

	/// 获取当所有反向代理列表
	pub fn reverse_list(&mut self) -> Result<Vec<MappingDevice>> {
		reverse_list(&mut self.inner, None)
	}

	pub fn devices(&mut self) -> Result<Vec<AdbDevice>> {
		self.inner.send("host:devices".as_bytes())?;
		self.inner.check_ok()?;
		let device_str = self.inner.read_string()?;
		let trim_space: &[_] = &[' ', '\t'];
		let devices = device_str
			.split("\n")
			.filter(|x| !x.trim_matches(trim_space).is_empty())
			.map(|line| {
				let serial = line.to_string();
				AdbDevice::new(self.inner.clone(), serial)
			}).collect::<Vec<AdbDevice>>();
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