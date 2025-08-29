#[derive(Debug, Clone)]
pub struct Record {
    pub name: String,
    pub record_type: String,
    pub embedding: Vec<f32>,
    pub is_removed: bool,
}
