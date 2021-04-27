mod client;
#[cfg(test)]
mod tests;

#[macro_use]
extern crate failure;

mod result;
mod protocol;

pub use client::{AdbClient, AdbConfig};
pub use result::CheckResult;
pub use protocol::{AdbMessage, AdbCommand};
