use actix_web::web;
use sea_orm::DatabaseConnection;

use crate::RedisClient;

pub struct AppState {
    pub redis_client: web::Data<RedisClient>,
}
