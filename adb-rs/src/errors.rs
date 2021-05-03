use std::io;
use std::num::ParseIntError;

#[derive(Debug, Fail)]
pub enum AdbError {
	#[fail(display = "parse adb server protocol error {}", why)]
	ProtocolParseError {
		why: String
	},
	#[fail(display = "got io error when using adb server connection {}", why)]
	IoError {
		why: io::Error,
	},
	#[fail(display = "transfer file error cause file path {} invalid", path)]
	FilePathErr {
		path: String,
	},
	#[fail(display = "failed to write msg to adb server, cause {}", cause)]
	WriteMsgErr {
		cause: String
	},
	#[fail(display = "failed to transfer file {} to android device", filename)]
	FileTransferErr {
		filename: String,
		cause: String,
	}
}

impl From<std::fmt::Error> for AdbError {
	fn from(e: std::fmt::Error) -> Self {
		Self::WriteMsgErr {
			cause: e.to_string()
		}
	}
}

impl From<io::Error> for AdbError {
	fn from(e: io::Error) -> Self {
		Self::IoError {
			why: e,
		}
	}
}

impl From<ParseIntError> for AdbError {
	fn from(e: ParseIntError) -> Self {
		Self::ProtocolParseError {
			why: e.to_string(),
		}
	}
}