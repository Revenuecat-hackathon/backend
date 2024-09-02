use std::collections::HashMap;

use crate::utils::{
    api_response::{self, ApiResponse},
    app_state::{self, AppState},
    global_variables::DYNAMO_DB_TABLE_NAME,
    jwt::{add_to_blacklist, encode_jwt},
    models::User,
    user::get_user_from_email,
};
use actix_web::{post, web, HttpRequest};
use anyhow::Result;
use aws_sdk_dynamodb::types::AttributeValue;
use sha256::digest;

#[derive(serde::Deserialize)]
struct RegisterRequest {
    name: String,
    email: String,
    password: String,
}

#[derive(serde::Deserialize)]
struct LoginRequest {
    email: String,
    #[allow(dead_code)]
    password: String,
}

#[post("/register")]
pub async fn register(
    app_state: web::Data<AppState>,
    request: web::Json<RegisterRequest>,
) -> Result<ApiResponse, ApiResponse> {
    let result = get_user_from_email(&app_state.dynamo_client, request.email.clone())
        .await
        .map_err(|err| ApiResponse::new(500, err.to_string()))?;

    if let Some(_) = result.items.and_then(|items| items.first().cloned()) {
        return Err(ApiResponse::new(409, "User already exists".to_string()));
    }

    let mut item = HashMap::new();

    item.insert(
        "id".to_string(),
        AttributeValue::S(format!("USER#{}", uuid::Uuid::new_v4())),
    );
    item.insert("name".to_string(), AttributeValue::S(request.name.clone()));
    item.insert(
        "email".to_string(),
        AttributeValue::S(request.email.clone()),
    );
    item.insert(
        "password".to_string(),
        AttributeValue::S(digest(request.password.clone())),
    );

    let table_name = DYNAMO_DB_TABLE_NAME.clone();

    app_state
        .dynamo_client
        .put_item()
        .table_name(table_name)
        .set_item(Some(item))
        .send()
        .await
        .map_err(|err| ApiResponse::new(500, err.to_string()))?;

    Ok(api_response::ApiResponse::new(
        200,
        format!("created user: {} - {}", request.name, request.email),
    ))
}

#[post("/login")]
pub async fn login(
    app_state: web::Data<app_state::AppState>,
    request: web::Json<LoginRequest>,
) -> Result<ApiResponse, ApiResponse> {
    let result = get_user_from_email(&app_state.dynamo_client, request.email.clone())
        .await
        .map_err(|err| ApiResponse::new(409, err.to_string()))?;

    let user = result
    .items
    .and_then(|items| items.first().cloned())
    .ok_or_else(|| ApiResponse::new(404, "User not found. Please check the email address.".to_string()))
    .and_then(|item| {
        User::from_item(&item).map_err(|err| {
            ApiResponse::new(
                500,
                format!("Failed to parse user data: {}. This might be due to data corruption or schema mismatch.", err),
            )
        })
    })?;

    let token =
        encode_jwt(user.email, user.id).map_err(|err| ApiResponse::new(500, err.to_string()))?;

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
