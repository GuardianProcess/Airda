use std::io;
use crate::client::{AdbClient, AdbConfig};

#[test]
fn test_adb_client() -> io::Result<()> {
	let config = AdbConfig::new("/usr/local/bin/adb", "127.0.0.1", 5037);
	let adb = AdbClient::new(config);
	Ok(())
}