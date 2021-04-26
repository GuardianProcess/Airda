use crate::errors::resource::ResourceResult;

#[derive(Debug)]
pub enum ResourceType {
	/// 软件资源 例如端口，
	Software,
	Hardware,
}

/// 测试过程中用到的资源
pub trait Resource {
	/// 返回资源类型
	fn resource_type(&self) -> ResourceType;

	fn create<T>(&mut self) -> ResourceResult<T>;
}