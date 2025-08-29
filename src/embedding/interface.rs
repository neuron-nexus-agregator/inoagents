use crate::embedding::model::Response;

pub trait Embedding {
    async fn get_embedding(&self, text: &str) -> Result<Response, reqwest::Error>;
}
