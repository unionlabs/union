use std::{collections::BTreeSet, num::NonZeroU32};

use alloy::sol_types::SolValue;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, wasm_execute, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response,
    StdResult,
};
use cw_storage_plus::Item;
use ibc_union_msg::{
    lightclient::{
        QueryMsg as LightClientQuery, Status, VerifyClientMessageUpdate, VerifyCreationResponse,
        VerifyCreationResponseEvent,
    },
    module::{ExecuteMsg as ModuleMsg, IbcUnionMsg},
    msg::{
        ExecuteMsg, InitMsg, MsgBatchAcks, MsgBatchSend, MsgChannelCloseConfirm,
        MsgChannelCloseInit, MsgChannelOpenAck, MsgChannelOpenConfirm, MsgChannelOpenInit,
        MsgChannelOpenTry, MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
        MsgConnectionOpenTry, MsgCreateClient, MsgIntentPacketRecv, MsgMigrateState,
        MsgPacketAcknowledgement, MsgPacketRecv, MsgPacketTimeout, MsgRegisterClient,
        MsgSendPacket, MsgUpdateClient, MsgWriteAcknowledgement,
    },
    query::QueryMsg,
};
use ibc_union_spec::{
    path::{
        BatchPacketsPath, BatchReceiptsPath, ChannelPath, ClientStatePath, ConnectionPath,
        ConsensusStatePath, COMMITMENT_MAGIC,
    },
    types::{Channel, ChannelState, Connection, ConnectionState, Packet},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use unionlabs::{
    ethereum::keccak256,
    primitives::{encoding::HexPrefixed, Bytes, H256},
};
use unionlabs_cosmwasm_upgradable::UpgradeMsg;

use crate::{
    state::{
        CHANNELS, CHANNEL_OWNER, CLIENT_CONSENSUS_STATES, CLIENT_IMPLS, CLIENT_REGISTRY,
        CLIENT_STATES, CLIENT_TYPES, CONNECTIONS, CONTRACT_CHANNELS, NEXT_CHANNEL_ID,
        NEXT_CLIENT_ID, NEXT_CONNECTION_ID, QUERY_STORE,
    },
    ContractError,
};

type ContractResult = Result<Response, ContractError>;

pub mod events {
    pub mod client {
        pub const REGISTER: &str = "register_client";
        pub const CREATE: &str = "create_client";
        pub const UPDATE: &str = "update_client";
    }
    pub mod connection {
        pub const OPEN_INIT: &str = "connection_open_init";
        pub const OPEN_TRY: &str = "connection_open_try";
        pub const OPEN_ACK: &str = "connection_open_ack";
        pub const OPEN_CONFIRM: &str = "connection_open_confirm";
    }
    pub mod channel {
        pub const OPEN_INIT: &str = "channel_open_init";
        pub const OPEN_TRY: &str = "channel_open_try";
        pub const OPEN_ACK: &str = "channel_open_ack";
        pub const OPEN_CONFIRM: &str = "channel_open_confirm";
        pub const CLOSE_INIT: &str = "channel_close_init";
        pub const CLOSE_CONFIRM: &str = "channel_close_confirm";
    }
    pub mod packet {
        pub const SEND: &str = "packet_send";
        pub const RECV: &str = "packet_recv";
        pub const INTENT_RECV: &str = "intent_packet_recv";
        pub const ACK: &str = "packet_ack";
        pub const TIMEOUT: &str = "packet_timeout";
        pub const BATCH_SEND: &str = "batch_send";
        pub const BATCH_ACKS: &str = "batch_acks";
    }
    pub mod attribute {
        pub const CLIENT_ID: &str = "client_id";
        pub const CONNECTION_ID: &str = "connection_id";
        pub const CHANNEL_ID: &str = "channel_id";
        pub const COUNTERPARTY_CHANNEL_ID: &str = "counterparty_channel_id";
        pub const COUNTERPARTY_HEIGHT: &str = "counterparty_height";
        pub const PACKET: &str = "packet";
        pub const PACKETS: &str = "packets";
        pub const ACKS: &str = "acks";
        pub const MAKER: &str = "maker";
        pub const MAKER_MSG: &str = "maker_msg";
        pub const ACKNOWLEDGEMENT: &str = "acknowledgement";
        pub const CLIENT_TYPE: &str = "client_type";
        pub const CLIENT_ADDRESS: &str = "client_address";
        pub const COUNTERPARTY_CHAIN_ID: &str = "counterparty_chain_id";
        pub const COUNTERPARTY_CLIENT_ID: &str = "counterparty_client_id";
        pub const COUNTERPARTY_CONNECTION_ID: &str = "counterparty_connection_id";
        pub const PORT_ID: &str = "port_id";
        pub const COUNTERPARTY_PORT_ID: &str = "counterparty_port_id";
        pub const VERSION: &str = "version";
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterClient(MsgRegisterClient {
            client_type,
            client_address,
        }) => {
            let address = deps.api.addr_validate(&client_address)?;
            register_client(deps.branch(), client_type, address)
        }
        ExecuteMsg::CreateClient(MsgCreateClient {
            client_type,
            client_state_bytes,
            consensus_state_bytes,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            create_client(
                deps.branch(),
                client_type,
                client_state_bytes.to_vec(),
                consensus_state_bytes.to_vec(),
                relayer,
            )
        }
        ExecuteMsg::UpdateClient(MsgUpdateClient {
            client_id,
            client_message,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            update_client(deps.branch(), client_id, client_message.to_vec(), relayer)
        }
        ExecuteMsg::ConnectionOpenInit(MsgConnectionOpenInit {
            client_id,
            counterparty_client_id,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            connection_open_init(deps.branch(), client_id, counterparty_client_id, relayer)
        }
        ExecuteMsg::ConnectionOpenTry(MsgConnectionOpenTry {
            counterparty_client_id,
            counterparty_connection_id,
            client_id,
            proof_init,
            proof_height,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            connection_open_try(
                deps.branch(),
                counterparty_client_id,
                counterparty_connection_id,
                client_id,
                proof_init.to_vec(),
                proof_height,
                relayer,
            )
        }
        ExecuteMsg::ConnectionOpenAck(MsgConnectionOpenAck {
            connection_id,
            counterparty_connection_id,
            proof_try,
            proof_height,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            connection_open_ack(
                deps.branch(),
                connection_id,
                counterparty_connection_id,
                proof_try.to_vec(),
                proof_height,
                relayer,
            )
        }
        ExecuteMsg::ConnectionOpenConfirm(MsgConnectionOpenConfirm {
            connection_id,
            proof_ack,
            proof_height,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            connection_open_confirm(
                deps.branch(),
                connection_id,
                proof_ack.to_vec(),
                proof_height,
                relayer,
            )
        }
        ExecuteMsg::ChannelOpenInit(MsgChannelOpenInit {
            port_id,
            counterparty_port_id,
            connection_id,
            version,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            channel_open_init(
                deps.branch(),
                port_id,
                counterparty_port_id,
                connection_id,
                version,
                relayer,
            )
        }
        ExecuteMsg::ChannelOpenTry(MsgChannelOpenTry {
            port_id,
            channel,
            counterparty_version,
            proof_init,
            proof_height,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            channel_open_try(
                deps.branch(),
                port_id,
                channel,
                counterparty_version,
                proof_init.to_vec(),
                proof_height,
                relayer,
            )
        }
        ExecuteMsg::ChannelOpenAck(MsgChannelOpenAck {
            channel_id,
            counterparty_version,
            counterparty_channel_id,
            proof_try,
            proof_height,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            channel_open_ack(
                deps.branch(),
                channel_id,
                counterparty_version,
                counterparty_channel_id,
                proof_try.to_vec(),
                proof_height,
                relayer,
            )
        }
        ExecuteMsg::ChannelOpenConfirm(MsgChannelOpenConfirm {
            channel_id,
            proof_ack,
            proof_height,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            channel_open_confirm(
                deps.branch(),
                channel_id,
                proof_ack.to_vec(),
                proof_height,
                relayer,
            )
        }
        ExecuteMsg::ChannelCloseInit(MsgChannelCloseInit {
            channel_id,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            channel_close_init(deps.branch(), channel_id, relayer)
        }
        ExecuteMsg::ChannelCloseConfirm(MsgChannelCloseConfirm {
            channel_id,
            proof_init,
            proof_height,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            channel_close_confirm(
                deps.branch(),
                channel_id,
                proof_init.to_vec(),
                proof_height,
                relayer,
            )
        }
        ExecuteMsg::PacketRecv(MsgPacketRecv {
            packets,
            relayer_msgs,
            relayer,
            proof,
            proof_height,
        }) => process_receive(
            deps,
            env,
            packets,
            relayer_msgs.into_iter().map(Into::into).collect(),
            relayer,
            proof.into(),
            proof_height,
            false,
        ),
        ExecuteMsg::PacketAck(MsgPacketAcknowledgement {
            packets,
            acknowledgements,
            proof,
            proof_height,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            acknowledge_packet(
                deps.branch(),
                packets,
                acknowledgements.into_iter().map(Into::into).collect(),
                proof.to_vec(),
                proof_height,
                relayer,
            )
        }
        ExecuteMsg::PacketTimeout(MsgPacketTimeout {
            packet,
            proof,
            proof_height,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            timeout_packet(deps.branch(), packet, proof.to_vec(), proof_height, relayer)
        }
        ExecuteMsg::IntentPacketRecv(MsgIntentPacketRecv {
            packets,
            market_maker_msgs,
            market_maker,
            empty_proof,
        }) => process_receive(
            deps,
            env,
            packets,
            market_maker_msgs.into_iter().map(Into::into).collect(),
            market_maker,
            empty_proof.into(),
            0,
            true,
        ),
        ExecuteMsg::WriteAcknowledgement(MsgWriteAcknowledgement {
            channel_id,
            packet,
            acknowledgement,
        }) => write_acknowledgement(
            deps.branch(),
            info.sender,
            channel_id,
            packet,
            acknowledgement.into_vec(),
        ),
        ExecuteMsg::PacketSend(MsgSendPacket {
            source_channel,
            timeout_height,
            timeout_timestamp,
            data,
        }) => send_packet(
            deps.branch(),
            info.sender,
            source_channel,
            timeout_height,
            timeout_timestamp,
            data.into_vec(),
        ),
        ExecuteMsg::BatchSend(MsgBatchSend {
            source_channel,
            packets,
        }) => batch_send(deps, source_channel, packets),
        ExecuteMsg::BatchAcks(MsgBatchAcks {
            source_channel,
            packets,
            acks,
        }) => batch_acks(
            deps,
            source_channel,
            packets,
            acks.into_iter().map(Into::into).collect(),
        ),
        ExecuteMsg::MigrateState(MsgMigrateState {
            client_id,
            client_state,
            consensus_state,
            height,
        }) => migrate_state(
            deps,
            info.sender,
            client_id,
            client_state,
            consensus_state,
            height,
        ),
    }
}

fn migrate_state(
    mut deps: DepsMut,
    sender: Addr,
    client_id: u32,
    client_state: unionlabs::primitives::Bytes,
    consensus_state: unionlabs::primitives::Bytes,
    height: u64,
) -> Result<Response, ContractError> {
    let client_addr = CLIENT_IMPLS.load(deps.storage, client_id)?;

    if client_addr != sender {
        return Err(ContractError::UnauthorizedMigration {
            client_id,
            caller: sender,
            client: client_addr,
        });
    }

    CLIENT_STATES.update(deps.storage, client_id, |s| {
        let _ = s.ok_or(ContractError::CannotMigrateWithNoClientState { client_id })?;
        Ok::<Binary, ContractError>(client_state.to_vec().into())
    })?;

    store_commit(
        deps.branch(),
        &ClientStatePath { client_id }.key(),
        &commit(client_state),
    )?;

    CLIENT_CONSENSUS_STATES.update(deps.storage, (client_id, height), |s| {
        let _ = s.ok_or(ContractError::CannotMigrateWithNoConsensusState { client_id, height })?;
        Ok::<Binary, ContractError>(consensus_state.to_vec().into())
    })?;

    store_commit(
        deps.branch(),
        &ConsensusStatePath { client_id, height }.key(),
        &commit(consensus_state),
    )?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(_: DepsMut, _: Env, _: MessageInfo, _: ()) -> StdResult<Response> {
    panic!("this contract cannot be instantiated directly, but must be migrated from an existing instantiated contract.");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct IbcUnionMigrateMsg {}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    msg: UpgradeMsg<InitMsg, IbcUnionMigrateMsg>,
) -> Result<Response, ContractError> {
    msg.run(deps, init, |_deps, _migrate, _version| {
        Ok((Response::default(), None))
    })
}

pub(crate) fn init(
    deps: DepsMut<'_>,
    InitMsg {}: InitMsg,
) -> Result<(Response, Option<NonZeroU32>), ContractError> {
    NEXT_CHANNEL_ID.save(deps.storage, &0)?;
    NEXT_CONNECTION_ID.save(deps.storage, &0)?;
    NEXT_CLIENT_ID.save(deps.storage, &0)?;

    Ok((Response::default(), None))
}

fn batch_send(deps: DepsMut, source_channel: u32, packets: Vec<Packet>) -> ContractResult {
    if packets.len() < 2 {
        return Err(ContractError::NotEnoughPackets);
    }
    for packet in &packets {
        let commitment_key = BatchPacketsPath {
            channel_id: source_channel,
            batch_hash: commit_packet(packet),
        }
        .key();
        let commitment = deps
            .storage
            .get(commitment_key.as_ref())
            .unwrap_or(H256::<HexPrefixed>::default().into_bytes().into());
        if commitment != COMMITMENT_MAGIC.as_ref() {
            return Err(ContractError::PacketCommitmentNotFound);
        }
    }
    store_commit(
        deps,
        &BatchPacketsPath {
            channel_id: source_channel,
            batch_hash: commit_packets(&packets),
        }
        .key(),
        &COMMITMENT_MAGIC,
    )?;
    Ok(
        Response::new().add_event(Event::new(events::packet::BATCH_SEND).add_attributes([
            (events::attribute::CHANNEL_ID, source_channel.to_string()),
            (
                events::attribute::PACKETS,
                serde_json::to_string(&packets).expect("packet serialization is infallible; qed;"),
            ),
        ])),
    )
}

fn batch_acks(
    deps: DepsMut,
    source_channel: u32,
    packets: Vec<Packet>,
    acks: Vec<Bytes>,
) -> ContractResult {
    if packets.len() < 2 {
        return Err(ContractError::NotEnoughPackets);
    }
    for (packet, ack) in packets.iter().zip(acks.iter()) {
        let commitment_key = BatchReceiptsPath {
            channel_id: source_channel,
            batch_hash: commit_packet(packet),
        }
        .key();
        let commitment = deps.storage.get(commitment_key.as_ref());
        match commitment {
            Some(acknowledgement) => {
                let expected_ack = commit_ack(ack);

                if acknowledgement == COMMITMENT_MAGIC.as_ref() {
                    return Err(ContractError::AcknowledgementIsEmpty);
                } else if acknowledgement != expected_ack.as_ref() {
                    return Err(ContractError::AcknowledgementMismatch {
                        found: acknowledgement.into(),
                        expected: expected_ack.into(),
                    });
                }
            }
            None => return Err(ContractError::PacketCommitmentNotFound),
        }
    }
    store_commit(
        deps,
        &BatchReceiptsPath {
            channel_id: source_channel,
            batch_hash: commit_packets(&packets),
        }
        .key(),
        &commit_acks(&acks),
    )?;
    Ok(
        Response::new().add_event(Event::new(events::packet::BATCH_ACKS).add_attributes([
            (events::attribute::CHANNEL_ID, source_channel.to_string()),
            (
                events::attribute::PACKETS,
                serde_json::to_string(&packets).expect("packet serialization is infallible; qed;"),
            ),
            (
                events::attribute::ACKS,
                serde_json::to_string(&acks).expect("bytes serialization is infallible; qed;"),
            ),
        ])),
    )
}

fn timeout_packet(
    mut deps: DepsMut,
    packet: Packet,
    proof: Vec<u8>,
    proof_height: u64,
    relayer: Addr,
) -> ContractResult {
    let source_channel = packet.source_channel_id;
    let destination_channel = packet.destination_channel_id;
    let channel = ensure_channel_state(deps.as_ref(), source_channel)?;
    let connection = ensure_connection_state(deps.as_ref(), channel.connection_id)?;

    let proof_timestamp =
        get_timestamp_at_height(deps.as_ref(), connection.client_id, proof_height)?;
    if proof_timestamp == 0 {
        return Err(ContractError::TimeoutProofTimestampNotFound);
    }

    let commitment_key = BatchReceiptsPath {
        channel_id: destination_channel,
        batch_hash: commit_packet(&packet),
    }
    .key();

    let client_impl = client_impl(deps.as_ref(), connection.client_id)?;
    query_light_client::<()>(
        deps.as_ref(),
        client_impl,
        LightClientQuery::VerifyNonMembership {
            client_id: connection.client_id,
            height: proof_height,
            proof: proof.to_vec().into(),
            path: commitment_key.into_bytes(),
        },
    )?;
    delete_packet_commitment(deps.branch(), source_channel, &packet)?;

    if packet.timeout_timestamp == 0 && packet.timeout_height == 0 {
        return Err(ContractError::TimeoutMustBeSet);
    }

    if packet.timeout_timestamp > 0 && packet.timeout_timestamp > proof_timestamp {
        return Err(ContractError::TimeoutTimestampNotReached);
    }

    if packet.timeout_height > 0 && packet.timeout_height > proof_height {
        return Err(ContractError::TimeoutHeightNotReached);
    }

    let port_id = CHANNEL_OWNER.load(deps.storage, source_channel)?;
    Ok(Response::new()
        .add_event(Event::new(events::packet::TIMEOUT).add_attributes([
            (
                events::attribute::PACKET,
                serde_json::to_string(&packet).expect("packet serialization is infallible; qed;"),
            ),
            (events::attribute::MAKER, relayer.to_string()),
        ]))
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnTimeoutPacket {
                packet,
                relayer: relayer.into(),
            }),
            vec![],
        )?))
}

fn acknowledge_packet(
    mut deps: DepsMut,
    packets: Vec<Packet>,
    acknowledgements: Vec<Bytes>,
    proof: Vec<u8>,
    proof_height: u64,
    relayer: Addr,
) -> ContractResult {
    let first = packets.first().ok_or(ContractError::NotEnoughPackets)?;

    let source_channel = first.source_channel_id;
    let destination_channel = first.destination_channel_id;

    let channel = ensure_channel_state(deps.as_ref(), source_channel)?;
    let connection = ensure_connection_state(deps.as_ref(), channel.connection_id)?;

    let (batch_hash, commitment_value) = match packets.len() {
        1 => (commit_packet(first), commit_ack(&acknowledgements[0])),
        _ => (commit_packets(&packets), commit_acks(&acknowledgements)),
    };

    let commitment_key = BatchReceiptsPath {
        channel_id: destination_channel,
        batch_hash,
    }
    .key();

    let client_impl = client_impl(deps.as_ref(), connection.client_id)?;
    query_light_client::<()>(
        deps.as_ref(),
        client_impl,
        LightClientQuery::VerifyMembership {
            client_id: connection.client_id,
            height: proof_height,
            proof: proof.to_vec().into(),
            path: commitment_key.into_bytes(),
            value: commitment_value.into_bytes(),
        },
    )?;

    let port_id = CHANNEL_OWNER.load(deps.storage, source_channel)?;
    let mut events = Vec::with_capacity(packets.len());
    let mut messages = Vec::with_capacity(packets.len());
    for (packet, ack) in packets.into_iter().zip(acknowledgements) {
        delete_packet_commitment(deps.branch(), source_channel, &packet)?;
        events.push(Event::new(events::packet::ACK).add_attributes([
            (
                events::attribute::PACKET,
                serde_json::to_string(&packet).expect("packet serialization is infallible; qed;"),
            ),
            (events::attribute::ACKNOWLEDGEMENT, hex::encode(&ack)),
            (events::attribute::MAKER, relayer.clone().to_string()),
        ]));
        messages.push(wasm_execute(
            port_id.clone(),
            &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnAcknowledgementPacket {
                packet,
                acknowledgement: ack.to_vec().into(),
                relayer: relayer.clone().into(),
            }),
            vec![],
        )?);
    }

    Ok(Response::new().add_events(events).add_messages(messages))
}

fn delete_packet_commitment(
    deps: DepsMut,
    source_channel: u32,
    packet: &Packet,
) -> Result<(), ContractError> {
    let commitment_key = BatchPacketsPath {
        channel_id: source_channel,
        batch_hash: commit_packet(packet),
    }
    .key();
    let commitment = deps
        .storage
        .get(commitment_key.as_ref())
        .unwrap_or(H256::<HexPrefixed>::default().into_bytes().into_vec());
    if commitment != COMMITMENT_MAGIC.as_ref() {
        return Err(ContractError::PacketCommitmentNotFound);
    }
    deps.storage.remove(commitment_key.as_ref());
    Ok(())
}

fn commit_packet(packet: &Packet) -> H256 {
    commit(packet.abi_encode())
}

fn commit_packets(packets: &[Packet]) -> H256 {
    commit(packets.abi_encode())
}

fn register_client(
    deps: DepsMut,
    client_type: String,
    client_address: Addr,
) -> Result<Response, ContractError> {
    if CLIENT_REGISTRY
        .may_load(deps.storage, &client_type)?
        .is_some()
    {
        return Err(ContractError::ClientTypeAlreadyExists);
    }
    CLIENT_REGISTRY.save(deps.storage, &client_type, &client_address)?;

    Ok(Response::new().add_event(
        Event::new(events::client::REGISTER)
            .add_attribute(events::attribute::CLIENT_TYPE, client_type)
            .add_attribute(events::attribute::CLIENT_ADDRESS, client_address),
    ))
}

fn create_client(
    mut deps: DepsMut,
    client_type: String,
    client_state_bytes: Vec<u8>,
    consensus_state_bytes: Vec<u8>,
    _relayer: Addr,
) -> Result<Response, ContractError> {
    let client_impl = CLIENT_REGISTRY.load(deps.storage, &client_type)?;
    let client_id = next_client_id(deps.branch())?;
    CLIENT_TYPES.save(deps.storage, client_id, &client_type)?;
    CLIENT_IMPLS.save(deps.storage, client_id, &client_impl)?;
    let verify_creation_response = query_light_client::<VerifyCreationResponse>(
        deps.as_ref(),
        client_impl,
        LightClientQuery::VerifyCreation {
            client_id,
            client_state: client_state_bytes.to_vec().into(),
            consensus_state: consensus_state_bytes.to_vec().into(),
        },
    )?;
    CLIENT_STATES.save(deps.storage, client_id, &client_state_bytes.to_vec().into())?;
    CLIENT_CONSENSUS_STATES.save(
        deps.storage,
        (client_id, verify_creation_response.latest_height),
        &consensus_state_bytes.to_vec().into(),
    )?;
    store_commit(
        deps.branch(),
        &ClientStatePath { client_id }.key(),
        &commit(client_state_bytes),
    )?;
    store_commit(
        deps,
        &ConsensusStatePath {
            client_id,
            height: verify_creation_response.latest_height,
        }
        .key(),
        &commit(consensus_state_bytes),
    )?;
    let mut response = Response::new();
    if let Some(events) = verify_creation_response.events {
        response = response.add_events(
            events
                .into_iter()
                .map(|e| make_verify_creation_event(client_id, e))
                .collect::<Vec<_>>(),
        );
    }
    Ok(
        response.add_event(Event::new(events::client::CREATE).add_attributes([
            (events::attribute::CLIENT_TYPE, client_type),
            (events::attribute::CLIENT_ID, client_id.to_string()),
            (
                events::attribute::COUNTERPARTY_CHAIN_ID,
                verify_creation_response.counterparty_chain_id,
            ),
        ])),
    )
}

fn update_client(
    mut deps: DepsMut,
    client_id: u32,
    client_message: Vec<u8>,
    relayer: Addr,
) -> Result<Response, ContractError> {
    let client_impl = client_impl(deps.as_ref(), client_id)?;
    let update = {
        // Ugly hack to allow for >64K messages (not configurable) to be threaded for the query.
        // See https://github.com/CosmWasm/cosmwasm/blob/e17ecc44cdebc84de1caae648c7a4f4b56846f8f/packages/vm/src/imports.rs#L47
        QUERY_STORE.save(deps.storage, &client_message.into())?;

        let status = query_light_client::<Status>(
            deps.as_ref(),
            client_impl.clone(),
            LightClientQuery::GetStatus { client_id },
        )?;

        if !matches!(status, Status::Active) {
            return Err(ContractError::ClientNotActive { client_id, status });
        }

        let update = query_light_client::<VerifyClientMessageUpdate>(
            deps.as_ref(),
            client_impl,
            LightClientQuery::VerifyClientMessage {
                client_id,
                caller: relayer.into(),
            },
        )?;
        QUERY_STORE.remove(deps.storage);
        update
    };
    CLIENT_STATES.save(
        deps.storage,
        client_id,
        &update.client_state.to_vec().into(),
    )?;
    CLIENT_CONSENSUS_STATES.save(
        deps.storage,
        (client_id, update.height),
        &update.consensus_state.to_vec().into(),
    )?;

    store_commit(
        deps.branch(),
        &ClientStatePath { client_id }.key(),
        &commit(update.client_state),
    )?;
    store_commit(
        deps.branch(),
        &ConsensusStatePath {
            client_id,
            height: update.height,
        }
        .key(),
        &commit(update.consensus_state),
    )?;
    Ok(
        Response::new().add_event(Event::new(events::client::UPDATE).add_attributes([
            (events::attribute::CLIENT_ID, client_id.to_string()),
            (
                events::attribute::COUNTERPARTY_HEIGHT,
                update.height.to_string(),
            ),
        ])),
    )
}

fn connection_open_init(
    mut deps: DepsMut,
    client_id: u32,
    counterparty_client_id: u32,
    _relayer: Addr,
) -> ContractResult {
    let connection_id = next_connection_id(deps.branch())?;
    let connection = Connection {
        state: ConnectionState::Init,
        client_id,
        counterparty_client_id,
        counterparty_connection_id: 0,
    };
    save_connection(deps.branch(), connection_id, &connection)?;
    Ok(
        Response::new().add_event(Event::new(events::connection::OPEN_INIT).add_attributes([
            (events::attribute::CONNECTION_ID, connection_id.to_string()),
            (events::attribute::CLIENT_ID, client_id.to_string()),
            (
                events::attribute::COUNTERPARTY_CLIENT_ID,
                counterparty_client_id.to_string(),
            ),
        ])),
    )
}

fn connection_open_try(
    mut deps: DepsMut,
    counterparty_client_id: u32,
    counterparty_connection_id: u32,
    client_id: u32,
    proof_init: Vec<u8>,
    proof_height: u64,
    _relayer: Addr,
) -> ContractResult {
    let connection_id = next_connection_id(deps.branch())?;
    let connection = Connection {
        state: ConnectionState::TryOpen,
        client_id,
        counterparty_client_id,
        counterparty_connection_id,
    };

    let expected_connection = Connection {
        state: ConnectionState::Init,
        client_id: counterparty_client_id,
        counterparty_client_id: client_id,
        counterparty_connection_id: 0,
    };

    let client_impl = client_impl(deps.as_ref(), client_id)?;
    query_light_client::<()>(
        deps.as_ref(),
        client_impl,
        LightClientQuery::VerifyMembership {
            client_id,
            height: proof_height,
            proof: proof_init.into(),
            path: ConnectionPath {
                connection_id: counterparty_connection_id,
            }
            .key()
            .into_bytes(),
            value: commit(expected_connection.abi_encode_params()).into_bytes(),
        },
    )?;
    save_connection(deps.branch(), connection_id, &connection)?;
    Ok(
        Response::new().add_event(Event::new(events::connection::OPEN_TRY).add_attributes([
            (events::attribute::CONNECTION_ID, connection_id.to_string()),
            (events::attribute::CLIENT_ID, client_id.to_string()),
            (
                events::attribute::COUNTERPARTY_CLIENT_ID,
                counterparty_client_id.to_string(),
            ),
            (
                events::attribute::COUNTERPARTY_CONNECTION_ID,
                counterparty_connection_id.to_string(),
            ),
        ])),
    )
}

fn connection_open_ack(
    mut deps: DepsMut,
    connection_id: u32,
    counterparty_connection_id: u32,
    proof_try: Vec<u8>,
    proof_height: u64,
    _relayer: Addr,
) -> ContractResult {
    let mut connection = CONNECTIONS.load(deps.storage, connection_id)?;
    if connection.state != ConnectionState::Init {
        return Err(ContractError::ConnectionInvalidState {
            got: connection.state,
            expected: ConnectionState::Init,
        });
    }
    let expected_connection = Connection {
        state: ConnectionState::TryOpen,
        client_id: connection.counterparty_client_id,
        counterparty_client_id: connection.client_id,
        counterparty_connection_id: connection_id,
    };
    let client_impl = client_impl(deps.as_ref(), connection.client_id)?;
    query_light_client::<()>(
        deps.as_ref(),
        client_impl,
        LightClientQuery::VerifyMembership {
            client_id: connection.client_id,
            height: proof_height,
            proof: proof_try.into(),
            path: ConnectionPath {
                connection_id: counterparty_connection_id,
            }
            .key()
            .into_bytes(),
            value: commit(expected_connection.abi_encode_params()).into_bytes(),
        },
    )?;
    connection.state = ConnectionState::Open;
    connection.counterparty_connection_id = counterparty_connection_id;
    save_connection(deps.branch(), connection_id, &connection)?;
    Ok(
        Response::new().add_event(Event::new(events::connection::OPEN_ACK).add_attributes([
            (events::attribute::CONNECTION_ID, connection_id.to_string()),
            (
                events::attribute::CLIENT_ID,
                connection.client_id.to_string(),
            ),
            (
                events::attribute::COUNTERPARTY_CLIENT_ID,
                connection.counterparty_client_id.to_string(),
            ),
            (
                events::attribute::COUNTERPARTY_CONNECTION_ID,
                connection.counterparty_connection_id.to_string(),
            ),
        ])),
    )
}

fn connection_open_confirm(
    mut deps: DepsMut,
    connection_id: u32,
    proof_ack: Vec<u8>,
    proof_height: u64,
    _relayer: Addr,
) -> ContractResult {
    let mut connection = CONNECTIONS.load(deps.storage, connection_id)?;
    if connection.state != ConnectionState::TryOpen {
        return Err(ContractError::ConnectionInvalidState {
            got: connection.state,
            expected: ConnectionState::TryOpen,
        });
    }
    let expected_connection = Connection {
        state: ConnectionState::Open,
        client_id: connection.counterparty_client_id,
        counterparty_client_id: connection.client_id,
        counterparty_connection_id: connection_id,
    };
    let client_impl = client_impl(deps.as_ref(), connection.client_id)?;
    query_light_client::<()>(
        deps.as_ref(),
        client_impl,
        LightClientQuery::VerifyMembership {
            client_id: connection.client_id,
            height: proof_height,
            proof: proof_ack.into(),
            path: ConnectionPath {
                connection_id: connection.counterparty_connection_id,
            }
            .key()
            .into_bytes(),
            value: commit(expected_connection.abi_encode_params()).into_bytes(),
        },
    )?;
    connection.state = ConnectionState::Open;
    save_connection(deps.branch(), connection_id, &connection)?;
    Ok(Response::new().add_event(
        Event::new(events::connection::OPEN_CONFIRM).add_attributes([
            (events::attribute::CONNECTION_ID, connection_id.to_string()),
            (
                events::attribute::CLIENT_ID,
                connection.client_id.to_string(),
            ),
            (
                events::attribute::COUNTERPARTY_CLIENT_ID,
                connection.counterparty_client_id.to_string(),
            ),
            (
                events::attribute::COUNTERPARTY_CONNECTION_ID,
                connection.counterparty_connection_id.to_string(),
            ),
        ]),
    ))
}

fn channel_open_init(
    mut deps: DepsMut,
    port_id: String,
    counterparty_port_id: Bytes,
    connection_id: u32,
    version: String,
    relayer: Addr,
) -> ContractResult {
    let port_id = deps.api.addr_validate(&port_id)?;
    ensure_connection_state(deps.as_ref(), connection_id)?;
    let (channel_id, _) = create_channel(
        deps.branch(),
        port_id.clone(),
        ChannelState::Init,
        connection_id,
        0,
        counterparty_port_id.clone(),
        version.clone(),
    )?;
    Ok(Response::new()
        .add_event(Event::new(events::channel::OPEN_INIT).add_attributes([
            (events::attribute::PORT_ID, port_id.to_string()),
            (events::attribute::CHANNEL_ID, channel_id.to_string()),
            (
                events::attribute::COUNTERPARTY_PORT_ID,
                hex::encode(&counterparty_port_id),
            ),
            (events::attribute::CONNECTION_ID, connection_id.to_string()),
            (events::attribute::VERSION, version.clone()),
        ]))
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenInit {
                connection_id,
                channel_id,
                version,
                relayer: relayer.into(),
            }),
            vec![],
        )?))
}

fn channel_open_try(
    mut deps: DepsMut,
    port_id: String,
    channel: Channel,
    counterparty_version: String,
    proof_init: Vec<u8>,
    proof_height: u64,
    relayer: Addr,
) -> ContractResult {
    if channel.state != ChannelState::TryOpen {
        return Err(ContractError::ChannelInvalidState {
            got: channel.state,
            expected: ChannelState::TryOpen,
        });
    }
    let connection = ensure_connection_state(deps.as_ref(), channel.connection_id)?;
    let expected_channel = Channel {
        state: ChannelState::Init,
        connection_id: connection.counterparty_connection_id,
        counterparty_channel_id: 0,
        counterparty_port_id: port_id.as_bytes().to_vec().into(),
        version: counterparty_version.clone(),
    };
    let client_impl = client_impl(deps.as_ref(), connection.client_id)?;
    query_light_client::<()>(
        deps.as_ref(),
        client_impl,
        LightClientQuery::VerifyMembership {
            client_id: connection.client_id,
            height: proof_height,
            proof: proof_init.into(),
            path: ChannelPath {
                channel_id: channel.counterparty_channel_id,
            }
            .key()
            .into_bytes(),
            value: commit(expected_channel.abi_encode()).into_bytes(),
        },
    )?;
    let port_id = deps.api.addr_validate(&port_id)?;
    let (channel_id, channel) = create_channel(
        deps.branch(),
        port_id.clone(),
        channel.state,
        channel.connection_id,
        channel.counterparty_channel_id,
        channel.counterparty_port_id,
        channel.version,
    )?;
    Ok(Response::new()
        .add_event(Event::new(events::channel::OPEN_TRY).add_attributes([
            (events::attribute::PORT_ID, port_id.to_string()),
            (events::attribute::CHANNEL_ID, channel_id.to_string()),
            (
                events::attribute::COUNTERPARTY_PORT_ID,
                hex::encode(&channel.counterparty_port_id),
            ),
            (
                events::attribute::COUNTERPARTY_CHANNEL_ID,
                channel.counterparty_channel_id.to_string(),
            ),
            ("connection_id", channel.connection_id.to_string()),
            ("counterparty_version", counterparty_version.clone()),
        ]))
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenTry {
                connection_id: channel.connection_id,
                channel_id,
                version: channel.version,
                counterparty_version,
                relayer: relayer.into(),
            }),
            vec![],
        )?))
}

fn channel_open_ack(
    mut deps: DepsMut,
    channel_id: u32,
    counterparty_version: String,
    counterparty_channel_id: u32,
    proof_try: Vec<u8>,
    proof_height: u64,
    relayer: Addr,
) -> ContractResult {
    let mut channel = CHANNELS.load(deps.storage, channel_id)?;
    if channel.state != ChannelState::Init {
        return Err(ContractError::ChannelInvalidState {
            got: channel.state,
            expected: ChannelState::Init,
        });
    }
    let connection = ensure_connection_state(deps.as_ref(), channel.connection_id)?;
    let port_id = CHANNEL_OWNER.load(deps.storage, channel_id)?;
    let expected_channel = Channel {
        state: ChannelState::TryOpen,
        connection_id: connection.counterparty_connection_id,
        counterparty_channel_id: channel_id,
        counterparty_port_id: port_id.as_bytes().to_vec().into(),
        version: counterparty_version.clone(),
    };
    let client_impl = client_impl(deps.as_ref(), connection.client_id)?;
    query_light_client::<()>(
        deps.as_ref(),
        client_impl,
        LightClientQuery::VerifyMembership {
            client_id: connection.client_id,
            height: proof_height,
            proof: proof_try.into(),
            path: ChannelPath {
                channel_id: counterparty_channel_id,
            }
            .key()
            .into_bytes(),
            value: commit(expected_channel.abi_encode()).into_bytes(),
        },
    )?;
    channel.state = ChannelState::Open;
    channel.version = counterparty_version.clone();
    channel.counterparty_channel_id = counterparty_channel_id;
    save_channel(deps.branch(), channel_id, &channel)?;
    Ok(Response::new()
        .add_event(Event::new(events::channel::OPEN_ACK).add_attributes([
            (events::attribute::PORT_ID, port_id.to_string()),
            (events::attribute::CHANNEL_ID, channel_id.to_string()),
            (
                events::attribute::COUNTERPARTY_PORT_ID,
                hex::encode(&channel.counterparty_port_id),
            ),
            (
                events::attribute::COUNTERPARTY_CHANNEL_ID,
                channel.counterparty_channel_id.to_string(),
            ),
            ("connection_id", channel.connection_id.to_string()),
        ]))
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenAck {
                channel_id,
                counterparty_channel_id,
                counterparty_version,
                relayer: relayer.into(),
            }),
            vec![],
        )?))
}

fn channel_open_confirm(
    mut deps: DepsMut,
    channel_id: u32,
    proof_ack: Vec<u8>,
    proof_height: u64,
    relayer: Addr,
) -> ContractResult {
    let mut channel = CHANNELS.load(deps.storage, channel_id)?;
    if channel.state != ChannelState::TryOpen {
        return Err(ContractError::ChannelInvalidState {
            got: channel.state,
            expected: ChannelState::TryOpen,
        });
    }
    let connection = ensure_connection_state(deps.as_ref(), channel.connection_id)?;
    let port_id = CHANNEL_OWNER.load(deps.storage, channel_id)?;
    let expected_channel = Channel {
        state: ChannelState::Open,
        connection_id: connection.counterparty_connection_id,
        counterparty_channel_id: channel_id,
        counterparty_port_id: port_id.clone().as_bytes().to_vec().into(),
        version: channel.version.clone(),
    };
    let client_impl = client_impl(deps.as_ref(), connection.client_id)?;
    query_light_client::<()>(
        deps.as_ref(),
        client_impl,
        LightClientQuery::VerifyMembership {
            client_id: connection.client_id,
            height: proof_height,
            proof: proof_ack.into(),
            path: ChannelPath {
                channel_id: channel.counterparty_channel_id,
            }
            .key()
            .into_bytes(),
            value: commit(expected_channel.abi_encode()).into_bytes(),
        },
    )?;
    channel.state = ChannelState::Open;
    save_channel(deps.branch(), channel_id, &channel)?;
    Ok(Response::new()
        .add_event(Event::new(events::channel::OPEN_CONFIRM).add_attributes([
            (events::attribute::PORT_ID, port_id.to_string()),
            (events::attribute::CHANNEL_ID, channel_id.to_string()),
            (
                events::attribute::COUNTERPARTY_PORT_ID,
                hex::encode(&channel.counterparty_port_id),
            ),
            (
                events::attribute::COUNTERPARTY_CHANNEL_ID,
                channel.counterparty_channel_id.to_string(),
            ),
            ("connection_id", channel.connection_id.to_string()),
        ]))
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenConfirm {
                channel_id,
                relayer: relayer.into(),
            }),
            vec![],
        )?))
}

