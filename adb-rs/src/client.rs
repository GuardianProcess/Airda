use std::net::TcpStream;
use std::io;
use crate::result::CheckResult;
use std::io::{Write, Read};
use bytes::BytesMut;
use std::str::FromStr;
use super::Result;
use crate::errors::AdbError;

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

#[derive(Debug, Eq, PartialEq)]
pub enum AdbConnState {
	OKAY,
	FAIL,
	DENT,
	DONE,
	DATA,
}

impl Into<String> for AdbConnState {
	fn into(self) -> String {
		match &self {
			Self::OKAY => "OKAY".to_string(),
			Self::FAIL => "FAIL".to_string(),
			Self::DENT => "DENT".to_string(),
			Self::DONE => "DONE".to_string(),
			Self::DATA => "DATA".to_string(),
		}
	}
}

impl From<&[u8; 4]> for AdbConnState {
	fn from(buf: &[u8; 4]) -> Self {
		let code = String::from_utf8_lossy(buf.as_ref());
		if code == "OKAY" {
			Self::OKAY
		} else {
			Self::FAIL
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

	pub fn recv_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
		self.tcp.read_to_string(buf)
	}

	pub fn read_string(&mut self) -> Result<String> {
		// adb返回数据并非整形数组而是字符串，因此需要转换为字符串后方可使用
		// 解析消息长度
		let mut len = [0u8; 4];
		self.tcp.read_exact(&mut len)?;
		let str_len = String::from_utf8_lossy(&len);
		let u32_len = u32::from_str(str_len.as_ref())?;
		// 读取消息
		Self::read_n_string(&mut self.tcp, u32_len as u64)
	}

	fn read_n_string<R>(read: R, n: u64) -> Result<String> where R: Read {
		let mut chunk = read.take(n);
		let mut res = String::new();
		let read_size = chunk.read_to_string(&mut res)?;
		if read_size < n as usize {
			return Err(AdbError::IoError {
				why: io::Error::new(io::ErrorKind::UnexpectedEof,
				                    format!("expect read {} bytes but read {} bytes", n, read_size))
			})
		}
		Ok(res)
	}


	pub fn check_ok(&mut self) -> io::Result<()> {
		let mut code = [0u8; 4];
		self.tcp.read_exact(&mut code)?;
		let state = AdbConnState::from(&code);
		if state == AdbConnState::FAIL {
			return Err(io::Error::new(io::ErrorKind::ConnectionRefused, "got wrong state from adb server"))
		}
		Ok(())
	}

	pub fn recv_full(&mut self, buf: &mut [u8]) -> io::Result<()> {
		self.tcp.read_exact(buf)
	}

	///
	pub fn send(&mut self, command: &[u8]) -> io::Result<usize> {
		let size = u32::to_be_bytes("host:version".len() as u32)
			.iter()
			.fold(String::new(), |mut x, ch| {
				x.push_str(format!("{:X}", ch).as_str());
				x
			});
		let mut buffer = BytesMut::new();
		buffer.extend_from_slice(size.as_bytes());
		buffer.extend_from_slice(command);
		self.tcp.write(buffer.as_ref())
	}
}

