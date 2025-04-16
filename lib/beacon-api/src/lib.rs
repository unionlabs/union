pub mod client;
pub mod errors;
pub mod types;

pub mod routes;

// #[cfg(test)]
// mod tests {
//     use beacon_api_types::custom_types::Slot;

//     use crate::client::BeaconApiClient;

//     const URL: &str = "https://lodestar-sepolia.chainsafe.io";

//     #[tokio::test]
//     async fn block() {
//         let client = BeaconApiClient::new(URL).await.unwrap();

//         let block = client.block(Slot::new(7118848).into()).await.unwrap();

//         dbg!(block);
//     }
// }
