use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{SMSResponseError, SmsClient, API_URL};

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckSMSResponse {
    pub status: i8,
    pub message: Option<String>,
    pub resend: Option<i8>,
    pub expiration: i64,
    pub time_left: Option<i32>,
    pub sms: Option<String>,
    pub full_sms: Option<String>,
}

impl SmsClient {
    pub async fn get_sms_code(self, order_id: &str) -> Result<CheckSMSResponse, SMSResponseError> {
        let request = self
            .client
            .post(format!("{}/sms/check", API_URL))
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.api_key),
            )
            .form(&[("orderid", order_id)])
            .send()
            .await
            .unwrap();

        if request.status() == 200 {
            let sms_info = request.json::<CheckSMSResponse>().await.unwrap();
            Ok(sms_info)
        } else {
            let info = request.json::<Value>().await.unwrap();

            let error_info = serde_json::from_value::<SMSResponseError>(info).unwrap();
            Err(error_info)
        }
    }
}
