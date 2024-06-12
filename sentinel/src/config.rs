use serde::{ Deserialize, Serialize };
use sqlx::FromRow;
use unionlabs::encoding::Proto;
use unionlabs::hash::H160;
use ucs01_relay::msg::{ ExecuteMsg, TransferMsg };
use unionlabs::cosmwasm::wasm::msg_execute_contract::MsgExecuteContract;
use unionlabs::cosmos::base::coin::Coin;
use protos::google::protobuf::Any;
use protos::ibc::applications::transfer::v1::MsgTransfer;
use unionlabs::google::protobuf::any;
use prost::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Protocol {
    Ucs01 {
        contract: String,
    },
    Ics20,
}

impl Protocol {
    pub fn transfer_message(
        &self,
        signer: &str,
        channel: &str,
        denom: &str,
        amount: &str,
        receiver: &str
    ) -> Any {
        match self {
            Protocol::Ucs01 { contract } => {
                let transfer_msg = ExecuteMsg::Transfer(TransferMsg {
                    channel: channel.to_string(),
                    receiver: receiver.to_string(),
                    memo: Default::default(),
                    timeout: None,
                });

                let transfer_msg_bytes = serde_json::to_vec(&transfer_msg).unwrap();

                any::Any(MsgExecuteContract {
                    sender: signer.to_string(),
                    contract: contract.clone(),
                    msg: transfer_msg_bytes,
                    funds: vec![Coin {
                        denom: denom.to_string(),
                        amount: amount.to_string(),
                    }],
                }).into()
            }
            Protocol::Ics20 => {
                let msg = MsgTransfer {
                    source_port: "transfer".into(),
                    source_channel: channel.to_string(),
                    token: Some(
                        (Coin {
                            denom: denom.to_string(),
                            amount: amount.parse().unwrap(),
                        }).into()
                    ),
                    sender: signer.to_string(),
                    receiver: receiver.to_string(),
                    timeout_height: None,
                    timeout_timestamp: u64::MAX / 2,
                    memo: String::new(),
                };

                Any {
                    type_url: "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
                    value: msg.encode_to_vec(),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub union: CosmosConfig<ChainConfig<chain_utils::union::Config, String>>,
    pub ethereum: ChainConfig<chain_utils::ethereum::Config, H160>,
    pub osmosis: CosmosConfig<ChainConfig<chain_utils::cosmos::Config, String>>,
    pub amount: String,
    pub db_url: String,
    pub connections: Vec<ConnectionPair>,
    // pub ethereum_priv_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CosmosConfig<T> {
    pub protocol: Protocol,
    pub chain_config: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChainConfig<T, A> {
    pub chain_config: T,
    pub address: A,
    pub channel: String,
    pub counterparty_channel: String,
}

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
    pub protocol: String,
    pub sequence_number: i64,
    pub send_packet: Option<serde_json::Value>,
    pub recv_packet: Option<serde_json::Value>,
    pub write_ack: Option<serde_json::Value>,
    pub acknowledge_packet: Option<serde_json::Value>,
    pub last_update: chrono::DateTime<chrono::Utc>,
}
impl PacketStatus {
    pub fn new(
        source_chain_name: &str,
        target_chain_name: &str,
        protocol: &str,
        sequence_number: i64
    ) -> Self {
        let source_chain_id = ChainId::from_str(source_chain_name).unwrap() as i32;
        let target_chain_id: i32 = ChainId::from_str(target_chain_name).unwrap() as i32;

        Self {
            source_chain_id,
            target_chain_id,
            protocol: protocol.to_string(),
            sequence_number,
            send_packet: None,
            recv_packet: None,
            write_ack: None,
            acknowledge_packet: None,
            last_update: chrono::Utc::now(),
        }
    }
}
