use serde::{Deserialize, Serialize};
use crate::types::Roles;

#[derive(Deserialize)]
pub struct DeleteUserPayload {
  pub username: String
}

#[derive(Deserialize, Serialize)]
pub struct UserWithPassword {
  pub id: i32,
  pub username: String,
  pub password: String,
  pub role: Roles
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct User {
  pub id: i32,
  pub username: String,
  pub role: Roles
}

#[derive(Deserialize)]
pub struct PromoteUserPayload {
  pub id: i32
}