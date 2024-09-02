use std::sync::Arc;

use anyhow::Result;

use actix_web::{middleware::Logger, web, App, HttpServer};
use aws_sdk_dynamodb::Client;
use redis::aio::MultiplexedConnection;
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

    println!("about to init env");
    dotenv::dotenv().ok();
    env_logger::init();

    println!("env initialized succesfully");
    let address = (utils::environment_variables::ADDRESS).clone();
    let port = (utils::environment_variables::PORT).clone();

    let redis_client = web::Data::new(RedisClient::new().expect("Failed to create Redis client"));

    let shared_config = aws_config::load_from_env().await;
    let dynamo_client = Arc::new(Client::new(&shared_config));

    println!("dynamodb setup done");

    println!("about to fire up web server!");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                redis_client: redis_client.clone(),
                dynamo_client: Arc::clone(&dynamo_client),
            }))
            .wrap(Logger::default())
            .configure(routes::auth_routes::config)
            .configure(routes::user_routes::config)
            .configure(routes::index_routes::config)
    })
    .bind((address, port))
    .map_err(anyhow::Error::from)?
    .run()
    .await
    .map_err(anyhow::Error::from)
}
