use alexandria_bytes::byte_array_ext::ByteArrayTraitExt;
use alexandria_math::opt_math::OptBitShift;
use core::keccak::compute_keccak_byte_array;
use ibc::contract::IbcHandler::Event;
use ibc::contract::{IIbcHandlerDispatcher, IIbcHandlerDispatcherTrait};
use ibc::event::*;
use ibc::path::{ClientStatePath, ConnectionPath, ConsensusStatePath, StorePathKeyTrait};
use ibc::types::{ClientIdImpl, Connection, ConnectionIdImpl, ConnectionState, ConnectionTrait};
use snforge_std::{
    ContractClassTrait, DeclareResultTrait, EventSpyAssertionsTrait, declare, load,
    map_entry_address, spy_events,
};
use starknet::storage_access::{storage_address_from_base, storage_base_address_from_felt252};
use starknet::{ContractAddress, SyscallResultTrait};

fn deploy_contract(name: ByteArray) -> ContractAddress {
    let contract = declare(name).unwrap_syscall().contract_class();
    let (contract_address, _) = contract.deploy(Default::default()).unwrap_syscall();
    contract_address
}

fn deploy_ibc_and_client() -> (IIbcHandlerDispatcher, ContractAddress, ContractAddress) {
    let ibc_contract = deploy_contract("IbcHandler");
    let light_client = deploy_contract("MockClient");

    let ibc_dispatcher = IIbcHandlerDispatcher { contract_address: ibc_contract };

    (ibc_dispatcher, ibc_contract, light_client)
}

/// Load a value from `Map<K, ByteArray>`.
///
/// **Important Note:** This function only supports `ByteArray` of length up to 31 * 256.
///
/// # Examples
/// ```
/// // for the following contract:
/// mod Contract {
///     #[storage]
///     struct Storage {
///         client_types: Map<ClientId, ByteArray>
///     }
/// }
///
/// // to get the value of `ClientId(1)`, do:
/// let value = load_byte_array_map_value(deployed_contract_address, selector!("client_types"),
/// ClientId(1)).unwrap();
/// ```
fn load_byte_array_map_value<K, +Serde<K>, +Drop<K>>(
    contract_address: ContractAddress, map_selector: felt252, key: K,
) -> Option<ByteArray> {
    let mut serialized_key = Default::default();
    key.serialize(ref serialized_key);

    // Compute the base address of the `ByteArray`. This only contains the size of the value.
    let key = map_entry_address(map_selector, serialized_key.span());

    // Parse the size of the `ByteArray` value.
    let mut size: usize = (*load(contract_address, key, 1).span()[0]).try_into().unwrap();

    assert!(size != 0);

    // The `ByteArray` is split into (31 * 256) bytes chunks and every chunk is written to a
    // different storage address as chunks of bytes31's. This function only supports array length up
    // to 31 * 256 to deal with only a single chunk of bytes31's. Hence, we compute the address by
    // using `0` as the chunk index.
    let (chunk_base, _, _) = core::poseidon::hades_permutation(key, 0, 'ByteArray');

    // We read `size / 31` + 1 subchunks from the memory because our `ByteArray` is splitted into
    // multiple `bytes31's`.
    let mut chunks = load(contract_address, chunk_base, (size / 31).into() + 1).span();

    let mut chunks_size = chunks.len();
    let mut out: ByteArray = Default::default();
    for i in 0..chunks_size {
        // All the chunks will have the size 31 except the latest chunk which might be less
        let size: u8 = if i == chunks_size - 1 {
            (size % 31).try_into().unwrap()
        } else {
            31
        };

        if size == 31 {
            // If the size is already 31, we already have a builtin function for this
            out.append_felt252(*chunks[i]);
        } else {
            // Else, we do it one by one. Note that we could have done `append_u128`, then
            // `append_u64` and so on, but let's not complicate things for a test helper.
            let chunk: u256 = (*chunks[i]).into();

            // The bytes are encoded in big endian format, hence we start from parsing the bytes
            // from the higher bytes
            let mut and_val: u256 = OptBitShift::shl(0xFF, (size - 1) * 8);
            let mut shr_val: u8 = size * 8 - 8;
            for _ in 0..size {
                // Parse the bytes one by one, by 'and'ing, we isolite the byte we want to parse as
                // u8
                let shifted = OptBitShift::shr(chunk & and_val, shr_val);
                out.append_u8(shifted.try_into().unwrap());

                and_val = OptBitShift::shr(and_val, 8);
                if shr_val != 0 {
                    shr_val -= 8;
                }
            }
        }
    }

    Some(out)
}

