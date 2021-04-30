use std::num::ParseIntError;
use std::io;

#[derive(Debug, Fail)]
pub enum AdbError {
	#[fail(display = "parse adb server protocol error {}", why)]
	ProtocolParseError {
		why: String
	},
	#[fail(display = "got io error when using adb server connection {}", why)]
	IoError {
		why: io::Error,
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