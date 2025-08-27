use crate::embedding::model::{Request, Response};

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

fn keep_russian_and_dot(input: &str) -> String {
    input
        .chars()
        .filter(|c| {
            // Русские буквы (А-Я, а-я, Ё, ё)
            (*c >= 'А' && *c <= 'я') || *c == 'Ё' || *c == 'ё' || *c == '.'
        })
        .collect()
}
