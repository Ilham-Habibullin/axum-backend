
use std::usize;

use axum_macros::debug_handler;

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

#[derive(serde::Deserialize)]
pub struct Search {
    pub search: Option<String>,
}

type QuieryBuildParam = (&'static str, Box<(dyn ToSql + Sync + Send)>);
type SqlParams = Vec<QuieryBuildParam>;

#[debug_handler]
pub async fn get_users(
    pagination: Query<Pagination>,
    role_option: Query<Role>,
    mut search_option: Query<Search>,
    state: State<AppState>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let mut for_users_query: SqlParams = vec![];
    let mut for_count_query: SqlParams = vec![];

    role_option.role.map(|role| {
        for_users_query.push(("role = ", Box::new(role)));
        for_count_query.push(("role = ", Box::new(role)));
    });

    search_option.search.take().map(|search| {
        let search_ = format!("%{search}%");
        for_users_query.push(("username LIKE ", Box::new(search_.clone())));
        for_count_query.push(("username LIKE ", Box::new(search_.clone())));
    });

    let conn = state.pool.get().await.map_err(internal_error)?;

    let mut query_users = format!("SELECT id, username, role FROM {USER_TABLE_NAME}");
    let mut query_count = format!("SELECT count(*) FROM {USER_TABLE_NAME}");

    if !for_users_query.is_empty() { query_users += " WHERE " }
    if !for_count_query.is_empty() { query_count += " WHERE " }

    let mut parameters_count: usize = 0;

    let mut params_users = for_users_query
        .iter()
        .enumerate()
        .map(|(i, (key, value))| {
            if i > 0 { query_users += " AND " }
            let parameter_number = i+1;
            query_users += &format!("{}${}", key, parameter_number);
            parameters_count = parameter_number;
            value.as_ref() as &(dyn ToSql + Sync)
        })
        .collect::<Vec<&(dyn ToSql + Sync)>>();

    params_users.push(&pagination.limit);
    params_users.push(&pagination.offset);

    let params_count = for_count_query
        .iter()
        .enumerate()
        .map(|(i, (key, value))| {
            if i > 0 { query_count += " AND " }
            query_count += &format!("{}${}", key, i+1);
            value.as_ref() as &(dyn ToSql + Sync)
        })
        .collect::<Vec<&(dyn ToSql + Sync)>>();

    if parameters_count != 0 {
        query_users += &format!(" ORDER BY id LIMIT ${} OFFSET ${}", parameters_count + 1, parameters_count + 2);
    } else {
        query_users += " ORDER BY id LIMIT $1 OFFSET $2";
    }

    let (rows, row_count) = tokio::try_join!(
        conn.query(&query_users, &params_users),
        conn.query_one(&query_count, &params_count)
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