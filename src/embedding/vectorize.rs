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

#[deprecated]
pub async fn get_embedding(
    text: &str,
    model: &str,
    token: &str,
    url: &str,
) -> Result<Response, reqwest::Error> {
    let sub_text = keep_russian_and_dot(text).to_lowercase();
    let request = Request {
        model_uri: model.to_string(),
        text: sub_text,
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .header("Authorization", format!("Api-Key {token}"))
        .json(&request)
        .send()
        .await?;

    let response: Response = resp.json().await?;
    Ok(response)
}
