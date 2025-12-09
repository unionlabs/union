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

use alexandria_bytes::byte_array_ext::ByteArrayTraitExt;
use core::hash::{Hash, HashStateExTrait, HashStateTrait};
use starknet::ContractAddress;
use crate::app::IIbcModule;
use crate::event::*;
use crate::lightclient::ILightClient;
use crate::path::*;
use crate::types::{Channel, ChannelId, ClientId, ConnectionId};

pub mod Error {
    pub const CLIENT_TYPE_ALREADY_REGISTERED: felt252 = 'CLIENT_TYPE_ALREADY_REGISTERED';
    pub const CLIENT_TYPE_NOT_FOUND: felt252 = 'CLIENT_TYPE_NOT_FOUND';
    pub const CLIENT_NOT_FOUND: felt252 = 'CLIENT_NOT_FOUND';
    pub const INVALID_PROOF: felt252 = 'INVALID_PROOF';
    pub const INVALID_CONNECTION_STATE: felt252 = 'INVALID_CONNECTION_STATE';
    pub const INVALID_CHANNEL_STATE: felt252 = 'INVALID_CHANNEL_STATE';
}

#[starknet::interface]
pub trait IIbcHandler<TContractState> {
    /// Register a client implementation at `client_address` with a client type. This
    /// `client_type` will be used later to call the correct client implementation.
    ///
    /// #### Params
    ///
    /// - `client_type`: The human-readable label for the client implementation. This will be used
    /// when creating a client of this type.
    /// - `client_address`: The address of the client implementation. This is expected to implement
    /// [`ILightClient`].
    ///
    /// #### Panics
    ///
    /// This function may panic with the following well-known error codes:
    ///
    /// - [`Error::CLIENT_TYPE_ALREADY_REGISTERED`]: The specified `client_type` is already
    /// registered.
    fn register_client(
        ref self: TContractState, client_type: ByteArray, client_address: ContractAddress,
    );

    /// Create a light client instance from a previously registered client type.
    ///
    /// #### Params
    ///
    /// - `client_type`: The light client implementation to create a client of. The implementation
    /// must have already been registered with [`IIbcHandler::register_client()`].
    /// - `client_state_bytes`: Arbitrary bytes to be interpreted by the light client upon client
    /// creation.
    /// - `consensus_state_bytes`: Arbitrary bytes to be interpreted by the light client upon
    /// client creation. Note that this may not be the exact state committed, since the client may
    /// return a different client state to save after creation. See [`ILightClient::create_client`]
    /// for more information.
    /// - `relayer`: An arbitrary address provided by the caller. This must not be used for any kind
    /// of authentication.
    ///
    /// The [`ClientId`] of the newly created client is returned upon success.
    ///
    /// #### Events
    ///
    /// Emits [`CreateClient`].
    ///
    /// #### Panics
    ///
    /// This function may panic with the following well-known error codes:
    ///
    /// - [`Error::CLIENT_TYPE_NOT_FOUND`]: The specified `client_type` is not registered.
    ///
    /// Additionally, the light client may panic while creating the client. In this case, this
    /// function will panic with the full panic message from the failed call.
    ///
    /// #### Commitments
    ///
    /// - Client state commitment is saved under [`ClientStatePath`].
    /// - Consensus state commitment is saved under [`ConsensusStatePath`].
    fn create_client(
        ref self: TContractState,
        client_type: ByteArray,
        client_state_bytes: ByteArray,
        consensus_state_bytes: ByteArray,
        relayer: ContractAddress,
    ) -> ClientId;

    /// Updates a light client to a new state. This state transition must be verified by the
    /// specified light client.
    ///
    /// #### Params
    ///
    /// - `client_message`: Arbitrary bytes to be interpreted by the light client. There is no
    /// assumption on the type or encoding of the client message by the core protocol.
    /// - `relayer`: An arbitrary address provided by the caller. This must not be used for any kind
    /// of authentication.
    ///
    /// #### Events
    ///
    /// Emits [`UpdateClient`].
    ///
    /// #### Panics
    ///
    /// This function may panic with the following well-known error codes:
    ///
    /// - [`Error::CLIENT_NOT_FOUND`]: Client with `client_id` is not found.
    ///
    /// Additionally, the light client may panic while updating the client. In this case, this
    /// function will panic with the full panic message from the failed call.
    ///
    /// #### Commitments
    ///
    /// - The [`ClientStatePath`] is updated with the commitment returned by the client.
    /// - The [`ConsensusStatePath`] is added or updated with the commitment returned by the
    /// client.
    fn update_client(
        ref self: TContractState,
        client_id: ClientId,
        client_message: ByteArray,
        relayer: ContractAddress,
    );

