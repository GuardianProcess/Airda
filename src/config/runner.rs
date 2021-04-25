use serde::Deserialize;
use std::fs::File;
use std::path::Path;
use std::io::Read;
use crate::errors::error::ExecuteResult;
use crate::utils::must_get_log_file;
use lazy_static::lazy_static;

lazy_static! {
	pub static ref CONFIG: Runner = init_log();
}

fn init_log() -> Runner {
	let log_path = must_get_log_file();
	match read_config(log_path) {
		Ok(e) => e,
		Err(e) => panic!(e)
	}
}

#[derive(Deserialize)]
pub struct Runner {
	pub server: Server,
	pub gateway: Gateway,
}

#[derive(Deserialize)]
pub struct Gateway {
	pub endpoint: String,
}

#[derive(Deserialize)]
pub struct Server {
	pub ip_addr: String,
	pub port: i32,
}

pub fn read_config<P: AsRef<Path>>(path: P) -> ExecuteResult<Runner> {
	let mut f = File::open(path)?;
	let mut data = String::new();
	f.read_to_string(&mut data)?;
	Ok(toml::from_str(data.as_str())?)
}