fn load_map_value<K, +Serde<K>, +Drop<K>, V, +Serde<V>>(
    contract_address: ContractAddress, map_selector: felt252, key: K, size: felt252,
) -> Option<V> {
    let mut serialized_key = Default::default();
    key.serialize(ref serialized_key);
    let key = map_entry_address(map_selector, serialized_key.span());
    let mut out = load(contract_address, key, size).span();
    Serde::deserialize(ref out)
}

fn load_map_value_custom<
    K,
    +Serde<K>,
    +Drop<K>,
    V,
    +Serde<V>,
    F,
    +Drop<F>,
    impl func: core::ops::Fn<F, (Array<felt252>,)>[Output: Option<V>],
>(
    contract_address: ContractAddress, map_selector: felt252, key: K, size: felt252, decode_fn: F,
) -> Option<V> {
    let mut serialized_key = Default::default();
    key.serialize(ref serialized_key);
    let key = map_entry_address(map_selector, serialized_key.span());
    let out = load(contract_address, key, size);

    decode_fn(out)
}

#[inline]
fn truncate(n: u256) -> felt252 {
    // https://docs.starknet.io/learn/protocol/cryptography#starknet-keccak
    // u250(u256::MAX);
    let mask = 0x3ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_u256;
    (n & mask).try_into().expect('value is <= 250 bits')
}

fn load_commitment<T, +StorePathKeyTrait<T>, +Drop<T>>(
    contract_address: ContractAddress, commitment_key: T,
) -> Option<felt252> {
    let mut out = load(
        contract_address,
        storage_address_from_base(storage_base_address_from_felt252(truncate(commitment_key.key())))
            .into(),
        1,
    )
        .span();

    Serde::deserialize(ref out)
}

mod register_client {
    use super::*;

    #[test]
    fn test_register_client_works() {
        let (ibc_dispatcher, ibc_contract, light_client) = deploy_ibc_and_client();

        let mut spy = spy_events();

        ibc_dispatcher.register_client("cometbls", light_client);

        let client_address = load_map_value(
            ibc_contract,
            selector!("client_type_registry"),
            compute_keccak_byte_array(@"cometbls"),
            1,
        )
            .unwrap();

        assert!(light_client == client_address);

        spy
            .assert_emitted(
                @array![
                    (
                        ibc_contract,
                        Event::RegisterClient(
                            RegisterClient {
                                client_type: "cometbls", client_address: light_client,
                            },
                        ),
                    ),
                ],
            );
    }

    #[test]
    #[should_panic(expected: 'CLIENT_TYPE_ALREADY_REGISTERED')]
    fn test_register_client_fails_client_type_already_registered() {
        let (ibc_dispatcher, _, light_client) = deploy_ibc_and_client();

        ibc_dispatcher.register_client("cometbls", light_client);
        ibc_dispatcher.register_client("cometbls", light_client);
    }
}

mod create_client {
    use super::*;

