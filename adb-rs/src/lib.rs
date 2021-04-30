mod client;
#[cfg(test)]
mod tests;

#[macro_use]
extern crate failure;

mod result;
mod errors;
mod protocol;

pub use client::{AdbClient, AdbConfig};
pub use result::CheckResult;
pub use protocol::{AdbCommand};

type Result<T> = std::result::Result<T, errors::AdbError>;