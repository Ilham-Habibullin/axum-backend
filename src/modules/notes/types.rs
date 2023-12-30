use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Note {
  pub id: i32,
  pub text: String
}

#[derive(Deserialize, Serialize)]
pub struct DeleteNotePayload {
  pub id: i32
}

#[derive(Deserialize, Serialize)]
pub struct CreateNotePayload {
  pub text: String
}