    #[test]
    fn test_create_client_works() {
        let (ibc_dispatcher, ibc_contract, light_client) = deploy_ibc_and_client();

        let mut spy = spy_events();

        ibc_dispatcher.register_client("cometbls", light_client);

        ibc_dispatcher
            .create_client("cometbls", "client_state_bytes", "consensus_state_bytes", ibc_contract);

        let client_id = ClientIdImpl::new(1);

        assert!(load(ibc_contract, selector!("next_client_id"), 1)[0] == @2);

        let client_state_commitment = load_commitment(ibc_contract, ClientStatePath { client_id })
            .unwrap();

        let consensus_state_commitment = load_commitment(
            ibc_contract, ConsensusStatePath { client_id, height: 10 },
        )
            .unwrap();

        assert!(
            client_state_commitment == truncate(compute_keccak_byte_array(@"client_state_bytes")),
        );

        assert!(
            consensus_state_commitment == truncate(
                compute_keccak_byte_array(@"consensus_state_bytes"),
            ),
        );
        assert!(
            "cometbls" == load_byte_array_map_value(
                ibc_contract, selector!("client_types"), client_id,
            )
                .unwrap(),
        );
        spy
            .assert_emitted(
                @array![
                    (
                        ibc_contract,
                        Event::CreateClient(
                            CreateClient {
                                client_type: "cometbls",
                                client_id,
                                counterparty_chain_id: "counterparty-chain",
                            },
                        ),
                    ),
                ],
            );
    }

    #[test]
    #[should_panic(expected: 'CLIENT_TYPE_NOT_FOUND')]
    fn test_create_client_fails_client_type_not_found() {
        let (ibc_dispatcher, ibc_contract, _) = deploy_ibc_and_client();

        ibc_dispatcher
            .create_client("cometbls", "client_state_bytes", "consensus_state_bytes", ibc_contract);
    }
}

mod update_client {
    use super::*;

    #[test]
    fn test_update_client_works() {
        let (ibc_dispatcher, ibc_contract, light_client) = deploy_ibc_and_client();

        let mut spy = spy_events();

        ibc_dispatcher.register_client("cometbls", light_client);

        ibc_dispatcher
            .create_client("cometbls", "client_state_bytes", "consensus_state_bytes", ibc_contract);

        let client_id = ClientIdImpl::new(1);

        let mut client_message = Default::default();
        let new_client_state: ByteArray = "new_client_state_bytes";
        let new_consensus_state: ByteArray = "new_consensus_state_bytes";
        let height = 15;
        client_message.append_u32(new_client_state.len());
        client_message.append(@new_client_state);
        client_message.append_u32(new_consensus_state.len());
        client_message.append(@new_consensus_state);
        // height
        client_message.append_u64(height);

        ibc_dispatcher.update_client(client_id, client_message, ibc_contract);

        let client_state_commitment = load_commitment(ibc_contract, ClientStatePath { client_id })
            .unwrap();

        let consensus_state_commitment = load_commitment(
            ibc_contract, ConsensusStatePath { client_id, height },
        )
            .unwrap();

        assert!(client_state_commitment == truncate(compute_keccak_byte_array(@new_client_state)));

        assert!(
            consensus_state_commitment == truncate(compute_keccak_byte_array(@new_consensus_state)),
        );
        spy
            .assert_emitted(
                @array![(ibc_contract, Event::UpdateClient(UpdateClient { client_id, height }))],
            );
    }

    #[test]
    #[should_panic(expected: 'CLIENT_NOT_FOUND')]
    fn test_update_client_fails_client_not_found() {
        let (ibc_dispatcher, ibc_contract, _) = deploy_ibc_and_client();

        ibc_dispatcher.update_client(ClientIdImpl::new(1), Default::default(), ibc_contract);
    }
}

mod connection_handshake {
    use super::{*, ConnectionState};

    pub(crate) fn decode_connection(data: Array<felt252>) -> Option<Connection> {
        let client_id: u32 = (*data[1]).try_into().unwrap();
        let counterparty_client_id: u32 = (*data[2]).try_into().unwrap();
        Some(
            Connection {
                state: match *data[0] {
                    1 => ConnectionState::Init,
                    2 => ConnectionState::TryOpen,
                    3 => ConnectionState::Open,
                    _ => panic!("non existent"),
                },
                client_id: ClientIdImpl::new(client_id.try_into().unwrap()),
                counterparty_client_id: ClientIdImpl::new(
                    counterparty_client_id.try_into().unwrap(),
                ),
                counterparty_connection_id: match *data[3] {
                    0 => None,
                    1 => {
                        let connection_id: u32 = (*data[4]).try_into().unwrap();

                        Some(ConnectionIdImpl::new(connection_id.try_into().unwrap()))
                    },
                    _ => panic!("non existent"),
                },
            },
        )
    }

