use serde::Serialize;

#[derive(Serialize)]
pub struct Item {
    pub name: String,
    pub embedding: Vec<f32>,
    pub status: String,
    pub is_removed: bool,
}
