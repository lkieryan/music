use reqwest::Client;
use serde::{Deserialize, Serialize};
use shared_models::{ApiResponse, ApiError};
use shared_utils::{AppError, AppResult};

pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn get<T>(&self, endpoint: &str) -> AppResult<ApiResponse<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            let api_response: ApiResponse<T> = response
                .json()
                .await
                .map_err(|e| AppError::ApiError(e.to_string()))?;
            Ok(api_response)
        } else {
            Err(AppError::ApiError("Request failed".to_string()))
        }
    }
}