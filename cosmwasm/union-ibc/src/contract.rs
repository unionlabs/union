use alloy::{primitives::Bytes, sol_types::SolValue};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, wasm_execute, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response,
};
use cw_storage_plus::Item;
use ibc_solidity::cosmwasm::types::ibc::{
    Channel, ChannelState, Connection, ConnectionState, MsgBatchAcks, MsgBatchSend,
    MsgChannelCloseConfirm, MsgChannelCloseInit, MsgChannelOpenAck, MsgChannelOpenConfirm,
    MsgChannelOpenInit, MsgChannelOpenTry, MsgConnectionOpenAck, MsgConnectionOpenConfirm,
    MsgConnectionOpenInit, MsgConnectionOpenTry, MsgCreateClient, MsgIntentPacketRecv,
    MsgPacketAcknowledgement, MsgPacketRecv, MsgPacketTimeout, MsgUpdateClient, Packet,
};
use unionlabs::{
    ethereum::keccak256,
    hash::{hash_v2::HexPrefixed, H256},
    ics24::ethabi::COMMITMENT_MAGIC,
};

use crate::{
    lightclient::query::{QueryMsg as LightClientQuery, VerifyClientMessageUpdate},
    module::msg::ExecuteMsg as ModuleMsg,
    msg::{ExecuteMsg, InitMsg, MsgRegisterClient, MsgSendPacket, MsgWriteAcknowledgement},
    query::QueryMsg,
    state::{
        CHANNELS, CHANNEL_OWNER, CLIENT_CONSENSUS_STATES, CLIENT_IMPLS, CLIENT_REGISTRY,
        CLIENT_STATES, CLIENT_TYPES, CONNECTIONS, NEXT_CHANNEL_ID, NEXT_CLIENT_ID,
        NEXT_CONNECTION_ID,
    },
    ContractError,
};

type ContractResult = Result<Response, ContractError>;