    /// Start the connection handshake.
    ///
    /// #### Params
    ///
    /// - `client_id`: The light client that will verify the counterparty chain.
    /// - `counterparty_client_id`: The light client on the counterparty chain that will verify
    /// state of this chain.
    ///
    /// The [`ConnectionId`] of the newly created connection is returned upon success.
    ///
    /// #### Events
    ///
    /// Emits [`ConnectionOpenInit`].
    ///
    /// #### Panics
    ///
    /// This function may panic with the following well-known error codes:
    ///
    /// - [`Error::CLIENT_NOT_FOUND`]: Client with `client_id` is not found.
    ///
    /// #### Commitments
    ///
    /// - The ethabi encoded and keccak hashed connection will be committed under
    /// [`ConnectionPath`].
    fn connection_open_init(
        ref self: TContractState, client_id: ClientId, counterparty_client_id: ClientId,
    ) -> ConnectionId;


    /// The second step of the connection handshake, after the `connection_open_init` on the
    /// counterparty chain.
    ///
    /// #### Params
    ///
    /// - `client_id`: The light client that will verify the counterparty chain.
    /// - `counterparty_client_id`: the light client on the counterparty chain that will verify
    /// state of this chain.
    /// - `proof_init`: The proof of the counterparty connection commitment, as stored under the
    /// [`ConnectionPath`] path in the counterparty's commitment store.
    /// - `proof_height`: The height that the `proof_init` proof is verifiable at.
    ///
    /// The [`ConnectionId`] of the newly created connection is returned upon success.
    ///
    /// #### Events
    ///
    /// Emits [`ConnectionOpenTry`].
    ///
    /// #### Panics
    ///
    /// This function may panic with the following well-known error codes:
    ///
    /// - [`Error::CLIENT_NOT_FOUND`]: The client `client_id` is not found.
    /// - [`Error::INVALID_PROOF`]: The `proof_init` cannot be verified by the light client.
    ///
    /// Additionally, the call to the light client may exit with an error that cannot be handled by
    /// the safe dispatcher. See the cairo documentation for more information on what errors can be
    /// caught.
    ///
    /// #### Commitments
    ///
    /// - The ethabi encoded and keccak hashed connection will be committed under
    /// [`ConnectionPath`].
    fn connection_open_try(
        ref self: TContractState,
        counterparty_client_id: ClientId,
        counterparty_connection_id: ConnectionId,
        client_id: ClientId,
        proof_init: ByteArray,
        proof_height: u64,
    ) -> ConnectionId;

    /// The second step of the connection handshake, after the `connection_open_try` on the
    /// counterparty chain. This is the final step of the connection handshake on this chain, and
    /// the `connection_open_confirm` must still be sent to the counterparty after this call to
    /// complete the handshake.
    ///
    /// #### Params
    ///
    /// - `connection_id`: The ID of the connection on this chain.
    /// - `counterparty_connection_id`: the ID of the connection on the counterparty chain.
    /// - `proof_try`: The proof of the counterparty connection commitment, as stored under the
    /// [`ConnectionPath`] path in the counterparty's commitment store.
    /// - `proof_height`: The height that the `proof_try` proof is verifiable at.
    ///
    /// #### Events
    ///
    /// Emits [`ConnectionOpenAck`].
    ///
    /// #### Panics
    ///
    /// This function may panic with the following well-known error codes:
    ///
    /// - [`Error::CONNECTION_NOT_FOUND`]: Connection `connection_id` is not found.
    /// - [`Error::INVALID_CONNECTION_STATE`]: Connection `connection_id` is in an invalid state.
    /// This can occur either because the `connection_open_ack/confirm` has already been run for
    /// this connection, or `connection_open_ack` is executed after `connection_open_try` but not
    /// `init`.
    /// - [`Error::INVALID_PROOF`]: The `proof_try` cannot be verified by the light client.
    ///
    /// Additionally, the call to the light client may exit with an error that cannot be handled by
    /// the safe dispatcher. See the cairo documentation for more information on what errors can be
    /// caught.
    ///
    /// #### Commitments
    ///
    /// - The ethabi encoded and keccak hashed connection will be committed under
    /// [`ConnectionPath`].
    fn connection_open_ack(
        ref self: TContractState,
        connection_id: ConnectionId,
        counterparty_connection_id: ConnectionId,
        proof_try: ByteArray,
        proof_height: u64,
    );

