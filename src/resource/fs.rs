use std::path::{Path, PathBuf};
use crate::resource::resource::{Resource, ResourceType};
use crate::errors::resource::{ResourceResult, ResourceError};
use std::fs;
use std::ffi::{OsString};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::fs::File;

pub struct FsResource<P: AsRef<Path>> {
	root_dir: P,
	file_mapping: HashMap<OsString, fs::File>,
}

impl<P: AsRef<Path>> FsResource<P> {
	pub fn new(path: P) -> ResourceResult<Self> {
		Self::check_exists(&path)?;
		Ok(Self {
			root_dir: path,
			file_mapping: HashMap::new(),
		})
	}

	fn check_exists(path: &P) -> ResourceResult<()> {
		let res = fs::metadata(&path);
		if let Err(ref e) = res {
			match e.kind() {
				ErrorKind::NotFound => fs::create_dir_all(path)?,
				_ => res.map(|_| ())?,
			}
		}
		Ok(())
	}

	pub fn getfile_by_name(&self, path: P) -> Option<&File> {
		self.file_mapping.get(path.as_ref().file_name()?)
	}

	pub fn create_dir(&mut self, path: P) -> ResourceResult<PathBuf> {
		let ph = Path::new(self.root_dir.as_ref()).join(path);
		fs::create_dir_all(&ph)?;
		Ok(ph.to_path_buf())
	}

	pub fn create_file(&mut self, path: P) -> ResourceResult<()> {
		let filename = path
			.as_ref()
			.file_name()
			.ok_or(ResourceError::CreateError {
				why: format!("file {} do not exists", path.as_ref().display())
			})?;

		let ph = Path::new(self.root_dir.as_ref()).join(filename);
		let filepath = ph.as_path();
		let file = fs::File::create(filepath)?;
		self.file_mapping.insert(filename.to_owned(), file);
		Ok(())
	}


}

impl<P: AsRef<Path>> Drop for FsResource<P> {
	fn drop(&mut self) {
		self.file_mapping.clear()
	}
}


impl<P: AsRef<Path>> Resource for FsResource<P> {
	fn res_type(&self) -> ResourceType {
		return ResourceType::Software;
	}

	fn create<T>(&mut self) -> ResourceResult<T> {
		unimplemented!()
	}
}