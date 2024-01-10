use serde::{Deserialize, Serialize};

use super::{
    deserialize_float, deserialize_int, ApiErrorInfo, ApiResponseError, SmsClient, API_URL,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct CountryPriceInfo {
    pub country: i32,
    #[serde(deserialize_with = "deserialize_int")]
    pub success_rate: i16,
    #[serde(deserialize_with = "deserialize_float")]
    pub price: f32,
    #[serde(deserialize_with = "deserialize_float")]
    pub low_price: f32,
    pub country_id: i32,
    pub name: String,
    #[serde(rename = "short_name")]
    pub iso: String,
}

impl SmsClient {
    pub async fn get_service_prices(
        self,
        service: &str,
    ) -> Result<Vec<CountryPriceInfo>, ApiResponseError> {
        let request = self
            .client
            .post(format!("{}/request/success_rate", API_URL))
            .form(&[("service", service)])
            .send()
            .await
            .unwrap();

        if request.status() == 200 {
            let service_info = request.json::<Vec<CountryPriceInfo>>().await;
            match service_info {
                Ok(services) => Ok(services),
                Err(err) => Err(ApiResponseError {
                    success: 1,
                    errors: vec![ApiErrorInfo {
                        param: "service".to_string(),
                        message: err.to_string(),
                        description: "The service name passed is not valid or is misspelled, please check our supported services.".to_string(),
                    }],
                }),
            }
        } else {
            let error_info = request.json::<ApiResponseError>().await.unwrap();
            return Err(error_info);
        }
    }
}