    /// The final step of the connection handshake, after the `connection_open_ack` on the
    /// counterparty chain.
    ///
    /// The connection `connection_id` on this chain will be fully opened.
    ///
    /// #### Params
    ///
    /// - `connection_id`: The ID of the connection on this chain.
    /// - `proof_ack`: The proof of the counterparty connection commitment, as stored under the
    /// [`ConnectionPath`] path in the counterparty's commitment store.
    /// - `proof_height`: The height that the `proof_ack` proof is verifiable at.
    ///
    /// Emits [`ConnectionOpenConfirm`]
    ///
    /// #### Panics
    ///
    /// This function may panic with the following well-known error codes:
    ///
    /// - [`Error::CONNECTION_NOT_FOUND`]: Connection `connection_id` is not found.
    /// - [`Error::INVALID_CONNECTION_STATE`]: Connection `connection_id` is in an invalid state.
    /// This can occur either because the `connection_open_ack/confirm` has already been run for
    /// this connection, or `connection_open_ack` is executed after `connection_open_try` but not
    /// `init`.
    /// - [`Error::INVALID_PROOF`]: The `proof_ack` cannot be verified by the light client.
    ///
    /// Additionally, the call to the light client may exit with an error that cannot be handled by
    /// the safe dispatcher. See the cairo documentation for more information on what errors can be
    /// caught.
    ///
    /// #### Commitments
    ///
    /// - The ethabi encoded and keccak hashed connection will be committed under
    /// [`ConnectionPath`].
    fn connection_open_confirm(
        ref self: TContractState,
        connection_id: ConnectionId,
        proof_ack: ByteArray,
        proof_height: u64,
    );

    /// Start the channel handshake.
    ///
    /// #### Params
    ///
    /// - `port_id`: The contract on this chain that implements [`IIbcModule`].
    /// - `counterparty_port_id`: The port on the counterparty chain that implements the same IBC
    /// app protocol as `port_id`. Note that this is arbitrary bytes from the perspective of this
    /// chain, since this is only needed to uniquely identify the port id on the counterparty chain.
    /// - `connection_id`: The connection on this chain to create the channel over.
    /// - `version`: The channel version for this protocol.
    /// - `relayer`: An arbitrary address provided by the caller. This must not be used for any kind
    /// of authentication.
    ///
    /// The [`ChannelId`] of the newly created channel is returned upon success.
    ///
    /// #### Events
    ///
    /// Emits [`ChannelOpenInit`].
    ///
    /// #### Panics
    ///
    /// This function will panic with a storage read error if the provided connection cannot be
    /// found.
    ///
    /// Additionally, the module may panic during the open init callback. In this case, this
    /// function will either panic with the full panic message from the failed call, or exit
    /// directly if the error cannot be caught. See the cairo documentation for more information on
    /// what errors can be caught.
    ///
    /// #### Commitments
    ///
    /// - The ethabi encoded and keccak hashed channel will be committed under [`ChannelPath`].
    fn channel_open_init(
        ref self: TContractState,
        port_id: ContractAddress,
        counterparty_port_id: ByteArray,
        connection_id: ConnectionId,
        version: ByteArray,
        relayer: ContractAddress,
    ) -> ChannelId;

    /// The second step of the channel handshake, after the `channel_open_init` on the counterpaty
    /// chain.
    ///
    /// #### Params
    ///
    /// - `port_id`: The contract on this chain that implements [`IIbcModule`].
    /// - `connection_id`: The connection on this chain to create the channel over.
    /// - `counterparty_channel_id`: The ID of the channel on the counterparty chain.
    /// - `counterparty_port_id`: The port id of the channel on the counterparty chain.
    /// - `counterparty_version`: The channel version as committed on the counterparty chain.
    /// - `proof_init`: The proof of the counterparty channel commitment, as stored under the
    /// [`ChannelPath`] path in the counterparty's commitment store.
    /// - `proof_height`: The height that the `proof_init` proof is verifiable at.
    /// - `relayer`: An arbitrary address provided by the caller. This must not be used for any kind
    /// of authentication.
    ///
    /// The [`ChannelId`] of the newly created channel is returned upon success.
    ///
    /// #### Events
    ///
    /// Emits [`ChannelOpenTry`].
    ///
    /// #### Panics
    ///
    /// This function will panic with a storage read error if the provided connection
    /// (`channel.connection_id`) cannot be found.
    ///
    /// This function may also panic with the following well-known error codes:
    ///
    /// - [`Error::INVALID_CONNECTION_STATE`]: The provided connection is not open
    /// ([`ConnectionState::Open`]).
    /// - [`Error::INVALID_PROOF`]: The `proof_init` cannot be verified by the light client.
    ///
    /// Additionally, the module may panic during the open init callback. In this case, this
    /// function will either panic with the full panic message from the failed call, or exit
    /// directly if the error cannot be caught. See the cairo documentation for more information on
    /// what errors can be caught.
    ///
    /// #### Commitments
    ///
    /// - The ethabi encoded and keccak hashed channel will be committed under [`ChannelPath`].
    fn channel_open_try(
        ref self: TContractState,
        port_id: ContractAddress,
        connection_id: ConnectionId,
        counterparty_channel_id: ChannelId,
        counterparty_port_id: ByteArray,
        counterparty_version: ByteArray,
        proof_init: ByteArray,
        proof_height: u64,
        relayer: ContractAddress,
    ) -> ChannelId;

