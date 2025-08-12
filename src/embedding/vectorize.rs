use crate::embedding::model::{Request, Response};

pub async fn get_embedding(
    text: &str,
    model: &str,
    token: &str,
    url: &str,
) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
    let request = Request {
        model_uri: model.to_string(),
        text: text.to_string(),
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
