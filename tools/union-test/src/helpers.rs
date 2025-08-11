use unionlabs::primitives::FixedBytes;
use alloy::{sol_types::SolType, primitives::{U256, Bytes}};
use ethers::abi::{self};
type Ack = (U256, Bytes);
use voyager_sdk::{
    anyhow};
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionConfirm {
    pub connection_id: u32,
    pub counterparty_connection_id: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelOpenConfirm {
    pub channel_id: u32,
    pub counterparty_channel_id: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateClientConfirm {
    pub client_id: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PacketRecv {
    pub packet_hash: FixedBytes<32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PacketTimeout {
    pub packet_hash: FixedBytes<32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PacketAck {
    pub packet_hash: FixedBytes<32>,
    pub tag: u128
}

#[derive(Debug, Clone, PartialEq)]
pub struct PacketSend {
    pub packet_hash: FixedBytes<32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Delegate {
    pub validator: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WithdrawRewards {
    pub validator: String,
    pub amount: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateClient {
    pub height: u64,
}


// fn decode_acknowledgement(ack_bytes: &[u8]) -> anyhow::Result<(U256, Bytes)> {
//     let (tag, inner_ack) = Ack::abi_decode(ack_bytes, true)?;
//     Ok((tag, inner_ack))
// }