    fn channel_open_ack(
        ref self: TContractState,
        channel_id: ChannelId,
        counterparty_version: ByteArray,
        counterparty_channel_id: ChannelId,
        proof_try: ByteArray,
        proof_height: u64,
        relayer: ContractAddress,
    );

    fn channel_open_confirm(
        ref self: TContractState,
        channel_id: ChannelId,
        proof_ack: ByteArray,
        proof_height: u64,
        relayer: ContractAddress,
    );
}

#[starknet::contract]
pub mod IbcHandler {
    use core::keccak::compute_keccak_byte_array;
    use core::num::traits::Zero;
    use starknet::event::EventEmitter;
    use starknet::storage::{
        Map, StorageMapReadAccess, StorageMapWriteAccess, StoragePointerReadAccess,
        StoragePointerWriteAccess,
    };
    use starknet::storage_access::{storage_address_from_base, storage_base_address_from_felt252};
    use starknet::syscalls::storage_write_syscall;
    use starknet::{ContractAddress, SyscallResultTrait, get_execution_info};
    use crate::app::{IIbcModuleDispatcher, IIbcModuleSafeDispatcher, IIbcModuleSafeDispatcherTrait};
    use crate::event::{
        ChannelCloseConfirm, ChannelCloseInit, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit,
        ChannelOpenTry, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
        ConnectionOpenTry, CreateClient, RegisterClient, UpdateClient,
    };
    use crate::lightclient::{
        ConsensusStateUpdate, ILightClientSafeDispatcher, ILightClientSafeDispatcherTrait,
    };
    use crate::path::{
        ChannelPath, ClientStatePath, ConnectionPath, ConsensusStatePath, StorePathKeyTrait,
    };
    use crate::types::{
        Channel, ChannelId, ChannelImpl, ChannelState, ClientId, ClientIdImpl, Connection,
        ConnectionId, ConnectionIdImpl, ConnectionImpl, ConnectionState, ConnectionTrait,
    };
    use super::{ByteArrayTraitExt, Error, to_byte_array};

    #[storage]
    struct Storage {
        commitments: Map<u256, u256>,
        client_type_registry: Map<u256, ContractAddress>,
        client_types: Map<ClientId, ByteArray>,
        client_impls: Map<ClientId, ContractAddress>,
        next_client_id: ClientId,
        next_connection_id: ConnectionId,
        next_channel_id: ChannelId,
        connections: Map<ConnectionId, Connection>,
        channels: Map<ChannelId, Channel>,
        channel_owners: Map<ChannelId, ContractAddress>,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        RegisterClient: RegisterClient,
        CreateClient: CreateClient,
        UpdateClient: UpdateClient,
        ConnectionOpenInit: ConnectionOpenInit,
        ConnectionOpenTry: ConnectionOpenTry,
        ConnectionOpenAck: ConnectionOpenAck,
        ConnectionOpenConfirm: ConnectionOpenConfirm,
        ChannelOpenInit: ChannelOpenInit,
        ChannelOpenTry: ChannelOpenTry,
        ChannelOpenAck: ChannelOpenAck,
        ChannelOpenConfirm: ChannelOpenConfirm,
        ChannelCloseInit: ChannelCloseInit,
        ChannelCloseConfirm: ChannelCloseConfirm,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        self.next_client_id.write(ClientIdImpl::new(1));
        self.next_connection_id.write(ConnectionIdImpl::new(1));
    }

    #[abi(embed_v0)]
    pub impl IbcHandlerImpl of super::IIbcHandler<ContractState> {
        fn register_client(
            ref self: ContractState, client_type: ByteArray, client_address: ContractAddress,
        ) {
            let key = compute_keccak_byte_array(@client_type);

            assert(
                self.client_type_registry.read(key).is_zero(),
                Error::CLIENT_TYPE_ALREADY_REGISTERED,
            );

            self.client_type_registry.write(key, client_address);

            self.emit(RegisterClient { client_type, client_address });
        }

