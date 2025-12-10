// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.
//
// Parameters
//
// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's cairo subdirectory
//                       The Licensed Work is (c) 2025 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
//
//
// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.
//
// Notice
//
// Business Source License 1.1
//
// Terms
//
// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.
//
// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.
//
// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.
//
// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.
//
// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.
//
// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.
//
// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).
//
// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

use starknet::ContractAddress;
use crate::types::{ChannelId, ConnectionId, Packet};

#[starknet::interface]
pub trait IIbcModuleSend<TContractState> {
    fn send_packet(
        ref self: TContractState,
        channel_id: ChannelId,
        timeout_height: u64,
        timeout_timestamp: u64,
        data: ByteArray,
    ) -> Result<Packet, ()>;
}

#[starknet::interface]
pub trait IIbcModule<TContractState> {
    fn on_recv_packet(
        ref self: TContractState,
        caller: ContractAddress,
        packet: Packet,
        relayer: ContractAddress,
        relayer_msg: ByteArray,
    ) -> Option<ByteArray>;

    fn on_recv_intent_packet(
        ref self: TContractState,
        caller: ContractAddress,
        packet: Packet,
        market_maker: ContractAddress,
        market_maker_msg: ByteArray,
    ) -> ByteArray;

    fn on_chan_open_init(
        ref self: TContractState,
        caller: ContractAddress,
        connection_id: ConnectionId,
        channel_id: ChannelId,
        version: ByteArray,
        relayer: ContractAddress,
    );

    fn on_chan_open_try(
        ref self: TContractState,
        caller: ContractAddress,
        connection_id: ConnectionId,
        channel_id: ChannelId,
        counterparty_channel_id: ChannelId,
        version: ByteArray,
        counterparty_version: ByteArray,
        relayer: ContractAddress,
    );

    fn on_chan_open_ack(
        ref self: TContractState,
        caller: ContractAddress,
        channel_id: ChannelId,
        counterparty_channel_id: ChannelId,
        counterparty_version: ByteArray,
        relayer: ContractAddress,
    );

    fn on_chan_open_confirm(
        ref self: TContractState,
        caller: ContractAddress,
        channel_id: ChannelId,
        relayer: ContractAddress,
    );

    fn on_chan_close_init(
        ref self: TContractState,
        caller: ContractAddress,
        channel_id: ChannelId,
        relayer: ContractAddress,
    );

    fn on_chan_close_confirm(
        ref self: TContractState,
        caller: ContractAddress,
        channel_id: ChannelId,
        relayer: ContractAddress,
    );

    fn on_acknowledge_packet(
        ref self: TContractState,
        caller: ContractAddress,
        packet: Packet,
        acknowledgement: ByteArray,
        relayer: ContractAddress,
    );

    fn on_timeout_packet(
        ref self: TContractState, caller: ContractAddress, packet: Packet, relayer: ContractAddress,
    );
}
