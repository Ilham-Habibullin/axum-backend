use axum::{
    extract::{State, Request},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse
};

use axum_extra::extract::cookie::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::{
    AppState,
    modules::users::types::User,
    modules::auth::types::TokenClaims,
    types::internal_error
};


pub async fn auth(
    cookie_jar: CookieJar,
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let token = cookie_jar
        .get("token")
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(
            || (StatusCode::UNAUTHORIZED, "You are not logged in, please provide token".to_string())
        )?;

    let claims = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(state.secret.as_ref()),
        &Validation::default(),
    )
    .map_err(
        |_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string())
    )?.claims;

    let user_id = claims.sub.parse::<i32>().map_err(internal_error)?;
    let conn = state.pool.get().await.map_err(internal_error)?;

    let row = conn.query_one(
        "SELECT id, username, role FROM users WHERE id = $1", &[&user_id]
    ).await.map_err(
        |_| (StatusCode::UNAUTHORIZED, "User not found".to_string())
    )?;

    let user = User {
        id: row.get(0),
        username: row.get(1),
        role: row.get::<usize, i16>(2).into()
    };

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}