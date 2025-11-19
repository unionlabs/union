use starknet::ContractAddress;
use crate::types::{ChannelId, ConnectionId, Packet};


#[starknet::interface]
pub trait IIbcModuleRecv<TContractState> {
    fn on_recv_packet(
        ref self: TContractState,
        caller: ContractAddress,
        packet: Packet,
        relayer: ContractAddress,
        relayer_msg: ByteArray,
    ) -> ByteArray;

    fn on_recv_intent_packet(
        ref self: TContractState,
        caller: ContractAddress,
        packet: Packet,
        market_maker: ContractAddress,
        market_maker_msg: ByteArray,
    ) -> ByteArray;
}

#[starknet::interface]
pub trait IIbcModule<TContractState> {
    fn on_chan_open_init(
        ref self: TContractState,
        caller: ContractAddress,
        connection_id: ConnectionId,
        channel_id: ChannelId,
        version: ByteArray,
        relayer: ContractAddress,
    ) -> ByteArray;

    fn on_chan_open_try(
        ref self: TContractState,
        caller: ContractAddress,
        connection_id: ConnectionId,
        channel_id: ChannelId,
        counterparty_channel_id: ChannelId,
        version: ByteArray,
        counterparty_version: ByteArray,
        relayer: ContractAddress,
    ) -> ByteArray;

    fn on_chan_open_ack(
        ref self: TContractState,
        caller: ContractAddress,
        channel_id: ChannelId,
        counterparty_channel_id: ChannelId,
        counterparty_version: ByteArray,
        relayer: ContractAddress,
    ) -> ByteArray;

    fn on_chan_open_confirm(
        ref self: TContractState,
        caller: ContractAddress,
        channel_id: ChannelId,
        relayer: ContractAddress,
    ) -> ByteArray;

    fn on_chan_close_init(
        ref self: TContractState,
        caller: ContractAddress,
        channel_id: ChannelId,
        relayer: ContractAddress,
    ) -> ByteArray;

    fn on_chan_close_confirm(
        ref self: TContractState,
        caller: ContractAddress,
        channel_id: ChannelId,
        relayer: ContractAddress,
    ) -> ByteArray;

    fn on_acknowledge_packet(
        ref self: TContractState,
        caller: ContractAddress,
        packet: Packet,
        acknowledgement: ByteArray,
        relayer: ContractAddress,
    ) -> ByteArray;

    fn on_timeout_packet(
        ref self: TContractState, caller: ContractAddress, packet: Packet, relayer: ContractAddress,
    ) -> ByteArray;
}
