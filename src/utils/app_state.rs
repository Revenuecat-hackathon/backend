use actix_web::web;

use crate::RedisClient;

pub struct AppState {
    pub redis_client: web::Data<RedisClient>,
}
