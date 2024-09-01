use std::collections::HashMap;

use actix_web::{post, web};
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};

use crate::utils::{
    api_response::ApiResponse, app_state::AppState, global_variables::DYNAMO_DB_TABLE_NAME,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDynamodbModelRequest {
    pub art_info: String,
    pub name: String,
    pub author: String,
    pub art_unique_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDynamodbModel {
    pub partition_key: String,
    pub sort_key: String,
    pub name: String,
    pub author: String,
    pub art_unique_id: String, // I think this is needed to query the item later
}

impl CreateDynamodbModel {
    fn to_item(&self) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();
        item.insert(
            "PK".to_string(),
            AttributeValue::S(self.partition_key.clone()),
        );
        item.insert("SK".to_string(), AttributeValue::S(self.sort_key.clone()));
        item.insert("name".to_string(), AttributeValue::S(self.name.clone()));
        item.insert("author".to_string(), AttributeValue::S(self.author.clone()));
        item.insert(
            "art_unique_id".to_string(),
            AttributeValue::S(self.art_unique_id.clone()),
        );
        item
    }
}

#[post("/dynamodb/create")]
pub async fn create_dynamodb(
    app_state: web::Data<AppState>,
    request: web::Json<CreateDynamodbModelRequest>,
) -> Result<ApiResponse, ApiResponse> {
    let create_dynamodb_model = CreateDynamodbModel {
        partition_key: format!("{}#{}", request.art_info.clone(), uuid::Uuid::new_v4()),
        sort_key: "ARTINFO".to_string(),
        name: request.name.clone(),
        author: request.author.clone(),
        art_unique_id: request.art_unique_id.clone(),
    };

    let table_name = (DYNAMO_DB_TABLE_NAME).clone();

    app_state
        .dynamo_client
        .put_item()
        .table_name(table_name)
        .set_item(Some(create_dynamodb_model.to_item()))
        .send()
        .await
        .map_err(|err| ApiResponse::new(500, err.to_string()))?;
    Ok(ApiResponse::new(
        200,
        format!("DynamoDB item is successfully created!"),
    ))
}
