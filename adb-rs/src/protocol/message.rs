use std::ops::BitXor;
use tokio_util::codec::Decoder;
use std::io::{Error, Read};

/// ADB AdbCommand Type
/// Message主要包含以下类型
/// ```c
/// #define A_SYNC 0x434e5953
/// #define A_CNXN 0x4e584e43
/// #define A_AUTH 0x48545541
/// #define A_OPEN 0x4e45504f
/// #define A_OKAY 0x59414b4f
/// #define A_CLSE 0x45534c43
/// #define A_WRTE 0x45545257
/// ```
/// See https://android.googlesource.com/platform/system/core/+/refs/heads/android10-c2f2-release/adb/protocol.txt
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum AdbCommand {
	SYNC,
	CNXN,
	AUTH,
	OPEN,
	OKAY,
	CLSE,
	WRTE
}

impl BitXor<u32> for AdbCommand {
	type Output = u32;

	fn bitxor(self, rhs: u32) -> Self::Output {
		let data: u32 = self.into();
		data.bitxor(rhs)
	}
}

impl Into<u32> for AdbCommand {
	fn into(self) -> u32 {
		match &self {
			AdbCommand::SYNC => 0x434e5953, // b'SYNC'
			AdbCommand::CNXN => 0x4e584e43, // b'CNXN'
			AdbCommand::AUTH => 0x48545541, // b'AUTH'
			AdbCommand::OPEN => 0x4e45504f, // b'OPEN'
			AdbCommand::OKAY => 0x59414b4f, // b'OKEY'
			AdbCommand::CLSE => 0x45534c43, // b'CLSE'
			AdbCommand::WRTE => 0x45545257, // b'WRTE'
		}
	}
}

impl Into<Vec<u8>> for AdbCommand {
	fn into(self) -> Vec<u8> {
		match &self {
			AdbCommand::SYNC => vec![0x43, 0x4E, 0x59, 0x53], // b'SYNC'
			AdbCommand::CNXN => vec![0x4E, 0x58, 0x4e, 0x43], // b'CNXN'
			AdbCommand::AUTH => vec![0x48, 0x54, 0x55, 0x41], // b'AUTH'
			AdbCommand::OPEN => vec![0x4E, 0x45, 0x50, 0x4F], // b'OPEN'
			AdbCommand::OKAY => vec![0x59, 0x41, 0x4B, 0x4F], // b'OKEY'
			AdbCommand::CLSE => vec![0x45, 0x53, 0x4C, 0x43], // b'CLSE'
			AdbCommand::WRTE => vec![0x45, 0x54, 0x52, 0x57], // b'WRTE'
		}
	}
}

/// AdbMessage
/// # English
/// The transport layer deals in "messages", which consist of a 24 byte
/// header followed (optionally) by a payload.  The header consists of 6
/// 32 bit words which are sent across the wire in little endian format.
/// Receipt of an invalid message header, corrupt message payload, or an
/// unrecognized command MUST result in the closing of the remote connection
/// # 中文
/// 处于传输层的`Message`包含24字节header组成的Payload。
/// Header由无线传输的6个小端序32字节组成。
/// 如果收到无效，损坏的header，或无法识别的命令会导致远程链接关闭
pub struct AdbMessage {
	/// command identifier constant (A_CNXN, ...)
	/// 固定不变的命令标示符，例如CNXN等
	command: AdbCommand,
	/// first argument
	/// 第一个参数
	arg0: u32,
	/// second argument
	/// 第二个参数
	arg1: u32,
	/// length of payload (0 is allowed)
	/// 整个payload（载荷/消息）的长度
	data_len: u32,
	/// crc32 of data payload
	/// payload的校验和（采用CRC32循环冗余校验和算法）
	data_crc32: u32,
	/// command ^ 0xffffffff
	/// 魔数，`命令标示符` 与 `0xffffffff` 的异或结果
	magic: u32,
	/// 数据
	data: Vec<u8>,
}

impl AdbMessage {
	pub fn create(command: AdbCommand, arg0: u32, arg1: u32, data: Vec<u8>) -> Self {
		let magic = command ^ 0xFFFFFFFF;
		let data_len = data.len() as u32;
		let crc32: u32 = data.iter().map(|x| *x as u32).sum();
		Self {
			command,
			arg0,
			arg1,
			data,
			magic,
			data_len,
			data_crc32: crc32 & 0xFFFFFFFF,
		}
	}
}

#[repr(C)]
pub struct RawMessage {
	command: u32,
	arg0: u32,
	arg1: u32,
	data_length: u32,
	data_checksum: u32,
	magic: u32,
}

impl Decoder for AdbMessage {
	type Item = Vec<u8>;
	type Error = std::io::Error;

	// +---+----+----+-----------+-------------+------------+
	// |cmd|arg0|arg1|data_length|data_checksum|unused_magic|
	// +---+----+----+-----------+-------------+------------+
	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		let header_size = 6usize * 4;
		/// 没有读到更多的数据则继续读取
		if src.len() < header_size {
			return Ok(None)
		}
		//todo: parse protocol
		unimplemented!()
	}
}
