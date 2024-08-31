use actix_web::middleware::from_fn;
use actix_web::web;

use super::{handlers, middlewares};

// This is in charge of every path in /test path
pub fn config(config: &mut web::ServiceConfig) {
    config.service(web::scope("/test").service(handlers::test_handlers::create_dynamodb));
}
