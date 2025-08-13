use regex::Regex;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Text {
    text: String,
}

pub async fn getText(id: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let username = "newrv";
    let password = "KKEmhu";
    let res = client
        .get(format!("https://rtgazeta.ru/api/news/{id}"))
        .basic_auth(username, Some(password))
        .send()
        .await?;
    let respose: Text = res.json().await?;
    Ok(strip_html(respose.text))
}

fn strip_html(input: String) -> String {
    let re = Regex::new(r"<[^>]*>").unwrap();
    re.replace_all(&input, "").to_string()
}
