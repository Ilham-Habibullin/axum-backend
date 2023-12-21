use axum::{
    routing::{get, delete, post},
    Router
};

use bb8::{Pool, ManageConnection};
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::{NoTls, Client};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod api;
pub mod types;

use crate::api::users::api::*;
use crate::api::testable::api::*;
use crate::types::AppState;

async fn run_migrations(client: &mut Client) {
    mod embedded {
        use refinery::embed_migrations;
        embed_migrations!("./migrations");
    }

    

    let migration_report = embedded::migrations::runner()
        .run_async(client)
        .await
        .unwrap();

    println!("{:?}", migration_report);

    for migration in migration_report.applied_migrations() {
        println!(
            "Migration Applied -  Name: {}, Version: {}",
            migration.name(),
            migration.version()
        );
    }

    println!("DB migrations finished!");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axum_admin_backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let contents = tokio::fs::read("postgres").await.unwrap();
    let postgres_config: String = String::from_utf8_lossy(&contents).parse().unwrap();

    // set up connection pool
    let manager = PostgresConnectionManager::new_from_stringlike(postgres_config, NoTls).unwrap();
    let mut client = manager.connect().await.unwrap();

    run_migrations(&mut client).await;

    let pool = Pool::builder().build(manager).await.unwrap();
    let state = AppState { pool };

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
        )
        .route(
            "/promote",
            post(promote_user)
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