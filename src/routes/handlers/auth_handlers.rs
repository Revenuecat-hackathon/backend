use crate::utils::{
    api_response::{self, ApiResponse},
    app_state,
    jwt::{add_to_blacklist, encode_jwt},
};
use actix_web::{post, web, HttpRequest};
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use sha256::digest;

#[derive(serde::Deserialize)]
struct RegisterModel {
    name: String,
    email: String,
    password: String,
}

#[derive(serde::Deserialize)]
struct LoginModel {
    email: String,
    password: String,
}

#[post("/register")]
pub async fn register(
    app_state: web::Data<app_state::AppState>,
    register_json: web::Json<RegisterModel>,
) -> Result<ApiResponse, ApiResponse> {
    let user_model = entity::user::ActiveModel {
        name: Set(register_json.name.clone()),
        email: Set(register_json.email.clone()),
        password: Set(digest(&register_json.password)),
        ..Default::default()
    }
    .insert(&app_state.db)
    .await
    .map_err(|err| ApiResponse::new(500, err.to_string()))?;

    Ok(api_response::ApiResponse::new(
        200,
        format!("{}", user_model.id),
    ))
}

#[post("/login")]
pub async fn login(
    app_state: web::Data<app_state::AppState>,
    login_json: web::Json<LoginModel>,
) -> Result<ApiResponse, ApiResponse> {
    let user_data = entity::user::Entity::find()
        .filter(
            Condition::all()
                .add(entity::user::Column::Email.eq(login_json.email.clone()))
                .add(entity::user::Column::Password.eq(digest(&login_json.password))),
        )
        .one(&app_state.db)
        .await
        .map_err(|err| ApiResponse::new(500, err.to_string()))?
        .ok_or(ApiResponse::new(404, "User not found".to_owned()))?;

    let token = encode_jwt(user_data.email, user_data.id)
        .map_err(|err| ApiResponse::new(500, err.to_string()))?;

    Ok(api_response::ApiResponse::new(
        200,
        format!("{{'token': '{}'}}", token),
    ))
}

#[post("/logout")]
async fn logout(
    app_state: web::Data<app_state::AppState>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = auth_str.replace("Bearer ", "");
                // Add token to blacklist with an expiry time of 24 hours
                if let Err(e) = add_to_blacklist(&app_state.redis_client, &token).await {
                    return Err(ApiResponse::new(
                        500,
                        format!("Failed to blacklist token: {}", e),
                    ));
                }
                return Ok(ApiResponse::new(200, "Successfully logged out".to_string()));
            }
        }
    }
    Err(ApiResponse::new(400, "Invalid token".to_string()))
}
