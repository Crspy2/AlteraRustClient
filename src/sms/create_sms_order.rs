use serde::{Deserialize, Serialize};

use crate::sms::SMSResponseError;

use super::{deserialize_to_float, ApiErrorInfo, SmsClient, API_URL};

#[derive(Serialize, Deserialize, Debug)]
pub struct SmsOrderInfo {
    #[serde(rename = "cc")]
    pub area_code: String,
    #[serde(deserialize_with = "deserialize_to_float")]
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
    ) -> Result<SmsOrderInfo, SMSResponseError> {
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
            let error = request.json::<SMSResponseError>().await;

            match error {
                Ok(info) => return Err(info),
                Err(_) => {
                    return Err(SMSResponseError {
                        success: 0,
                        errors: vec![ApiErrorInfo {
                            message: "No numbers available at the moment, please try again later."
                                .to_string(),
                            description:
                                "No numbers available at the moment, please try again later."
                                    .to_string(),
                            param: "pool".to_string(),
                        }],
                    })
                }
            };
        }
    }
}
