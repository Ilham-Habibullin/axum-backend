use axum::{
    Json,
    extract::State,
    response::IntoResponse,
    http::{StatusCode, Response, header, HeaderValue},
};

use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{encode, EncodingKey, Header};

use sha256::digest;

use crate::types::{internal_error, AppState, Roles};

use crate::modules::auth::types::*;
use crate::modules::users::types::*;

pub async fn sign_up(
    State(state): State<AppState>,
    Json(body): Json<SignUpPayload>,
) -> Result<Json<User>, (StatusCode, String)> {
    let conn = state.pool.get().await.map_err(internal_error)?;

    let username = body.username;
    let password = body.password;
    let salt = state.salt;

    let hashed_password = digest(password+&salt);

    let user_row = conn
        .query_one(
            &format!("INSERT INTO {} (username, password) VALUES ($1, $2) RETURNING id, username", crate::USER_TABLE_NAME),
            &[&username, &hashed_password]
        )
        .await
        .map_err(internal_error)?;

    let user = User {
        id: user_row.get(0),
        username: user_row.get(1),
        role: Roles::Basic
    };
    Ok(Json(user))
}

pub async fn sign_in(
    State(state): State<AppState>,
    Json(body): Json<SignInPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = state.pool.get().await.map_err(internal_error)?;

    let username = body.username;
    let password = body.password;
    let salt = &state.salt;

    let hashed_password = digest(password+salt);

    let query = &format!("SELECT id, username, password, role FROM {} WHERE username = $1", crate::USER_TABLE_NAME);
    let user_row = conn.query_one(query, &[&username]).await.map_err(internal_error)?;

    let user = UserWithPassword {
        id: user_row.get(0),
        username: user_row.get(1),
        password: user_row.get(2),
        role: user_row.get::<usize, i16>(3).into()
    };

    if user.password != hashed_password {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "User does not exist or password was wrong!".to_string()))
    } else {
        let now = chrono::Utc::now();
        let iat = now.timestamp() as usize;
        let exp = (now + chrono::Duration::minutes(60)).timestamp() as usize;

        let claims: TokenClaims = TokenClaims {
            sub: user.id.to_string(),
            role: 1,
            exp,
            iat,
        };

        let x = state.secret.as_ref();

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(x)
        )
        .unwrap();

        let cookie = Cookie::build(("token", token))
            .path("/")
            .max_age(time::Duration::hours(1))
            .same_site(SameSite::Lax)
            .http_only(true);

        let header_cookie_value = HeaderValue::from_str(&cookie.to_string()).map_err(internal_error)?;

        let mut response: Response<String> = Response::default();
        response.headers_mut().insert(header::SET_COOKIE, header_cookie_value);

        return Ok(response)
    }
}

pub async fn sign_out() {
  
}