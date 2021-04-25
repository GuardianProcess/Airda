use std::collections::BTreeMap;
use crate::execution::task::Task;
use std::path::Path;
use std::ffi::OsStr;
use std::hash::Hash;
use std::fmt::Debug;
use log::error;
use std::sync::{Mutex, Arc};
use std::cell::RefCell;
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct Launcher<P: 'static>
	where P: AsRef<Path> + AsRef<OsStr> + Hash + Eq + Clone + Debug,
{
	man: Mutex<BTreeMap<String, Arc<RefCell<Task<P>>>>>,
	task_chan_r: Receiver<String>,
	task_chan_s: Arc<Sender<String>>,
}

impl<P: 'static> Launcher<P>
	where P: AsRef<Path> + AsRef<OsStr> + Hash + Eq + Clone + Debug,
{
	pub fn new() -> Self {
		let (sx, rx) = channel();
		Self {
			man: Mutex::new(Default::default()),
			task_chan_r: rx,
			task_chan_s: Arc::new(sx),
		}
	}

	async fn task(task: Arc<RefCell<Task<P>>>, sender: Arc<Sender<String>>) {
		match task.try_borrow_mut() {
			Ok(mut t) => {
				if let Err(e) = t.wait_for_end() {
					error!("task {} ->  execute task error {}", t.task_id(), e);
				}
				if let Err(e) = sender.send(t.task_id()) {
					error!("send task to channel error {}", e);
				}
			}
			Err(e) => error!("try borrow mut task error {}", e)
		}
		drop(sender);
	}

	pub fn submit_task(&mut self, task: Task<P>) {
		let task = Arc::new(RefCell::new(task));
		let run_task = task.clone();
		let sender = self.task_chan_s.clone();

		actix::spawn(Self::task(run_task, sender));

		if let Ok(man) = self.man.get_mut() {
			let id = task.borrow().task_id();
			man.insert(id, task);
		}
	}

	// pub fn poll(&mut self) -> Option<&String> {
	// 	if let Ok(man) = self.man.get_mut() {
	// 		for (id, task) in man {
	// 			match task.borrow_mut().try_wait() {
	// 				Ok(exited) => match exited {
	// 					Some(_) => return Some(id),
	// 					None => continue,
	// 				},
	// 				Err(e) => {
	// 					error!("try wait task {} error {}", id, e);
	// 					return Some(id)
	// 				}
	// 			}
	// 		}
	// 	}
	// 	None
	// }

	fn task_loop(&mut self) {
		loop {
			match self.task_chan_r.recv() {
				Ok(task) => {
					if let Ok(man) = self.man.get_mut() {
						man.remove(&task);
					}
				}
				Err(e) => {
					error!("recv error {}", e);
				}
			}
		}
	}
}


