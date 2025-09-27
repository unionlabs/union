use alexandria_bytes::byte_array_ext::ByteArrayTraitExt;
use core::hash::{Hash, HashStateExTrait, HashStateTrait};
use crate::msg::{MsgCreateClient, MsgRegisterClient, MsgUpdateClient};
use crate::types::ClientId;
// pub fn to_be_bytes<
//     N,
//     +Div<N>,
//     +Rem<N>,
//     +PartialOrd<N>,
//     +TryInto<N, u8>,
//     +One<N>,
//     +Zero<N>,
//     +Add<N>,
//     +Mul<N>,
//     +Pow<N, usize>[Output: N],
//     +Drop<N>,
//     +Copy<N>,
//     +BitSize<N>,
// >(
//     mut n: N,
// ) -> ByteArray {
//     let modulus = (One::<N>::one() + One::<N>::one()).pow(8);

//     let mut bz: ByteArray = "";

//     let mut len = BitSize::<N>::bits() / 8;

//     while n > Zero::<N>::zero() {
//         len -= 1;
//         let b = n % modulus;
//         n = n / modulus;
//         bz.append_byte(b.try_into().unwrap());
//     }

//     for _ in 0..len {
//         bz.append_byte(0);
//     }

//     bz.rev().into()
// }

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
    fn register_client(ref self: TContractState, msg: MsgRegisterClient);

    fn create_client(ref self: TContractState, msg: MsgCreateClient) -> ClientId;

    fn update_client(ref self: TContractState, msg: MsgUpdateClient);
}

#[starknet::contract]
mod IbcHandler {
    use core::keccak::compute_keccak_byte_array;
    use core::num::traits::{Pow, Zero};
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
        ConsensusStateUpdate, ILightClient, ILightClientDispatcher, ILightClientSafeDispatcher,
        ILightClientSafeDispatcherTrait,
    };
    use crate::msg::{
        MsgChannelOpenInit, MsgChannelOpenTry, MsgConnectionOpenAck, MsgConnectionOpenConfirm,
        MsgConnectionOpenInit, MsgConnectionOpenTry, MsgCreateClient, MsgRegisterClient,
        MsgUpdateClient,
    };
    use crate::path::{
        ChannelPath, ClientStatePath, ConnectionPath, ConsensusStatePath, StorePathKeyTrait,
    };
    use crate::types::{
        Channel, ChannelId, ChannelState, ChannelTrait, ClientId, ClientIdImpl, Connection,
        ConnectionId, ConnectionImpl, ConnectionState, ConnectionTrait,
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
        let t = ClientIdImpl::new(1);
        self.next_client_id.write(t);
    }

    #[abi(embed_v0)]
    impl IbcHandlerImpl of super::IIbcHandler<ContractState> {
        fn register_client(ref self: ContractState, msg: MsgRegisterClient) {
            let key = compute_keccak_byte_array(@msg.client_type);

            assert(
                self.client_type_registry.read(key).is_zero(),
                Error::CLIENT_TYPE_ALREADY_REGISTERED,
            );

            self.client_type_registry.write(key, msg.client_address);

            self
                .emit(
                    RegisterClient {
                        client_type: msg.client_type, client_address: msg.client_address,
                    },
                );
        }

        fn create_client(ref self: ContractState, msg: MsgCreateClient) -> ClientId {
            let client_address = self.client_type_impl(@msg.client_type);

            let client_id = self.get_next_client_id();

            #[feature("safe_dispatcher")]
            let res = ILightClientSafeDispatcher { contract_address: client_address }
                .create_client(
                    get_execution_info().caller_address,
                    client_id,
                    msg.client_state_bytes,
                    msg.consensus_state_bytes,
                    msg.relayer,
                );

            match res {
                Ok((
                    ConsensusStateUpdate {
                        client_state_commitment, consensus_state_commitment, height,
                    }, counterparty_chain_id,
                )) => {
                    self.commit(@ClientStatePath { client_id }, client_state_commitment);
                    self
                        .commit(
                            @ConsensusStatePath { client_id, height }, consensus_state_commitment,
                        );

                    self.client_impls.write(client_id, client_address);
                    self.client_types.write(client_id, msg.client_type.clone());

                    self
                        .emit(
                            CreateClient {
                                client_type: msg.client_type, client_id, counterparty_chain_id,
                            },
                        );

                    client_id
                },
                Err(err) => { panic!("error when creating client: {err:?}"); },
            }
        }

        fn update_client(ref self: ContractState, msg: MsgUpdateClient) {
            #[feature("safe_dispatcher")]
            let res = self
                .client_impl(msg.client_id)
                .update_client(
                    get_execution_info().caller_address,
                    msg.client_id,
                    msg.client_message,
                    msg.relayer,
                );

            match res {
                Ok(ConsensusStateUpdate {
                    client_state_commitment, consensus_state_commitment, height,
                }) => {
                    self
                        .commit(
                            @ClientStatePath { client_id: msg.client_id }, client_state_commitment,
                        );
                    self
                        .commit(
                            @ConsensusStatePath { client_id: msg.client_id, height },
                            consensus_state_commitment,
                        );

                    self.emit(UpdateClient { client_id: msg.client_id, height });
                },
                Err(err) => { panic!("error when creating client: {err:?}"); },
            }
        }
    }

    #[generate_trait]
    impl IbcHandlerUtilsImpl of IbcHandlerUtilsTrait {
        fn connection_open_init(
            ref self: ContractState, msg: MsgConnectionOpenInit,
        ) -> ConnectionId {
            let connection_id = self.get_next_connection_id();

            let connection = Connection {
                state: ConnectionState::Init,
                client_id: msg.client_id,
                counterparty_client_id: msg.counterparty_client_id,
                counterparty_connection_id: None,
            };

            self.commit(@ConnectionPath { connection_id }, connection.commit());

            self
                .emit(
                    ConnectionOpenInit {
                        connection_id,
                        client_id: msg.client_id,
                        counterparty_client_id: msg.counterparty_client_id,
                    },
                );

            connection_id
        }

        fn connection_open_try(ref self: ContractState, msg: MsgConnectionOpenTry) -> ConnectionId {
            let expected_connection = Connection {
                state: ConnectionState::Init,
                client_id: msg.counterparty_client_id,
                counterparty_client_id: msg.client_id,
                counterparty_connection_id: None,
            };

            assert(
                self
                    .verify_connection_state(
                        msg.client_id,
                        msg.proof_height,
                        msg.proof_init,
                        msg.counterparty_connection_id,
                        expected_connection,
                    ),
                Error::INVALID_PROOF,
            );

            let connection_id = self.get_next_connection_id();

            let connection = Connection {
                state: ConnectionState::TryOpen,
                client_id: msg.client_id,
                counterparty_client_id: msg.counterparty_client_id,
                counterparty_connection_id: Some(msg.counterparty_connection_id),
            };

            self.save_connection(connection_id, connection);

            self
                .emit(
                    ConnectionOpenTry {
                        connection_id,
                        client_id: msg.client_id,
                        counterparty_client_id: msg.counterparty_client_id,
                        counterparty_connection_id: msg.counterparty_connection_id,
                    },
                );

            connection_id
        }

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

            self.save_connection(msg.connection_id, connection);
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

            self.save_connection(msg.connection_id, connection);
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

        fn save_connection(
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
                (n ^ mask).try_into().expect('value is <= 250 bits')
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
