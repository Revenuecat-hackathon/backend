use lazy_static::lazy_static;

lazy_static! {
    pub static ref JWT_EXPIRY: i64 = set_jwt_expiry();
}

fn set_jwt_expiry() -> i64 {
    24
}
