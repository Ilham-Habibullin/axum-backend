use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, State},
    http::{request::Parts, StatusCode}
};

use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

use crate::types::{internal_error, AppState};

// we can also write a custom extractor that grabs a connection from the pool
// which setup is appropriate depends on your application
pub struct DatabaseConnection(PooledConnection<'static, PostgresConnectionManager<NoTls>>);

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let conn = state.pool.get_owned().await.map_err(internal_error)?;
        Ok(Self(conn))
    }
}

pub async fn using_connection_extractor(
    DatabaseConnection(conn): DatabaseConnection,
) -> Result<String, (StatusCode, String)> {
let row = conn
    .query_one("select 1 + 1", &[])
    .await
    .map_err(internal_error)?;

    let two: i32 = row.try_get(0).map_err(internal_error)?;
    Ok(two.to_string())
}

pub async fn using_connection_pool_extractor(
    State(state): State<AppState>,
) -> Result<String, (StatusCode, String)> {
    let conn = state.pool.get().await.map_err(internal_error)?;

    let row = conn
        .query_one("select 1 + 1", &[])
        .await
        .map_err(internal_error)?;

    let two: i32 = row.try_get(0).map_err(internal_error)?;
    Ok(two.to_string())
}