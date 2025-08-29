use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Request {
    #[serde(rename = "modelUri")]
    pub model_uri: String,
    pub text: String,
}

#[derive(Deserialize)]
pub struct Response {
    pub embedding: Option<Vec<f32>>,
}
