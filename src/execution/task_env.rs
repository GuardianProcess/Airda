use crate::execution::task::ArgsProtocol;

pub struct TaskEnvironment {
	app: String,

}


impl ArgsProtocol for TaskEnvironment {
	fn to_command_args(&self) -> Vec<String> {
		unimplemented!()
	}
}