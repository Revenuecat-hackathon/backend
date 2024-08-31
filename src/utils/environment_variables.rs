use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref ADDRESS: String = set_address();
    pub static ref REDIS_URL: String = set_redis_url();
    pub static ref PORT: u16 = set_port();
    pub static ref SECRET: String = set_secret();
}

fn set_address() -> String {
    dotenv::dotenv().ok();
    env::var("ADDRESS").unwrap_or("0.0.0.0".to_string())
}

fn set_redis_url() -> String {
    dotenv::dotenv().ok();
    env::var("REDIS_URL").expect("REDIS_URL must be set")
}

fn set_port() -> u16 {
    dotenv::dotenv().ok();
    env::var("PORT")
        .unwrap_or("5050".to_string())
        .parse::<u16>()
        .expect("Cant parse PORT")
}

fn set_secret() -> String {
    dotenv::dotenv().ok();
    env::var("SECRET").unwrap_or("SECRET".to_string())
}
