use axum::{
    Extension,
    extract::{State, Query},
    http::{StatusCode, HeaderValue, Response},
    Json, response::IntoResponse,
    body::Body
};
use tokio_postgres::types::ToSql;

use crate::types::{internal_error, AppState, Pagination};

use crate::modules::notes::types::*;
use crate::modules::users::types::User;

pub async fn get_notes(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    pagination: Query<Pagination>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let conn = state.pool.get().await.map_err(internal_error)?;

    let limit = pagination.0.limit;
    let offset = pagination.0.offset;

    let query_notes = format!("SELECT id, text FROM {} WHERE user_id=$3 LIMIT $1 OFFSET $2", crate::NOTES_TABLE_NAME);
    let params_notes: Vec<&(dyn ToSql + Sync)> = vec![&limit, &offset, &user.id];

    let query_count = format!("SELECT count(*) FROM {} WHERE user_id=$1", crate::NOTES_TABLE_NAME);
    let params_count: Vec<&(dyn ToSql + Sync)> = vec![&user.id];

    let (rows, row_count) = tokio::try_join!(
        conn.query(&query_notes, &params_notes),
        conn.query_one(&query_count, &params_count)
    ).map_err(internal_error)?;

    let notes: Vec<Note> = rows.iter().map(|row| Note {
        id: row.get(0),
        text: row.get(1)
    }).collect();

    let count: i64 = row_count.get(0);
    let header_count_value = HeaderValue::from_str(&count.to_string()).map_err(internal_error)?;

    let mut response = Json(notes).into_response();
    response.headers_mut().insert("x-total-count", header_count_value);

    Ok(response)
}

pub async fn delete_note(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    payload: Query<DeleteNotePayload>,
) -> Result<Json<Note>, (StatusCode, String)> {
    let conn = state.pool.get().await.map_err(internal_error)?;
    let note_id = payload.0.id;

    let row = conn.query_one(
        &format!("DELETE FROM {} WHERE id=$1 AND user_id=$2 RETURNING id, text", crate::NOTES_TABLE_NAME),
        &[&note_id, &user.id]
    ).await.map_err(internal_error)?;

    let deleted_note = Note {
        id: row.get(0),
        text: row.get(1)
    };

    Ok(Json(deleted_note))
}

pub async fn create_note(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(body): Json<CreateNotePayload>,
) -> Result<Json<Note>, (StatusCode, String)> {
    let conn = state.pool.get().await.map_err(internal_error)?;

    let row = conn.query_one(
        &format!("INSERT INTO {} (text, user_id) VALUES ($1, $2) RETURNING id, text", crate::NOTES_TABLE_NAME),
        &[&body.text, &user.id]
    ).await.map_err(internal_error)?;

    let created_note = Note {
        id: row.get(0),
        text: row.get(1)
    };

    Ok(Json(created_note))
}