use anyhow::Result;
use std::sync::Arc;

use aws_sdk_dynamodb::{operation::query::QueryOutput, types::AttributeValue, Client};

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
