use serde::{Deserialize, Serialize};

use crate::sms::ApiResponseError;

use super::{deserialize_float, SmsClient, API_URL};

#[derive(Serialize, Deserialize, Debug)]
pub struct SmsOrderInfo {
    pub cc: String,
    #[serde(deserialize_with = "deserialize_float")]
    pub cost: f32,
    pub country: String,
    pub expires_in: i64,
    pub message: String,
    pub number: i64,
    pub order_id: String,
    pub phonenumber: String,
    pub pool: i8,
    pub service: String,
    pub success: i8,
}

impl SmsClient {
    pub async fn create_sms_order(
        self,
        service: &str,
        country: &str,
    ) -> Result<SmsOrderInfo, ApiResponseError> {
        let request = self
            .client
            .post(format!("{}/purchase/sms", API_URL))
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.api_key),
            )
            .form(&[
                ("service", service),
                ("country", country),
                ("pricing_option", "1"),
            ])
            .send()
            .await
            .unwrap();

        if request.status() == 200 {
            let order_info = request.json::<SmsOrderInfo>().await.unwrap();
            Ok(order_info)
        } else {
            let error_info = request.json::<ApiResponseError>().await.unwrap();
            Err(error_info)
        }
    }
}
