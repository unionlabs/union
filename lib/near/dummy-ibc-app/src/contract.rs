use ibc_vm_rs::IbcVmResponse;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, ext_contract, near_bindgen, AccountId, Promise,
};
#[allow(unused)]
use near_sdk_contract_tools::owner::OwnerExternal;
#[allow(clippy::wildcard_imports)]
use near_sdk_contract_tools::Owner;
use unionlabs::{
    ibc::core::{
        channel::{self, packet::Packet},
        client::height::Height,
    },
    id::{ChannelId, ConnectionId, PortId},
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default, Owner)]
pub struct Contract {}

#[near_bindgen]
impl Contract {
    #[allow(unused)]
    pub fn on_channel_open_init(
        order: channel::order::Order,
        connection_hops: Vec<ConnectionId>,
        port_id: PortId,
        channel_id: ChannelId,
        counterparty: channel::counterparty::Counterparty,
        version: String,
    ) -> Option<String> {
        None
    }

    #[allow(unused)]
    pub fn on_channel_open_try(
        order: channel::order::Order,
        connection_hops: Vec<ConnectionId>,
        port_id: PortId,
        channel_id: ChannelId,
        counterparty: channel::counterparty::Counterparty,
        counterparty_version: String,
    ) -> Option<String> {
        None
    }

    #[allow(unused)]
    pub fn on_channel_open_ack(
        port_id: PortId,
        channel_id: ChannelId,
        counterparty_channel_id: String,
        counterparty_version: String,
    ) -> Option<String> {
        None
    }

    #[allow(unused)]
    pub fn on_channel_open_confirm(port_id: PortId, channel_id: ChannelId) -> Option<String> {
        None
    }

    #[allow(unused)]
    pub fn on_acknowledge_packet(packet: Packet, ack: Vec<u8>) -> Option<String> {
        None
    }

    #[allow(unused)]
    pub fn recv_packet(packet: Packet) -> Vec<u8> {
        env::log_str("how do we do the async ack??");
        vec![1, 2, 3]
    }

    pub fn ping(ibc_addr: AccountId, source_channel: ChannelId) -> Promise {
        ext_ibc::ext(ibc_addr)
            .send_packet(
                PortId::new(env::current_account_id().to_string()).unwrap(),
                source_channel,
                Height::new(1_000_000_000),
                0,
                b"hello world!".to_vec(),
            )
            .then(Contract::ext(env::current_account_id()).ping_callback())
    }

    #[private]
    pub fn ping_callback(&self, #[callback_unwrap] sequence: IbcVmResponse) {
        env::log_str(&format!("packet with sequence {sequence:?} is sent!!"));
    }
}

#[ext_contract(ext_ibc)]
pub trait Ibc {
    fn send_packet(
        &mut self,
        source_port: PortId,
        source_channel: ChannelId,
        timeout_height: Height,
        timeout_timestamp: u64,
        data: Vec<u8>,
    ) -> u64;
}
