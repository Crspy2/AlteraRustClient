use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

const API_URL: &str = "https://api.smspool.net";

#[derive(Debug, Clone)]

pub struct SmsClient {
    api_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct APIErrorInfo {
    pub message: String,
    pub param: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponseError {
    pub success: i8,
    pub errors: Vec<APIErrorInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BalanceResponseType {
    pub balance: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CountryResponseType {
    pub iso: BTreeMap<String, i32>,
    pub prefix: BTreeMap<String, i32>,
    pub text_en: String,
}

impl SmsClient {
    pub fn new(api_key: String) -> Self {
        return Self { api_key };
    }

    pub async fn get_api_balance(self) -> Result<f32, ApiResponseError> {
        let request = reqwest::get(format!("{}/request/balance?key={}", API_URL, self.api_key))
            .await
            .unwrap();

        if request.status() == 200 {
            let balance_info = request.json::<BalanceResponseType>().await.unwrap();
            let balance = balance_info.balance.parse::<f32>().unwrap();
            return Ok(balance);
        } else {
            let error_info = request.json::<ApiResponseError>().await.unwrap();
            return Err(error_info);
        }
    }
}
