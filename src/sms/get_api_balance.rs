use serde::{Deserialize, Serialize};

use super::{ApiResponseError, SmsClient, API_URL};

#[derive(Serialize, Deserialize, Debug)]
pub struct BalanceResponseType {
    pub balance: String,
}

impl SmsClient {
    pub async fn get_api_balance(self) -> Result<f32, ApiResponseError> {
        let request = self
            .client
            .post(format!("{}/request/balance", API_URL))
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.api_key),
            )
            .send()
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