    #[test]
    fn test_connection_open_init_works() {
        let (ibc_dispatcher, ibc_contract, light_client) = deploy_ibc_and_client();

        let mut spy = spy_events();

        ibc_dispatcher.register_client("cometbls", light_client);

        let client_id = ibc_dispatcher
            .create_client("cometbls", "client_state_bytes", "consensus_state_bytes", ibc_contract);

        let counterparty_client_id = ClientIdImpl::new(2);

        let connection_id = ibc_dispatcher.connection_open_init(client_id, counterparty_client_id);

        // connections start from 1
        assert_eq!(connection_id.raw(), 1);

        let expected_connection = Connection {
            state: ConnectionState::Init,
            client_id,
            counterparty_client_id,
            counterparty_connection_id: None,
        };

        assert_eq!(
            load_commitment(ibc_contract, ConnectionPath { connection_id }).unwrap(),
            truncate(expected_connection.commit()),
        );

        let connection: Connection = load_map_value_custom(
            ibc_contract, selector!("connections"), connection_id, 5, |out| decode_connection(out),
        )
            .unwrap();

        assert_eq!(connection, expected_connection);

        spy
            .assert_emitted(
                @array![
                    (
                        ibc_contract,
                        Event::ConnectionOpenInit(
                            ConnectionOpenInit { connection_id, client_id, counterparty_client_id },
                        ),
                    ),
                ],
            );

        let new_connection_id = ibc_dispatcher
            .connection_open_init(client_id, counterparty_client_id);
        assert_eq!(connection_id.increment(), new_connection_id);
    }

    #[test]
    #[should_panic(expected: 'CLIENT_NOT_FOUND')]
    fn test_connection_open_init_fails_client_not_found() {
        let (ibc_dispatcher, _, _) = deploy_ibc_and_client();

        ibc_dispatcher.connection_open_init(ClientIdImpl::new(1), ClientIdImpl::new(2));
    }

    #[test]
    fn test_connection_open_try_works() {
        let (ibc_dispatcher, ibc_contract, light_client) = deploy_ibc_and_client();

        let mut spy = spy_events();

        ibc_dispatcher.register_client("cometbls", light_client);

        let client_id = ibc_dispatcher
            .create_client("cometbls", "client_state_bytes", "consensus_state_bytes", ibc_contract);

        let counterparty_client_id = ClientIdImpl::new(2);
        let counterparty_connection_id = ConnectionIdImpl::new(3);

        let connection_id = ibc_dispatcher
            .connection_open_try(
                counterparty_client_id,
                counterparty_connection_id,
                client_id,
                // We don't care about the proof part as it's the job of the respective unit tests
                // of the clients
                // TODO(aeryz): although we don't care, we still have to assert whether the calling
                // of the membership verification is done correctly. We can do that by putting the
                // key and the value to the `proof` and assert that the key and value given by the
                // protocol to the client is correct
                Default::default(),
                10,
            );

        // connections start from 1
        assert_eq!(connection_id.raw(), 1);

        let expected_connection = Connection {
            state: ConnectionState::TryOpen,
            client_id,
            counterparty_client_id,
            counterparty_connection_id: Some(counterparty_connection_id),
        };

        assert_eq!(
            load_commitment(ibc_contract, ConnectionPath { connection_id }).unwrap(),
            truncate(expected_connection.commit()),
        );
        let connection: Connection = load_map_value_custom(
            ibc_contract, selector!("connections"), connection_id, 5, |out| decode_connection(out),
        )
            .unwrap();

        assert_eq!(connection, expected_connection);

        spy
            .assert_emitted(
                @array![
                    (
                        ibc_contract,
                        Event::ConnectionOpenTry(
                            ConnectionOpenTry {
                                connection_id,
                                client_id,
                                counterparty_client_id,
                                counterparty_connection_id,
                            },
                        ),
                    ),
                ],
            );

        let new_connection_id = ibc_dispatcher
            .connection_open_init(client_id, counterparty_client_id);
        assert_eq!(connection_id.increment(), new_connection_id);
    }

