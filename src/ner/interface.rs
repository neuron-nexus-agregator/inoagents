use crate::ner::model;
pub trait Entities {
    async fn get_entities(&self, text: &str) -> Result<model::Response, anyhow::Error>;
}
