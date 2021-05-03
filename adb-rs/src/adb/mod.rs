mod server;
mod base_type;

pub use base_type::{MappingType, MappingDevice, DeviceStatus, DeviceEvent, ShellResult};


mod test {
	use crate::adb::server::AndroidDebugBridge;
	use crate::AdbConfig;
	use crate::Result;

	#[test]
	#[cfg(feature = "server")]
	pub fn test_version() -> Result<()> {
		let config = AdbConfig::new("/usr/local/bin/adb", "127.0.0.1", 5037);
		let mut adb = AndroidDebugBridge::new(config)?;
		let version = adb.adb_version()?;
		assert_eq!(29, version);
		Ok(())
	}
}