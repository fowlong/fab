use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct OpenResponse {
  pub doc_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PatchRequest {
  Array(Vec<serde_json::Value>),
  Object(serde_json::Value),
}
