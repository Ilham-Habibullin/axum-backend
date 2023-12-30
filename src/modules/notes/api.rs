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

use crate::modules::notes::types::*;
use crate::modules::users::types::User;

pub async fn get_notes(
    pagination: Query<Pagination>,
    State(state): State<AppState>,
    Extension(user): Extension<User>
) -> Result<Json<Vec<Note>>, (StatusCode, String)> {
    let conn = state.pool.get().await.map_err(internal_error)?;

    let limit = pagination.0.limit;
    let offset = pagination.0.offset;

    let notes = vec![Note{}, Note{}, Note{}];

    Ok(Json(notes))
}

pub async fn delete_note(
    payload: Query<DeleteNotePayload>,
    State(state): State<AppState>,
) -> Result<Json<Note>, (StatusCode, String)> {
    let conn = state.pool.get().await.map_err(internal_error)?;

    Ok(Json(Note{}))
}

pub async fn create_note(
    State(state): State<AppState>,
    Json(body): Json<CreateNotePayload>,
) -> Result<Json<Note>, (StatusCode, String)> {
    let conn = state.pool.get().await.map_err(internal_error)?;

    Ok(Json(Note{}))
}