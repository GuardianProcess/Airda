use crate::resource::fs::FsResource;
use crate::errors::resource::{ResourceResult, ResourceError};
use std::io::Write;
use std::fs;

#[test]
fn test_fs() -> ResourceResult<()> {
	let mut f = FsResource::new("cache")?;
	let filename = "test.txt";
	f.create_file(filename)?;
	let mut file = f.getfile_by_name(filename).ok_or(ResourceError::CreateError { why: "file do not exists".to_string() })?;
	file.write(String::from("hello").as_bytes())?;
	Ok(())
}

#[test]
fn test_dir() -> ResourceResult<()> {
	let mut f = FsResource::new("cache")?;
	let abs = f.create_dir("TestLog")?;
	let mut file = fs::File::create(abs.join("file.txt"))?;
	file.write(String::from("hello").as_bytes())?;
	Ok(())
}

