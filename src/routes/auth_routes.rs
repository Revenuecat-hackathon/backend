use actix_web::{middleware::from_fn, web};

use super::{handlers, middlewares};

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/auth")
            .service(handlers::auth_handlers::register)
            .service(handlers::auth_handlers::login)
            .service(
                web::scope("")
                    .wrap(from_fn(middlewares::auth_middleware::check_auth_middleware))
                    .service(handlers::auth_handlers::logout),
            ),
    );
}