pub mod events {
    pub mod client {
        pub const REGISTER: &str = "client_register";
        pub const CREATE: &str = "client_create";
        pub const UPDATE: &str = "client_update";
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
        pub const INTENT_RECV: &str = "packet_intent_recv";
        pub const ACK: &str = "packet_ack";
        pub const TIMEOUT: &str = "packet_timeout";
        pub const BATCH_SEND: &str = "batch_send";
        pub const BATCH_ACKS: &str = "batch_acks";
    }
    pub mod attribute {
        pub const CLIENT_ID: &str = "client_id";
        pub const CONNECTION_ID: &str = "connection_id";
        pub const CHANNEL_ID: &str = "channel_id";
        pub const COUNTERPARTY_CHANNEL_ID: &str = "counterpary_channel_id";
        pub const HEIGHT: &str = "height";
        pub const PACKET: &str = "packet";
        pub const PACKETS: &str = "packets";
        pub const ACKS: &str = "acks";
        pub const MAKER: &str = "maker";
        pub const MAKER_MSG: &str = "maker_msg";
        pub const ACKNOWLEDGEMENT: &str = "acknowledgement";
        pub const CLIENT_TYPE: &str = "client_type";
        pub const CLIENT_ADDRESS: &str = "client_address";
        pub const COUNTERPARTY_CLIENT_ID: &str = "counterparty_client_id";
        pub const COUNTERPARTY_CONNECTION_ID: &str = "counterparty_connection_id";
        pub const PORT_ID: &str = "port_id";
        pub const COUNTERPARTY_PORT_ID: &str = "port_id";
        pub const VERSION: &str = "version";
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InitMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
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
        }) => register_client(deps.branch(), client_type, client_address),
        ExecuteMsg::CreateClient(MsgCreateClient {
            clientType,
            clientStateBytes,
            consensusStateBytes,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            create_client(
                deps.branch(),
                clientType,
                clientStateBytes.to_vec(),
                consensusStateBytes.to_vec(),
                relayer,
            )
        }
        ExecuteMsg::UpdateClient(MsgUpdateClient {
            clientId,
            clientMessage,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            update_client(deps.branch(), clientId, clientMessage.to_vec(), relayer)
        }
        ExecuteMsg::ConnectionOpenInit(MsgConnectionOpenInit {
            clientId,
            counterpartyClientId,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            connection_open_init(deps.branch(), clientId, counterpartyClientId, relayer)
        }
        ExecuteMsg::ConnectionOpenTry(MsgConnectionOpenTry {
            counterpartyClientId,
            counterpartyConnectionId,
            clientId,
            proofInit,
            proofHeight,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            connection_open_try(
                deps.branch(),
                counterpartyClientId,
                counterpartyConnectionId,
                clientId,
                proofInit.to_vec(),
                proofHeight,
                relayer,
            )
        }
        ExecuteMsg::ConnectionOpenAck(MsgConnectionOpenAck {
            connectionId,
            counterpartyConnectionId,
            proofTry,
            proofHeight,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            connection_open_ack(
                deps.branch(),
                connectionId,
                counterpartyConnectionId,
                proofTry.to_vec(),
                proofHeight,
                relayer,
            )
        }
        ExecuteMsg::ConnectionOpenConfirm(MsgConnectionOpenConfirm {
            connectionId,
            proofAck,
            proofHeight,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            connection_open_confirm(
                deps.branch(),
                connectionId,
                proofAck.to_vec(),
                proofHeight,
                relayer,
            )
        }
        ExecuteMsg::ChannelOpenInit(MsgChannelOpenInit {
            portId,
            counterpartyPortId,
            connectionId,
            version,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            channel_open_init(
                deps.branch(),
                portId,
                counterpartyPortId,
                connectionId,
                version,
                relayer,
            )
        }
        ExecuteMsg::ChannelOpenTry(MsgChannelOpenTry {
            portId,
            channel,
            counterpartyVersion,
            proofInit,
            proofHeight,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            channel_open_try(
                deps.branch(),
                portId,
                channel,
                counterpartyVersion,
                proofInit.to_vec(),
                proofHeight,
                relayer,
            )
        }
        ExecuteMsg::ChannelOpenAck(MsgChannelOpenAck {
            channelId,
            counterpartyVersion,
            counterpartyChannelId,
            proofTry,
            proofHeight,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            channel_open_ack(
                deps.branch(),
                channelId,
                counterpartyVersion,
                counterpartyChannelId,
                proofTry.to_vec(),
                proofHeight,
                relayer,
            )
        }
        ExecuteMsg::ChannelOpenConfirm(MsgChannelOpenConfirm {
            channelId,
            proofAck,
            proofHeight,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            channel_open_confirm(
                deps.branch(),
                channelId,
                proofAck.to_vec(),
                proofHeight,
                relayer,
            )
        }
        ExecuteMsg::ChannelCloseInit(MsgChannelCloseInit { channelId, relayer }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            channel_close_init(deps.branch(), channelId, relayer)
        }
        ExecuteMsg::ChannelCloseConfirm(MsgChannelCloseConfirm {
            channelId,
            proofInit,
            proofHeight,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            channel_close_confirm(
                deps.branch(),
                channelId,
                proofInit.to_vec(),
                proofHeight,
                relayer,
            )
        }
        ExecuteMsg::PacketRecv(MsgPacketRecv {
            packets,
            relayerMsgs,
            relayer,
            proof,
            proofHeight,
        }) => process_receive(
            deps,
            env,
            packets,
            relayerMsgs,
            relayer,
            proof,
            proofHeight,
            false,
        ),
        ExecuteMsg::PacketAck(MsgPacketAcknowledgement {
            packets,
            acknowledgements,
            proof,
            proofHeight,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            acknowledge_packet(
                deps.branch(),
                packets,
                acknowledgements,
                proof.to_vec(),
                proofHeight,
                relayer,
            )
        }
        ExecuteMsg::PacketTimeout(MsgPacketTimeout {
            packet,
            proof,
            proofHeight,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            timeout_packet(deps.branch(), packet, proof.to_vec(), proofHeight, relayer)
        }
        ExecuteMsg::IntentPacketRecv(MsgIntentPacketRecv {
            packets,
            marketMakerMsgs,
            marketMaker,
            emptyProof,
        }) => process_receive(
            deps,
            env,
            packets,
            marketMakerMsgs,
            marketMaker,
            emptyProof,
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
            acknowledgement,
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
            data,
        ),
        ExecuteMsg::BatchSend(MsgBatchSend {
            sourceChannel,
            packets,
        }) => batch_send(deps, sourceChannel, packets),
        ExecuteMsg::BatchAcks(MsgBatchAcks {
            sourceChannel,
            packets,
            acks,
        }) => batch_acks(deps, sourceChannel, packets, acks),
    }
}

fn batch_send(deps: DepsMut, source_channel: u32, packets: Vec<Packet>) -> ContractResult {
    if packets.len() < 2 {
        return Err(ContractError::NotEnoughPackets);
    }
    for packet in &packets {
        let commitment_key =
            unionlabs::ics24::ethabi::batch_packets_key(source_channel, commit_packet(packet));
        let commitment = deps
            .storage
            .get(commitment_key.as_ref())
            .unwrap_or(H256::<HexPrefixed>::default().into_bytes());
        if commitment != COMMITMENT_MAGIC.as_ref() {
            return Err(ContractError::PacketCommitmentNotFound);
        }
    }
    store_commit(
        deps,
        &unionlabs::ics24::ethabi::batch_packets_key(source_channel, commit_packets(&packets)),
        &COMMITMENT_MAGIC,
    )?;
    Ok(
        Response::new().add_event(Event::new(events::packet::BATCH_SEND).add_attributes([
            (events::attribute::CHANNEL_ID, source_channel.to_string()),
            (
                events::attribute::PACKETS,
                serde_json::to_string(&packets).unwrap(),
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
        let commitment_key =
            unionlabs::ics24::ethabi::batch_receipts_key(source_channel, commit_packet(packet));
        let commitment = deps.storage.get(commitment_key.as_ref());
        match commitment {
            Some(acknowledgement) => {
                if acknowledgement == COMMITMENT_MAGIC.as_ref() {
                    return Err(ContractError::AcknowledgementIsEmpty);
                } else if acknowledgement != commit_ack(ack.to_vec()).as_ref() {
                    return Err(ContractError::AcknowledgementMismatch);
                }
            }
            None => return Err(ContractError::PacketCommitmentNotFound),
        }
    }
    store_commit(
        deps,
        &unionlabs::ics24::ethabi::batch_receipts_key(source_channel, commit_packets(&packets)),
        &commit_acks(&acks),
    )?;
    Ok(
        Response::new().add_event(Event::new(events::packet::BATCH_ACKS).add_attributes([
            (events::attribute::CHANNEL_ID, source_channel.to_string()),
            (
                events::attribute::PACKETS,
                serde_json::to_string(&packets).unwrap(),
            ),
            (
                events::attribute::ACKS,
                serde_json::to_string(&acks).unwrap(),
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
    let source_channel = packet.sourceChannel;
    let destination_channel = packet.destinationChannel;
    let channel = ensure_channel_state(deps.as_ref(), source_channel)?;
    let connection = ensure_connection_state(deps.as_ref(), channel.connectionId)?;

    let proof_timestamp =
        get_timestamp_at_height(deps.as_ref(), connection.clientId, proof_height)?;
    if proof_timestamp == 0 {
        return Err(ContractError::TimeoutProofTimestampNotFound);
    }

    let commitment_key =
        unionlabs::ics24::ethabi::batch_receipts_key(destination_channel, commit_packet(&packet));

    let client_impl = client_impl(deps.as_ref(), connection.clientId)?;
    deps.querier.query_wasm_smart::<()>(
        &client_impl,
        &LightClientQuery::VerifyNonMembership {
            client_id: connection.clientId,
            height: proof_height,
            proof: proof.to_vec().into(),
            path: commitment_key.into_bytes().into(),
        },
    )?;
    delete_packet_commitment(deps.branch(), source_channel, &packet)?;

    if packet.timeoutTimestamp == 0 && packet.timeoutHeight == 0 {
        return Err(ContractError::TimeoutMustBeSet);
    }

    if packet.timeoutTimestamp > 0 && packet.timeoutTimestamp > proof_timestamp {
        return Err(ContractError::TimeoutTimestampNotReached);
    }

    if packet.timeoutHeight > 0 && packet.timeoutHeight > proof_height {
        return Err(ContractError::TimeoutHeightNotReached);
    }

    let port_id = CHANNEL_OWNER.load(deps.storage, source_channel)?;
    Ok(Response::new()
        .add_event(Event::new(events::packet::TIMEOUT).add_attributes([
            (
                events::attribute::PACKET,
                serde_json::to_string(&packet).unwrap(),
            ),
            (events::attribute::MAKER, relayer.to_string()),
        ]))
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::OnTimeoutPacket { packet, relayer },
            vec![],
        )?))
}

fn acknowledge_packet(
    mut deps: DepsMut,
    packets: Vec<Packet>,
    acknowledgements: Vec<alloy::primitives::Bytes>,
    proof: Vec<u8>,
    proof_height: u64,
    relayer: Addr,
) -> ContractResult {
    let first = packets.first().ok_or(ContractError::NotEnoughPackets)?;
    let source_channel = first.sourceChannel;
    let destination_channel = first.destinationChannel;
    let channel = ensure_channel_state(deps.as_ref(), source_channel)?;
    let connection = ensure_connection_state(deps.as_ref(), channel.connectionId)?;

    let commitment_key = match packets.len() {
        1 => {
            unionlabs::ics24::ethabi::batch_receipts_key(destination_channel, commit_packet(first))
        }
        _ => unionlabs::ics24::ethabi::batch_receipts_key(
            destination_channel,
            commit_packets(&packets),
        ),
    };

    let client_impl = client_impl(deps.as_ref(), connection.clientId)?;
    deps.querier.query_wasm_smart::<()>(
        &client_impl,
        &LightClientQuery::VerifyMembership {
            client_id: connection.clientId,
            height: proof_height,
            proof: proof.to_vec().into(),
            path: commitment_key.into_bytes().into(),
            value: commit_acks(&acknowledgements).into_bytes().into(),
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
                serde_json::to_string(&packet).unwrap(),
            ),
            (events::attribute::ACKNOWLEDGEMENT, hex::encode(&ack)),
            (events::attribute::MAKER, relayer.clone().to_string()),
        ]));
        messages.push(wasm_execute(
            port_id.clone(),
            &ModuleMsg::OnAcknowledgementPacket {
                packet,
                acknowledgement: ack.to_vec().into(),
                relayer: relayer.clone(),
            },
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
    let commitment_key =
        unionlabs::ics24::ethabi::batch_packets_key(source_channel, commit_packet(packet));
    let commitment = deps
        .storage
        .get(commitment_key.as_ref())
        .unwrap_or(H256::<HexPrefixed>::default().into_bytes());
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
    let latest_height = deps.querier.query_wasm_smart(
        &client_impl,
        &LightClientQuery::VerifyCreation {
            client_id,
            client_state: client_state_bytes.to_vec().into(),
            consensus_state: consensus_state_bytes.to_vec().into(),
        },
    )?;
    CLIENT_STATES.save(deps.storage, client_id, &client_state_bytes.to_vec().into())?;
    CLIENT_CONSENSUS_STATES.save(
        deps.storage,
        (client_id, latest_height),
        &consensus_state_bytes.to_vec().into(),
    )?;
    store_commit(
        deps.branch(),
        &unionlabs::ics24::ethabi::client_state_key(client_id),
        &commit(client_state_bytes),
    )?;
    store_commit(
        deps,
        &unionlabs::ics24::ethabi::consensus_state_key(client_id, latest_height),
        &commit(consensus_state_bytes),
    )?;
    Ok(
        Response::new().add_event(Event::new(events::client::CREATE).add_attributes([
            (events::attribute::CLIENT_TYPE, client_type),
            (events::attribute::CLIENT_ID, client_id.to_string()),
        ])),
    )
}

fn update_client(
    mut deps: DepsMut,
    client_id: u32,
    client_message: Vec<u8>,
    _relayer: Addr,
) -> Result<Response, ContractError> {
    let client_impl = client_impl(deps.as_ref(), client_id)?;
    let update = deps.querier.query_wasm_smart::<VerifyClientMessageUpdate>(
        &client_impl,
        &LightClientQuery::VerifyClientMessage {
            client_id,
            message: client_message.to_vec().into(),
        },
    )?;
    store_commit(
        deps.branch(),
        &unionlabs::ics24::ethabi::client_state_key(client_id),
        &commit(update.client_state),
    )?;
    store_commit(
        deps.branch(),
        &unionlabs::ics24::ethabi::consensus_state_key(client_id, update.height),
        &commit(update.consensus_state),
    )?;
    Ok(
        Response::new().add_event(Event::new(events::client::UPDATE).add_attributes([
            (events::attribute::CLIENT_ID, client_id.to_string()),
            (events::attribute::HEIGHT, update.height.to_string()),
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
        clientId: client_id,
        counterpartyClientId: counterparty_client_id,
        counterpartyConnectionId: 0,
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
        clientId: client_id,
        counterpartyClientId: counterparty_client_id,
        counterpartyConnectionId: counterparty_connection_id,
    };
    let expected_connection = Connection {
        state: ConnectionState::Init,
        clientId: counterparty_client_id,
        counterpartyClientId: client_id,
        counterpartyConnectionId: 0,
    };
    let client_impl = client_impl(deps.as_ref(), client_id)?;
    deps.querier.query_wasm_smart::<()>(
        &client_impl,
        &LightClientQuery::VerifyMembership {
            client_id,
            height: proof_height,
            proof: proof_init.to_vec().into(),
            path: unionlabs::ics24::ethabi::connection_key(counterparty_connection_id)
                .into_bytes()
                .into(),
            value: commit(expected_connection.abi_encode()).into_bytes().into(),
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
        clientId: connection.counterpartyClientId,
        counterpartyClientId: connection.clientId,
        counterpartyConnectionId: connection_id,
    };
    let client_impl = client_impl(deps.as_ref(), connection.clientId)?;
    deps.querier.query_wasm_smart::<()>(
        &client_impl,
        &LightClientQuery::VerifyMembership {
            client_id: connection.clientId,
            height: proof_height,
            proof: proof_try.to_vec().into(),
            path: unionlabs::ics24::ethabi::connection_key(counterparty_connection_id)
                .into_bytes()
                .into(),
            value: commit(expected_connection.abi_encode()).into_bytes().into(),
        },
    )?;
    connection.state = ConnectionState::Open;
    connection.counterpartyConnectionId = counterparty_connection_id;
    save_connection(deps.branch(), connection_id, &connection)?;
    Ok(
        Response::new().add_event(Event::new(events::connection::OPEN_ACK).add_attributes([
            (events::attribute::CONNECTION_ID, connection_id.to_string()),
            (
                events::attribute::CLIENT_ID,
                connection.clientId.to_string(),
            ),
            (
                events::attribute::COUNTERPARTY_CLIENT_ID,
                connection.counterpartyClientId.to_string(),
            ),
            (
                events::attribute::COUNTERPARTY_CONNECTION_ID,
                connection.counterpartyConnectionId.to_string(),
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
        clientId: connection.counterpartyClientId,
        counterpartyClientId: connection.clientId,
        counterpartyConnectionId: connection_id,
    };
    let client_impl = client_impl(deps.as_ref(), connection.clientId)?;
    deps.querier.query_wasm_smart::<()>(
        &client_impl,
        &LightClientQuery::VerifyMembership {
            client_id: connection.clientId,
            height: proof_height,
            proof: proof_ack.to_vec().into(),
            path: unionlabs::ics24::ethabi::connection_key(connection.counterpartyConnectionId)
                .into_bytes()
                .into(),
            value: commit(expected_connection.abi_encode()).into_bytes().into(),
        },
    )?;
    connection.state = ConnectionState::Open;
    save_connection(deps.branch(), connection_id, &connection)?;
    Ok(Response::new().add_event(
        Event::new(events::connection::OPEN_CONFIRM).add_attributes([
            (events::attribute::CONNECTION_ID, connection_id.to_string()),
            (
                events::attribute::CLIENT_ID,
                connection.clientId.to_string(),
            ),
            (
                events::attribute::COUNTERPARTY_CLIENT_ID,
                connection.counterpartyClientId.to_string(),
            ),
            (
                events::attribute::COUNTERPARTY_CONNECTION_ID,
                connection.counterpartyConnectionId.to_string(),
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
    let channel_id = next_channel_id(deps.branch())?;
    let channel = Channel {
        state: ChannelState::Init,
        connectionId: connection_id,
        counterpartyChannelId: 0,
        counterpartyPortId: counterparty_port_id.clone(),
        version: version.clone(),
    };
    save_channel(deps.branch(), channel_id, &channel)?;
    CHANNEL_OWNER.save(deps.storage, channel_id, &port_id)?;
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
            &ModuleMsg::OnChannelOpenInit {
                connection_id,
                channel_id,
                version,
                relayer,
            },
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
    let connection = ensure_connection_state(deps.as_ref(), channel.connectionId)?;
    let expected_channel = Channel {
        state: ChannelState::Init,
        connectionId: connection.counterpartyConnectionId,
        counterpartyChannelId: 0,
        counterpartyPortId: port_id.as_bytes().to_vec().into(),
        version: counterparty_version.clone(),
    };
    let client_impl = client_impl(deps.as_ref(), connection.clientId)?;
    deps.querier.query_wasm_smart::<()>(
        &client_impl,
        &LightClientQuery::VerifyMembership {
            client_id: connection.clientId,
            height: proof_height,
            proof: proof_init.to_vec().into(),
            path: unionlabs::ics24::ethabi::channel_key(channel.counterpartyChannelId)
                .into_bytes()
                .into(),
            value: commit(expected_channel.abi_encode()).into_bytes().into(),
        },
    )?;
    let channel_id = next_channel_id(deps.branch())?;
    let port_id = deps.api.addr_validate(&port_id)?;
    save_channel(deps.branch(), channel_id, &channel)?;
    CHANNEL_OWNER.save(deps.storage, channel_id, &port_id)?;
    Ok(Response::new()
        .add_event(Event::new(events::channel::OPEN_TRY).add_attributes([
            (events::attribute::PORT_ID, port_id.to_string()),
            (events::attribute::CHANNEL_ID, channel_id.to_string()),
            (
                events::attribute::COUNTERPARTY_PORT_ID,
                hex::encode(&channel.counterpartyPortId),
            ),
            (
                events::attribute::COUNTERPARTY_CHANNEL_ID,
                channel.counterpartyChannelId.to_string(),
            ),
            ("connection_id", channel.connectionId.to_string()),
            ("counterparty_version", counterparty_version.clone()),
        ]))
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::OnChannelOpenTry {
                connection_id: channel.connectionId,
                channel_id,
                version: channel.version,
                counterparty_version,
                relayer,
            },
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
    let connection = ensure_connection_state(deps.as_ref(), channel.connectionId)?;
    let port_id = CHANNEL_OWNER.load(deps.storage, channel_id)?;
    let expected_channel = Channel {
        state: ChannelState::TryOpen,
        connectionId: connection.counterpartyConnectionId,
        counterpartyChannelId: channel_id,
        counterpartyPortId: port_id.as_bytes().to_vec().into(),
        version: counterparty_version.clone(),
    };
    let client_impl = client_impl(deps.as_ref(), connection.clientId)?;
    deps.querier.query_wasm_smart::<()>(
        &client_impl,
        &LightClientQuery::VerifyMembership {
            client_id: connection.clientId,
            height: proof_height,
            proof: proof_try.to_vec().into(),
            path: unionlabs::ics24::ethabi::channel_key(counterparty_channel_id)
                .into_bytes()
                .into(),
            value: commit(expected_channel.abi_encode()).into_bytes().into(),
        },
    )?;
    channel.state = ChannelState::Open;
    channel.version = counterparty_version.clone();
    channel.counterpartyChannelId = counterparty_channel_id;
    save_channel(deps.branch(), channel_id, &channel)?;
    Ok(Response::new()
        .add_event(Event::new(events::channel::OPEN_ACK).add_attributes([
            (events::attribute::PORT_ID, port_id.to_string()),
            (events::attribute::CHANNEL_ID, channel_id.to_string()),
            (
                events::attribute::COUNTERPARTY_PORT_ID,
                hex::encode(&channel.counterpartyPortId),
            ),
            (
                events::attribute::COUNTERPARTY_CHANNEL_ID,
                channel.counterpartyChannelId.to_string(),
            ),
            ("connection_id", channel.connectionId.to_string()),
        ]))
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::OnChannelOpenAck {
                channel_id,
                counterparty_channel_id,
                counterparty_version,
                relayer,
            },
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
    let connection = ensure_connection_state(deps.as_ref(), channel.connectionId)?;
    let port_id = CHANNEL_OWNER.load(deps.storage, channel_id)?;
    let expected_channel = Channel {
        state: ChannelState::Open,
        connectionId: connection.counterpartyConnectionId,
        counterpartyChannelId: channel_id,
        counterpartyPortId: port_id.clone().as_bytes().to_vec().into(),
        version: channel.version.clone(),
    };
    let client_impl = client_impl(deps.as_ref(), connection.clientId)?;
    deps.querier.query_wasm_smart::<()>(
        &client_impl,
        &LightClientQuery::VerifyMembership {
            client_id: connection.clientId,
            height: proof_height,
            proof: proof_ack.to_vec().into(),
            path: unionlabs::ics24::ethabi::channel_key(channel.counterpartyChannelId)
                .into_bytes()
                .into(),
            value: commit(expected_channel.abi_encode()).into_bytes().into(),
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
                hex::encode(&channel.counterpartyPortId),
            ),
            (
                events::attribute::COUNTERPARTY_CHANNEL_ID,
                channel.counterpartyChannelId.to_string(),
            ),
            ("connection_id", channel.connectionId.to_string()),
        ]))
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::OnChannelOpenConfirm {
                channel_id,
                relayer,
            },
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
    ensure_connection_state(deps.as_ref(), channel.connectionId)?;
    channel.state = ChannelState::Closed;
    save_channel(deps.branch(), channel_id, &channel)?;
    let port_id = CHANNEL_OWNER.load(deps.storage, channel_id)?;
    Ok(Response::new()
        .add_event(Event::new(events::channel::CLOSE_INIT).add_attributes([
            (events::attribute::PORT_ID, port_id.to_string()),
            (events::attribute::CHANNEL_ID, channel_id.to_string()),
            (
                events::attribute::COUNTERPARTY_PORT_ID,
                hex::encode(&channel.counterpartyPortId),
            ),
            (
                events::attribute::COUNTERPARTY_CHANNEL_ID,
                channel.counterpartyChannelId.to_string(),
            ),
        ]))
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::OnChannelCloseInit {
                channel_id,
                relayer,
            },
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
    let connection = ensure_connection_state(deps.as_ref(), channel.connectionId)?;
    let port_id = CHANNEL_OWNER.load(deps.storage, channel_id)?;
    let expected_channel = Channel {
        state: ChannelState::Closed,
        connectionId: connection.counterpartyConnectionId,
        counterpartyChannelId: channel_id,
        counterpartyPortId: port_id.as_bytes().to_vec().into(),
        version: channel.version.clone(),
    };
    let client_impl = client_impl(deps.as_ref(), connection.clientId)?;
    deps.querier.query_wasm_smart::<()>(
        &client_impl,
        &LightClientQuery::VerifyMembership {
            client_id: connection.clientId,
            height: proof_height,
            proof: proof_init.to_vec().into(),
            path: unionlabs::ics24::ethabi::channel_key(channel.counterpartyChannelId)
                .into_bytes()
                .into(),
            value: commit(expected_channel.abi_encode()).into_bytes().into(),
        },
    )?;
    channel.state = ChannelState::Closed;
    CHANNELS.save(deps.storage, channel_id, &channel)?;
    store_commit(
        deps.branch(),
        &unionlabs::ics24::ethabi::channel_key(channel_id),
        &commit(channel.abi_encode()),
    )?;
    Ok(Response::new()
        .add_event(Event::new(events::channel::CLOSE_CONFIRM).add_attributes([
            (events::attribute::PORT_ID, port_id.to_string()),
            (events::attribute::CHANNEL_ID, channel_id.to_string()),
            (
                events::attribute::COUNTERPARTY_PORT_ID,
                hex::encode(&channel.counterpartyPortId),
            ),
            (
                events::attribute::COUNTERPARTY_CHANNEL_ID,
                channel.counterpartyChannelId.to_string(),
            ),
        ]))
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::OnChannelOpenConfirm {
                channel_id,
                relayer,
            },
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
    let source_channel = first.sourceChannel;
    let destination_channel = first.destinationChannel;

    let channel = ensure_channel_state(deps.as_ref(), destination_channel)?;
    let connection = ensure_connection_state(deps.as_ref(), channel.connectionId)?;

    if !intent {
        let proof_commitment_key = match packets.len() {
            1 => unionlabs::ics24::ethabi::batch_receipts_key(source_channel, commit_packet(first)),
            _ => unionlabs::ics24::ethabi::batch_receipts_key(
                source_channel,
                commit_packets(&packets),
            ),
        };

        let client_impl = client_impl(deps.as_ref(), connection.clientId)?;
        deps.querier.query_wasm_smart::<()>(
            &client_impl,
            &LightClientQuery::VerifyMembership {
                client_id: connection.clientId,
                height: proof_height,
                proof: proof.to_vec().into(),
                path: proof_commitment_key.into_bytes().into(),
                value: COMMITMENT_MAGIC.into_bytes().into(),
            },
        )?;
    }

    let mut events = vec![];
    let mut messages = vec![];
    let port_id = CHANNEL_OWNER.load(deps.storage, destination_channel)?;
    for (packet, relayer_msg) in packets.into_iter().zip(relayer_msgs) {
        if packet.timeoutHeight > 0 && (env.block.height >= packet.timeoutHeight) {
            return Err(ContractError::ReceivedTimedOutPacketHeight {
                timeout_height: packet.timeoutHeight,
                current_height: env.block.height,
            });
        }

        let current_timestamp = env.block.time.nanos();
        if packet.timeoutTimestamp != 0 && (current_timestamp >= packet.timeoutTimestamp) {
            return Err(ContractError::ReceivedTimedOutPacketTimestamp {
                timeout_timestamp: packet.timeoutTimestamp,
                current_timestamp,
            });
        }

        let commitment_key = unionlabs::ics24::ethabi::batch_receipts_key(
            destination_channel,
            commit_packet(&packet),
        );

        if !set_packet_receive(deps.branch(), commitment_key) {
            if intent {
                events.push(Event::new(events::packet::INTENT_RECV).add_attributes([
                    (
                        events::attribute::PACKET,
                        serde_json::to_string(&packet).unwrap(),
                    ),
                    (events::attribute::MAKER, relayer.clone()),
                    (events::attribute::MAKER_MSG, hex::encode(&relayer_msg)),
                ]));

                messages.push(wasm_execute(
                    port_id.clone(),
                    &ModuleMsg::OnIntentRecvPacket {
                        packet,
                        market_maker: deps.api.addr_validate(&relayer)?,
                        market_maker_msg: relayer_msg.to_vec().into(),
                    },
                    vec![],
                )?);
            } else {
                events.push(Event::new(events::packet::RECV).add_attributes([
                    (
                        events::attribute::PACKET,
                        serde_json::to_string(&packet).unwrap(),
                    ),
                    (events::attribute::MAKER, relayer.clone()),
                    (events::attribute::MAKER_MSG, hex::encode(&relayer_msg)),
                ]));

                messages.push(wasm_execute(
                    port_id.clone(),
                    &ModuleMsg::OnRecvPacket {
                        packet,
                        relayer: deps.api.addr_validate(&relayer)?,
                        relayer_msg: relayer_msg.to_vec().into(),
                    },
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
    // make sure the caller owns the channel
    let port_id = CHANNEL_OWNER.load(deps.storage, channel_id)?;
    if port_id != sender {
        return Err(ContractError::Unauthorized);
    }

    let commitment_key =
        unionlabs::ics24::ethabi::batch_receipts_key(channel_id, commit_packet(&packet));

    // the receipt must present, but ack shouldn't
    match deps.storage.get(commitment_key.as_ref()) {
        Some(commitment) => {
            if commitment != COMMITMENT_MAGIC.into_bytes() {
                return Err(ContractError::AlreadyAcknowledged);
            }
        }
        None => return Err(ContractError::PacketNotReceived),
    }

    store_commit(
        deps.branch(),
        &commitment_key,
        &commit_ack(acknowledgement.clone()),
    )?;

    Ok(
        Response::new().add_event(Event::new("write_acknowledgement").add_attributes([
            (
                events::attribute::PACKET,
                serde_json::to_string(&packet).unwrap(),
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
    source_channel: u32,
    timeout_height: u64,
    timeout_timestamp: u64,
    data: Vec<u8>,
) -> ContractResult {
    if timeout_timestamp == 0 && timeout_height == 0 {
        return Err(ContractError::TimeoutMustBeSet);
    }

    let port_id = CHANNEL_OWNER.load(deps.storage, source_channel)?;
    if port_id != sender {
        return Err(ContractError::Unauthorized);
    }

    let channel = ensure_channel_state(deps.as_ref(), source_channel)?;
    let packet = Packet {
        sourceChannel: source_channel,
        destinationChannel: channel.counterpartyChannelId,
        data: data.into(),
        timeoutHeight: timeout_height,
        timeoutTimestamp: timeout_timestamp,
    };

    let commitment_key =
        unionlabs::ics24::ethabi::batch_packets_key(source_channel, commit_packet(&packet));

    if deps.storage.get(commitment_key.as_ref()).is_some() {
        return Err(ContractError::PacketCommitmentAlreadyExist);
    }

    store_commit(deps.branch(), &commitment_key, &COMMITMENT_MAGIC)?;

    Ok(Response::new()
        .add_event(Event::new("send_packet").add_attribute(
            events::attribute::PACKET,
            serde_json::to_string(&packet).unwrap(),
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

fn commit_ack(ack: Vec<u8>) -> H256 {
    merge_ack(commit(ack))
}

fn commit_acks(acks: &Vec<alloy::primitives::Bytes>) -> H256 {
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

fn save_connection(
    deps: DepsMut,
    connection_id: u32,
    connection: &Connection,
) -> Result<(), ContractError> {
    CONNECTIONS.save(deps.storage, connection_id, connection)?;
    store_commit(
        deps,
        &unionlabs::ics24::ethabi::connection_key(connection_id),
        &commit(connection.abi_encode()),
    )?;
    Ok(())
}

fn save_channel(deps: DepsMut, channel_id: u32, channel: &Channel) -> Result<(), ContractError> {
    CHANNELS.save(deps.storage, channel_id, channel)?;
    store_commit(
        deps,
        &unionlabs::ics24::ethabi::channel_key(channel_id),
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
    let consensus_state = CLIENT_CONSENSUS_STATES.load(deps.storage, (client_id, height))?;
    let timestamp = deps.querier.query_wasm_smart(
        client_impl,
        &LightClientQuery::GetTimestamp { consensus_state },
    )?;
    Ok(timestamp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetClientType { client_id } => Ok(to_json_binary(
            &CLIENT_TYPES.load(deps.storage, client_id)?,
        )?),
        QueryMsg::GetTimestampAtHeight { client_id, height } => Ok(to_json_binary(
            &get_timestamp_at_height(deps, client_id, height)?,
        )?),
        QueryMsg::GetLatestHeight { client_id } => {
            let client_impl = client_impl(deps, client_id)?;
            let client_state = CLIENT_STATES.load(deps.storage, client_id)?;
            let latest_height = deps.querier.query_wasm_smart::<u64>(
                client_impl,
                &LightClientQuery::GetLatestHeight { client_state },
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
            let client_state = CLIENT_STATES.load(deps.storage, client_id)?;
            Ok(deps
                .querier
                .query_wasm_smart(client_impl, &LightClientQuery::GetStatus { client_state })?)
        }
    }
}
