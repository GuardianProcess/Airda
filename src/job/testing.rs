use crate::errors::error::ExecuteResult;
use crate::resource::resource::Resource;


/// 自动化测试任务Trait，实现Drop方法，用于资源释放操作
pub trait TestJob: Drop + Sync {
	/// 测试任务所需要的资源列表
	fn need_resources<V: Resource>(resources: Vec<V>);

	/// 添加测试任务所需要的资源
	fn need_resource<V: Resource>(resource: Vec<V>);

	/// 初始化所有测试任务所需要的资源，任务准备等
	fn initialize() -> ExecuteResult<()>;

	/// 资源初始化执行前执行该函数，如果执行失败则不会执行该任务
	fn before_init() -> ExecuteResult<()>;

	/// 资源初始化后执行该函数，如果执行失败则不会执行该任务
	fn after_init() -> ExecuteResult<()>;

	/// 测试执行前执行该函数，如果执行失败则不会执行该任务
	fn before_start() -> ExecuteResult<()>;

	/// 测试结束后执行该函数，，如果执行失败则不会执行该任务
	fn after_start() -> ExecuteResult<()>;
}


