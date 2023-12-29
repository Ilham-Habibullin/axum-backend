use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SignUpPayload {
  pub username: String,
  pub password: String,
}

#[derive(Deserialize)]
pub struct SignInPayload {
  pub username: String,
  pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub role: usize,
    pub iat: usize,
    pub exp: usize,
}