use anyhow::Result;

use actix_web::{middleware::Logger, web, App, HttpServer};
use redis::aio::MultiplexedConnection;
use sea_orm::Database;
use utils::app_state::AppState;

mod routes;
mod utils;

pub struct RedisClient {
    client: redis::Client,
}

impl RedisClient {
    pub fn new() -> Result<Self> {
        let redis_url = (utils::environment_variables::REDIS_URL).clone();
        let client = redis::Client::open(redis_url).map_err(anyhow::Error::from)?;
        Ok(Self { client })
    }

    pub async fn get_async_connection(&self) -> Result<MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(anyhow::Error::from)
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }

    dotenv::dotenv().ok();
    env_logger::init();

    let address = (utils::environment_variables::ADDRESS).clone();
    let port = (utils::environment_variables::PORT).clone();

    let redis_client = web::Data::new(RedisClient::new().expect("Failed to create Redis client"));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                redis_client: redis_client.clone(),
            }))
            .wrap(Logger::default())
            .configure(routes::auth_routes::config)
            .configure(routes::user_routes::config)
    })
    .bind((address, port))
    .map_err(anyhow::Error::from)?
    .run()
    .await
    .map_err(anyhow::Error::from)
}
