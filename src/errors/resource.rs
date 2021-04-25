use crate::errors::error::ExecuteError;
use std::env::JoinPathsError;


pub type ResourceResult<T> = std::result::Result<T, ResourceError>;

#[derive(Fail, Debug)]
pub enum ResourceError {
	#[fail(display = "create resource error {}", why)]
	CreateError {
		why: String
	},
	#[fail(display = "execute command error {}", e)]
	ExecError {
		e: ExecuteError,
	}
}

impl From<ExecuteError> for ResourceError {
	fn from(e: ExecuteError) -> Self {
		Self::ExecError {
			e
		}
	}
}


impl From<std::io::Error> for ResourceError {
	fn from(e: std::io::Error) -> Self {
		Self::CreateError {
			why: e.to_string(),
		}
	}
}

impl From<JoinPathsError> for ResourceError {
	fn from(e: JoinPathsError) -> Self {
		Self::CreateError {
			why: e.to_string(),
		}
	}
}
