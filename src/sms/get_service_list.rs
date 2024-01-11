use serde::{Deserialize, Serialize};

use super::{ApiResponseError, SmsClient, API_URL};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceResponse {
    #[serde(rename = "ID")]
    pub id: i16,
    pub name: String,
    pub favourite: i8,
}

impl SmsClient {
    pub async fn get_service_list(self) -> Result<Vec<ServiceResponse>, ApiResponseError> {
        let request = self
            .client
            .post(format!("{}/service/retrieve_all", API_URL))
            .send()
            .await
            .unwrap();

        if request.status() == 200 {
            let service_info = request.json::<Vec<ServiceResponse>>().await.unwrap();
            Ok(service_info)
        } else {
            let error_info = request.json::<ApiResponseError>().await.unwrap();
            Err(error_info)
        }
    }
}
