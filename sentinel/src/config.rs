use serde::{ Deserialize, Serialize };
use sqlx::FromRow;
use unionlabs::hash::H160;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub union: chain_utils::union::Config,
    pub ethereum: chain_utils::ethereum::Config,
    pub osmosis: chain_utils::cosmos::Config,
    pub union_contract: String,
    pub osmosis_contract: String,
    pub ethereum_contract: H160,
    pub channel: String,
    pub counterparty_channel: String,
    pub amount: String,
    pub db_url: String,
    // pub datadog_data: DatadogData,
    pub connections: Vec<ConnectionPair>,
    pub ethereum_priv_key: String,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(deny_unknown_fields)]
// pub struct DatadogData {
//     pub datadog_api_key: String,
//     pub datadog_log_host: String,
//     pub datadog_validate_host: String,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionPair {
    pub source_chain: String,
    pub target_chain: String,
    pub send_packet_interval: i32,
    pub expect_full_circle: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Chain {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum ChainId {
    Union = 0,
    Osmosis = 1,
    Ethereum = 2,
    // Add other chains as needed
}

impl ChainId {
    pub fn from_str(chain_name: &str) -> Option<ChainId> {
        match chain_name.to_lowercase().as_str() {
            "union" => Some(ChainId::Union),
            "osmosis" => Some(ChainId::Osmosis),
            "ethereum" => Some(ChainId::Ethereum),
            // Add other chain mappings as needed
            _ => None,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            ChainId::Union => "union",
            ChainId::Osmosis => "osmosis",
            ChainId::Ethereum => "ethereum",
            // Add other chain mappings as needed
        }
    }

    pub fn from_i32(chain_id: &i32) -> &'static str {
        match chain_id {
            0 => "union",
            1 => "osmosis",
            2 => "ethereum",
            // Add other chain mappings as needed
            _ => "unknown",
        }
    }
}

// Define a struct to store events for a packet sequence
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PacketStatus {
    // pub id: i32,
    pub source_chain_id: i32,
    pub target_chain_id: i32,
    pub sequence_number: i64,
    pub send_packet: Option<serde_json::Value>,
    pub recv_packet: Option<serde_json::Value>,
    pub write_ack: Option<serde_json::Value>,
    pub acknowledge_packet: Option<serde_json::Value>,
    pub last_update: chrono::DateTime<chrono::Utc>,
}
impl PacketStatus {
    pub fn new(source_chain_name: &str, target_chain_name: &str, sequence_number: i64) -> Self {
        let source_chain_id = ChainId::from_str(source_chain_name).unwrap() as i32;
        let target_chain_id: i32 = ChainId::from_str(target_chain_name).unwrap() as i32;

        Self {
            source_chain_id,
            target_chain_id,
            sequence_number,
            send_packet: None,
            recv_packet: None,
            write_ack: None,
            acknowledge_packet: None,
            last_update: chrono::Utc::now(),
        }
    }
}