fn channel_close_init(mut deps: DepsMut, channel_id: u32, relayer: Addr) -> ContractResult {
    let mut channel = CHANNELS.load(deps.storage, channel_id)?;
    if channel.state != ChannelState::Open {
        return Err(ContractError::ChannelInvalidState {
            got: channel.state,
            expected: ChannelState::Open,
        });
    }
    ensure_connection_state(deps.as_ref(), channel.connection_id)?;
    channel.state = ChannelState::Closed;
    save_channel(deps.branch(), channel_id, &channel)?;
    let port_id = CHANNEL_OWNER.load(deps.storage, channel_id)?;
    Ok(Response::new()
        .add_event(Event::new(events::channel::CLOSE_INIT).add_attributes([
            (events::attribute::PORT_ID, port_id.to_string()),
            (events::attribute::CHANNEL_ID, channel_id.to_string()),
            (
                events::attribute::COUNTERPARTY_PORT_ID,
                hex::encode(&channel.counterparty_port_id),
            ),
            (
                events::attribute::COUNTERPARTY_CHANNEL_ID,
                channel.counterparty_channel_id.to_string(),
            ),
        ]))
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnChannelCloseInit {
                channel_id,
                relayer: relayer.into(),
            }),
            vec![],
        )?))
}

fn channel_close_confirm(
    mut deps: DepsMut,
    channel_id: u32,
    proof_init: Vec<u8>,
    proof_height: u64,
    relayer: Addr,
) -> ContractResult {
    let mut channel = CHANNELS.load(deps.storage, channel_id)?;
    if channel.state != ChannelState::Open {
        return Err(ContractError::ChannelInvalidState {
            got: channel.state,
            expected: ChannelState::Open,
        });
    }
    let connection = ensure_connection_state(deps.as_ref(), channel.connection_id)?;
    let port_id = CHANNEL_OWNER.load(deps.storage, channel_id)?;
    let expected_channel = Channel {
        state: ChannelState::Closed,
        connection_id: connection.counterparty_connection_id,
        counterparty_channel_id: channel_id,
        counterparty_port_id: port_id.as_bytes().to_vec().into(),
        version: channel.version.clone(),
    };
    let client_impl = client_impl(deps.as_ref(), connection.client_id)?;
    query_light_client::<()>(
        deps.as_ref(),
        client_impl,
        LightClientQuery::VerifyMembership {
            client_id: connection.client_id,
            height: proof_height,
            proof: proof_init.into(),
            path: ChannelPath {
                channel_id: channel.counterparty_channel_id,
            }
            .key()
            .into_bytes(),
            value: commit(expected_channel.abi_encode()).into_bytes(),
        },
    )?;
    channel.state = ChannelState::Closed;
    CHANNELS.save(deps.storage, channel_id, &channel)?;
    store_commit(
        deps.branch(),
        &ChannelPath { channel_id }.key(),
        &commit(channel.abi_encode()),
    )?;
    Ok(Response::new()
        .add_event(Event::new(events::channel::CLOSE_CONFIRM).add_attributes([
            (events::attribute::PORT_ID, port_id.to_string()),
            (events::attribute::CHANNEL_ID, channel_id.to_string()),
            (
                events::attribute::COUNTERPARTY_PORT_ID,
                hex::encode(&channel.counterparty_port_id),
            ),
            (
                events::attribute::COUNTERPARTY_CHANNEL_ID,
                channel.counterparty_channel_id.to_string(),
            ),
        ]))
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenConfirm {
                channel_id,
                relayer: relayer.into(),
            }),
            vec![],
        )?))
}

