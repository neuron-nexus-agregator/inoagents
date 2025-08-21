use regex::Regex;
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
pub struct Text {
    text: String,
}

pub async fn get_text(id: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let username = env::var("RVUSER").ok().unwrap();
    let password = env::var("RVPASS").ok().unwrap();
    let res = client
        .get(format!("https://rtgazeta.ru/api/news/{id}"))
        .basic_auth(username, Some(password))
        .send()
        .await?;
    let respose: Text = res.json().await?;
    Ok(strip_html(respose.text))
}

pub fn strip_html(input: String) -> String {
    let re = Regex::new(r"<[^>]*>").unwrap();
    re.replace_all(&input, "").to_string()
}
