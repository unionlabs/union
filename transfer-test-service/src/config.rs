use serde::{ Deserialize, Serialize };
use sqlx::{ FromRow, PgPool, Postgres, QueryBuilder };
use std::{
    collections::HashMap,
    fs::{ File, OpenOptions },
    pin::Pin,
    sync::Arc,
    time::{ SystemTime, UNIX_EPOCH },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub union: chain_utils::union::Config,
    pub ethereum: chain_utils::ethereum::Config,
    pub osmosis: chain_utils::cosmos::Config,
    pub union_contract: String,
    pub osmosis_contract: String,
    pub channel: String,
    pub port: String,
    pub amount: String,
    pub datadog_data: DatadogData,
    pub connections: Vec<ConnectionPair>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatadogData {
    pub datadog_api_key: String,
    pub datadog_log_host: String,
    pub datadog_validate_host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionPair {
    pub source_chain: String,
    pub target_chain: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Chain {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PacketStatus {
    pub id: i32,
    pub source_chain_id: i32,
    pub target_chain_id: i32,
    pub sequence_number: i64,
    pub send_packet: Option<serde_json::Value>,
    pub recv_packet: Option<serde_json::Value>,
    pub write_ack: Option<serde_json::Value>,
    pub acknowledge_packet: Option<serde_json::Value>,
    pub last_update: SystemTime,
}

impl PacketStatus {
    pub fn new(source_chain_id: i32, target_chain_id: i32, sequence_number: i64) -> Self {
        Self {
            id: 0,
            source_chain_id,
            target_chain_id,
            sequence_number,
            send_packet: None,
            recv_packet: None,
            write_ack: None,
            acknowledge_packet: None,
            last_update: SystemTime::now(),
        }
    }
}