#[allow(clippy::too_many_arguments)]
fn process_receive(
    mut deps: DepsMut,
    env: Env,
    packets: Vec<Packet>,
    relayer_msgs: Vec<alloy::primitives::Bytes>,
    relayer: String,
    proof: alloy::primitives::Bytes,
    proof_height: u64,
    intent: bool,
) -> Result<Response, ContractError> {
    let first = packets.first().ok_or(ContractError::NotEnoughPackets)?;
    let source_channel = first.source_channel_id;
    let destination_channel = first.destination_channel_id;

    let channel = ensure_channel_state(deps.as_ref(), destination_channel)?;
    let connection = ensure_connection_state(deps.as_ref(), channel.connection_id)?;

    if !intent {
        let proof_commitment_key = BatchPacketsPath {
            channel_id: source_channel,
            batch_hash: match packets.len() {
                1 => commit_packet(first),
                _ => commit_packets(&packets),
            },
        }
        .key();

        let client_impl = client_impl(deps.as_ref(), connection.client_id)?;
        query_light_client::<()>(
            deps.as_ref(),
            client_impl,
            LightClientQuery::VerifyMembership {
                client_id: connection.client_id,
                height: proof_height,
                proof: proof.to_vec().into(),
                path: proof_commitment_key.into_bytes(),
                value: COMMITMENT_MAGIC.into_bytes(),
            },
        )?;
    }

    let mut events = Vec::with_capacity(packets.len());
    let mut messages = Vec::with_capacity(packets.len());
    let port_id = CHANNEL_OWNER.load(deps.storage, destination_channel)?;
    for (packet, relayer_msg) in packets.into_iter().zip(relayer_msgs) {
        if packet.timeout_height > 0 && (env.block.height >= packet.timeout_height) {
            return Err(ContractError::ReceivedTimedOutPacketHeight {
                timeout_height: packet.timeout_height,
                current_height: env.block.height,
            });
        }

        let current_timestamp = env.block.time.nanos();
        if packet.timeout_timestamp != 0 && (current_timestamp >= packet.timeout_timestamp) {
            return Err(ContractError::ReceivedTimedOutPacketTimestamp {
                timeout_timestamp: packet.timeout_timestamp,
                current_timestamp,
            });
        }

        let commitment_key = BatchReceiptsPath {
            channel_id: destination_channel,
            batch_hash: commit_packet(&packet),
        }
        .key();

        if !set_packet_receive(deps.branch(), commitment_key) {
            if intent {
                events.push(
                    Event::new(events::packet::INTENT_RECV).add_attributes([
                        (
                            events::attribute::PACKET,
                            serde_json::to_string(&packet)
                                .expect("packet serialization is infallible; qed;"),
                        ),
                        (events::attribute::MAKER, relayer.clone()),
                        (events::attribute::MAKER_MSG, hex::encode(&relayer_msg)),
                    ]),
                );

                messages.push(wasm_execute(
                    port_id.clone(),
                    &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnIntentRecvPacket {
                        packet,
                        market_maker: deps.api.addr_validate(&relayer)?.into(),
                        market_maker_msg: relayer_msg.to_vec().into(),
                    }),
                    vec![],
                )?);
            } else {
                events.push(
                    Event::new(events::packet::RECV).add_attributes([
                        (
                            events::attribute::PACKET,
                            serde_json::to_string(&packet)
                                .expect("packet serialization is infallible; qed;"),
                        ),
                        (events::attribute::MAKER, relayer.clone()),
                        (events::attribute::MAKER_MSG, hex::encode(&relayer_msg)),
                    ]),
                );

                messages.push(wasm_execute(
                    port_id.clone(),
                    &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnRecvPacket {
                        packet,
                        relayer: deps.api.addr_validate(&relayer)?.into(),
                        relayer_msg: relayer_msg.to_vec().into(),
                    }),
                    vec![],
                )?);
            }
        }
    }

    Ok(Response::new().add_events(events).add_messages(messages))
}

