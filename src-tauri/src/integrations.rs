use reqwest::Client;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiConfig {
    pub api_key: String,
    pub endpoints: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub struct IntegrationService {
    client: Client,
    config: ApiConfig,
}

impl IntegrationService {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            config: ApiConfig {
                api_key,
                endpoints: HashMap::from([
                    ("helius".into(), "https://api.helius.xyz".into()),
                    ("birdeye".into(), "https://public-api.birdeye.so".into()),
                ]),
            },
        }
    }

    pub async fn fetch_data(&self, service: &str, path: &str) -> Result<ApiResponse, String> {
        let endpoint = self.config.endpoints.get(service)
            .ok_or("Unknown service")?;
        
        let url = format!("{}{}", endpoint, path);
        
        let response = self.client.get(&url)
            .header("Authorization", &self.config.api_key)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        
        let response_text = response.text().await.map_err(|e| e.to_string())?;
        let data: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| e.to_string())?;
        
        Ok(ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        })
    }
}

#[tauri::command]
pub async fn fetch_api_data(
    api_key: String,
    service: String,
    path: String
) -> Result<ApiResponse, String> {
    let integration = IntegrationService::new(api_key);
    integration.fetch_data(&service, &path).await
}
