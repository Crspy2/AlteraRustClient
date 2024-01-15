use serde::{Deserialize, Serialize};

use super::{deserialize_to_float, ApiErrorInfo, SMSResponseError, SmsClient, API_URL};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CountryPriceInfo {
    pub country_id: i32,
    pub name: String,
    #[serde(rename = "short_name")]
    pub iso: String,
    #[serde(deserialize_with = "deserialize_to_float")]
    pub price: f32,
    #[serde(deserialize_with = "deserialize_to_float")]
    pub low_price: f32,
    pub success_rate: i32,
}

impl SmsClient {
    pub async fn get_country_prices(
        self,
        service: &str,
    ) -> Result<Vec<CountryPriceInfo>, SMSResponseError> {
        let request = self
            .client
            .post(format!("{}/request/success_rate", API_URL))
            .form(&[("service", service)])
            .send()
            .await
            .unwrap();

        if request.status() == 200 {
            let service_info = request.json::<Vec<CountryPriceInfo>>().await;
            // let service_info = request.json::<Value>().await;
            match service_info {
                Ok(services) => Ok(services),
                Err(err) => Err(SMSResponseError {
                    success: 1,
                    errors: vec![ApiErrorInfo {
                        param: "service".to_string(),
                        message: err.to_string(),
                        description: "The service name passed is not valid or is misspelled, please check our supported services.".to_string(),
                    }],
                }),
            }
        } else {
            let error_info = request.json::<SMSResponseError>().await.unwrap();
            return Err(error_info);
        }
    }
}