        fn create_client(
            ref self: ContractState,
            client_type: ByteArray,
            client_state_bytes: ByteArray,
            consensus_state_bytes: ByteArray,
            relayer: ContractAddress,
        ) -> ClientId {
            // TODO(aeryz): check the client status and revert if its already non-active

            let client_address = self.client_type_impl(@client_type);

            let client_id = self.get_next_client_id();

            let res = ILightClientSafeDispatcher { contract_address: client_address }
                .create_client(
                    get_execution_info().caller_address,
                    client_id,
                    client_state_bytes,
                    consensus_state_bytes,
                    relayer,
                );

            match res {
                Ok((
                    ConsensusStateUpdate {
                        client_state_commitment, consensus_state_commitment, height,
                    }, counterparty_chain_id,
                )) => {
                    // Note that the light clients define how the commitment should be since the
                    // commitments are verified on the counterparty chains which might not natively
                    // support certain encoding schemes.
                    self.commit(@ClientStatePath { client_id }, client_state_commitment);
                    self
                        .commit(
                            @ConsensusStatePath { client_id, height }, consensus_state_commitment,
                        );

                    self.client_impls.write(client_id, client_address);
                    self.client_types.write(client_id, client_type.clone());

                    self.emit(CreateClient { client_type, client_id, counterparty_chain_id });

                    client_id
                },
                Err(err) => panic!("error creating client: {err:?}"),
            }
        }

        fn update_client(
            ref self: ContractState,
            client_id: ClientId,
            client_message: ByteArray,
            relayer: ContractAddress,
        ) {
            // TODO(aeryz): check the client status

            let res = self
                .client_impl(client_id)
                .update_client(
                    get_execution_info().caller_address, client_id, client_message, relayer,
                );

            match res {
                Ok(ConsensusStateUpdate {
                    client_state_commitment, consensus_state_commitment, height,
                }) => {
                    // Update or add the commitments.
                    self.commit(@ClientStatePath { client_id }, client_state_commitment);
                    self
                        .commit(
                            @ConsensusStatePath { client_id, height }, consensus_state_commitment,
                        );

                    self.emit(UpdateClient { client_id, height });
                },
                Err(err) => panic!("error updating client: {err:?}"),
            }
        }

        fn connection_open_init(
            ref self: ContractState, client_id: ClientId, counterparty_client_id: ClientId,
        ) -> ConnectionId {
            // Acts as an assertion that the client exists.
            let _ = self.client_impl(client_id);

            // We get the next connection ID, and increment it.
            let connection_id = self.get_next_connection_id();

            let connection = Connection {
                state: ConnectionState::Init,
                client_id,
                counterparty_client_id,
                // This is `None` because we don't know it yet. It will be filled by the
                // counterparty chain on `connection_open_ack`.
                counterparty_connection_id: None,
            };

            self.save_and_commit_connection(connection_id, connection);

            self.emit(ConnectionOpenInit { connection_id, client_id, counterparty_client_id });

            connection_id
        }

        fn connection_open_try(
            ref self: ContractState,
            counterparty_client_id: ClientId,
            counterparty_connection_id: ConnectionId,
            client_id: ClientId,
            proof_init: ByteArray,
            proof_height: u64,
        ) -> ConnectionId {
            // The expected state of the connection after `connection_open_init` is run on the
            // counterparty chain. That's the reason why `client_id` and `counterparty_client_id` is
            // flipped.
            let expected_connection = Connection {
                state: ConnectionState::Init,
                client_id: counterparty_client_id,
                counterparty_client_id: client_id,
                counterparty_connection_id: None,
            };

            // Verify that the counterparty chain actually performed the `connection_open_init` and
            // properly did the commitment
            assert(
                self
                    .verify_connection_state(
                        client_id,
                        proof_height,
                        proof_init,
                        counterparty_connection_id,
                        expected_connection,
                    ),
                Error::INVALID_PROOF,
            );

            let connection_id = self.get_next_connection_id();

            let connection = Connection {
                state: ConnectionState::TryOpen,
                client_id,
                counterparty_client_id,
                counterparty_connection_id: Some(counterparty_connection_id),
            };

            self.save_and_commit_connection(connection_id, connection);

            self
                .emit(
                    ConnectionOpenTry {
                        connection_id,
                        client_id,
                        counterparty_client_id,
                        counterparty_connection_id,
                    },
                );

            connection_id
        }

