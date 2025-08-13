use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Request {
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct Response {
    pub entities: Vec<Entity>,
}

#[derive(Deserialize, Debug)]
pub struct Entity {
    pub name: String,
    pub norm_name: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub context: String,
}