fn write_acknowledgement(
    mut deps: DepsMut,
    sender: Addr,
    channel_id: u32,
    packet: Packet,
    acknowledgement: Vec<u8>,
) -> ContractResult {
    if acknowledgement.is_empty() {
        return Err(ContractError::AcknowledgementIsEmpty);
    }

    // make sure the caller owns the channel
    let port_id = CHANNEL_OWNER.load(deps.storage, channel_id)?;
    if port_id != sender {
        return Err(ContractError::Unauthorized {
            channel_id,
            owner: port_id,
            caller: sender,
        });
    }

    let commitment_key = BatchReceiptsPath {
        channel_id,
        batch_hash: commit_packet(&packet),
    }
    .key();

    // the receipt must present, but ack shouldn't
    match deps.storage.get(commitment_key.as_ref()) {
        Some(commitment) => {
            if commitment != COMMITMENT_MAGIC.into_bytes().into_vec() {
                return Err(ContractError::AlreadyAcknowledged);
            }
        }
        None => return Err(ContractError::PacketNotReceived),
    }

    store_commit(
        deps.branch(),
        &commitment_key,
        &commit_ack(&acknowledgement.clone().into()),
    )?;

    Ok(
        Response::new().add_event(Event::new("write_ack").add_attributes([
            (
                events::attribute::PACKET,
                serde_json::to_string(&packet).expect("packet serialization is infallible; qed;"),
            ),
            (
                events::attribute::ACKNOWLEDGEMENT,
                hex::encode(acknowledgement),
            ),
        ])),
    )
}

