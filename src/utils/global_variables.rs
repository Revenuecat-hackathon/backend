use super::environment_variables::ENVIRONMENT;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref JWT_EXPIRY: i64 = set_jwt_expiry();
    pub static ref DYNAMO_DB_TABLE_NAME: String = set_dynamo_db_table_name();
}

fn set_jwt_expiry() -> i64 {
    24
}

fn set_dynamo_db_table_name() -> String {
    let environment = (ENVIRONMENT).clone();
    format!("artizans_{environment}")
}
