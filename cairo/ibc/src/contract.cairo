// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's cairo subdirectory
//                       The Licensed Work is (c) 2025 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
//

// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.

// Notice

// Business Source License 1.1

// Terms

// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.

// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.

// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.

// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.

// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.

// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.

// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).

// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

use alexandria_bytes::byte_array_ext::ByteArrayTraitExt;
use core::hash::{Hash, HashStateExTrait, HashStateTrait};
use starknet::ContractAddress;
use crate::msg::{MsgCreateClient, MsgRegisterClient, MsgUpdateClient};
use crate::types::{ClientId, ConnectionId};

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
    /// ## Panics
    ///
    /// This function will panic if the `client_type` is already registered
    /// (CLIENT_TYPE_ALREADY_REGISTERED).
    fn register_client(
        ref self: TContractState, client_type: ByteArray, client_address: ContractAddress,
    );

    /// Create a light client instance
    ///
    /// The light client must be registered with `client_type` before. The `client_state_bytes`
    /// and the `consensus_state_bytes` will be provided as the initial state to the client. But
    /// the client is free to return a different data to be saved. `relayer` IS PROVIDED BY THE
    /// USER, hence DO NOT USE it for authentication.
    ///
    /// Returns the ID of the client.
    ///
    /// Emits [`CreateClient`].
    ///
    /// ## Panics
    /// This function will panic if:
    /// 1. the `client_type` is not registered (CLIENT_TYPE_NOT_FOUND),
    /// 2. the light client returns an error (custom error)
    ///
    /// ## Commitments
    /// 1. Client state commitment is written under `ClientStatePath`
    /// 2. Consensus state commitment is written under `ConsensusStatePath`
    fn create_client(
        ref self: TContractState,
        client_type: ByteArray,
        client_state_bytes: ByteArray,
        consensus_state_bytes: ByteArray,
        relayer: ContractAddress,
    ) -> ClientId;

    /// Updates a light client to a new state. This state transition MUST be verified by the
    /// light client.
    ///
    /// The light client has the full control over the `client_message`. There is no assumption
    /// over the encoding by the core protocol. `relayer` IS PROVIDED BY THE USER, hence DO NOT
    /// USE it for authentication.
    ///
    /// Emits [`UpdateClient`].
    ///
    /// ## Panics
    /// 1. Client with `client_id` is not found. (CLIENT_NOT_FOUND)
    /// 2. The light client returns an error. (custom error)
    ///
    /// ## Commitments
    /// 1. Client state commitment is updated with the commitment returned by the client.
    /// 2. Consensus state commitent is added or updated with the commitment returned by the
    /// client.
    fn update_client(
        ref self: TContractState,
        client_id: ClientId,
        client_message: ByteArray,
        relayer: ContractAddress,
    );

    /// Starts the connection handshake.
    ///
    /// `client_id` will be the verifier of the packets on this
    /// chain using this connection, and the `counterparty_client_id` is the same for the
    /// counterparty chain.
    ///
    /// Returns the ID of the connection.
    ///
    /// Emits [`ConnectionOpenInit`]
    ///
    /// ## Panics
    /// 1. Client with `client_id` is not found. (CLIENT_NOT_FOUND)
    ///
    /// ## Commitments
    /// 1. The ethabi encoded and keccak hashed connection will be committed under `ConnectionPath`
    fn connection_open_init(
        ref self: TContractState, client_id: ClientId, counterparty_client_id: ClientId,
    ) -> ConnectionId;


    /// Second step of the connection handshake meant to run after the `connection_open_init` runs
    /// on the counterparty chain.
    ///
    /// `client_id` will be the verifier of the packets on this
    /// chain using this connection, and the `counterparty_client_id` is the same for the
    /// counterparty chain.
    ///
    /// The `proof_init` is the proof of the `Connection` commitment, created at height
    /// `proof_height`. The commitment is expected to be done under `ConnectionPath`.
    ///
    /// Returns the ID of the connection.
    ///
    /// Emits [`ConnectionOpenTry`]
    ///
    /// ## Panics
    /// 1. Client with `client_id` is not found. (CLIENT_NOT_FOUND)
    /// 2. The `proof_init` cannot be verified by the light client. (INVALID_PROOF)
    ///
    /// ## Commitments
    /// 1. The ethabi encoded and keccak hashed connection will be committed under `ConnectionPath`
    fn connection_open_try(
        ref self: TContractState,
        counterparty_client_id: ClientId,
        counterparty_connection_id: ConnectionId,
        client_id: ClientId,
        proof_init: ByteArray,
        proof_height: u64,
    ) -> ConnectionId;
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
    use crate::event::{
        ChannelCloseConfirm, ChannelCloseInit, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit,
        ChannelOpenTry, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
        ConnectionOpenTry, CreateClient, RegisterClient, UpdateClient,
    };
    use crate::lightclient::{
        ConsensusStateUpdate, ILightClientSafeDispatcher, ILightClientSafeDispatcherTrait,
    };
    use crate::msg::{
        MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit, MsgConnectionOpenTry,
        MsgCreateClient, MsgRegisterClient, MsgUpdateClient,
    };
    use crate::path::{ClientStatePath, ConnectionPath, ConsensusStatePath, StorePathKeyTrait};
    use crate::types::{
        Channel, ChannelId, ClientId, ClientIdImpl, Connection, ConnectionId, ConnectionIdImpl,
        ConnectionImpl, ConnectionState, ConnectionTrait,
    };
    use super::{Error, to_byte_array};

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

            // TODO(aeryz): this code depends on this feature, we should remove this? (cc: @bonlulu)
            #[feature("safe_dispatcher")]
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
                Err(err) => { panic!("error when creating client: {err:?}"); },
            }
        }

        fn update_client(
            ref self: ContractState,
            client_id: ClientId,
            client_message: ByteArray,
            relayer: ContractAddress,
        ) {
            // TODO(aeryz): check the client status

            // TODO(aeryz): this code depends on this feature, we should remove this? (cc: @bonlulu)
            #[feature("safe_dispatcher")]
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
                Err(err) => { panic!("error when updating client: {err:?}"); },
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
    }

    #[generate_trait]
    impl IbcHandlerUtilsImpl of IbcHandlerUtilsTrait {
        fn connection_open_ack(ref self: ContractState, msg: MsgConnectionOpenAck) {
            let mut connection = self
                .ensure_connection_state(msg.connection_id, ConnectionState::Init);

            let expected_connection = Connection {
                state: ConnectionState::Init,
                client_id: connection.counterparty_client_id,
                counterparty_client_id: connection.client_id,
                counterparty_connection_id: Some(msg.connection_id),
            };

            assert(
                self
                    .verify_connection_state(
                        connection.client_id,
                        msg.proof_height,
                        msg.proof_try,
                        msg.counterparty_connection_id,
                        expected_connection,
                    ),
                Error::INVALID_PROOF,
            );

            connection.state = ConnectionState::Open;

            self
                .emit(
                    ConnectionOpenAck {
                        connection_id: msg.connection_id,
                        client_id: connection.client_id,
                        counterparty_client_id: connection.counterparty_client_id,
                        counterparty_connection_id: msg.counterparty_connection_id,
                    },
                );

            self.save_and_commit_connection(msg.connection_id, connection);
        }

        fn connection_open_confirm(ref self: ContractState, msg: MsgConnectionOpenConfirm) {
            let mut connection = self
                .ensure_connection_state(msg.connection_id, ConnectionState::TryOpen);

            let expected_connection = Connection {
                state: ConnectionState::TryOpen,
                client_id: connection.counterparty_client_id,
                counterparty_client_id: connection.client_id,
                counterparty_connection_id: Some(msg.connection_id),
            };

            assert(
                self
                    .verify_connection_state(
                        connection.client_id,
                        msg.proof_height,
                        msg.proof_ack,
                        connection.counterparty_connection_id.expect('must be set'),
                        expected_connection,
                    ),
                Error::INVALID_PROOF,
            );

            connection.state = ConnectionState::Open;

            self
                .emit(
                    ConnectionOpenConfirm {
                        connection_id: msg.connection_id,
                        client_id: connection.client_id,
                        counterparty_client_id: connection.counterparty_client_id,
                        counterparty_connection_id: connection
                            .counterparty_connection_id
                            .expect('must be set'),
                    },
                );

            self.save_and_commit_connection(msg.connection_id, connection);
        }

        // fn channel_open_init(ref self: ContractState, msg: MsgChannelOpenInit) -> ChannelId {
        //     let channel_id = self.get_next_channel_id();

        //     let channel = Channel {
        //         state: ChannelState::Init,
        //         connection_id: msg.connection_id,
        //         counterparty_channel_id: None,
        //         counterparty_port_id: msg.counterparty_port_id.clone(),
        //         version: msg.version.clone(),
        //     };

        //     self.commit(@ChannelPath { channel_id }, channel.commit());

        //     self
        //         .emit(
        //             ChannelOpenInit {
        //                 port_id: msg.port_id,
        //                 channel_id,
        //                 counterparty_port_id: msg.counterparty_port_id,
        //                 connection_id: msg.connection_id,
        //                 version: msg.version,
        //             },
        //         );

        //     channel_id
        // }

        // fn channel_open_try(ref self: ContractState, msg: MsgChannelOpenTry) -> ChannelId {
        //     let expected_Channel = Channel {
        //         state: ChannelState::Init,
        //         connection_id: msg.,
        //         counterparty_channel_id: (),
        //         counterparty_port_id: (),
        //         version: (),
        //     };

        //     assert(
        //         self
        //             .verify_channel_state(
        //                 msg.client_id,
        //                 msg.proof_height,
        //                 msg.proof_init,
        //                 msg.counterparty_channel_id,
        //                 expected_Channel,
        //             ),
        //         Error::INVALID_PROOF,
        //     );

        //     let channel_id = self.get_next_channel_id();

        //     let Channel = Channel {
        //         state: ChannelState::TryOpen,
        //         client_id: msg.client_id,
        //         counterparty_client_id: msg.counterparty_client_id,
        //         counterparty_channel_id: Some(msg.counterparty_channel_id),
        //     };

        //     self.save_Channel(channel_id, Channel);

        //     self
        //         .emit(
        //             ChannelOpenTry {
        //                 channel_id,
        //                 client_id: msg.client_id,
        //                 counterparty_client_id: msg.counterparty_client_id,
        //                 counterparty_channel_id: msg.counterparty_channel_id,
        //             },
        //         );

        //     channel_id
        // }

        // fn channel_open_ack(ref self: ContractState, msg: MsgChannelOpenAck) {
        //     let mut Channel = self.ensure_channel_state(msg.channel_id, ChannelState::Init);

        //     let expected_Channel = Channel {
        //         state: ChannelState::Init,
        //         client_id: Channel.counterparty_client_id,
        //         counterparty_client_id: Channel.client_id,
        //         counterparty_channel_id: Some(msg.channel_id),
        //     };

        //     assert(
        //         self
        //             .verify_channel_state(
        //                 Channel.client_id,
        //                 msg.proof_height,
        //                 msg.proof_try,
        //                 msg.counterparty_channel_id,
        //                 expected_Channel,
        //             ),
        //         Error::INVALID_PROOF,
        //     );

        //     Channel.state = ChannelState::Open;

        //     self
        //         .emit(
        //             ChannelOpenAck {
        //                 channel_id: msg.channel_id,
        //                 client_id: Channel.client_id,
        //                 counterparty_client_id: Channel.counterparty_client_id,
        //                 counterparty_channel_id: msg.counterparty_channel_id,
        //             },
        //         );

        //     self.save_Channel(msg.channel_id, Channel);
        // }

        // fn channel_open_confirm(ref self: ContractState, msg: MsgChannelOpenConfirm) {
        //     let mut Channel = self.ensure_channel_state(msg.channel_id, ChannelState::TryOpen);

        //     let expected_Channel = Channel {
        //         state: ChannelState::TryOpen,
        //         client_id: Channel.counterparty_client_id,
        //         counterparty_client_id: Channel.client_id,
        //         counterparty_channel_id: Some(msg.channel_id),
        //     };

        //     assert(
        //         self
        //             .verify_channel_state(
        //                 Channel.client_id,
        //                 msg.proof_height,
        //                 msg.proof_ack,
        //                 Channel.counterparty_channel_id.expect('must be set'),
        //                 expected_Channel,
        //             ),
        //         Error::INVALID_PROOF,
        //     );

        //     Channel.state = ChannelState::Open;

        //     self
        //         .emit(
        //             ChannelOpenConfirm {
        //                 channel_id: msg.channel_id,
        //                 client_id: Channel.client_id,
        //                 counterparty_client_id: Channel.counterparty_client_id,
        //                 counterparty_channel_id: Channel
        //                     .counterparty_channel_id
        //                     .expect('must be set'),
        //             },
        //         );

        //     self.save_Channel(msg.channel_id, Channel);
        // }

        fn ensure_connection_state(
            self: @ContractState, connection_id: ConnectionId, state: ConnectionState,
        ) -> Connection {
            let connection = self.connections.read(connection_id);
            assert(connection.state != state, Error::INVALID_CONNECTION_STATE);
            connection
        }

        fn save_and_commit_connection(
            ref self: ContractState, connection_id: ConnectionId, connection: Connection,
        ) {
            self.commit(@ConnectionPath { connection_id }, connection.commit());
            self.connections.write(connection_id, connection);
        }

        fn verify_connection_state(
            self: @ContractState,
            client_id: ClientId,
            height: u64,
            proof: ByteArray,
            connection_id: ConnectionId,
            counterparty_connection: Connection,
        ) -> bool {
            #[feature("safe_dispatcher")]
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
