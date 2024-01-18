use serde::{Deserialize, Serialize};

use super::{deserialize_to_float, SmsClient, API_URL};

#[derive(Serialize, Deserialize, Debug)]
pub struct SmsOrderInfo {
    #[serde(rename = "cc")]
    pub area_code: String,
    #[serde(deserialize_with = "deserialize_to_float")]
    pub cost: f32,
    pub country: String,
    pub expires_in: i16,
    pub expiration: i64,
    pub message: String,
    pub number: i64,
    pub order_id: String,
    pub phonenumber: String,
    pub pool: i8,
    pub service: String,
    pub success: i8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SMSOrderError {
    pub success: u8,
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
}

impl SmsClient {
    pub async fn create_sms_order(
        self,
        service: &str,
        country: &str,
    ) -> Result<SmsOrderInfo, SMSOrderError> {
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
            let error = request.json::<SMSOrderError>().await.unwrap();
            Err(error)
        }
    }
}
