use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ShortenPayload {
    pub url: String,
    pub name: Option<String>,
    pub password: String,
}