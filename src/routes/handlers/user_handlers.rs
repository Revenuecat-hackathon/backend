use actix_web::{get, web};
use aws_sdk_dynamodb::types::AttributeValue;

use crate::utils::{
    api_response::{self, ApiResponse},
    app_state,
    global_variables::DYNAMO_DB_TABLE_NAME,
    jwt::Claims,
    models::User,
};

#[derive(serde::Serialize, serde::Deserialize)]
struct UpdateUserModel {
    name: String,
}

#[get("")]
pub async fn user(
    app_state: web::Data<app_state::AppState>,
    claim_data: Claims,
) -> Result<ApiResponse, ApiResponse> {
    let table_name = DYNAMO_DB_TABLE_NAME.clone();
    let result = app_state
        .dynamo_client
        .query()
        .table_name(table_name)
        .key_condition_expression("id = :id")
        .expression_attribute_values(":id", AttributeValue::S(claim_data.id))
        .send()
        .await
        .map_err(|err| ApiResponse::new(500, err.to_string()))?;

    let user = result
        .items
        .and_then(|items| items.first().cloned())
        .ok_or_else(|| ApiResponse::new(404, "User not found".to_string()))
        .and_then(|item| {
            User::from_item(&item).map_err(|err| ApiResponse::new(500, err.to_string()))
        })?;
    Ok(api_response::ApiResponse::new(
        200,
        format!("{{ 'name': '{}', 'email': '{}'}}", user.name, user.email),
    ))
}

// #[post("update")]
// pub async fn update_user(
//     app_state: web::Data<app_state::AppState>,
//     user_data: web::Json<UpdateUserModel>,
//     claim_data: Claims,
// ) -> Result<ApiResponse, ApiResponse> {
//     let mut user_model = entity::user::Entity::find_by_id(claim_data.id)
//         .one(&app_state.db)
//         .await
//         .map_err(|err| api_response::ApiResponse::new(500, err.to_string()))?
//         .ok_or(api_response::ApiResponse::new(
//             404,
//             "User not found".to_owned(),
//         ))?
//         .into_active_model();

//     user_model.name = Set(user_data.name.clone());
//     user_model
//         .update(&app_state.db)
//         .await
//         .map_err(|err| api_response::ApiResponse::new(500, err.to_string()))?;

//     Ok(api_response::ApiResponse::new(
//         200,
//         "User updated".to_owned(),
//     ))
// }