fn send_packet(
    mut deps: DepsMut,
    sender: Addr,
    source_channel_id: u32,
    timeout_height: u64,
    timeout_timestamp: u64,
    data: Vec<u8>,
) -> ContractResult {
    if timeout_timestamp == 0 && timeout_height == 0 {
        return Err(ContractError::TimeoutMustBeSet);
    }

    let port_id = CHANNEL_OWNER.load(deps.storage, source_channel_id)?;
    if port_id != sender {
        return Err(ContractError::Unauthorized {
            channel_id: source_channel_id,
            owner: port_id,
            caller: sender,
        });
    }

    let channel = ensure_channel_state(deps.as_ref(), source_channel_id)?;
    let packet = Packet {
        source_channel_id,
        destination_channel_id: channel.counterparty_channel_id,
        data: data.into(),
        timeout_height,
        timeout_timestamp,
    };

    let commitment_key = BatchPacketsPath {
        channel_id: source_channel_id,
        batch_hash: commit_packet(&packet),
    }
    .key();

    if deps.storage.get(commitment_key.as_ref()).is_some() {
        return Err(ContractError::PacketCommitmentAlreadyExist);
    }

    store_commit(deps.branch(), &commitment_key, &COMMITMENT_MAGIC)?;

    Ok(Response::new()
        .add_event(Event::new(events::packet::SEND).add_attribute(
            events::attribute::PACKET,
            serde_json::to_string(&packet).expect("packet serialization is infallible; qed;"),
        ))
        .set_data(to_json_binary(&packet)?))
}

