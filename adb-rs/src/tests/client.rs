use crate::client::{AdbClient, AdbConfig};
use crate::Result;
use std::str::FromStr;

#[test]
fn test_adb_client() -> Result<()> {
	let config = AdbConfig::new("/usr/local/bin/adb", "127.0.0.1", 5037);
	let _adb = AdbClient::new(config)?;
	Ok(())
}

#[test]
fn test_get_adb_version() -> Result<()> {
	let config = AdbConfig::new("/usr/local/bin/adb", "127.0.0.1", 5037);
	let mut adb = AdbClient::new(config)?;
	let command = String::from("host:version");
	adb.send(command.as_bytes())?;
	adb.check_ok()?;
	let res = adb.read_string()?;
	assert_eq!(Ok(29), u32::from_str(res.as_str()));
	Ok(())
}