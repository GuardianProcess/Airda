use std::path::PathBuf;

pub fn must_get_root_path() -> PathBuf {
	if let Ok(path) = std::env::current_dir() {
		return path;
	}
	panic!("There are insufficient permissions to access the current directory")
}

pub fn must_get_log_file() -> PathBuf {
	let work_dir = must_get_root_path();
	let config = work_dir.join("config.toml");
	if !config.exists() {
		panic!("config.toml not found in {}", work_dir.display())
	}
	config
}
