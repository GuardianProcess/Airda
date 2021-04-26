use std::net::TcpStream;
use std::io;
use crate::result::CheckResult;


/// Adb 配置选项，用于Adb Client配置
/// AdbConfig实现Default trait，意味着根据不同平台上尝试查找adb路径
pub struct AdbConfig {
	/// adb 二进制路径，MACOS为/usr/local/bin/adb
	/// Windows则会尝试获取ANDROID_HONE环境变量
	/// Linux平台默认路径为/usr/bin/adb或/usr/local/adb
	pub adb_path: String,
	/// adb server端口号，默认为5037
	pub adb_port: i32,
	/// adb server地址，默认为本机回环地址localhost(127.0.0.1)
	pub adb_host: String,
}

impl Default for AdbConfig {
	fn default() -> Self {
		let adb_path = if cfg!(windows) {
			"".to_string()
		} else if cfg!(macos) { "/usr/local/bin/adb".to_string() } else { "".to_string() };
		Self {
			adb_path: adb_path.to_string(),
			adb_host: "127.0.0.1".to_string(),
			adb_port: 5037,
		}
	}
}

impl AdbConfig {
	pub fn new(adb_path: &str, adb_host: &str, adb_port: i32) -> Self {
		Self {
			adb_path: adb_path.to_string(),
			adb_host: adb_host.to_string(),
			adb_port,
		}
	}
}


/// adb 客户端实现adb server通信该版本不支持USB直连
/// 某些功能无法使用、adb server本身完成，需要配置adb二进制路径
pub struct AdbClient {
	/// adb客户端配置
	pub config: AdbConfig,
	tcp: TcpStream,
}

impl AdbClient {

	/// 创建AdbClient并链接adb server
	/// # Result
	/// 如果无法链接到adb server返回`io::Error`
	pub fn new(config: AdbConfig) -> io::Result<Self> {
		let tcp = TcpStream::connect(format!("{}:{}", config.adb_host, config.adb_port))?;
		Ok(Self {
			config,
			tcp,
		})
	}

	pub fn check_adb_is_fine(&self) -> CheckResult<()> {
		//todo: send data to query adb client, make sure adb it work
		Ok(())
	}

	///
	pub fn send(&self){

	}

}

