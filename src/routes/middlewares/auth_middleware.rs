use actix_web::middleware::Next;
use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    http::header::AUTHORIZATION,
    web, Error, HttpMessage,
};

use crate::utils::{
    api_response::{self, ApiResponse},
    app_state::AppState,
    jwt::{decode_jwt, is_blacklisted},
};

// middleware that checks if the request has a valid token
pub async fn check_auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let app_state = req.app_data::<web::Data<AppState>>().cloned().unwrap();
    let auth = req.headers().get(AUTHORIZATION);
    if auth.is_none() {
        return Err(Error::from(api_response::ApiResponse::new(
            401,
            "Unauthorized".to_string(),
        )));
    }
    let token = auth
        .ok_or_else(|| {
            Error::from(ApiResponse::new(
                401,
                "Missing Authorization header".to_string(),
            ))
        })?
        .to_str()
        .map_err(|_| {
            Error::from(ApiResponse::new(
                401,
                "Invalid Authorization header".to_string(),
            ))
        })?
        .strip_prefix("Bearer ")
        .ok_or_else(|| Error::from(ApiResponse::new(401, "Invalid token format".to_string())))?
        .to_owned();

    // Check if token is blacklisted meaning user logged out already within past 24 hours
    match is_blacklisted(&app_state.redis_client, &token).await {
        Ok(true) => {
            return Err(Error::from(ApiResponse::new(
                401,
                "Token is invalid".to_string(),
            )))
        }
        Ok(false) => {} // Token is not blacklisted, continue
        Err(e) => {
            return Err(Error::from(ApiResponse::new(
                500,
                format!("Failed to check token: {}", e),
            )))
        }
    }

    let claim = decode_jwt(token).unwrap();
    req.extensions_mut().insert(claim.claims);

    next.call(req)
        .await
        .map_err(|err| Error::from(ApiResponse::new(500, err.to_string())))
}
