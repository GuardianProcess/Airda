use std::io;

pub type CheckResult<T> = std::result::Result<T, CheckError>;


#[derive(Fail, Debug)]
pub enum CheckError {
	#[fail(display = "check adb error cause {} the raw error is {}", cause, raw_err)]
	AdbCheckError {
		raw_err: io::Error,
		cause: String
	}
}