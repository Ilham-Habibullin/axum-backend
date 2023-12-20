use axum::{
  extract::{State, Query},
  http::StatusCode,
  Json
};

use crate::types::{
  internal_error,
  AppState,
  Pagination
};

use crate::api::users::types::*;

const USER_TABLE_NAME: &'static str = "users_ax";

pub async fn get_users(
  pagination: Query<Pagination>,
  State(state): State<AppState>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
  let conn = state.pool.get().await.map_err(internal_error)?;

  let limit = pagination.0.limit;
  let offset = pagination.0.offset;

  let row = conn
      .query(
        &format!("SELECT id, username, password FROM {} LIMIT $1 OFFSET $2", USER_TABLE_NAME), 
        &[&limit, &offset]
      )
      .await
      .map_err(internal_error)?;

  let reply: Vec<User> = row.iter().map(|r| {
    let id = r.get(0);
    let username = r.get(1);
    let password = r.get(2);

    User { id, username, password }
  }).collect();

  Ok(Json(reply))
}

pub async fn create_user(
  payload: Query<CreateUserPayload>,
  State(state): State<AppState>,
) -> Result<Json<User>, (StatusCode, String)> {
  let conn = state.pool.get().await.map_err(internal_error)?;

  let username = payload.0.username;
  let password = payload.0.password;

  let row = conn
    .query(
      &format!("INSERT INTO {} (username, password) VALUES ($1, $2) RETURNING id, username, password", USER_TABLE_NAME),
      &[&username, &password]
    )
    .await
    .map_err(internal_error)?;

  let mut reply: Vec<User> = row.iter().map(|r| {
    let id = r.get(0);
    let username = r.get(1);
    let password = r.get(2);

    User { id, username, password }
  }).collect();

  let created_user = match reply.pop() {
    Some(user) => user,
    None => return Err((StatusCode::INTERNAL_SERVER_ERROR, "user was not created for some reason".to_string()))
  };

  Ok(Json(created_user))
}

pub async fn delete_user(
  payload: Query<DeleteUserPayload>,
  State(state): State<AppState>,
) -> Result<Json<User>, (StatusCode, String)> {
  let conn = state.pool.get().await.map_err(internal_error)?;

  let username = payload.0.username;

  let row = conn
    .query(
      &format!("DELETE FROM {} WHERE username=$1 RETURNING id, username, password", USER_TABLE_NAME),
      &[&username]
    )
    .await
    .map_err(internal_error)?;

  let mut reply: Vec<User> = row.iter().map(|r| {
    let id = r.get(0);
    let username = r.get(1);
    let password = r.get(2);

    User { id, username, password }
  }).collect();

  let deleted_user = match reply.pop() {
    Some(user) => user,
    None => return Err((StatusCode::INTERNAL_SERVER_ERROR, "user was not created for some reason".to_string()))
  };

  Ok(Json(deleted_user))
}



pub async fn promote_user(
    payload: Query<PromoteUserPayload>,
    State(state): State<AppState>,
) -> Result<Json<User>, (StatusCode, String)> {
    
}