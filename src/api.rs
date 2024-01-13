use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use twilight_model::id::{marker::UserMarker, Id};

const BASE_URL: &str = "http://localhost:8080/api";

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse {
    success: bool,
    message: String,
    pub resource: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(rename = "Balance")]
    pub balance: i32,
    #[serde(rename = "CreatedAt")]
    pub created_at: String,
    #[serde(rename = "UpdatedAt")]
    pub updated_at: String,
    #[serde(rename = "DiscordID")]
    pub discord_id: String,
    #[serde(rename = "Email")]
    pub email: String,
    #[serde(rename = "ID")]
    pub id: u32,
    #[serde(rename = "Image")]
    pub image: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Role")]
    pub role: u8,
    #[serde(rename = "Deposits")]
    pub deposits: Vec<Value>,
}

pub async fn get_user_data(user_id: Id<UserMarker>) -> Result<User, ApiResponse> {
    let client = reqwest::Client::new();

    let request = client
        .get(format!("{}/user/discord/{}", BASE_URL, user_id))
        .header(
            reqwest::header::AUTHORIZATION,
            env::var("ADMIN_TOKEN").unwrap().parse::<String>().unwrap(),
        )
        .send()
        .await
        .unwrap()
        .json::<ApiResponse>()
        .await
        .unwrap();

    if request.success {
        let user_info: User = serde_json::from_value(request.resource).unwrap();
        return Ok(user_info);
    } else {
        return Err(request);
    }
}
