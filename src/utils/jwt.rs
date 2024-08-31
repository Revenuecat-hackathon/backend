use std::future;

use actix_web::FromRequest;
use actix_web::HttpMessage;
use anyhow::Result;
use chrono::Duration;
use chrono::Utc;
use jsonwebtoken::decode;
use jsonwebtoken::encode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Header;
use jsonwebtoken::TokenData;
use jsonwebtoken::Validation;
use redis::AsyncCommands;

use crate::RedisClient;

use super::environment_variables;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub email: String,
    pub id: i32,
}

impl FromRequest for Claims {
    type Error = actix_web::Error;

    type Future = future::Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> future::Ready<Result<Claims, actix_web::Error>> {
        match req.extensions().get::<Claims>() {
            Some(claim) => future::ready(Ok(claim.clone())),
            None => future::ready(Err(actix_web::error::ErrorBadRequest("Bad Claims"))),
        }
    }
}

pub fn encode_jwt(email: String, id: i32) -> Result<String> {
    let now = Utc::now();
    let expire = Duration::hours((super::global_variables::JWT_EXPIRY).clone());

    let claims = Claims {
        exp: (now + expire).timestamp() as usize,
        iat: now.timestamp() as usize,
        email,
        id,
    };

    let secret = (*environment_variables::SECRET).clone();

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(anyhow::Error::from)
}

pub fn decode_jwt(jwt: String) -> Result<TokenData<Claims>> {
    let secret = (*environment_variables::SECRET).clone();
    let claim_data: Result<TokenData<Claims>> = decode(
        &jwt,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(anyhow::Error::from);
    claim_data
}

// Function to add a token to the blacklist
pub async fn add_to_blacklist(redis_client: &RedisClient, token: &str) -> Result<()> {
    let expiry = Duration::hours((super::global_variables::JWT_EXPIRY).clone()).num_seconds();
    let mut conn = redis_client.get_async_connection().await?;
    conn.set_ex(format!("blacklist:{}", token), "1", expiry as u64)
        .await?;
    Ok(())
}

// Function to check if a token is blacklisted
pub async fn is_blacklisted(redis_client: &RedisClient, token: &str) -> Result<bool> {
    let mut conn = redis_client.get_async_connection().await?;
    let result: Option<String> = conn.get(format!("blacklist:{}", token)).await?;
    Ok(result.is_some())
}