fn increment(deps: DepsMut, item: Item<u32>) -> Result<u32, ContractError> {
    item.update(deps.storage, |item| {
        item.checked_add(1).ok_or(ContractError::ArithmeticOverflow)
    })
}

fn next_channel_id(deps: DepsMut) -> Result<u32, ContractError> {
    increment(deps, NEXT_CHANNEL_ID)
}

fn next_connection_id(deps: DepsMut) -> Result<u32, ContractError> {
    increment(deps, NEXT_CONNECTION_ID)
}

fn next_client_id(deps: DepsMut) -> Result<u32, ContractError> {
    increment(deps, NEXT_CLIENT_ID)
}

fn client_impl(deps: Deps, client_id: u32) -> Result<Addr, ContractError> {
    Ok(CLIENT_IMPLS.load(deps.storage, client_id)?)
}

fn commit(bytes: impl AsRef<[u8]>) -> H256 {
    keccak256(bytes)
}

fn commit_ack(ack: &Bytes) -> H256 {
    merge_ack(commit(ack))
}

fn commit_acks(acks: &Vec<Bytes>) -> H256 {
    merge_ack(commit(acks.abi_encode()))
}

fn merge_ack(mut ack: H256) -> H256 {
    ack.get_mut()[0] = 0x01;
    ack
}

