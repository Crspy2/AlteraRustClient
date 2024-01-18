use serde::{Deserialize, Serialize};

use super::{SMSResponseError, SmsClient, API_URL};

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckSMSResponse {
    pub status: i8,
    pub message: Option<String>,
    pub resend: i8,
    pub expiration: i64,
    pub time_left: Option<i32>,
    pub full_sms: String,
}

impl SmsClient {
    pub async fn get_sms_code(self, order_id: &str) -> Result<CheckSMSResponse, SMSResponseError> {
        let request = self
            .client
            .post(format!("{}/sms/check", API_URL))
            .header(reqwest::header::AUTHORIZATION, self.api_key)
            .form(&[("order_id", order_id)])
            .send()
            .await
            .unwrap();

        if request.status() == 200 {
            let sms_info = request.json::<CheckSMSResponse>().await.unwrap();
            Ok(sms_info)
        } else {
            let error_info = request.json::<SMSResponseError>().await.unwrap();
            Err(error_info)
        }
    }
}
