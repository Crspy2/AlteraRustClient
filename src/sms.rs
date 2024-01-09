use std::collections::{HashMap, BTreeMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;

const API_URL: &str = "https://5sim.net/v1";

#[derive(Debug, Clone)]

pub struct SmsClient {
    api_key: String,
}

#[derive(Deserialize, Debug)]
struct BalanceResponseType {
    pub balance: i32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CountryResponseType {
    pub iso: HashMap<String, i32>,
    pub prefix: HashMap<String, i32>,
    pub text_en: String,
}

impl SmsClient {
    pub fn new(api_key: String) -> Self {
        return Self {
            api_key
        }
    }

    pub async fn get_api_balance(self) -> Option<f32> {
        let client = reqwest::Client::new();
        let request = client.get(format!("{}/user/profile", API_URL))
            .header(reqwest::header::AUTHORIZATION, format!("{}", self.api_key))
            .send()
            .await
            .unwrap()
            .json::<BalanceResponseType>()
            .await;

        println!("{:?}", request);

        match request {
            Ok(item) => return Some((item.balance / 100) as f32),
            Err(_) => return None
        }
        
    }

    pub async fn get_countries_list(self) -> Option<BTreeMap<String, CountryResponseType>> {
        let request = reqwest::get(format!("{}/guest/countries", API_URL)).await.unwrap();
        
        if request.status() == 200 {
            let country_info = request.json::<BTreeMap<String, CountryResponseType>>().await;
            match country_info {
                Ok(contries) => Some(contries),
                Err(_) => None
            }
        } else {
            None
        }
    }
}