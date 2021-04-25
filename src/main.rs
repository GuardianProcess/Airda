#![feature(try_trait)]

mod errors;
mod resource;
mod execution;
mod controller;
mod config;
mod utils;

#[macro_use]
extern crate failure;

use actix_web::{App, HttpServer, web};
use crate::controller::api::create_task;
use crate::config::runner::CONFIG;
use crate::errors::error::ExecuteResult;
use actix_web::middleware::Logger;
use crate::utils::must_get_log_file;

fn init() {
	log4rs::init_file(must_get_log_file(), Default::default()).expect("init logger error");
}

#[actix_web::main]
async fn main() -> ExecuteResult<()> {
	init();
	HttpServer::new(||
		App::new()
			.wrap(Logger::default())
			.wrap(Logger::new("%a %t %{User-Agent}i"))
			.service(web::scope("/task").service(create_task))
	).bind(&CONFIG.server.ip_addr)?
		.run()
		.await?;
	Ok(())
}
