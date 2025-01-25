use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;
use std::time::Duration;

pub mod create_sms_order;
pub mod get_api_balance;
pub mod get_country_prices;
pub mod get_service_list;
pub mod get_sms_code;

const API_URL: &str = "....";

#[derive(Debug, Clone)]

pub struct SmsClient {
    client: reqwest::Client,
    api_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiErrorInfo {
    pub message: String,
    pub param: String,
    pub description: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct SMSResponseError {
    pub success: i8,
    pub errors: Vec<ApiErrorInfo>,
}

impl SmsClient {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
        return Self { client, api_key };
    }
}

fn deserialize_to_float<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    f32::from_str(&s).map_err(serde::de::Error::custom)
}
