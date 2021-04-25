use actix_web::{post, web, HttpResponse, Result};
use super::defines::TaskParam;
use crate::controller::defines::CreateResult;

#[post("/create")]
pub(crate) async fn create_task(_param: web::Json<TaskParam>) -> Result<HttpResponse> {
	let data = CreateResult::ok_data("ok");
	let resp = HttpResponse::Ok().json(&data);
	Ok(resp)
}
