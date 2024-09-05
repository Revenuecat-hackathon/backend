use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use actix_web::dev::{forward_ready, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use actix_web::{dev::Service, middleware::Logger, web, App, HttpServer};
use anyhow::Result;
use aws_sdk_bedrockruntime as bedrock;
use aws_sdk_dynamodb::Client;
use futures_util::future::LocalBoxFuture;
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

struct LastActivityTime(Mutex<Instant>);

struct InactivityMiddleware {
    last_activity: Arc<LastActivityTime>,
    shutdown_duration: Duration,
}

impl<S, B> Transform<S, ServiceRequest> for InactivityMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = InactivityMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(InactivityMiddlewareService {
            service,
            last_activity: self.last_activity.clone(),
            shutdown_duration: self.shutdown_duration,
        }))
    }
}

struct InactivityMiddlewareService<S> {
    service: S,
    last_activity: Arc<LastActivityTime>,
    #[allow(dead_code)]
    shutdown_duration: Duration,
}

impl<S, B> Service<ServiceRequest> for InactivityMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let last_activity = self.last_activity.clone();
        let mut last_activity = last_activity.0.lock().unwrap();
        *last_activity = Instant::now();
        drop(last_activity);

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
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
                shutdown_duration: Duration::from_secs(300), // 5 minutes
            })
            .configure(routes::auth_routes::config)
            .configure(routes::user_routes::config)
            .configure(routes::index_routes::config)
            .configure(routes::map_routes::config)
    })
    .bind((address, port))?;

    let server_handle = server.run();

    // Spawn a task to check for inactivity
    actix_web::rt::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            let last_activity = last_activity.0.lock().unwrap();
            if last_activity.elapsed() > Duration::from_secs(300) {
                println!("[-] No activity for 5 minutes. Shutting down.");
                std::process::exit(0);
            }
        }
    });

    server_handle.await.map_err(anyhow::Error::from)
}
