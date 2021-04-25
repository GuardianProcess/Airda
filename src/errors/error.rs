use std::string::FromUtf8Error;
use std::num::ParseIntError;
use std::option::NoneError;


#[derive(Debug, Fail)]
pub enum ExecuteError {
	#[fail(display = "adb execute error {}", why)]
	AdbExecuteError {
		why: String
	},
	#[fail(display = "program execute error {}", why)]
	TaskError {
		why: String
	},
	#[fail(display = "io error {}", e)]
	IoError {
		e: std::io::Error,
	},
	#[fail(display = "utf8 convert error {}", e)]
	Utf8Error {
		e: FromUtf8Error,
	},
	#[fail(display = "parse int error {}", e)]
	ParseIntError {
		e: ParseIntError,
	},
	#[fail(display = "read toml config error {}", e)]
	TomlConfigError {
		e: toml::de::Error,
	},
}

impl From<toml::de::Error> for ExecuteError {
	fn from(e: toml::de::Error) -> Self {
		Self::TomlConfigError {
			e,
		}
	}
}

impl From<NoneError> for ExecuteError {
	fn from(_: NoneError) -> Self {
		Self::AdbExecuteError {
			why: "adb no output".to_string()
		}
	}
}

impl From<ParseIntError> for ExecuteError {
	fn from(e: ParseIntError) -> Self {
		Self::ParseIntError {
			e
		}
	}
}

impl From<FromUtf8Error> for ExecuteError {
	fn from(e: FromUtf8Error) -> Self {
		Self::Utf8Error {
			e
		}
	}
}

impl From<nix::Error> for ExecuteError {
	fn from(e: nix::Error) -> Self {
		Self::TaskError {
			why: e.to_string()
		}
	}
}

impl From<std::io::Error> for ExecuteError {
	fn from(e: std::io::Error) -> Self {
		Self::IoError {
			e
		}
	}
}

pub type ExecuteResult<T> = std::result::Result<T, ExecuteError>;