fn store_commit(deps: DepsMut, key: &H256, value: &H256) -> Result<(), ContractError> {
    deps.storage.set(key.as_ref(), value.as_ref());
    Ok(())
}

fn read_commit(deps: Deps, key: &H256) -> Option<H256> {
    deps.storage.get(key.as_ref()).map(|bz| {
        bz.try_into()
            .expect("H256 is the only value ever written to this storage; qed;")
    })
}

fn save_connection(
    deps: DepsMut,
    connection_id: u32,
    connection: &Connection,
) -> Result<(), ContractError> {
    CONNECTIONS.save(deps.storage, connection_id, connection)?;
    store_commit(
        deps,
        &ConnectionPath { connection_id }.key(),
        &commit(connection.abi_encode_params()),
    )?;
    Ok(())
}

fn create_channel(
    mut deps: DepsMut,
    owner: Addr,
    state: ChannelState,
    connection_id: u32,
    counterparty_channel_id: u32,
    counterparty_port_id: Bytes,
    version: String,
) -> Result<(u32, Channel), ContractError> {
    let channel_id = next_channel_id(deps.branch())?;
    let channel = Channel {
        state,
        connection_id,
        counterparty_channel_id,
        counterparty_port_id,
        version,
    };
    CHANNEL_OWNER.save(deps.storage, channel_id, &owner)?;
    CONTRACT_CHANNELS.update(deps.storage, owner, |v| -> Result<_, ContractError> {
        Ok(match v {
            Some(mut set) => {
                let inserted = set.insert(channel_id);
                assert!(inserted, "impossible, channel has been just created");
                set
            }
            None => BTreeSet::from([channel_id]),
        })
    })?;
    save_channel(deps, channel_id, &channel)?;
    Ok((channel_id, channel))
}

