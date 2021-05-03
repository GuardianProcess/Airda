use std::io;
use std::io::{Read, Write};
use std::iter::FromIterator;
use std::net::TcpStream;
use std::str::FromStr;
use std::time::Duration;

use bytes::BytesMut;

use crate::errors::AdbError;
use crate::result::CheckResult;

use super::Result;

/// Adb 配置选项，用于Adb Client配置
/// AdbConfig实现Default trait，意味着根据不同平台上尝试查找adb路径
#[derive(Clone, Debug, Eq, PartialEq)]
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
#[derive(Debug)]
pub struct AdbClient {
    /// adb客户端配置
    pub config: AdbConfig,
    tcp: TcpStream,
}

impl Read for AdbClient {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.tcp.read(buf)
    }
}

impl Write for AdbClient {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.tcp.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.tcp.flush()
    }
}


impl AdbClient {
    /// 创建AdbClient并链接adb server
    /// # Result
    /// 如果无法链接到adb server返回`io::Error`
    pub fn new(config: AdbConfig) -> Result<Self> {
        let tcp = TcpStream::connect(format!("{}:{}", config.adb_host, config.adb_port))?;
        Ok(Self {
            config,
            tcp,
        })
    }

    pub fn set_timeout(&self, timeout: Option<Duration>) -> Result<()> {
        self.tcp.set_read_timeout(timeout)?;
        self.tcp.set_write_timeout(timeout)?;
        Ok(())
    }

    pub fn check_adb_is_fine(&self) -> CheckResult<()> {
        //todo: send data to query adb client, make sure adb it work
        Ok(())
    }

    pub fn rend_full_string(&mut self) -> Result<String> {
        let mut buf = String::new();
        self.tcp.read_to_string(&mut buf)?;
        Ok(buf)
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

    pub fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read(&mut buf)?;
        let data = u32::from_le_bytes(buf);
        Ok(data)
    }

    pub fn read_n(&mut self, n: u64) -> Result<BytesMut> {
        Self::read_n_bytes(&mut self.tcp, n)
    }

    pub fn read_n_bytes<R>(read: R, n: u64) -> Result<BytesMut> where R: Read {
        let mut res = Vec::new();
        let read_size = read.take(n).read_to_end(&mut res)?;
        if read_size < n as usize {
            return Err(AdbError::IoError {
                why: io::Error::new(io::ErrorKind::UnexpectedEof,
                                    format!("expect read {} bytes but read {} bytes", n, read_size))
            });
        }
        Ok(BytesMut::from_iter(res))
    }

    pub fn read_n_string<R>(read: R, n: u64) -> Result<String> where R: Read {
        let mut chunk = read.take(n);
        let mut res = String::new();
        let read_size = chunk.read_to_string(&mut res)?;
        if read_size < n as usize {
            return Err(AdbError::IoError {
                why: io::Error::new(io::ErrorKind::UnexpectedEof,
                                    format!("expect read {} bytes but read {} bytes", n, read_size))
            });
        }
        Ok(res)
    }

    pub fn conn_code(&mut self) -> Result<AdbConnState> {
        let mut code = [0u8; 4];
        self.tcp.read_exact(&mut code)?;
        Ok(AdbConnState::from(&code))
    }

    pub fn check_code_is(&mut self, except_code: AdbConnState) -> Result<bool> {
        let state = self.conn_code()?;
        Ok(state == except_code)
    }

    pub fn check_ok(&mut self) -> Result<()> {
        if !self.check_code_is(AdbConnState::OKAY)? {
            return Err(AdbError::IoError {
                why: io::Error::new(io::ErrorKind::ConnectionRefused, "got wrong state from adb server")
            });
        }
        Ok(())
    }

    pub fn recv_full(&mut self, buf: &mut [u8]) -> Result<()> {
        self.tcp.read_exact(buf)?;
        Ok(())
    }

    /// 发送adb协议，send方法会对发送的协议计算长度
    /// 并按照<header><command>形式发送至adb server
    pub fn send(&mut self, command: &[u8]) -> Result<usize> {
        let size = u32::to_be_bytes("host:version".len() as u32)
            .iter()
            .fold(String::new(), |mut x, ch| {
                x.push_str(format!("{:X}", ch).as_str());
                x
            });
        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(size.as_bytes());
        buffer.extend_from_slice(command);
        let size = self.tcp.write(buffer.as_ref())?;
        Ok(size)
    }
}

impl Clone for AdbClient {
    fn clone(&self) -> Self {
        let config = self.config.clone();
        AdbClient::new(config).unwrap()
    }
}