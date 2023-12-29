use axum::{
    Extension,
    extract::{State, Query},
    http::StatusCode,
    Json
};

use crate::types::{
    internal_error,
    AppState,
    Pagination
};

use crate::modules::users::types::*;

pub async fn get_users(
    pagination: Query<Pagination>,
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let conn = state.pool.get().await.map_err(internal_error)?;

    let limit = pagination.0.limit;
    let offset = pagination.0.offset;

    let row = conn
        .query(
            &format!("SELECT id, username, role FROM {} LIMIT $1 OFFSET $2", crate::USER_TABLE_NAME), 
            &[&limit, &offset]
        )
        .await
        .map_err(internal_error)?;

    let reply: Vec<User> = row.iter().map(|r| {
        let id = r.get(0);
        let username = r.get(1);
        let role = r.get::<usize, i16>(2).into();

        User { id, username, role }
    }).collect();

    Ok(Json(reply))
}

pub async fn delete_user(
    payload: Query<DeleteUserPayload>,
    State(state): State<AppState>,
) -> Result<Json<User>, (StatusCode, String)> {
    let conn = state.pool.get().await.map_err(internal_error)?;

    let username = payload.0.username;

    let row = conn
        .query_one(
            &format!("DELETE FROM {} WHERE username=$1 RETURNING id, username", crate::USER_TABLE_NAME),
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
    payload: Query<PromoteUserPayload>,
    State(state): State<AppState>,
    Extension(user): Extension<User>
) -> Result<Json<User>, (StatusCode, String)> {
    // let user = User { id: 1, username: "none".to_string() };

    Ok(Json(user))
}