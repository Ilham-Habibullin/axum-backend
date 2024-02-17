use axum::{
    Router, routing::{get, post},
    middleware::from_fn_with_state, handler::Handler
};
use bb8::{Pool, ManageConnection};
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::{NoTls, Client};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use http::{Method, HeaderValue, HeaderName};
use tower_http::cors::{CorsLayer, AllowOrigin, AllowCredentials};

pub mod modules;
pub mod types;
pub mod middleware;

use crate::modules::users::api::*;
use crate::modules::testable::api::*;
use crate::modules::auth::api::*;
use crate::modules::notes::api::*;

use crate::types::{AppState, Roles};
use crate::middleware::*;

pub const USER_TABLE_NAME: &'static str = "users";
pub const NOTES_TABLE_NAME: &'static str = "notes";


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

async fn read_file(filename: &str) -> Result<String, String> {
    let contents = tokio::fs::read(filename).await.map_err(|e| e.to_string())?;
    let parsed = String::from_utf8_lossy(&contents).to_string();
    return Ok(parsed)
}



#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axum_backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (postgres_config, jwt_secret, salt) = tokio::try_join!(
        read_file("postgres"),
        read_file("jwt_secret"),
        read_file("salt")
    ).unwrap();

    // set up connection pool
    let manager = PostgresConnectionManager::new_from_stringlike(postgres_config, NoTls).unwrap();
    let mut client = manager.connect().await.unwrap();

    run_migrations(&mut client).await;

    let pool = Pool::builder().build(manager).await.unwrap();
    let state = AppState {
        pool,
        secret: jwt_secret,
        salt
    };


    use http::header::{AUTHORIZATION, ACCEPT, CONTENT_TYPE, COOKIE, SET_COOKIE, CONTENT_LENGTH};

    let origins = [
        HeaderValue::from_static("http://localhost:3000"),
        HeaderValue::from_static("http://127.0.0.1:3000"),
        HeaderValue::from_static("http://localhost"),
        HeaderValue::from_static("http://127.0.0.1"),
    ];

    let x_total_count = HeaderName::from_static("x-total-count");

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_origin(AllowOrigin::list(origins))
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE, CONTENT_LENGTH, COOKIE, SET_COOKIE])
        .allow_credentials(AllowCredentials::yes())
        .expose_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE, CONTENT_LENGTH, COOKIE, SET_COOKIE, x_total_count]);

    // build our application with some routes
    let app  = Router::new()
        .route("/",
             get(using_connection_pool_extractor)
            .post(using_connection_extractor),
        )
        .route("/users",
             get(get_users)
            .delete(delete_user.layer(from_fn_with_state(Roles::Admin, roles::roles)))
        )
        .route("/notes",
             get(get_notes)
            .delete(delete_note)
            .post(create_note)
            .route_layer(from_fn_with_state(state.clone(), auth::auth))
        )
        .route("/promote",
            post(promote_user)
                .route_layer(from_fn_with_state(Roles::Basic, roles::roles))
                .route_layer(from_fn_with_state(state.clone(), auth::auth))
        )
        .nest("/auth",
            Router::new()
                .route("/me", get(me).layer(from_fn_with_state(state.clone(), auth::auth)))
                .route("/signup", post(sign_up))
                .route("/signin", post(sign_in))
                .route("/signout", get(sign_out))
        )
        .with_state(state.clone())
        .layer(cors);

    // let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    // let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let addr = "[::]:8080".parse::<std::net::SocketAddr>().unwrap();

    tracing::debug!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();

}