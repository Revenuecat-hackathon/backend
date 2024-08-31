use actix_web::middleware::from_fn;
use actix_web::web;

use super::{handlers::post_handlers, middlewares};

// This in charge of every path in /user path
pub fn config(config: &mut web::ServiceConfig) {
    config
        .service(
            web::scope("/post")
                .service(post_handlers::one_posts)
                .service(post_handlers::all_posts),
        )
        .service(
            web::scope("/secure/post")
                .wrap(from_fn(middlewares::auth_middleware::check_auth_middleware))
                .service(post_handlers::create_post)
                .service(post_handlers::my_posts),
        );
}
