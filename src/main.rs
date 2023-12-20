use axum::{routing::get, routing::delete, Router};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod api;
pub mod types;

use crate::api::users::api::*;
use crate::api::testable::api::*;
use crate::types::AppState;

use std::fs;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axum_admin_backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    

    // set up connection pool
    let manager = PostgresConnectionManager::new_from_stringlike("host=localhost user=postgres dbname=", NoTls).unwrap();

    let pool = Pool::builder().build(manager).await.unwrap();

    let state = AppState {
        pool: pool
    };

    // build our application with some routes
    let level_0_access = Router::new()
        .route(
            "/",
             get(using_connection_pool_extractor)
            .post(using_connection_extractor),
        )
        .route(
            "/users",
             get(get_users)
            .post(create_user)
            
        )
        .with_state(state.clone());

    let level_1_access = Router::new()
        .route(
            "/users",
            delete(delete_user)
            .post(promote_user)
        )
        .with_state(state.clone());

    let app = Router::new()
        .merge(level_0_access)
        .merge(level_1_access);

    // let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    // let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let addr = "[::]:8080".parse::<std::net::SocketAddr>().unwrap();

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

}