use axum::{
    Extension,
    extract::{State, Query},
    http::{StatusCode, HeaderValue, Response},
    Json, response::IntoResponse,
    body::Body
};

use tokio_postgres::types::ToSql;

use crate::{
    types::{internal_error, AppState, Pagination},
    modules::notes::types::*,
    modules::users::types::User,
    modules::common::{Search, SqlParams},
    NOTES_TABLE_NAME
};

pub async fn get_notes(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    pagination: Query<Pagination>,
    mut search_option: Query<Search>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let mut query_notes = format!("SELECT id, text FROM {NOTES_TABLE_NAME} WHERE ");
    let mut query_count = format!("SELECT count(*) FROM {NOTES_TABLE_NAME} WHERE ");

    let mut for_notes_query: SqlParams = vec![("user_id = ", Box::new(user.id))];
    let mut for_count_query: SqlParams = vec![("user_id = ", Box::new(user.id))];

    if let Some(search) = search_option.search.take() {
        let search_ = format!("%{search}%");
        for_notes_query.push(("text LIKE ", Box::new(search_.clone())));
        for_count_query.push(("text LIKE ", Box::new(search_.clone())));
    }

    let mut parameters_count = 0;

    let mut params_notes = for_notes_query
        .iter()
        .enumerate()
        .map(|(i, (key, value))| {
            if i > 0 { query_notes += " AND " }
            let parameters_number = i + 1;
            query_notes += &format!("{}${}", key, parameters_number);
            parameters_count = parameters_number;
            value.as_ref() as &(dyn ToSql + Sync)
        })
        .collect::<Vec<&(dyn ToSql + Sync)>>();

    params_notes.push(&pagination.limit);
    params_notes.push(&pagination.offset);

    if parameters_count != 0 {
        query_notes += &format!(" ORDER BY id LIMIT ${} OFFSET ${}", parameters_count + 1, parameters_count + 2);
    } else {
        query_notes += " ORDER BY id LIMIT $1 OFFSET $2";
    }

    let params_count =  for_count_query
        .iter()
        .enumerate()
        .map(|(i, (key, value))| {
            if i > 0 { query_count += " AND " }
            query_count += &format!("{}${}", key, i+1);
            value.as_ref() as &(dyn ToSql + Sync)
        })
        .collect::<Vec<&(dyn ToSql + Sync)>>();

    let conn = state.pool.get().await.map_err(internal_error)?;

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
        &format!("DELETE FROM {NOTES_TABLE_NAME} WHERE id=$1 AND user_id=$2 RETURNING id, text"),
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
        &format!("INSERT INTO {NOTES_TABLE_NAME} (text, user_id) VALUES ($1, $2) RETURNING id, text"),
        &[&body.text, &user.id]
    ).await.map_err(internal_error)?;

    let created_note = Note {
        id: row.get(0),
        text: row.get(1)
    };

    Ok(Json(created_note))
}