    #[test]
    #[should_panic(expected: 'CLIENT_NOT_FOUND')]
    fn test_connection_open_try_fails_client_not_found() {
        let (ibc_dispatcher, _, _) = deploy_ibc_and_client();

        ibc_dispatcher
            .connection_open_try(
                ClientIdImpl::new(1),
                ConnectionIdImpl::new(2),
                ClientIdImpl::new(3),
                Default::default(),
                10,
            );
        // TODO(aeryz): I mentioned this previously but if we make membership proof verification
    // controllable, we can also assert `INVALID_PROOF`
    }

    #[test]
    fn test_connection_open_ack_works() {
        let (ibc_dispatcher, ibc_contract, light_client) = deploy_ibc_and_client();

        let mut spy = spy_events();

        ibc_dispatcher.register_client("cometbls", light_client);

        let client_id = ibc_dispatcher
            .create_client("cometbls", "client_state_bytes", "consensus_state_bytes", ibc_contract);

        let counterparty_client_id = ClientIdImpl::new(2);

        let counterparty_connection_id = ConnectionIdImpl::new(10);

        let connection_id = ibc_dispatcher.connection_open_init(client_id, counterparty_client_id);

        ibc_dispatcher
            .connection_open_ack(connection_id, counterparty_connection_id, Default::default(), 10);

        let expected_connection = Connection {
            state: ConnectionState::Open,
            client_id,
            counterparty_client_id,
            counterparty_connection_id: Some(counterparty_connection_id),
        };

        assert_eq!(
            load_commitment(ibc_contract, ConnectionPath { connection_id }).unwrap(),
            truncate(expected_connection.commit()),
        );
        let connection: Connection = load_map_value_custom(
            ibc_contract, selector!("connections"), connection_id, 5, |out| decode_connection(out),
        )
            .unwrap();

        assert_eq!(connection, expected_connection);

        spy
            .assert_emitted(
                @array![
                    (
                        ibc_contract,
                        Event::ConnectionOpenAck(
                            ConnectionOpenAck {
                                connection_id,
                                client_id,
                                counterparty_client_id,
                                counterparty_connection_id,
                            },
                        ),
                    ),
                ],
            );
    }

    #[test]
    #[should_panic]
    fn test_connection_open_ack_fails_connection_not_found() {
        let (ibc_dispatcher, _, _) = deploy_ibc_and_client();

        ibc_dispatcher
            .connection_open_ack(
                ConnectionIdImpl::new(1), ConnectionIdImpl::new(2), Default::default(), 10,
            );
    }

    #[test]
    #[should_panic(expected: 'INVALID_CONNECTION_STATE')]
    fn test_connection_open_ack_fails_invalid_connection_state() {
        let (ibc_dispatcher, ibc_contract, light_client) = deploy_ibc_and_client();

        ibc_dispatcher.register_client("cometbls", light_client);

        let client_id = ibc_dispatcher
            .create_client("cometbls", "client_state_bytes", "consensus_state_bytes", ibc_contract);
        let counterparty_client_id = ClientIdImpl::new(2);

        let counterparty_connection_id = ConnectionIdImpl::new(10);

        let connection_id = ibc_dispatcher
            .connection_open_try(
                counterparty_client_id,
                counterparty_connection_id,
                client_id,
                Default::default(),
                10,
            );

        ibc_dispatcher
            .connection_open_ack(connection_id, counterparty_connection_id, Default::default(), 10);
    }

    #[test]
    #[should_panic(expected: 'INVALID_CONNECTION_STATE')]
    fn test_connection_open_ack_fails_invalid_connection_state2() {
        let (ibc_dispatcher, ibc_contract, light_client) = deploy_ibc_and_client();

        ibc_dispatcher.register_client("cometbls", light_client);

        let client_id = ibc_dispatcher
            .create_client("cometbls", "client_state_bytes", "consensus_state_bytes", ibc_contract);

        let counterparty_client_id = ClientIdImpl::new(2);

        let counterparty_connection_id = ConnectionIdImpl::new(10);

        let connection_id = ibc_dispatcher.connection_open_init(client_id, counterparty_client_id);

        ibc_dispatcher
            .connection_open_ack(connection_id, counterparty_connection_id, Default::default(), 10);

        ibc_dispatcher
            .connection_open_ack(connection_id, counterparty_connection_id, Default::default(), 10);
    }


