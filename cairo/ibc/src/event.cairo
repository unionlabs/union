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
use crate::types::{ChannelId, ClientId, ConnectionId};

#[derive(Drop, starknet::Event)]
pub struct RegisterClient {
    #[key]
    pub client_type: ByteArray,
    pub client_address: ContractAddress,
}

#[derive(Drop, starknet::Event)]
pub struct CreateClient {
    #[key]
    pub client_type: ByteArray,
    #[key]
    pub client_id: ClientId,
    #[key]
    pub counterparty_chain_id: ByteArray,
}

#[derive(Drop, starknet::Event)]
pub struct UpdateClient {
    #[key]
    pub client_id: ClientId,
    #[key]
    pub height: u64,
}

#[derive(Drop, starknet::Event)]
pub struct ConnectionOpenInit {
    #[key]
    pub connection_id: ConnectionId,
    #[key]
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
}

#[derive(Drop, starknet::Event)]
pub struct ConnectionOpenTry {
    #[key]
    pub connection_id: ConnectionId,
    #[key]
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Drop, starknet::Event)]
pub struct ConnectionOpenAck {
    #[key]
    pub connection_id: ConnectionId,
    #[key]
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Drop, starknet::Event)]
pub struct ConnectionOpenConfirm {
    #[key]
    pub connection_id: ConnectionId,
    #[key]
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Drop, starknet::Event)]
pub struct ChannelOpenInit {
    #[key]
    pub port_id: ContractAddress,
    #[key]
    pub channel_id: ChannelId,
    pub counterparty_port_id: ByteArray,
    pub connection_id: ConnectionId,
    #[key]
    pub version: ByteArray,
}

#[derive(Drop, starknet::Event)]
pub struct ChannelOpenTry {
    #[key]
    pub port_id: ContractAddress,
    #[key]
    pub channel_id: ChannelId,
    pub counterparty_port_id: ByteArray,
    pub counterparty_channel_id: ChannelId,
    pub connection_id: ConnectionId,
    #[key]
    pub counterparty_version: ByteArray,
}

#[derive(Drop, starknet::Event)]
pub struct ChannelOpenAck {
    #[key]
    pub port_id: ContractAddress,
    #[key]
    pub channel_id: ChannelId,
    pub counterparty_port_id: ByteArray,
    pub counterparty_channel_id: ChannelId,
    pub connection_id: ConnectionId,
}

#[derive(Drop, starknet::Event)]
pub struct ChannelOpenConfirm {
    #[key]
    pub port_id: ContractAddress,
    #[key]
    pub channel_id: ChannelId,
    pub counterparty_port_id: ByteArray,
    pub counterpartyChannel_id: ChannelId,
    pub connection_id: ConnectionId,
}

#[derive(Drop, starknet::Event)]
pub struct ChannelCloseInit {
    #[key]
    pub port_id: ContractAddress,
    #[key]
    pub channel_id: ChannelId,
    pub counterparty_port_id: ByteArray,
    pub counterparty_channel_id: ChannelId,
}

#[derive(Drop, starknet::Event)]
pub struct ChannelCloseConfirm {
    #[key]
    pub port_id: ContractAddress,
    #[key]
    pub channel_id: ChannelId,
    pub counterparty_port_id: ByteArray,
    pub counterparty_channel_id: ChannelId,
}