fn save_channel(deps: DepsMut, channel_id: u32, channel: &Channel) -> Result<(), ContractError> {
    CHANNELS.save(deps.storage, channel_id, channel)?;
    store_commit(
        deps,
        &ChannelPath { channel_id }.key(),
        &commit(channel.abi_encode()),
    )?;
    Ok(())
}

fn ensure_connection_state(deps: Deps, connection_id: u32) -> Result<Connection, ContractError> {
    let connection = CONNECTIONS.load(deps.storage, connection_id)?;
    if connection.state != ConnectionState::Open {
        Err(ContractError::ConnectionInvalidState {
            got: connection.state,
            expected: ConnectionState::Open,
        })
    } else {
        Ok(connection)
    }
}

fn ensure_channel_state(deps: Deps, channel_id: u32) -> Result<Channel, ContractError> {
    let channel = CHANNELS.load(deps.storage, channel_id)?;
    if channel.state != ChannelState::Open {
        Err(ContractError::ChannelInvalidState {
            got: channel.state,
            expected: ChannelState::Open,
        })
    } else {
        Ok(channel)
    }
}

fn set_packet_receive(deps: DepsMut, commitment_key: H256) -> bool {
    if deps.storage.get(commitment_key.as_ref()).is_some() {
        true
    } else {
        deps.storage
            .set(commitment_key.as_ref(), COMMITMENT_MAGIC.as_ref());
        false
    }
}

fn get_timestamp_at_height(deps: Deps, client_id: u32, height: u64) -> Result<u64, ContractError> {
    let client_impl = client_impl(deps, client_id)?;
    let timestamp = query_light_client(
        deps,
        client_impl,
        LightClientQuery::GetTimestamp { client_id, height },
    )?;
    Ok(timestamp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetClientType { client_id } => Ok(to_json_binary(
            &CLIENT_TYPES.load(deps.storage, client_id)?,
        )?),
        QueryMsg::GetClientImpl { client_id } => {
            Ok(to_json_binary(&client_impl(deps, client_id)?)?)
        }
        QueryMsg::GetRegisteredClientType { client_type } => Ok(to_json_binary(
            &CLIENT_REGISTRY.load(deps.storage, &client_type)?,
        )?),
        QueryMsg::GetTimestampAtHeight { client_id, height } => Ok(to_json_binary(
            &get_timestamp_at_height(deps, client_id, height)?,
        )?),
        QueryMsg::GetLatestHeight { client_id } => {
            let client_impl = client_impl(deps, client_id)?;
            let latest_height = query_light_client::<u64>(
                deps,
                client_impl,
                LightClientQuery::GetLatestHeight { client_id },
            )?;
            Ok(to_json_binary(&latest_height)?)
        }
        QueryMsg::GetClientState { client_id } => {
            let client_state = CLIENT_STATES.load(deps.storage, client_id)?;
            Ok(to_json_binary(&client_state)?)
        }
        QueryMsg::GetConsensusState { client_id, height } => {
            let consensus_state =
                CLIENT_CONSENSUS_STATES.load(deps.storage, (client_id, height))?;
            Ok(to_json_binary(&consensus_state)?)
        }
        QueryMsg::GetStatus { client_id } => {
            let client_impl = client_impl(deps, client_id)?;
            let status = query_light_client::<Status>(
                deps,
                client_impl,
                LightClientQuery::GetStatus { client_id },
            )?;
            Ok(to_json_binary(&status)?)
        }
        QueryMsg::GetChannels { contract } => {
            let contract = deps.api.addr_validate(&contract)?;
            let channels = CONTRACT_CHANNELS.load(deps.storage, contract)?;
            Ok(to_json_binary(&channels)?)
        }
        QueryMsg::GetChannel { channel_id } => {
            let channel = CHANNELS.load(deps.storage, channel_id)?;
            Ok(to_json_binary(&channel)?)
        }
        QueryMsg::GetConnection { connection_id } => {
            let connection = CONNECTIONS.load(deps.storage, connection_id)?;
            Ok(to_json_binary(&connection)?)
        }
        QueryMsg::GetBatchPackets {
            channel_id,
            batch_hash,
        } => {
            let commit = read_commit(
                deps,
                &BatchPacketsPath {
                    channel_id,
                    batch_hash,
                }
                .key(),
            );
            Ok(to_json_binary(&commit)?)
        }
        QueryMsg::GetBatchReceipts {
            channel_id,
            batch_hash,
        } => {
            let commit = read_commit(
                deps,
                &BatchReceiptsPath {
                    channel_id,
                    batch_hash,
                }
                .key(),
            );
            Ok(to_json_binary(&commit)?)
        }
    }
}

fn query_light_client<T: DeserializeOwned>(
    deps: Deps,
    client_impl: Addr,
    query: LightClientQuery,
) -> Result<T, ContractError> {
    deps.querier
        .query_wasm_smart::<T>(&client_impl, &query)
        .map_err(|error| ContractError::CannotQueryLightClient {
            client_impl,
            query: Box::new(query),
            error,
        })
}

fn make_verify_creation_event(client_id: u32, event: VerifyCreationResponseEvent) -> Event {
    match event {
        VerifyCreationResponseEvent::CreateLensClient {
            l1_client_id,
            l2_client_id,
            l2_chain_id,
        } => Event::new("create_lens_client").add_attributes([
            ("client_id", client_id.to_string()),
            ("l1_client_id", l1_client_id.to_string()),
            ("l2_client_id", l2_client_id.to_string()),
            ("l2_chain_id", l2_chain_id),
        ]),
    }
}

#[cfg(test)]
mod tests {
    use alloy::hex;

    use super::*;

    #[test]
    fn channel_value() {
        let channel = Channel {
            state: ChannelState::Init,
            connection_id: 0,
            counterparty_channel_id: 0,
            counterparty_port_id: hex!("30783735366536393666366533313337373033393732376137373665366536363738363336613730333333323735366533393735363733373739363836383761363737343662363837363663333936613636366237333761373436373737333537353638333633393737363136333332373036373733333936383337373736613637").into(),
            version: "ucs01-relay-1".to_owned()
        };

        let value = commit(channel.abi_encode());

        dbg!(value);
        dbg!("0x68361972d5315b7a497e342405661930a6bdb0c17ce58a87227bb676fbcfc3ce");
        dbg!("0x9f4901a9b797640d3d2507111018b5130d209eb7305bdda6ed3163a6ec4d4c9b");
    }
}
