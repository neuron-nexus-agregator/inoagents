use crate::ner::model::{Request, Response};

pub async fn get_entities(text: &str, url: &str) -> Result<Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let req = Request {
        text: text.to_string(),
    };
    let resp = client.post(url).json(&req).send().await?;
    let res: Response = resp.json().await?;
    Ok(res)
}
