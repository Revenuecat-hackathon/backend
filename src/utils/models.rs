use std::collections::HashMap;

use anyhow::anyhow;
use anyhow::Result;
use aws_sdk_dynamodb::types::AttributeValue;

#[derive(Debug)]
pub(crate) struct User {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) email: String,
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
