use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use actix_web::{middleware::Logger, web, App, HttpServer};
use anyhow::Result;
use aws_sdk_bedrockruntime as bedrock;
use aws_sdk_dynamodb::Client;
use redis::aio::MultiplexedConnection;
use routes::middlewares::inactivity_middleware::{InactivityMiddleware, LastActivityTime};
use utils::app_state::AppState;
use utils::global_variables::SHUTDOWN_DURATION;

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

    println!("[+] about to init env");
    dotenv::dotenv().ok();
    env_logger::init();

    println!("[+] env initialized successfully");
    let address = (utils::environment_variables::ADDRESS).clone();
    let port = (utils::environment_variables::PORT).clone();

    let redis_client = web::Data::new(RedisClient::new().expect("Failed to create Redis client"));

    let shared_config = aws_config::load_from_env().await;
    let dynamo_client = Arc::new(Client::new(&shared_config));
    let bedrock_client = Arc::new(bedrock::Client::new(&shared_config));

    println!("[+] dynamodb setup done");

    println!("[+] server start listening...");

    let last_activity = Arc::new(LastActivityTime(Mutex::new(Instant::now())));
    let last_activity_clone = last_activity.clone();

    let shutdown_duration = SHUTDOWN_DURATION.clone();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                redis_client: redis_client.clone(),
                dynamo_client: Arc::clone(&dynamo_client),
                bedrock_client: Arc::clone(&bedrock_client),
            }))
            .wrap(Logger::default())
            .wrap(InactivityMiddleware {
                last_activity: last_activity_clone.clone(),
                shutdown_duration: Duration::from_secs(shutdown_duration as u64),
            })
            .configure(routes::auth_routes::config)
            .configure(routes::user_routes::config)
            .configure(routes::index_routes::config)
            .configure(routes::map_routes::config)
    })
    .bind((address, port))?;

    let server_handle = server.run();

    actix_web::rt::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            let last_activity = last_activity.0.lock().unwrap();
            if last_activity.elapsed() > Duration::from_secs(shutdown_duration as u64) {
                println!("[-] No activity for 5 minutes. Shutting down.");
                std::process::exit(0);
            }
        }
    });

    server_handle.await.map_err(anyhow::Error::from)
}
