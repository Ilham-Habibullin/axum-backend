use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Note {}

#[derive(Deserialize, Serialize)]
pub struct DeleteNotePayload {}

#[derive(Deserialize, Serialize)]
pub struct CreateNotePayload {}
