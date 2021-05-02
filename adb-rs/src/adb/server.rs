use crate::{AdbClient, AdbConfig, Result};
use std::str::{FromStr, SplitAsciiWhitespace};
use crate::adb::base_type::{ForwardDevice, MappingType};


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

	pub fn forward(&mut self, local: u32, remote: MappingType, norebind: Option<bool>) -> Result<()> {
		forward(&mut self.inner, &self.serial, local, remote, norebind)
	}

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


fn forward_list(client: &mut AdbClient, serial: Option<&String>) -> Result<Vec<ForwardDevice>> {
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
			handle_forward(&mut acc, &mut item);
			acc
		});
	Ok(res)
}

fn handle_forward(acc: &mut Vec<ForwardDevice>, item: &mut SplitAsciiWhitespace) {
	let serial = item.next();
	let local = item.next();
	let remote = item.next();
	if serial.is_some() && local.is_some() && remote.is_some() {
		if let Ok(port) = u32::from_str(local.unwrap()) {
			acc.push(ForwardDevice::new(serial.unwrap().to_string(), port, remote.unwrap()));
		}
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
