use crate::ner::interface::Entities;
use crate::ner::model::{Request, Response};

pub struct PythonEntities {
    url: String,
}

impl PythonEntities {
    pub fn new(url: String) -> Self {
        PythonEntities { url }
    }
}

impl Entities for PythonEntities {
    async fn get_entities(&self, text: &str) -> Result<Response, anyhow::Error> {
        let client = reqwest::Client::new();
        let req = Request {
            text: text.to_string(),
        };
        let resp = client.post(&self.url).json(&req).send().await?;
        let res: Response = resp.json().await?;
        Ok(res)
    }
}