        fn connection_open_ack(
            ref self: ContractState,
            connection_id: ConnectionId,
            counterparty_connection_id: ConnectionId,
            proof_try: ByteArray,
            proof_height: u64,
        ) {
            // We are at the ack phase, meaning we should be at the state where only the
            // `connection_open_init` ran on this chain.
            let mut connection = self.ensure_connection_state(connection_id, ConnectionState::Init);

            // The expected state of the connection after `connection_open_try` is run on the
            // counterparty chain. That's the reason why `client_id` and `counterparty_client_id` is
            // flipped.
            let expected_connection = Connection {
                state: ConnectionState::TryOpen,
                client_id: connection.counterparty_client_id,
                counterparty_client_id: connection.client_id,
                counterparty_connection_id: Some(connection_id),
            };

            assert(
                self
                    .verify_connection_state(
                        connection.client_id,
                        proof_height,
                        proof_try,
                        counterparty_connection_id,
                        expected_connection,
                    ),
                Error::INVALID_PROOF,
            );

            // We mark this connection as open since it's the last step on this chain but it won't
            // matter until `confirm` runs on the other chain.
            connection.state = ConnectionState::Open;
            // We previously didn't have this info in the `init` phase, hence we set it now.
            connection.counterparty_connection_id = Some(counterparty_connection_id);

            self
                .emit(
                    ConnectionOpenAck {
                        connection_id,
                        client_id: connection.client_id,
                        counterparty_client_id: connection.counterparty_client_id,
                        counterparty_connection_id,
                    },
                );

            self.save_and_commit_connection(connection_id, connection);
        }

        fn connection_open_confirm(
            ref self: ContractState,
            connection_id: ConnectionId,
            proof_ack: ByteArray,
            proof_height: u64,
        ) {
            // We are at the confirm phase, meaning we should be at the state where only the
            // `connection_open_try` ran on this chain.
            let mut connection = self
                .ensure_connection_state(connection_id, ConnectionState::TryOpen);

            // The expected state of the connection after `connection_open_ack` is run on the
            // counterparty chain. That's the reason why `client_id` and `counterparty_client_id` is
            // flipped.
            let expected_connection = Connection {
                state: ConnectionState::TryOpen,
                client_id: connection.counterparty_client_id,
                counterparty_client_id: connection.client_id,
                counterparty_connection_id: Some(connection_id),
            };

            assert(
                self
                    .verify_connection_state(
                        connection.client_id,
                        proof_height,
                        proof_ack,
                        connection.counterparty_connection_id.expect('must be set'),
                        expected_connection,
                    ),
                Error::INVALID_PROOF,
            );

            connection.state = ConnectionState::Open;

            self
                .emit(
                    ConnectionOpenConfirm {
                        connection_id,
                        client_id: connection.client_id,
                        counterparty_client_id: connection.counterparty_client_id,
                        counterparty_connection_id: connection
                            .counterparty_connection_id
                            .expect('must be set'),
                    },
                );

            self.save_and_commit_connection(connection_id, connection);
        }

        fn channel_open_init(
            ref self: ContractState,
            port_id: ContractAddress,
            counterparty_port_id: ByteArray,
            connection_id: ConnectionId,
            version: ByteArray,
            relayer: ContractAddress,
        ) -> ChannelId {
            // assert that the connection exists and is open
            let _ = self.ensure_connection_state(connection_id, ConnectionState::Open);

            let channel_id = self.get_next_channel_id();

            let channel = Channel {
                state: ChannelState::Init,
                connection_id,
                counterparty_channel_id: None,
                counterparty_port_id: counterparty_port_id.clone(),
                version: version.clone(),
            };

            self.save_and_commit_channel(channel_id, channel);

            self.channel_owners.write(channel_id, port_id);

            let res = IIbcModuleSafeDispatcher { contract_address: port_id }
                .on_chan_open_init(
                    get_execution_info().caller_address,
                    connection_id,
                    channel_id,
                    version.clone(),
                    relayer,
                );

            match res {
                Ok(()) => {},
                Err(err) => panic!("error in channel open init callback: {err:?}"),
            }

            self
                .emit(
                    ChannelOpenInit {
                        port_id, channel_id, counterparty_port_id, connection_id, version,
                    },
                );

            channel_id
        }

