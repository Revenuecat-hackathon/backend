use anyhow::Result;
use std::collections::HashMap;

use anyhow::anyhow;
use aws_sdk_dynamodb::types::AttributeValue;
use std::sync::Arc;

use aws_sdk_dynamodb::{operation::query::QueryOutput, Client};

use super::global_variables::DYNAMO_DB_TABLE_NAME;

pub(crate) async fn get_user_from_email(
    dynamo_client: &Arc<Client>,
    email: String,
) -> Result<QueryOutput> {
    let table_name = DYNAMO_DB_TABLE_NAME.clone();
    dynamo_client
        .query()
        .table_name(table_name)
        .index_name("EmailIndex") // Assuming you've created a GSI named "EmailIndex"
        .key_condition_expression("email = :email")
        .expression_attribute_values(":email", AttributeValue::S(email))
        .select(aws_sdk_dynamodb::types::Select::AllAttributes)
        .send()
        .await
        .map_err(anyhow::Error::from)
}

#[derive(Debug)]
pub(crate) struct User {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) email: String,
    #[allow(dead_code)]
    pub(crate) password: String,
}

impl User {
    pub(crate) fn from_item(item: &HashMap<String, AttributeValue>) -> Result<Self> {
        Ok(User {
            id: item
                .get("id")
                .and_then(|v| v.as_s().ok())
                .ok_or_else(|| anyhow!("Missing id"))?
                .to_string(),
            name: item
                .get("name")
                .and_then(|v| v.as_s().ok())
                .ok_or_else(|| anyhow!("Missing name"))?
                .to_string(),
            email: item
                .get("email")
                .and_then(|v| v.as_s().ok())
                .ok_or_else(|| anyhow!("Missing email"))?
                .to_string(),
            password: item
                .get("password")
                .and_then(|v| v.as_s().ok())
                .ok_or_else(|| anyhow!("Missing password"))?
                .to_string(),
        })
    }
}
