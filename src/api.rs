use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
pub struct Number {
    #[serde(rename = "Number")]
    pub number: String,
    #[serde(rename = "Service")]
    pub service: String,
    #[serde(rename = "Country")]
    pub country: String,
    #[serde(rename = "Price")]
    pub price: i32,
    #[serde(rename = "OrderID")]
    pub order_id: String,
    #[serde(rename = "Received")]
    pub received: bool,
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
    #[serde(rename = "Numbers")]
    pub numbers: Vec<Number>,
    #[serde(rename = "Token")]
    pub token: String,
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
        let user_info: User = serde_json::from_value(request.resource.clone()).unwrap();
        Ok(user_info)
    } else {
        Err(request)
    }
}

pub async fn post_user_number(
    number: &str,
    service: &str,
    country: &str,
    price: i32,
    order_id: &str,
    user_id: u32,
) -> Result<(), ApiResponse> {
    let client = reqwest::Client::new();

    let request = client
        .post(format!("{}/user/numbers", BASE_URL))
        .header(
            reqwest::header::AUTHORIZATION,
            env::var("ADMIN_TOKEN").unwrap().parse::<String>().unwrap(),
        )
        .json(&serde_json::json!({
            "number": number,
            "service": service,
            "country": country,
            "price": price,
            "order_id": order_id,
            "user_id": user_id,
        }))
        .send()
        .await
        .unwrap()
        .json::<ApiResponse>()
        .await
        .unwrap();

    if request.success {
        Ok(())
    } else {
        Err(request)
    }
}

pub async fn mark_number_received(number: String) -> Result<(), ApiResponse> {
    let client = reqwest::Client::new();

    let request = client
        .put(format!("{}/user/numbers/{}/received", BASE_URL, number))
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
        Ok(())
    } else {
        Err(request)
    }
}