        fn channel_open_try(
            ref self: ContractState,
            port_id: ContractAddress,
            connection_id: ConnectionId,
            counterparty_channel_id: ChannelId,
            counterparty_port_id: ByteArray,
            counterparty_version: ByteArray,
            proof_init: ByteArray,
            proof_height: u64,
            relayer: ContractAddress,
        ) -> ChannelId {
            let connection = self.ensure_connection_state(connection_id, ConnectionState::Open);

            assert(
                self
                    .verify_channel_state(
                        connection.client_id,
                        proof_height,
                        proof_init,
                        counterparty_channel_id,
                        Channel {
                            state: ChannelState::Init,
                            // connection is open, counterparty connection id will exist; qed;
                            connection_id: connection.counterparty_connection_id.unwrap(),
                            counterparty_channel_id: None,
                            counterparty_port_id: {
                                let mut bz = "";
                                bz.append_address(port_id);
                                bz
                            },
                            version: counterparty_version.clone(),
                        },
                    ),
                Error::INVALID_PROOF,
            );

            let channel_id = self.get_next_channel_id();

            let channel = Channel {
                state: ChannelState::TryOpen,
                connection_id,
                counterparty_channel_id: Some(counterparty_channel_id),
                counterparty_port_id: counterparty_port_id.clone(),
                version: counterparty_version.clone(),
            };

            self.save_and_commit_channel(channel_id, channel.clone());

            self.channel_owners.write(channel_id, port_id);

            let res = IIbcModuleSafeDispatcher { contract_address: port_id }
                .on_chan_open_try(
                    get_execution_info().caller_address,
                    channel.connection_id,
                    channel_id,
                    counterparty_channel_id,
                    channel.version,
                    counterparty_version.clone(),
                    relayer,
                );

            match res {
                Ok(()) => {},
                Err(err) => panic!("error in channel open try callback: {err:?}"),
            }

            self
                .emit(
                    ChannelOpenTry {
                        port_id,
                        channel_id,
                        counterparty_port_id: channel.counterparty_port_id,
                        counterparty_channel_id,
                        connection_id: channel.connection_id,
                        counterparty_version,
                    },
                );

            channel_id
        }

        fn channel_open_ack(
            ref self: ContractState,
            channel_id: ChannelId,
            counterparty_version: ByteArray,
            counterparty_channel_id: ChannelId,
            proof_try: ByteArray,
            proof_height: u64,
            relayer: ContractAddress,
        ) {
            let mut channel = self.ensure_channel_state(channel_id, ChannelState::Init);

            let connection = self
                .ensure_connection_state(channel.connection_id, ConnectionState::Open);

            let port_id = self.channel_owners.read(channel_id);

            assert!(!port_id.is_zero(), "channel owner is set in init; qed;");

            assert(
                self
                    .verify_channel_state(
                        connection.client_id,
                        proof_height,
                        proof_try,
                        counterparty_channel_id,
                        Channel {
                            state: ChannelState::TryOpen,
                            // connection is open, counterparty connection id will exist; qed;
                            connection_id: connection.counterparty_connection_id.unwrap(),
                            counterparty_channel_id: Some(channel_id),
                            counterparty_port_id: {
                                let mut bz = "";
                                bz.append_address(port_id);
                                bz
                            },
                            version: counterparty_version.clone(),
                        },
                    ),
                Error::INVALID_PROOF,
            );

            channel.state = ChannelState::Open;

            self.save_and_commit_channel(channel_id, channel.clone());

            let res = IIbcModuleSafeDispatcher { contract_address: port_id }
                .on_chan_open_ack(
                    get_execution_info().caller_address,
                    channel_id,
                    counterparty_channel_id,
                    counterparty_version,
                    relayer,
                );

            match res {
                Ok(()) => {},
                Err(err) => panic!("error in channel open ack callback: {err:?}"),
            }

            self
                .emit(
                    ChannelOpenAck {
                        channel_id,
                        port_id,
                        connection_id: channel.connection_id,
                        counterparty_channel_id: counterparty_channel_id,
                        counterparty_port_id: channel.counterparty_port_id,
                    },
                );
        }

        fn channel_open_confirm(
            ref self: ContractState,
            channel_id: ChannelId,
            proof_ack: ByteArray,
            proof_height: u64,
            relayer: ContractAddress,
        ) {
            let mut channel = self.ensure_channel_state(channel_id, ChannelState::TryOpen);

            let connection = self.connections.read(channel.connection_id);

            let port_id = self.channel_owners.read(channel_id);

            assert!(!port_id.is_zero(), "channel owner is set in init; qed;");

            let counterparty_channel_id = channel
                .counterparty_channel_id
                .expect(Error::INVALID_CHANNEL_STATE);

            assert(
                self
                    .verify_channel_state(
                        connection.client_id,
                        proof_height,
                        proof_ack,
                        counterparty_channel_id,
                        Channel {
                            state: ChannelState::Open,
                            // connection is open, counterparty connection id will exist; qed;
                            connection_id: connection.counterparty_connection_id.unwrap(),
                            counterparty_channel_id: Some(counterparty_channel_id),
                            counterparty_port_id: {
                                let mut bz = "";
                                bz.append_address(port_id);
                                bz
                            },
                            version: channel.version.clone(),
                        },
                    ),
                Error::INVALID_PROOF,
            );

            channel.state = ChannelState::Open;

            self.save_and_commit_channel(channel_id, channel.clone());

            let res = IIbcModuleSafeDispatcher { contract_address: port_id }
                .on_chan_open_confirm(get_execution_info().caller_address, channel_id, relayer);

            match res {
                Ok(()) => {},
                Err(err) => panic!("error in channel open confirm callback: {err:?}"),
            }

            self
                .emit(
                    ChannelOpenAck {
                        channel_id,
                        port_id,
                        connection_id: channel.connection_id,
                        counterparty_channel_id,
                        counterparty_port_id: channel.counterparty_port_id,
                    },
                );
        }
    }

