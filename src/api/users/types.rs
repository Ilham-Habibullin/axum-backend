use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateUserPayload {
  pub username: String,
  pub password: String,
}

#[derive(Deserialize)]
pub struct DeleteUserPayload {
  pub username: String
}

#[derive(Deserialize, Serialize)]
pub struct User {
  pub id: i32,
  pub username: String,
  pub password: String
}

#[derive(Deserialize)]
pub struct PromoteUserPayload {
  pub id: i32
}