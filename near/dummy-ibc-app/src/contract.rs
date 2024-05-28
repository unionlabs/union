use ibc_vm_rs::{IbcEvent, Status};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    near_bindgen, PanicOnDefault,
};
#[allow(unused)]
use near_sdk_contract_tools::owner::OwnerExternal;
#[allow(clippy::wildcard_imports)]
use near_sdk_contract_tools::Owner;
use unionlabs::{
    events::SendPacket,
    ibc::core::{channel, client::height::Height, commitment::merkle_path::MerklePath},
    id::{ChannelId, ClientId, ConnectionId},
};

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize, Owner)]
pub struct Contract {}

#[near_bindgen]
impl Contract {
    pub fn on_channel_open_init(
        order: channel::order::Order,
        connection_hops: Vec<ConnectionId>,
        port_id: unionlabs::id::PortId,
        channel_id: ChannelId,
        counterparty: channel::counterparty::Counterparty,
        version: String,
    ) -> bool {
        false
    }

    pub fn on_channel_open_try(
        order: channel::order::Order,
        connection_hops: Vec<ConnectionId>,
        port_id: unionlabs::id::PortId,
        channel_id: ChannelId,
        counterparty: channel::counterparty::Counterparty,
        counterparty_version: String,
    ) -> bool {
        false
    }

    pub fn on_channel_open_ack(
        port_id: unionlabs::id::PortId,
        channel_id: ChannelId,
        counterparty_channel_id: String,
        counterparty_version: String,
    ) -> bool {
        false
    }

    pub fn on_channel_open_confirm(port_id: unionlabs::id::PortId, channel_id: ChannelId) -> bool {
        false
    }
}
