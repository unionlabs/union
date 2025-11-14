use core::hash::{HashStateExTrait, HashStateTrait};
use crate::msg::{MsgCreateClient, MsgRegisterClient, MsgUpdateClient};
use crate::types::ClientId;

pub mod event;
pub mod lightclient;
pub mod msg;
#[feature(
    "deprecated-new_empty",
)] // they deprecated the Bytes type in favour of ByteArray but their own libraries are not compatible with ByteArray
pub mod path;
pub mod types;

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
    use core::num::traits::Zero;
    use starknet::event::EventEmitter;
    use starknet::storage::{
        Map, StorageMapReadAccess, StorageMapWriteAccess, StoragePointerReadAccess,
        StoragePointerWriteAccess,
    };
    use starknet::storage_access::{storage_address_from_base, storage_base_address_from_felt252};
    use starknet::syscalls::storage_write_syscall;
    use starknet::{ContractAddress, SyscallResultTrait, get_execution_info};
    use crate::event::{CreateClient, RegisterClient, UpdateClient};
    use crate::lightclient::{
        ConsensusStateUpdate, ILightClientSafeDispatcher, ILightClientSafeDispatcherTrait,
    };
    use crate::path::{ClientStatePath, ConsensusStatePath, StorePathKeyTrait};
    use crate::types::{ClientId, ClientIdImpl};
    use crate::{Error, MsgCreateClient, MsgRegisterClient, MsgUpdateClient, poseidon};

    #[storage]
    struct Storage {
        commitments: Map<u256, u256>,
        client_type_registry: Map<u256, ContractAddress>,
        client_types: Map<ClientId, ByteArray>,
        client_impls: Map<ClientId, ContractAddress>,
        next_client_id: ClientId,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        RegisterClient: RegisterClient,
        CreateClient: CreateClient,
        UpdateClient: UpdateClient,
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

            let client_id = self.next_client_id.read().increment();
            self.next_client_id.write(client_id);

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
            let client_address = self.client_impl(msg.client_id);

            #[feature("safe_dispatcher")]
            let res = ILightClientSafeDispatcher { contract_address: client_address }
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
        fn client_type_impl(self: @ContractState, client_type: @ByteArray) -> ContractAddress {
            let key = compute_keccak_byte_array(client_type);
            let client_address = self.client_type_registry.read(key);

            assert(!client_address.is_zero(), Error::CLIENT_TYPE_NOT_FOUND);

            client_address
        }

        fn client_impl(self: @ContractState, client_id: ClientId) -> ContractAddress {
            let client_address = self.client_impls.read(client_id);

            assert(!client_address.is_zero(), Error::CLIENT_NOT_FOUND);

            client_address
        }

        fn commit<T, +StorePathKeyTrait<T>>(ref self: ContractState, key: @T, value: u256) {
            storage_write_syscall(
                0,
                storage_address_from_base(
                    storage_base_address_from_felt252(
                        poseidon(key.key()),
                    ) // REVIEW: This wraps if it doesn't fit, is that behaviour ok?
                ),
                poseidon(value),
            )
                .unwrap_syscall();
        }
    }
}

pub fn poseidon(n: u256) -> felt252 {
    core::poseidon::PoseidonImpl::new().update_with(n).finalize()
}