    #[generate_trait]
    impl IbcHandlerUtilsImpl of IbcHandlerUtilsTrait {
        fn ensure_connection_state(
            self: @ContractState, connection_id: ConnectionId, state: ConnectionState,
        ) -> Connection {
            let connection = self.connections.read(connection_id);
            assert(connection.state == state, Error::INVALID_CONNECTION_STATE);
            connection
        }

        fn ensure_channel_state(
            self: @ContractState, channel_id: ChannelId, state: ChannelState,
        ) -> Channel {
            let channel = self.channels.read(channel_id);
            assert(channel.state == state, Error::INVALID_CHANNEL_STATE);
            channel
        }

        fn save_and_commit_connection(
            ref self: ContractState, connection_id: ConnectionId, connection: Connection,
        ) {
            self.commit(@ConnectionPath { connection_id }, connection.commit());
            self.connections.write(connection_id, connection);
        }

        fn save_and_commit_channel(
            ref self: ContractState, channel_id: ChannelId, channel: Channel,
        ) {
            self.commit(@ChannelPath { channel_id }, channel.commit());
            self.channels.write(channel_id, channel);
        }

        fn verify_connection_state(
            self: @ContractState,
            client_id: ClientId,
            height: u64,
            proof: ByteArray,
            connection_id: ConnectionId,
            counterparty_connection: Connection,
        ) -> bool {
            self
                .client_impl(client_id)
                .verify_membership(
                    client_id,
                    height,
                    proof,
                    to_byte_array(ConnectionPath { connection_id }.key()),
                    to_byte_array(counterparty_connection.commit()),
                )
                .unwrap_or(false)
        }

        fn verify_channel_state(
            self: @ContractState,
            client_id: ClientId,
            height: u64,
            proof: ByteArray,
            channel_id: ChannelId,
            counterparty_channel: Channel,
        ) -> bool {
            self
                .client_impl(client_id)
                .verify_membership(
                    client_id,
                    height,
                    proof,
                    to_byte_array(ChannelPath { channel_id }.key()),
                    to_byte_array(counterparty_channel.commit()),
                )
                .unwrap_or(false)
        }


        fn get_next_client_id(ref self: ContractState) -> ClientId {
            let client_id = self.next_client_id.read();
            self.next_client_id.write(client_id.increment());
            client_id
        }

        fn get_next_connection_id(ref self: ContractState) -> ConnectionId {
            let connection_id = self.next_connection_id.read();
            self.next_connection_id.write(connection_id.increment());
            connection_id
        }

        fn get_next_channel_id(ref self: ContractState) -> ChannelId {
            let channel_id = self.next_channel_id.read();
            self.next_channel_id.write(channel_id.increment());
            channel_id
        }

        fn client_type_impl(self: @ContractState, client_type: @ByteArray) -> ContractAddress {
            let key = compute_keccak_byte_array(client_type);
            let client_address = self.client_type_registry.read(key);

            assert(!client_address.is_zero(), Error::CLIENT_TYPE_NOT_FOUND);

            client_address
        }

        fn client_impl(self: @ContractState, client_id: ClientId) -> ILightClientSafeDispatcher {
            let contract_address = self.client_impls.read(client_id);

            assert(!contract_address.is_zero(), Error::CLIENT_NOT_FOUND);

            ILightClientSafeDispatcher { contract_address }
        }

        fn commit<T, +StorePathKeyTrait<T>>(ref self: ContractState, key: @T, value: u256) {
            // https://docs.starknet.io/learn/protocol/cryptography#starknet-keccak
            let truncate = |n: u256| -> felt252 {
                // u250(u256::MAX);
                let mask = 0x3ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_u256;
                (n & mask).try_into().expect('value is <= 250 bits')
            };

            storage_write_syscall(
                0,
                storage_address_from_base(storage_base_address_from_felt252(truncate(key.key()))),
                truncate(value),
            )
                .unwrap_syscall();
        }
    }
}

pub fn poseidon<T, +Drop<T>, +Hash<T, core::poseidon::HashState>>(t: T) -> felt252 {
    core::poseidon::PoseidonImpl::new().update_with(t).finalize()
}

pub fn to_byte_array(n: u256) -> ByteArray {
    let mut out = ByteArrayTraitExt::new_empty();
    out.append_u256(n);
    out
}
