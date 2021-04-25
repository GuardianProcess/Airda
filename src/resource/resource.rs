use crate::errors::resource::ResourceResult;

#[derive(Debug)]
pub enum ResourceType {
	Software,
	Hardware,
}

pub trait Resource {
	fn res_type(&self) -> ResourceType;

	fn create<T>(&mut self) -> ResourceResult<T>;

}