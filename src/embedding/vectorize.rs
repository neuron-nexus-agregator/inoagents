use crate::embedding::interface::Embedding;
use crate::embedding::model::{Request, Response};
use crate::utils::funcs::keep_russian_and_dot;

pub struct YandexEmbedding {
    model: String,
    token: String,
    url: String,
}

impl YandexEmbedding {
    pub fn new(model: String, token: String, url: String) -> Self {
        YandexEmbedding { model, token, url }
    }
}

impl Embedding for YandexEmbedding {
    /// Получение векторного представления текста в виде 256-мерного массива f32
    async fn get_embedding(&self, text: &str) -> Result<Response, reqwest::Error> {
        let sub_text = keep_russian_and_dot(text).to_lowercase();
        let request = Request {
            model_uri: self.model.clone(),
            text: sub_text,
        };

        let client = reqwest::Client::new();
        let resp = client
            .post(&self.url)
            .header("Authorization", format!("Api-Key {}", self.token))
            .json(&request)
            .send()
            .await?;

        let response: Response = resp.json().await?;
        Ok(response)
    }
}
