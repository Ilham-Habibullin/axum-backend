use axum::http::StatusCode;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

use serde::{Deserialize, Serialize};

pub type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

/// Utility function for mapping any error into a `500 Internal Server Error` response.
pub fn internal_error<E>(err: E) -> (StatusCode, String) where E: std::error::Error {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

#[derive(Deserialize)]
pub struct Pagination {
    pub offset: i64,
    pub limit: i64,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<PostgresConnectionManager<NoTls>>,
    pub secret: String,
    pub salt: String
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Roles {
    Basic = 0,
    Moderator = 1,
    Admin = 2
}

impl From<i16> for Roles {
    fn from(number: i16) -> Roles {
        match number {
            0 => Self::Basic,
            1 => Self::Moderator,
            2 => Self::Admin,
            _ => panic!("Tried to convert u8 to not existing role")
        }
    }
}