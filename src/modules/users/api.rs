use std::{borrow::Borrow, rc::Rc};

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

type SqlParams<'a> = Vec<&'a (dyn ToSql + Sync)>;

type QueryAndParams = (
    String,
    String,
    Vec<Rc<(dyn ToSql + Sync)>>,
    Vec<Rc<(dyn ToSql + Sync)>>,
    bool
);

fn obtain<T: ToSql + Sync + Clone + 'static>(
    optional: Option<T>,
    qap: QueryAndParams,
    (for_query_and, for_query_count_and, for_query_where, for_query_count_where): (&str, &str, &str, &str)
) -> Option<QueryAndParams> {
    let (
        mut query,
        mut query_count,
        mut params,
        mut params_count,
        where_clause_set
    ) = qap;

    let unwrapped = match optional.clone() {
        Some(r) => r,
        None => return Some((query, query_count, params.clone(), params_count.clone(), false))
    };

    if where_clause_set {
        query += for_query_and;
        query_count += for_query_count_and;
    } else {
        query += for_query_where;
        query_count += for_query_count_where;
    }

    let unwrapped_rc = Rc::new(unwrapped);

    params.push(unwrapped_rc.clone());
    params_count.push(unwrapped_rc.clone());

    Some((query, query_count, params, params_count, true))
}

impl Role {
    fn obtain(&self, query_and_params: QueryAndParams) -> Option<QueryAndParams> {
        obtain::<i16>(
            self.role,
            query_and_params,
            (
                " AND role=$3", " AND role=$1",
                " WHERE role=$3", " WHERE role=$1"
            )
        )
    }
}

impl Search {
    fn obtain(&self, query_and_params: QueryAndParams) -> Option<QueryAndParams> {
        obtain::<String>(
            self.search.clone(),
            query_and_params,
            (
                " AND username LIKE $4", " AND username LIKE $2",
                " WHERE username LIKE $3", " WHERE username LIKE $1"
            )
        )
    }
}

pub async fn get_users_x(
    pagination: Query<Pagination>,
    role_option: Query<Role>,
    search_option: Query<Search>,
    state: State<AppState>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let limit = pagination.0.limit;
    let offset = pagination.0.offset;

    let (mut query_users, query_count, params_users_rc, params_count_rc, _where_clause_set) = role_option
        .obtain((
            format!("SELECT id, username, role FROM {USER_TABLE_NAME}"),
            format!("SELECT count(*) FROM {USER_TABLE_NAME}"),
            vec![Rc::new(limit), Rc::new(offset)],
            vec![],
            false
        )).and_then(
            |qap| search_option.obtain(qap)
        ).expect("initial values should have been returned anyway");

    query_users += " ORDER BY id LIMIT $1 OFFSET $2";

    let conn = state.pool.get().await.map_err(internal_error)?;

    fn turn_rc_into_ref(rcs: &Vec<Rc<dyn ToSql + Sync>>) -> SqlParams {
        rcs
            .iter()
            .map(Rc::borrow)
            .collect::<SqlParams>()
    }

    let params_users = turn_rc_into_ref(&params_users_rc);
    let params_count = turn_rc_into_ref(&params_count_rc);

    let (rows, row_count) = tokio::try_join!(
        conn.query(&query_users, &params_users),
        conn.query_one(&query_count, &params_count)
    ).map_err(internal_error)?;

    let reply = rows
        .iter()
        .map(|r| User {
            id: r.get(0),
            username: r.get(1),
            role: r.get::<usize, i16>(2).into()
        }).collect::<Vec<User>>();

    let count: i64 = row_count.get(0);
    let header_count_value = HeaderValue::from_str(&count.to_string()).map_err(internal_error)?;

    let mut response = Json(reply).into_response();
    response.headers_mut().insert("x-total-count", header_count_value);

    Ok(response)
}

pub async fn get_users(
    pagination: Query<Pagination>,
    role_option: Query<Role>,
    search_option: Query<Search>,
    state: State<AppState>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let limit = pagination.0.limit;
    let offset = pagination.0.offset;

    let mut role_: i16 = 0;
    let mut search_ = String::default();

    let mut query_users = format!("SELECT id, username, role FROM {USER_TABLE_NAME}");
    let mut params_users: SqlParams = vec![&limit, &offset];

    let mut query_count = format!("SELECT count(*) FROM {USER_TABLE_NAME}");
    let mut params_count: SqlParams = vec![];

    let mut where_clause_set = false;

    role_option.role.map(|role| {
        role_ = role;

        query_users += " WHERE role=$3";
        params_users.push(&role_ as &(dyn ToSql + Sync));

        query_count += " WHERE role=$1";
        params_count.push(&role_ as &(dyn ToSql + Sync));

        where_clause_set = true;
    });

    search_option.search.clone().map(|search| {
        if where_clause_set {
            query_users += " AND username LIKE $4";
            query_count += " AND username LIKE $2";
        } else {
            query_users += " WHERE username LIKE $3";
            query_count += " WHERE username LIKE $1";
        }

        search_ = format!("%{search}%");
        params_users.push(&search_ as &(dyn ToSql + Sync));
        params_count.push(&search_ as &(dyn ToSql + Sync));

        where_clause_set = true;
    });

    query_users += " ORDER BY id LIMIT $1 OFFSET $2";

    let conn = state.pool.get().await.map_err(internal_error)?;

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