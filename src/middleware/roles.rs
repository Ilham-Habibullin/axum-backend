use axum::{
    extract::{State, Request},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse
};

use crate::{Roles, modules::users::types::User};

pub async fn roles(State(role): State<Roles>, req: Request, next: Next) -> Result<impl IntoResponse, (StatusCode, String)> {
    let x = req.extensions().get::<User>().ok_or_else(|| {
        (StatusCode::FORBIDDEN, "Not enough rights".to_string())
    })?;

    if (x.role.clone() as i16) < (role as i16) {
        Err((StatusCode::FORBIDDEN, "Not enough rights".to_string()))
    } else {
        Ok(next.run(req).await)
    }
}

