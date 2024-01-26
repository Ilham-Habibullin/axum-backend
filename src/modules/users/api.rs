use axum::{
    Extension,
    extract::{State, Query},
    http::{StatusCode, HeaderValue, Response},
    Json, response::IntoResponse, body::Body
};
use tokio_postgres::types::ToSql;

use crate::types::{
    internal_error,
    AppState,
    Pagination
};

use crate::{
    modules::users::types::*,
    USER_TABLE_NAME
};


#[derive(serde::Deserialize)]
pub struct Role {
    pub role: Option<i16>,
}

pub async fn get_users(
    pagination: Query<Pagination>,
    role_option: Query<Role>,
    state: State<AppState>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let limit = pagination.0.limit;
    let offset = pagination.0.offset;

    match role_option.role {
        None => complete_get_user(
            format!("SELECT id, username, role FROM {USER_TABLE_NAME} ORDER BY id LIMIT $1 OFFSET $2"),
            vec![&limit, &offset], 
            state).await,
        Some(role) => complete_get_user(
            format!("SELECT id, username, role FROM {USER_TABLE_NAME} WHERE role=$3 ORDER BY id LIMIT $1 OFFSET $2 "),
            vec![&limit, &offset, &role],
            state).await
    }
}

async fn complete_get_user(
    query_users: String,
    param_users: Vec<&(dyn ToSql + Sync)>,
    state: State<AppState>
) -> Result<Response<Body>, (StatusCode, String)> {
    let conn = state.pool.get().await.map_err(internal_error)?;

    let query_count = format!("SELECT count(*) FROM {USER_TABLE_NAME}");

    let (rows, row_count) = tokio::try_join!(
        conn.query(&query_users, &param_users),
        conn.query_one(&query_count, &[])
    ).map_err(internal_error)?;

    let reply: Vec<User> = rows.iter().map(|r| User {
        id: r.get(0),
        username: r.get(1),
        role: r.get::<usize, i16>(2).into()
    }).collect();

    let count: i64 = row_count.get(0);
    let header_count_value = HeaderValue::from_str(&count.to_string()).map_err(internal_error)?;

    let mut response = Json(reply).into_response();
    response.headers_mut().insert("x-total-count", header_count_value);

    Ok(response)
}

pub async fn delete_user(
    payload: Query<DeleteUserPayload>,
    State(state): State<AppState>,
) -> Result<Json<User>, (StatusCode, String)> {
    let conn = state.pool.get().await.map_err(internal_error)?;

    let username = payload.0.username;

    let row = conn
        .query_one(
            &format!("DELETE FROM {USER_TABLE_NAME} WHERE username=$1 RETURNING id, username"),
            &[&username]
        )
        .await
        .map_err(internal_error)?;

    let id = row.get(0);
    let username = row.get(1);
    let role = row.get::<usize, i16>(2).into();

    let deleted_user = User { id, username, role };
    Ok(Json(deleted_user))
}

pub async fn promote_user(
    _payload: Query<PromoteUserPayload>,
    State(_state): State<AppState>,
    Extension(_user): Extension<User>
) -> Result<Json<User>, (StatusCode, String)> {
    todo!();
    // let user = User { id: 1, username: "none".to_string() };
    // Ok(Json(user))
}