use actix_web::{get, web};

use crate::utils::{api_response::ApiResponse, app_state};

#[get("/index")]
pub async fn index(
    #[allow(unused_variables)] app_state: web::Data<app_state::AppState>,
) -> Result<ApiResponse, ApiResponse> {
    Ok(ApiResponse::new(200, format!("OK")))
}
