use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref ADDRESS: String = set_address();
    pub static ref DATABASE_URL: String = set_database_url();
    pub static ref REDIS_URL: String = set_redis_url();
    pub static ref PORT: u16 = set_port();
    pub static ref SECRET: String = set_secret();
    pub static ref MAX_FILE_SIZE: u64 = set_max_file_size();
}

fn set_address() -> String {
    dotenv::dotenv().ok();
    env::var("ADDRESS").unwrap_or("0.0.0.0".to_string())
}

fn set_database_url() -> String {
    dotenv::dotenv().ok();
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
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

fn set_max_file_size() -> u64 {
    dotenv::dotenv().ok();
    env::var("MAX_FILE_SIZE")
        .unwrap_or("10485760".to_string())
        .parse::<u64>()
        .expect("Cant parse MAX_FILE_SIZE")
}
