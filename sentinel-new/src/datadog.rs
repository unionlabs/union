// use reqwest::Client;
// use serde::{Deserialize, Serialize};

// #[derive(Serialize)]
// pub struct Data {
//     series: Vec<Metric>,
// }

// #[derive(Serialize)]
// pub struct Metric {
//     metric: String,
//     points: Vec<(u64, f64)>,
//     tags: Vec<String>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Log {
//     message: String,
//     ddtags: String,
//     hostname: String,
//     service: String,
//     status: String,
// }

// pub async fn validate_api_key(api_key: &str) -> Result<bool, reqwest::Error> {
//     let client = Client::new();
//     let url = "https://api.datadoghq.eu/api/v1/validate";

//     let response = client.get(url).header("DD-API-KEY", api_key).send().await?;

//     let status = response.status();
//     let text = response.text().await?;
//     println!("Validation Response Status: {}", status);
//     println!("Validation Response Body: {}", text);

//     if status == 200 {
//         Ok(true)
//     } else {
//         Ok(false)
//     }
// }

// pub async fn send_to_datadog(api_key: &str, data: &Data) -> Result<(), reqwest::Error> {
//     let client = Client::new();
//     let url = "https://api.datadoghq.eu/api/v1/series?api_key=".to_string() + api_key;

//     let response = client.post(&url).json(data).send().await?;

//     println!("Response: {:?}", response.text().await?);

//     Ok(())
// }

// pub async fn send_log_to_datadog(
//     api_key: &str,
//     log: &Log,
//     host: &str,
// ) -> Result<(), reqwest::Error> {
//     let client = Client::new();
//     let url = host.to_owned() + "/input" + "?dd-api-key=" + api_key;

//     let response = client.post(&url).json(log).send().await?;

//     println!("Response: {:?}", response.text().await?);

//     Ok(())
// }

// pub fn log_builder(
//     message: String,
//     ddtags: Option<String>,
//     hostname: Option<String>,
//     service: Option<String>,
//     status: Option<String>,
// ) -> Log {
//     let log_info = Log {
//         message: message,
//         ddtags: ddtags.unwrap_or_else(|| "env:dev,version:1.0".to_string()),
//         hostname: hostname.unwrap_or_else(|| "my-rust-app".to_string()),
//         service: service.unwrap_or_else(|| "rust-logging-service".to_string()),
//         status: status.unwrap_or_else(|| "info".to_string()),
//     };
//     log_info
// }