    #[test]
    fn test_connection_open_confirm_works() {
        let (ibc_dispatcher, ibc_contract, light_client) = deploy_ibc_and_client();

        let mut spy = spy_events();

        ibc_dispatcher.register_client("cometbls", light_client);

        let client_id = ibc_dispatcher
            .create_client("cometbls", "client_state_bytes", "consensus_state_bytes", ibc_contract);

        let counterparty_client_id = ClientIdImpl::new(2);

        let counterparty_connection_id = ConnectionIdImpl::new(10);

        let connection_id = ibc_dispatcher
            .connection_open_try(
                counterparty_client_id,
                counterparty_connection_id,
                client_id,
                Default::default(),
                10,
            );

        ibc_dispatcher.connection_open_confirm(connection_id, Default::default(), 10);

        let expected_connection = Connection {
            state: ConnectionState::Open,
            client_id,
            counterparty_client_id,
            counterparty_connection_id: Some(counterparty_connection_id),
        };

        assert_eq!(
            load_commitment(ibc_contract, ConnectionPath { connection_id }).unwrap(),
            truncate(expected_connection.commit()),
        );
        let connection: Connection = load_map_value_custom(
            ibc_contract, selector!("connections"), connection_id, 5, |out| decode_connection(out),
        )
            .unwrap();

        assert_eq!(connection, expected_connection);

        spy
            .assert_emitted(
                @array![
                    (
                        ibc_contract,
                        Event::ConnectionOpenConfirm(
                            ConnectionOpenConfirm {
                                connection_id,
                                client_id,
                                counterparty_client_id,
                                counterparty_connection_id,
                            },
                        ),
                    ),
                ],
            );
    }

    #[test]
    #[should_panic]
    fn test_connection_open_confirm_fails_connection_not_found() {
        let (ibc_dispatcher, _, _) = deploy_ibc_and_client();

        ibc_dispatcher.connection_open_confirm(ConnectionIdImpl::new(1), Default::default(), 10);
    }

    #[test]
    #[should_panic(expected: 'INVALID_CONNECTION_STATE')]
    fn test_connection_open_confirm_fails_invalid_connection_state() {
        let (ibc_dispatcher, ibc_contract, light_client) = deploy_ibc_and_client();

        ibc_dispatcher.register_client("cometbls", light_client);

        let client_id = ibc_dispatcher
            .create_client("cometbls", "client_state_bytes", "consensus_state_bytes", ibc_contract);

        let counterparty_client_id = ClientIdImpl::new(2);

        let connection_id = ibc_dispatcher.connection_open_init(client_id, counterparty_client_id);

        ibc_dispatcher.connection_open_confirm(connection_id, Default::default(), 10);
    }

    #[test]
    #[should_panic(expected: 'INVALID_CONNECTION_STATE')]
    fn test_connection_open_confirm_fails_invalid_connection_state2() {
        let (ibc_dispatcher, ibc_contract, light_client) = deploy_ibc_and_client();

        ibc_dispatcher.register_client("cometbls", light_client);

        let client_id = ibc_dispatcher
            .create_client("cometbls", "client_state_bytes", "consensus_state_bytes", ibc_contract);

        let counterparty_client_id = ClientIdImpl::new(2);

        let counterparty_connection_id = ConnectionIdImpl::new(10);

        let connection_id = ibc_dispatcher
            .connection_open_try(
                counterparty_client_id,
                counterparty_connection_id,
                client_id,
                Default::default(),
                10,
            );

        ibc_dispatcher.connection_open_confirm(connection_id, Default::default(), 10);

        ibc_dispatcher.connection_open_confirm(connection_id, Default::default(), 10);
    }
}
