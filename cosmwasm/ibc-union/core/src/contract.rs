use std::{collections::BTreeSet, num::NonZeroU32};

use alloy_sol_types::SolValue;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, wasm_execute, Addr, Attribute, Binary, Deps, DepsMut, Env, Event, MessageInfo,
    OverflowError, OverflowOperation, Response, StdError, StdResult, Storage,
};
use depolama::{RawStore, StorageExt};
use frissitheto::{UpgradeError, UpgradeMsg};
use ibc_union_msg::{
    lightclient::{
        QueryMsg as LightClientQuery, Status, UpdateStateResponse, VerifyCreationResponse,
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
        commit_packets, BatchPacketsPath, BatchReceiptsPath, ChannelPath, ClientStatePath,
        ConnectionPath, ConsensusStatePath, COMMITMENT_MAGIC, COMMITMENT_MAGIC_ACK,
    },
    Channel, ChannelId, ChannelState, ClientId, Connection, ConnectionId, ConnectionState, Packet,
    Timestamp,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use unionlabs::{
    ethereum::keccak256,
    primitives::{Bytes, H256},
};

use crate::{
    state::{
        ChannelOwner, Channels, ClientConsensusStates, ClientImpls, ClientRegistry, ClientStates,
        ClientStore, ClientTypes, Commitments, Connections, ContractChannels, NextChannelId,
        NextClientId, NextConnectionId, QueryStore, WhitelistedRelayers, WhitelistedRelayersAdmin,
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
        pub const WRITE_ACK: &str = "write_ack";
    }
    pub mod attribute {
        pub const CLIENT_ID: &str = "client_id";
        pub const CONNECTION_ID: &str = "connection_id";
        pub const CHANNEL_ID: &str = "channel_id";
        pub const COUNTERPARTY_CHANNEL_ID: &str = "counterparty_channel_id";
        pub const COUNTERPARTY_HEIGHT: &str = "counterparty_height";
        pub const PACKET_HASH: &str = "packet_hash";
        pub const PACKET_SOURCE_CHANNEL_ID: &str = "packet_source_channel_id";
        pub const PACKET_DESTINATION_CHANNEL_ID: &str = "packet_destination_channel_id";
        pub const PACKET_DATA: &str = "packet_data";
        pub const PACKET_TIMEOUT_HEIGHT: &str = "packet_timeout_height";
        pub const PACKET_TIMEOUT_TIMESTAMP: &str = "packet_timeout_timestamp";
        pub const BATCH_HASH: &str = "batch_hash";
        pub const MAKER: &str = "maker";
        pub const MAKER_MSG: &str = "maker_msg";
        pub const ACKNOWLEDGEMENT: &str = "acknowledgement";
        pub const ACKNOWLEDGEMENT_HASH: &str = "acknowledgement_hash";
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

#[must_use]
fn packet_to_attrs(packet: &Packet) -> [Attribute; 5] {
    [
        (
            events::attribute::PACKET_SOURCE_CHANNEL_ID,
            packet.source_channel_id.to_string(),
        ),
        (
            events::attribute::PACKET_DESTINATION_CHANNEL_ID,
            packet.destination_channel_id.to_string(),
        ),
        (events::attribute::PACKET_DATA, packet.data.to_string()),
        (
            events::attribute::PACKET_TIMEOUT_HEIGHT,
            packet.timeout_height.to_string(),
        ),
        (
            events::attribute::PACKET_TIMEOUT_TIMESTAMP,
            packet.timeout_timestamp.to_string(),
        ),
    ]
    .map(Into::into)
}

#[must_use]
fn packet_to_attr_hash(channel_id: ChannelId, packet: &Packet) -> [Attribute; 2] {
    [
        (events::attribute::CHANNEL_ID, channel_id.to_string()),
        (
            events::attribute::PACKET_HASH,
            commit_packets(&[packet.clone()]).to_string(),
        ),
    ]
    .map(Into::into)
}

fn ensure_relayer_admin(storage: &mut dyn Storage, sender: &Addr) -> Result<(), ContractError> {
    if sender != storage.read_item::<WhitelistedRelayersAdmin>()? {
        Err(ContractError::OnlyRelayerAdmin)
    } else {
        Ok(())
    }
}

fn ensure_relayer(storage: &mut dyn Storage, sender: &Addr) -> Result<(), ContractError> {
    if storage.read::<WhitelistedRelayers>(sender).is_ok() {
        Ok(())
    } else {
        Err(ContractError::OnlyWhitelistedRelayer)
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
            ensure_relayer(deps.storage, &info.sender)?;
            let address = deps.api.addr_validate(&client_address)?;
            register_client(deps.branch(), client_type, address)
        }
        ExecuteMsg::CreateClient(MsgCreateClient {
            client_type,
            client_state_bytes,
            consensus_state_bytes,
            relayer,
        }) => {
            ensure_relayer(deps.storage, &info.sender)?;
            let relayer = deps.api.addr_validate(&relayer)?;
            create_client(
                deps.branch(),
                info,
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
            ensure_relayer(deps.storage, &info.sender)?;
            let relayer = deps.api.addr_validate(&relayer)?;
            update_client(
                deps.branch(),
                info,
                client_id,
                client_message.to_vec(),
                relayer,
            )
        }
        ExecuteMsg::ConnectionOpenInit(MsgConnectionOpenInit {
            client_id,
            counterparty_client_id,
        }) => connection_open_init(deps.branch(), client_id, counterparty_client_id),
        ExecuteMsg::ConnectionOpenTry(MsgConnectionOpenTry {
            counterparty_client_id,
            counterparty_connection_id,
            client_id,
            proof_init,
            proof_height,
        }) => connection_open_try(
            info,
            deps.branch(),
            counterparty_client_id,
            counterparty_connection_id,
            client_id,
            proof_init.to_vec(),
            proof_height,
            true,
        ),
        ExecuteMsg::ForceConnectionOpenTry(MsgConnectionOpenTry {
            counterparty_client_id,
            counterparty_connection_id,
            client_id,
            proof_init,
            proof_height,
        }) => connection_open_try(
            info,
            deps.branch(),
            counterparty_client_id,
            counterparty_connection_id,
            client_id,
            proof_init.to_vec(),
            proof_height,
            false,
        ),
        ExecuteMsg::ConnectionOpenAck(MsgConnectionOpenAck {
            connection_id,
            counterparty_connection_id,
            proof_try,
            proof_height,
        }) => connection_open_ack(
            info,
            deps.branch(),
            connection_id,
            counterparty_connection_id,
            proof_try.to_vec(),
            proof_height,
            true,
        ),
        ExecuteMsg::ForceConnectionOpenAck(MsgConnectionOpenAck {
            connection_id,
            counterparty_connection_id,
            proof_try,
            proof_height,
        }) => connection_open_ack(
            info,
            deps.branch(),
            connection_id,
            counterparty_connection_id,
            proof_try.to_vec(),
            proof_height,
            false,
        ),
        ExecuteMsg::ConnectionOpenConfirm(MsgConnectionOpenConfirm {
            connection_id,
            proof_ack,
            proof_height,
        }) => connection_open_confirm(
            info,
            deps.branch(),
            connection_id,
            proof_ack.to_vec(),
            proof_height,
            true,
        ),
        ExecuteMsg::ForceConnectionOpenConfirm(MsgConnectionOpenConfirm {
            connection_id,
            proof_ack,
            proof_height,
        }) => connection_open_confirm(
            info,
            deps.branch(),
            connection_id,
            proof_ack.to_vec(),
            proof_height,
            false,
        ),
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
                info,
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
                info,
                port_id,
                channel,
                counterparty_version,
                proof_init.to_vec(),
                proof_height,
                relayer,
                true,
            )
        }
        ExecuteMsg::ForceChannelOpenTry(MsgChannelOpenTry {
            port_id,
            channel,
            counterparty_version,
            proof_init,
            proof_height,
            relayer,
        }) => {
            ensure_relayer_admin(deps.storage, &info.sender)?;
            let relayer = deps.api.addr_validate(&relayer)?;
            channel_open_try(
                deps.branch(),
                info,
                port_id,
                channel,
                counterparty_version,
                proof_init.to_vec(),
                proof_height,
                relayer,
                false,
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
                info,
                channel_id,
                counterparty_version,
                counterparty_channel_id,
                proof_try.to_vec(),
                proof_height,
                relayer,
                true,
            )
        }
        ExecuteMsg::ForceChannelOpenAck(MsgChannelOpenAck {
            channel_id,
            counterparty_version,
            counterparty_channel_id,
            proof_try,
            proof_height,
            relayer,
        }) => {
            ensure_relayer_admin(deps.storage, &info.sender)?;
            let relayer = deps.api.addr_validate(&relayer)?;
            channel_open_ack(
                deps.branch(),
                info,
                channel_id,
                counterparty_version,
                counterparty_channel_id,
                proof_try.to_vec(),
                proof_height,
                relayer,
                false,
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
                info,
                channel_id,
                proof_ack.to_vec(),
                proof_height,
                relayer,
                true,
            )
        }
        ExecuteMsg::ForceChannelOpenConfirm(MsgChannelOpenConfirm {
            channel_id,
            proof_ack,
            proof_height,
            relayer,
        }) => {
            ensure_relayer_admin(deps.storage, &info.sender)?;
            let relayer = deps.api.addr_validate(&relayer)?;
            channel_open_confirm(
                deps.branch(),
                info,
                channel_id,
                proof_ack.to_vec(),
                proof_height,
                relayer,
                false,
            )
        }
        ExecuteMsg::ChannelCloseInit(MsgChannelCloseInit {
            channel_id,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            channel_close_init(deps.branch(), info, channel_id, relayer)
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
                info,
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
        }) => {
            ensure_relayer(deps.storage, &info.sender)?;
            process_receive(
                deps,
                env,
                info,
                packets,
                relayer_msgs.into_iter().collect(),
                relayer,
                proof,
                proof_height,
                false,
            )
        }
        ExecuteMsg::PacketAck(MsgPacketAcknowledgement {
            packets,
            acknowledgements,
            proof,
            proof_height,
            relayer,
        }) => {
            ensure_relayer(deps.storage, &info.sender)?;
            let relayer = deps.api.addr_validate(&relayer)?;
            acknowledge_packet(
                deps.branch(),
                info,
                packets,
                acknowledgements.into_iter().collect(),
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
            ensure_relayer(deps.storage, &info.sender)?;
            let relayer = deps.api.addr_validate(&relayer)?;
            timeout_packet(
                deps.branch(),
                info,
                packet,
                proof.to_vec(),
                proof_height,
                relayer,
            )
        }
        ExecuteMsg::IntentPacketRecv(MsgIntentPacketRecv {
            packets,
            market_maker_msgs,
            market_maker,
            empty_proof,
        }) => {
            ensure_relayer(deps.storage, &info.sender)?;
            process_receive(
                deps,
                env,
                info,
                packets,
                market_maker_msgs.into_iter().collect(),
                market_maker,
                empty_proof,
                0,
                true,
            )
        }
        ExecuteMsg::WriteAcknowledgement(MsgWriteAcknowledgement {
            packet,
            acknowledgement,
        }) => write_acknowledgement(
            deps.branch(),
            info.sender,
            packet,
            acknowledgement.into_vec(),
        ),
        ExecuteMsg::PacketSend(MsgSendPacket {
            source_channel_id,
            timeout_height,
            timeout_timestamp,
            data,
        }) => send_packet(
            deps.branch(),
            info.sender,
            source_channel_id,
            timeout_height,
            timeout_timestamp,
            data.into_vec(),
        ),
        ExecuteMsg::BatchSend(MsgBatchSend { packets }) => {
            ensure_relayer(deps.storage, &info.sender)?;
            batch_send(deps, packets)
        }
        ExecuteMsg::BatchAcks(MsgBatchAcks { packets, acks }) => {
            ensure_relayer(deps.storage, &info.sender)?;
            batch_acks(deps, packets, acks.into_iter().collect())
        }
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
        ExecuteMsg::AddRelayer(relayer) => {
            ensure_relayer_admin(deps.storage, &info.sender)?;
            let relayer = deps.api.addr_validate(&relayer)?;
            deps.storage.write::<WhitelistedRelayers>(&relayer, &());
            Ok(Response::new().add_event(
                Event::new("whitelisted_relayers")
                    .add_attribute("action", "grant")
                    .add_attribute("relayer", relayer),
            ))
        }
        ExecuteMsg::RemoveRelayer(relayer) => {
            ensure_relayer_admin(deps.storage, &info.sender)?;
            let relayer = deps.api.addr_validate(&relayer)?;
            deps.storage.delete::<WhitelistedRelayers>(&relayer);
            Ok(Response::new().add_event(
                Event::new("whitelisted_relayers")
                    .add_attribute("action", "revoke")
                    .add_attribute("relayer", relayer),
            ))
        }
    }
}

fn migrate_state(
    mut deps: DepsMut,
    sender: Addr,
    client_id: ClientId,
    client_state: unionlabs::primitives::Bytes,
    consensus_state: unionlabs::primitives::Bytes,
    height: u64,
) -> Result<Response, ContractError> {
    let client_addr = deps.storage.read::<ClientImpls>(&client_id)?;

    if client_addr != sender {
        return Err(ContractError::UnauthorizedMigration {
            client_id,
            caller: sender,
            client: client_addr,
        });
    }

    deps.storage.upsert::<ClientStates, _>(&client_id, |s| {
        let _ = s.ok_or(ContractError::CannotMigrateWithNoClientState { client_id })?;
        Ok::<_, ContractError>(client_state.to_vec().into())
    })?;

    store_commit(
        deps.branch(),
        &ClientStatePath { client_id }.key(),
        &commit(client_state),
    );

    deps.storage
        .upsert::<ClientConsensusStates, _>(&(client_id, height), |s| {
            let _ =
                s.ok_or(ContractError::CannotMigrateWithNoConsensusState { client_id, height })?;
            Ok::<Bytes, ContractError>(consensus_state.to_vec().into())
        })?;

    store_commit(
        deps.branch(),
        &ConsensusStatePath { client_id, height }.key(),
        &commit(consensus_state),
    );

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(_: DepsMut, _: Env, _: MessageInfo, _: ()) -> StdResult<Response> {
    panic!("this contract cannot be instantiated directly, but must be migrated from an existing instantiated contract.");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct IbcUnionMigrateMsg {}

pub mod version {
    use std::num::NonZeroU32;

    use unionlabs::option_unwrap;

    pub const INIT: NonZeroU32 = option_unwrap!(NonZeroU32::new(1));
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    msg: UpgradeMsg<InitMsg, IbcUnionMigrateMsg>,
) -> Result<Response, ContractError> {
    msg.run(deps, init, |_deps, _migrate, version| match version {
        version::INIT => Ok((Response::new(), None)),
        _ => Err(UpgradeError::UnknownStateVersion(version).into()),
    })
}

pub(crate) fn init(
    deps: DepsMut<'_>,
    InitMsg {
        relayers_admin,
        relayers,
    }: InitMsg,
) -> Result<(Response, Option<NonZeroU32>), ContractError> {
    deps.storage.write_item::<NextChannelId>(&ChannelId!(1));
    deps.storage
        .write_item::<NextConnectionId>(&ConnectionId!(1));
    deps.storage.write_item::<NextClientId>(&ClientId!(1));
    if let Some(relayers_admin) = relayers_admin {
        let relayers_admin = deps.api.addr_validate(&relayers_admin)?;
        deps.storage
            .write_item::<WhitelistedRelayersAdmin>(&relayers_admin);
    }
    for relayer in relayers {
        let relayer = deps.api.addr_validate(&relayer)?;
        deps.storage.write::<WhitelistedRelayers>(&relayer, &());
    }
    Ok((Response::default(), None))
}

fn batch_send(deps: DepsMut, packets: Vec<Packet>) -> ContractResult {
    if packets.len() < 2 {
        return Err(ContractError::NotEnoughPackets);
    }
    let channel_id = packets[0].source_channel_id;
    let batch_hash = commit_packets(&packets);
    let batch_commitment_key = BatchPacketsPath::from_packets(&packets).key();
    let mut events = Vec::new();
    for packet in packets {
        events.push(
            Event::new(events::packet::BATCH_SEND)
                .add_attributes(packet_to_attr_hash(channel_id, &packet))
                .add_attribute(events::attribute::BATCH_HASH, batch_hash.to_string()),
        );
        if packet.source_channel_id != channel_id {
            return Err(ContractError::BatchSameChannelOnly);
        }
        let commitment_key = BatchPacketsPath::from_packets(&[packet]).key();

        let commitment = deps
            .storage
            .maybe_read::<Commitments>(&commitment_key)?
            .ok_or(ContractError::PacketCommitmentNotFound)?;

        if commitment != COMMITMENT_MAGIC {
            return Err(ContractError::PacketCommitmentNotFound);
        }
    }
    store_commit(deps, &batch_commitment_key, &COMMITMENT_MAGIC);
    Ok(Response::new().add_events(events))
}

fn batch_acks(deps: DepsMut, packets: Vec<Packet>, acks: Vec<Bytes>) -> ContractResult {
    if packets.len() < 2 {
        return Err(ContractError::NotEnoughPackets);
    }
    let channel_id = packets[0].destination_channel_id;
    let batch_hash = commit_packets(&packets);
    let batch_commitment_key = BatchReceiptsPath::from_packets(&packets).key();
    let mut events = Vec::new();
    for (packet, ack) in packets.into_iter().zip(acks.iter()) {
        events.push(
            Event::new(events::packet::BATCH_ACKS)
                .add_attributes(packet_to_attr_hash(channel_id, &packet))
                .add_attribute(events::attribute::BATCH_HASH, batch_hash.to_string()),
        );
        if packet.destination_channel_id != channel_id {
            return Err(ContractError::BatchSameChannelOnly);
        }
        let commitment_key = BatchReceiptsPath::from_packets(&[packet]).key();
        let commitment = read_commit(deps.as_ref(), &commitment_key);
        match commitment {
            Some(ack_commitment) => {
                let ack_commitment_expected = commit_ack(ack);
                if ack_commitment == COMMITMENT_MAGIC {
                    return Err(ContractError::AcknowledgementIsEmpty);
                } else if ack_commitment != ack_commitment_expected {
                    return Err(ContractError::AcknowledgementMismatch {
                        found: ack_commitment.into(),
                        expected: ack_commitment_expected.into(),
                    });
                }
            }
            None => return Err(ContractError::PacketCommitmentNotFound),
        }
    }
    store_commit(deps, &batch_commitment_key, &commit_acks(&acks));
    Ok(Response::new().add_events(events))
}

fn timeout_packet(
    mut deps: DepsMut,
    info: MessageInfo,
    packet: Packet,
    proof: Vec<u8>,
    proof_height: u64,
    relayer: Addr,
) -> ContractResult {
    let source_channel = packet.source_channel_id;
    let channel = ensure_channel_state(deps.as_ref(), source_channel)?;
    let connection = ensure_connection_state(deps.as_ref(), channel.connection_id)?;

    let proof_timestamp =
        get_timestamp_at_height(deps.as_ref(), connection.client_id, proof_height)?;
    if proof_timestamp.is_zero() {
        return Err(ContractError::TimeoutProofTimestampNotFound);
    }

    let commitment_key = BatchReceiptsPath::from_packets(&[packet.clone()]).key();

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
    mark_packet_as_acknowledged(deps.branch(), &packet)?;

    if packet.timeout_timestamp.is_zero() && packet.timeout_height == 0 {
        return Err(ContractError::TimeoutMustBeSet);
    }

    if !packet.timeout_timestamp.is_zero() && packet.timeout_timestamp > proof_timestamp {
        return Err(ContractError::TimeoutTimestampNotReached);
    }

    if !packet.timeout_timestamp.is_zero() && packet.timeout_height > proof_height {
        return Err(ContractError::TimeoutHeightNotReached);
    }

    let port_id = deps.storage.read::<ChannelOwner>(&source_channel)?;
    Ok(Response::new()
        .add_event(
            Event::new(events::packet::TIMEOUT)
                .add_attributes(packet_to_attr_hash(source_channel, &packet))
                .add_attributes([(events::attribute::MAKER, relayer.to_string())]),
        )
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnTimeoutPacket {
                caller: info.sender.into_string(),
                packet,
                relayer: relayer.into(),
            }),
            vec![],
        )?))
}

fn acknowledge_packet(
    mut deps: DepsMut,
    info: MessageInfo,
    packets: Vec<Packet>,
    acknowledgements: Vec<Bytes>,
    proof: Vec<u8>,
    proof_height: u64,
    relayer: Addr,
) -> ContractResult {
    let first = packets.first().ok_or(ContractError::NotEnoughPackets)?;

    let source_channel_id = first.source_channel_id;

    let channel = ensure_channel_state(deps.as_ref(), source_channel_id)?;
    let connection = ensure_connection_state(deps.as_ref(), channel.connection_id)?;

    let commitment_key = BatchReceiptsPath::from_packets(&packets).key();
    let commitment_value = commit_acks(&acknowledgements);

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

    let port_id = deps.storage.read::<ChannelOwner>(&source_channel_id)?;
    let mut events = Vec::with_capacity(packets.len());
    let mut messages = Vec::with_capacity(packets.len());
    for (packet, ack) in packets.into_iter().zip(acknowledgements) {
        if packet.source_channel_id != source_channel_id {
            return Err(ContractError::BatchSameChannelOnly);
        }
        mark_packet_as_acknowledged(deps.branch(), &packet)?;
        events.push(
            Event::new(events::packet::ACK)
                .add_attributes(packet_to_attr_hash(source_channel_id, &packet))
                .add_attributes([
                    (events::attribute::ACKNOWLEDGEMENT, hex::encode(&ack)),
                    (events::attribute::MAKER, relayer.clone().to_string()),
                ]),
        );
        messages.push(wasm_execute(
            port_id.clone(),
            &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnAcknowledgementPacket {
                caller: info.sender.clone().into_string(),
                packet,
                acknowledgement: ack.to_vec().into(),
                relayer: relayer.clone().into(),
            }),
            vec![],
        )?);
    }

    Ok(Response::new().add_events(events).add_messages(messages))
}

fn mark_packet_as_acknowledged(deps: DepsMut, packet: &Packet) -> Result<(), ContractError> {
    let commitment_key = BatchPacketsPath::from_packets(&[packet.clone()]).key();
    let commitment = deps
        .storage
        .maybe_read::<Commitments>(&commitment_key)?
        .ok_or(ContractError::PacketCommitmentNotFound)?;

    if commitment == COMMITMENT_MAGIC_ACK {
        return Err(ContractError::PacketAlreadyAcknowledged);
    }

    if commitment != COMMITMENT_MAGIC {
        return Err(ContractError::PacketCommitmentNotFound);
    }

    store_commit(deps, &commitment_key, &COMMITMENT_MAGIC_ACK);

    Ok(())
}

fn register_client(
    deps: DepsMut,
    client_type: String,
    client_address: Addr,
) -> Result<Response, ContractError> {
    let client_address = deps
        .storage
        .upsert::<ClientRegistry, _>(&client_type, |client_impl| match client_impl {
            Some(_) => Err(ContractError::ClientTypeAlreadyExists),
            None => Ok(client_address),
        })?;

    Ok(Response::new().add_event(
        Event::new(events::client::REGISTER)
            .add_attribute(events::attribute::CLIENT_TYPE, client_type)
            .add_attribute(events::attribute::CLIENT_ADDRESS, client_address),
    ))
}

fn create_client(
    mut deps: DepsMut,
    info: MessageInfo,
    client_type: String,
    client_state_bytes: Vec<u8>,
    consensus_state_bytes: Vec<u8>,
    relayer: Addr,
) -> Result<Response, ContractError> {
    let client_impl = deps.storage.read::<ClientRegistry>(&client_type)?;
    let client_id = next_client_id(deps.branch())?;
    deps.storage.write::<ClientTypes>(&client_id, &client_type);
    deps.storage.write::<ClientImpls>(&client_id, &client_impl);

    // Ugly hack to allow for >64K messages (not configurable) to be threaded for the query.
    // See https://github.com/CosmWasm/cosmwasm/blob/e17ecc44cdebc84de1caae648c7a4f4b56846f8f/packages/vm/src/imports.rs#L47

    // 1. write these states first, so they can be read by the light client contract during VerifyCreation
    deps.storage
        .write::<ClientStates>(&client_id, &client_state_bytes.to_vec().into());
    // 2. once the client state is saved, query the light client impl for the height of that client state (the state we just saved is the latest height)...
    let latest_height = query_light_client::<u64>(
        deps.as_ref(),
        client_impl.clone(),
        LightClientQuery::GetLatestHeight { client_id },
    )?;
    // 3. save the consensus state, so that it can be read during VerifyCreation as well
    deps.storage.write::<ClientConsensusStates>(
        &(client_id, latest_height),
        &consensus_state_bytes.to_vec().into(),
    );
    // 4. finally, call VerifyCreation, which will read the states we just stored
    let verify_creation_response = query_light_client::<VerifyCreationResponse>(
        deps.as_ref(),
        client_impl,
        LightClientQuery::VerifyCreation {
            caller: info.sender.into(),
            client_id,
            relayer: relayer.into(),
        },
    )?;
    if let Some(client_state_bytes) = verify_creation_response.client_state_bytes {
        // ...if VerifyCreation returns a new client state to save, overwrite the state we just wrote.
        deps.storage
            .write::<ClientStates>(&client_id, &client_state_bytes.to_vec().into());
    }
    store_commit(
        deps.branch(),
        &ClientStatePath { client_id }.key(),
        &commit(client_state_bytes),
    );
    store_commit(
        deps.branch(),
        &ConsensusStatePath {
            client_id,
            height: latest_height,
        }
        .key(),
        &commit(consensus_state_bytes),
    );

    for (k, v) in verify_creation_response.storage_writes {
        deps.storage
            .write::<ClientStore<RawStore>>(&(client_id, k), &v);
    }
    let response = verify_creation_response
        .events
        .into_iter()
        .fold(Response::new(), |response, e| {
            response.add_event(make_verify_creation_event(client_id, e))
        });
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
    info: MessageInfo,
    client_id: ClientId,
    client_message: Vec<u8>,
    relayer: Addr,
) -> Result<Response, ContractError> {
    let client_impl = client_impl(deps.as_ref(), client_id)?;
    let update = {
        let status = query_light_client::<Status>(
            deps.as_ref(),
            client_impl.clone(),
            LightClientQuery::GetStatus { client_id },
        )?;

        if !matches!(status, Status::Active) {
            return Err(ContractError::ClientNotActive { client_id, status });
        }

        // Ugly hack to allow for >64K messages (not configurable) to be threaded for the query.
        // See https://github.com/CosmWasm/cosmwasm/blob/e17ecc44cdebc84de1caae648c7a4f4b56846f8f/packages/vm/src/imports.rs#L47
        deps.storage
            .write_item::<QueryStore>(&client_message.into());

        let update = query_light_client::<UpdateStateResponse>(
            deps.as_ref(),
            client_impl,
            LightClientQuery::UpdateState {
                caller: info.sender.into(),
                client_id,
                relayer: relayer.into(),
            },
        )?;

        deps.storage.delete_item::<QueryStore>();

        update
    };

    if let Some(client_state_bytes) = update.client_state_bytes {
        store_commit(
            deps.branch(),
            &ClientStatePath { client_id }.key(),
            &commit(&client_state_bytes),
        );
        deps.storage
            .write::<ClientStates>(&client_id, &client_state_bytes.to_vec().into());
    }

    store_commit(
        deps.branch(),
        &ConsensusStatePath {
            client_id,
            height: update.height,
        }
        .key(),
        &commit(&update.consensus_state_bytes),
    );

    deps.storage.write::<ClientConsensusStates>(
        &(client_id, update.height),
        &update.consensus_state_bytes.into_vec().into(),
    );

    for (k, v) in update.storage_writes {
        deps.storage
            .write::<ClientStore<RawStore>>(&(client_id, k), &v);
    }

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
    client_id: ClientId,
    counterparty_client_id: ClientId,
) -> ContractResult {
    let connection_id = next_connection_id(deps.branch())?;
    let connection = Connection {
        state: ConnectionState::Init,
        client_id,
        counterparty_client_id,
        counterparty_connection_id: None,
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

#[allow(clippy::too_many_arguments)]
fn connection_open_try(
    info: MessageInfo,
    mut deps: DepsMut,
    counterparty_client_id: ClientId,
    counterparty_connection_id: ConnectionId,
    client_id: ClientId,
    proof_init: Vec<u8>,
    proof_height: u64,
    verify: bool,
) -> ContractResult {
    let connection_id = next_connection_id(deps.branch())?;
    let connection = Connection {
        state: ConnectionState::TryOpen,
        client_id,
        counterparty_client_id,
        counterparty_connection_id: Some(counterparty_connection_id),
    };
    let expected_connection = Connection {
        state: ConnectionState::Init,
        client_id: counterparty_client_id,
        counterparty_client_id: client_id,
        counterparty_connection_id: None,
    };
    if verify {
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
    } else {
        ensure_relayer_admin(deps.storage, &info.sender)?;
    }
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
    info: MessageInfo,
    mut deps: DepsMut,
    connection_id: ConnectionId,
    counterparty_connection_id: ConnectionId,
    proof_try: Vec<u8>,
    proof_height: u64,
    verify: bool,
) -> ContractResult {
    let mut connection = deps.storage.read::<Connections>(&connection_id)?;
    if connection.state != ConnectionState::Init {
        return Err(ContractError::ConnectionInvalidState {
            got: connection.state,
            expected: ConnectionState::Init,
        });
    }
    if verify {
        let expected_connection = Connection {
            state: ConnectionState::TryOpen,
            client_id: connection.counterparty_client_id,
            counterparty_client_id: connection.client_id,
            counterparty_connection_id: Some(connection_id),
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
    } else {
        ensure_relayer_admin(deps.storage, &info.sender)?;
    }
    connection.state = ConnectionState::Open;
    connection.counterparty_connection_id = Some(counterparty_connection_id);
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
                counterparty_connection_id.to_string(),
            ),
        ])),
    )
}

fn connection_open_confirm(
    info: MessageInfo,
    mut deps: DepsMut,
    connection_id: ConnectionId,
    proof_ack: Vec<u8>,
    proof_height: u64,
    verify: bool,
) -> ContractResult {
    let mut connection = deps.storage.read::<Connections>(&connection_id)?;
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
        counterparty_connection_id: Some(connection_id),
    };
    if verify {
        let client_impl = client_impl(deps.as_ref(), connection.client_id)?;
        query_light_client::<()>(
            deps.as_ref(),
            client_impl,
            LightClientQuery::VerifyMembership {
                client_id: connection.client_id,
                height: proof_height,
                proof: proof_ack.into(),
                path: ConnectionPath {
                    connection_id: connection
                        .counterparty_connection_id
                        .expect("state is open, counterparty exists; qed;"),
                }
                .key()
                .into_bytes(),
                value: commit(expected_connection.abi_encode_params()).into_bytes(),
            },
        )?;
    } else {
        ensure_relayer_admin(deps.storage, &info.sender)?;
    }
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
                connection
                    .counterparty_connection_id
                    .expect("state is open, counterparty exists; qed;")
                    .to_string(),
            ),
        ]),
    ))
}

fn channel_open_init(
    mut deps: DepsMut,
    info: MessageInfo,
    port_id: String,
    counterparty_port_id: Bytes,
    connection_id: ConnectionId,
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
        None,
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
                caller: info.sender.into_string(),
                connection_id,
                channel_id,
                version,
                relayer: relayer.into(),
            }),
            vec![],
        )?))
}

#[allow(clippy::too_many_arguments)]
fn channel_open_try(
    mut deps: DepsMut,
    info: MessageInfo,
    port_id: String,
    channel: Channel,
    counterparty_version: String,
    proof_init: Vec<u8>,
    proof_height: u64,
    relayer: Addr,
    verify: bool,
) -> ContractResult {
    if channel.state != ChannelState::TryOpen {
        return Err(ContractError::ChannelInvalidState {
            got: channel.state,
            expected: ChannelState::TryOpen,
        });
    }
    let connection = ensure_connection_state(deps.as_ref(), channel.connection_id)?;
    let connection_id = connection
        .counterparty_connection_id
        .expect("connection is open; qed;");
    let expected_channel = Channel {
        state: ChannelState::Init,
        connection_id,
        counterparty_channel_id: None,
        counterparty_port_id: port_id.as_bytes().to_vec().into(),
        version: counterparty_version.clone(),
    };
    let client_impl = client_impl(deps.as_ref(), connection.client_id)?;
    let counterparty_channel_id = channel
        .counterparty_channel_id
        .ok_or(ContractError::CounterpartyChannelIdInvalid)?;
    if verify {
        query_light_client::<()>(
            deps.as_ref(),
            client_impl,
            LightClientQuery::VerifyMembership {
                client_id: connection.client_id,
                height: proof_height,
                proof: proof_init.into(),
                path: ChannelPath {
                    channel_id: counterparty_channel_id,
                }
                .key()
                .into_bytes(),
                value: commit(expected_channel.abi_encode()).into_bytes(),
            },
        )?;
    }
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
                counterparty_channel_id.to_string(),
            ),
            (
                events::attribute::CONNECTION_ID,
                channel.connection_id.to_string(),
            ),
            ("counterparty_version", counterparty_version.clone()),
        ]))
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenTry {
                caller: info.sender.into_string(),
                connection_id,
                channel_id,
                version: channel.version,
                counterparty_version,
                relayer: relayer.into(),
            }),
            vec![],
        )?))
}

#[allow(clippy::too_many_arguments)]
fn channel_open_ack(
    mut deps: DepsMut,
    info: MessageInfo,
    channel_id: ChannelId,
    counterparty_version: String,
    counterparty_channel_id: ChannelId,
    proof_try: Vec<u8>,
    proof_height: u64,
    relayer: Addr,
    verify: bool,
) -> ContractResult {
    let mut channel = deps.storage.read::<Channels>(&channel_id)?;
    if channel.state != ChannelState::Init {
        return Err(ContractError::ChannelInvalidState {
            got: channel.state,
            expected: ChannelState::Init,
        });
    }
    let connection = ensure_connection_state(deps.as_ref(), channel.connection_id)?;
    let port_id = deps.storage.read::<ChannelOwner>(&channel_id)?;
    let expected_channel = Channel {
        state: ChannelState::TryOpen,
        connection_id: connection
            .counterparty_connection_id
            .expect("connection is open; qed;"),
        counterparty_channel_id: Some(channel_id),
        counterparty_port_id: port_id.as_bytes().to_vec().into(),
        version: counterparty_version.clone(),
    };
    let client_impl = client_impl(deps.as_ref(), connection.client_id)?;
    if verify {
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
    }
    channel.state = ChannelState::Open;
    channel.version = counterparty_version.clone();
    channel.counterparty_channel_id = Some(counterparty_channel_id);
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
                counterparty_channel_id.to_string(),
            ),
            ("connection_id", channel.connection_id.to_string()),
        ]))
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenAck {
                caller: info.sender.into_string(),
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
    info: MessageInfo,
    channel_id: ChannelId,
    proof_ack: Vec<u8>,
    proof_height: u64,
    relayer: Addr,
    verify: bool,
) -> ContractResult {
    let mut channel = deps.storage.read::<Channels>(&channel_id)?;
    if channel.state != ChannelState::TryOpen {
        return Err(ContractError::ChannelInvalidState {
            got: channel.state,
            expected: ChannelState::TryOpen,
        });
    }
    let connection = ensure_connection_state(deps.as_ref(), channel.connection_id)?;
    let port_id = deps.storage.read::<ChannelOwner>(&channel_id)?;
    let counterparty_connection_id = connection
        .counterparty_connection_id
        .expect("connection is open; qed;");
    let expected_channel = Channel {
        state: ChannelState::Open,
        connection_id: counterparty_connection_id,
        counterparty_channel_id: Some(channel_id),
        counterparty_port_id: port_id.clone().as_bytes().to_vec().into(),
        version: channel.version.clone(),
    };
    let client_impl = client_impl(deps.as_ref(), connection.client_id)?;
    let counterparty_channel_id = channel
        .counterparty_channel_id
        .expect("channel state is try open; qed;");
    if verify {
        query_light_client::<()>(
            deps.as_ref(),
            client_impl,
            LightClientQuery::VerifyMembership {
                client_id: connection.client_id,
                height: proof_height,
                proof: proof_ack.into(),
                path: ChannelPath {
                    channel_id: counterparty_channel_id,
                }
                .key()
                .into_bytes(),
                value: commit(expected_channel.abi_encode()).into_bytes(),
            },
        )?;
    }
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
                counterparty_channel_id.to_string(),
            ),
            (
                events::attribute::CONNECTION_ID,
                channel.connection_id.to_string(),
            ),
        ]))
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenConfirm {
                caller: info.sender.into_string(),
                channel_id,
                relayer: relayer.into(),
            }),
            vec![],
        )?))
}

fn channel_close_init(
    mut deps: DepsMut,
    info: MessageInfo,
    channel_id: ChannelId,
    relayer: Addr,
) -> ContractResult {
    let mut channel = deps.storage.read::<Channels>(&channel_id)?;
    if channel.state != ChannelState::Open {
        return Err(ContractError::ChannelInvalidState {
            got: channel.state,
            expected: ChannelState::Open,
        });
    }
    ensure_connection_state(deps.as_ref(), channel.connection_id)?;
    channel.state = ChannelState::Closed;
    save_channel(deps.branch(), channel_id, &channel)?;
    let port_id = deps.storage.read::<ChannelOwner>(&channel_id)?;
    Ok(Response::new()
        .add_event(
            Event::new(events::channel::CLOSE_INIT).add_attributes([
                (events::attribute::PORT_ID, port_id.to_string()),
                (events::attribute::CHANNEL_ID, channel_id.to_string()),
                (
                    events::attribute::COUNTERPARTY_PORT_ID,
                    hex::encode(&channel.counterparty_port_id),
                ),
                (
                    events::attribute::COUNTERPARTY_CHANNEL_ID,
                    channel
                        .counterparty_channel_id
                        .expect("channel is open; qed;")
                        .to_string(),
                ),
            ]),
        )
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnChannelCloseInit {
                caller: info.sender.into_string(),
                channel_id,
                relayer: relayer.into(),
            }),
            vec![],
        )?))
}

fn channel_close_confirm(
    mut deps: DepsMut,
    info: MessageInfo,
    channel_id: ChannelId,
    proof_init: Vec<u8>,
    proof_height: u64,
    relayer: Addr,
) -> ContractResult {
    let mut channel = deps.storage.read::<Channels>(&channel_id)?;
    if channel.state != ChannelState::Open {
        return Err(ContractError::ChannelInvalidState {
            got: channel.state,
            expected: ChannelState::Open,
        });
    }
    let connection = ensure_connection_state(deps.as_ref(), channel.connection_id)?;
    let port_id = deps.storage.read::<ChannelOwner>(&channel_id)?;
    let connection_id = connection
        .counterparty_connection_id
        .expect("connection is open; qed;");
    let expected_channel = Channel {
        state: ChannelState::Closed,
        connection_id,
        counterparty_channel_id: Some(channel_id),
        counterparty_port_id: port_id.as_bytes().to_vec().into(),
        version: channel.version.clone(),
    };
    let client_impl = client_impl(deps.as_ref(), connection.client_id)?;
    let counterparty_channel_id = channel
        .counterparty_channel_id
        .expect("channel is open; qed;");
    query_light_client::<()>(
        deps.as_ref(),
        client_impl,
        LightClientQuery::VerifyMembership {
            client_id: connection.client_id,
            height: proof_height,
            proof: proof_init.into(),
            path: ChannelPath {
                channel_id: counterparty_channel_id,
            }
            .key()
            .into_bytes(),
            value: commit(expected_channel.abi_encode()).into_bytes(),
        },
    )?;
    channel.state = ChannelState::Closed;
    deps.storage.write::<Channels>(&channel_id, &channel);
    store_commit(
        deps.branch(),
        &ChannelPath { channel_id }.key(),
        &commit(channel.abi_encode()),
    );
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
                counterparty_channel_id.to_string(),
            ),
        ]))
        .add_message(wasm_execute(
            port_id,
            &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnChannelOpenConfirm {
                caller: info.sender.into_string(),
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
    info: MessageInfo,
    packets: Vec<Packet>,
    relayer_msgs: Vec<Bytes>,
    relayer: String,
    proof: Bytes,
    proof_height: u64,
    intent: bool,
) -> Result<Response, ContractError> {
    let first = packets.first().ok_or(ContractError::NotEnoughPackets)?;
    let destination_channel_id = first.destination_channel_id;

    let channel = ensure_channel_state(deps.as_ref(), destination_channel_id)?;
    let connection = ensure_connection_state(deps.as_ref(), channel.connection_id)?;

    if !intent {
        let proof_commitment_key = BatchPacketsPath::from_packets(&packets).key();
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
    let port_id = deps.storage.read::<ChannelOwner>(&destination_channel_id)?;
    for (packet, relayer_msg) in packets.into_iter().zip(relayer_msgs) {
        if packet.destination_channel_id != destination_channel_id {
            return Err(ContractError::BatchSameChannelOnly);
        }

        if packet.timeout_height != 0 {
            return Err(ContractError::TimeoutHeightUnsupported);
        }

        let current_timestamp = Timestamp::from_nanos(env.block.time.nanos());
        if current_timestamp >= packet.timeout_timestamp {
            return Err(ContractError::ReceivedTimedOutPacketTimestamp {
                timeout_timestamp: packet.timeout_timestamp,
                current_timestamp,
            });
        }

        let commitment_key = BatchReceiptsPath::from_packets(&[packet.clone()]).key();
        if !set_packet_receive(deps.branch(), commitment_key) {
            if intent {
                events.push(
                    Event::new(events::packet::INTENT_RECV)
                        .add_attributes(packet_to_attr_hash(destination_channel_id, &packet))
                        .add_attributes([
                            (events::attribute::MAKER, relayer.clone()),
                            (events::attribute::MAKER_MSG, hex::encode(&relayer_msg)),
                        ]),
                );

                messages.push(wasm_execute(
                    port_id.clone(),
                    &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnIntentRecvPacket {
                        caller: info.sender.clone().into_string(),
                        packet,
                        market_maker: deps.api.addr_validate(&relayer)?.into(),
                        market_maker_msg: relayer_msg.to_vec().into(),
                    }),
                    vec![],
                )?);
            } else {
                events.push(
                    Event::new(events::packet::RECV)
                        .add_attributes(packet_to_attr_hash(destination_channel_id, &packet))
                        .add_attributes([
                            (events::attribute::MAKER, relayer.clone()),
                            (events::attribute::MAKER_MSG, hex::encode(&relayer_msg)),
                        ]),
                );

                messages.push(wasm_execute(
                    port_id.clone(),
                    &ModuleMsg::IbcUnionMsg(IbcUnionMsg::OnRecvPacket {
                        caller: info.sender.clone().into_string(),
                        packet,
                        relayer: deps.api.addr_validate(&relayer)?.into(),
                        relayer_msg: relayer_msg.to_vec().into(),
                    }),
                    // TODO: this is incorrect and we should allow the relayer
                    // to specify how to split the total funds individually for
                    // each batched packet. because voyager does not do any
                    // batching at ibc level, this will always work.
                    info.funds.clone(),
                )?);
            }
        }
    }

    Ok(Response::new().add_events(events).add_messages(messages))
}

fn write_acknowledgement(
    mut deps: DepsMut,
    sender: Addr,
    packet: Packet,
    acknowledgement: Vec<u8>,
) -> ContractResult {
    if acknowledgement.is_empty() {
        return Err(ContractError::AcknowledgementIsEmpty);
    }

    let channel_id = packet.destination_channel_id;

    // make sure the caller owns the channel
    let port_id = deps.storage.read::<ChannelOwner>(&channel_id)?;
    if port_id != sender {
        return Err(ContractError::Unauthorized {
            channel_id,
            owner: port_id,
            caller: sender,
        });
    }

    let commitment_key = BatchReceiptsPath::from_packets(&[packet.clone()]).key();

    // the receipt must present, but ack shouldn't
    match read_commit(deps.as_ref(), &commitment_key) {
        Some(commitment) => {
            if commitment != COMMITMENT_MAGIC {
                return Err(ContractError::AlreadyAcknowledged);
            }
        }
        None => return Err(ContractError::PacketNotReceived),
    }

    let acknowledgement_serialized = hex::encode(&acknowledgement);

    store_commit(
        deps.branch(),
        &commitment_key,
        &commit_ack(&acknowledgement.into()),
    );

    Ok(Response::new().add_event(
        Event::new(events::packet::WRITE_ACK)
            .add_attributes(packet_to_attr_hash(channel_id, &packet))
            .add_attributes([(
                events::attribute::ACKNOWLEDGEMENT,
                acknowledgement_serialized,
            )]),
    ))
}

fn send_packet(
    mut deps: DepsMut,
    sender: Addr,
    source_channel_id: ChannelId,
    timeout_height: u64,
    timeout_timestamp: Timestamp,
    data: Vec<u8>,
) -> ContractResult {
    if timeout_height != 0 {
        return Err(ContractError::TimeoutHeightUnsupported);
    }

    if timeout_timestamp.is_zero() {
        return Err(ContractError::TimeoutMustBeSet);
    }

    let port_id = deps.storage.read::<ChannelOwner>(&source_channel_id)?;
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
        destination_channel_id: channel
            .counterparty_channel_id
            .expect("channel is open; qed;"),
        data: data.into(),
        timeout_height,
        timeout_timestamp,
    };

    let serialized_packet =
        serde_json_wasm::to_string(&packet).expect("packet serialization is infallible; qed;");

    let packet_attrs = packet_to_attrs(&packet);
    let packet_attr_hash = packet_to_attr_hash(source_channel_id, &packet);

    let commitment_key = BatchPacketsPath::from_packets(&[packet]).key();

    if read_commit(deps.as_ref(), &commitment_key).is_some() {
        return Err(ContractError::PacketCommitmentAlreadyExist);
    }

    store_commit(deps.branch(), &commitment_key, &COMMITMENT_MAGIC);

    Ok(Response::new()
        .add_event(
            Event::new(events::packet::SEND)
                .add_attributes(packet_attr_hash)
                .add_attributes(packet_attrs),
        )
        .set_data(serialized_packet.as_bytes()))
}

fn next_channel_id(deps: DepsMut) -> Result<ChannelId, ContractError> {
    let next = deps.storage.read_item::<NextChannelId>()?;

    let v = next
        .checked_add(1)
        .ok_or(StdError::overflow(OverflowError::new(
            OverflowOperation::Add,
        )))?;

    deps.storage.write_item::<NextChannelId>(&v);

    Ok(next)
}

fn next_connection_id(deps: DepsMut) -> Result<ConnectionId, ContractError> {
    let next = deps.storage.read_item::<NextConnectionId>()?;

    let v = next
        .checked_add(1)
        .ok_or(StdError::overflow(OverflowError::new(
            OverflowOperation::Add,
        )))?;

    deps.storage.write_item::<NextConnectionId>(&v);

    Ok(next)
}

fn next_client_id(deps: DepsMut) -> Result<ClientId, ContractError> {
    let next = deps.storage.read_item::<NextClientId>()?;

    let v = next
        .checked_add(1)
        .ok_or(StdError::overflow(OverflowError::new(
            OverflowOperation::Add,
        )))?;

    deps.storage.write_item::<NextClientId>(&v);

    Ok(next)
}

fn client_impl(deps: Deps, client_id: ClientId) -> Result<Addr, ContractError> {
    Ok(deps.storage.read::<ClientImpls>(&client_id)?)
}

fn commit(bytes: impl AsRef<[u8]>) -> H256 {
    keccak256(bytes)
}

fn commit_ack(ack: &Bytes) -> H256 {
    commit_acks(&[ack.clone()])
}

fn commit_acks(acks: &[Bytes]) -> H256 {
    merge_ack(commit(acks.abi_encode()))
}

fn merge_ack(mut ack: H256) -> H256 {
    ack.get_mut()[0] = 0x01;
    ack
}

fn store_commit(deps: DepsMut, key: &H256, value: &H256) {
    deps.storage.write::<Commitments>(key, value);
}

fn read_commit(deps: Deps, key: &H256) -> Option<H256> {
    deps.storage
        .maybe_read::<Commitments>(key)
        .expect("H256 is the only value ever written to this storage; qed;")
}

fn save_connection(
    deps: DepsMut,
    connection_id: ConnectionId,
    connection: &Connection,
) -> Result<(), ContractError> {
    deps.storage
        .write::<Connections>(&connection_id, connection);
    store_commit(
        deps,
        &ConnectionPath { connection_id }.key(),
        &commit(connection.abi_encode_params()),
    );
    Ok(())
}

fn create_channel(
    mut deps: DepsMut,
    owner: Addr,
    state: ChannelState,
    connection_id: ConnectionId,
    counterparty_channel_id: Option<ChannelId>,
    counterparty_port_id: Bytes,
    version: String,
) -> Result<(ChannelId, Channel), ContractError> {
    let channel_id = next_channel_id(deps.branch())?;
    let channel = Channel {
        state,
        connection_id,
        counterparty_channel_id,
        counterparty_port_id,
        version,
    };
    deps.storage.write::<ChannelOwner>(&channel_id, &owner);
    deps.storage
        .upsert::<ContractChannels, _>(&owner, |v| -> Result<_, ContractError> {
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

fn save_channel(
    deps: DepsMut,
    channel_id: ChannelId,
    channel: &Channel,
) -> Result<(), ContractError> {
    deps.storage.write::<Channels>(&channel_id, channel);
    store_commit(
        deps,
        &ChannelPath { channel_id }.key(),
        &commit(channel.abi_encode()),
    );
    Ok(())
}

fn ensure_connection_state(
    deps: Deps,
    connection_id: ConnectionId,
) -> Result<Connection, ContractError> {
    let connection = deps.storage.read::<Connections>(&connection_id)?;
    if connection.state != ConnectionState::Open {
        Err(ContractError::ConnectionInvalidState {
            got: connection.state,
            expected: ConnectionState::Open,
        })
    } else {
        Ok(connection)
    }
}

fn ensure_channel_state(deps: Deps, channel_id: ChannelId) -> Result<Channel, ContractError> {
    let channel = deps.storage.read::<Channels>(&channel_id)?;
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
    if read_commit(deps.as_ref(), &commitment_key).is_some() {
        true
    } else {
        store_commit(deps, &commitment_key, &COMMITMENT_MAGIC);
        false
    }
}

fn get_timestamp_at_height(
    deps: Deps,
    client_id: ClientId,
    height: u64,
) -> Result<Timestamp, ContractError> {
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
            &deps.storage.read::<ClientTypes>(&client_id)?,
        )?),
        QueryMsg::GetClientImpl { client_id } => {
            Ok(to_json_binary(&client_impl(deps, client_id)?)?)
        }
        QueryMsg::GetRegisteredClientType { client_type } => Ok(to_json_binary(
            &deps.storage.read::<ClientRegistry>(&client_type)?,
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
        QueryMsg::GetClientState { client_id } => Ok(to_json_binary(
            &deps.storage.read::<ClientStates>(&client_id)?,
        )?),
        QueryMsg::GetConsensusState { client_id, height } => Ok(to_json_binary(
            &deps
                .storage
                .read::<ClientConsensusStates>(&(client_id, height))?,
        )?),
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
            let channels = deps.storage.read::<ContractChannels>(&contract)?;
            Ok(to_json_binary(&channels)?)
        }
        QueryMsg::GetChannel { channel_id } => {
            let channel = deps.storage.read::<Channels>(&channel_id)?;
            Ok(to_json_binary(&channel)?)
        }
        QueryMsg::GetConnection { connection_id } => {
            let connection = deps.storage.read::<Connections>(&connection_id)?;
            Ok(to_json_binary(&connection)?)
        }
        QueryMsg::GetBatchPackets { batch_hash } => {
            let commit = read_commit(deps, &BatchPacketsPath { batch_hash }.key());
            Ok(to_json_binary(&commit)?)
        }
        QueryMsg::GetBatchReceipts { batch_hash } => {
            let commit = read_commit(deps, &BatchReceiptsPath { batch_hash }.key());
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

fn make_verify_creation_event(client_id: ClientId, event: VerifyCreationResponseEvent) -> Event {
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
    use hex_literal::hex;

    use super::*;

    #[test]
    fn channel_value() {
        let channel = Channel {
            state: ChannelState::Init,
            connection_id: ConnectionId!(1),
            counterparty_channel_id: None,
            counterparty_port_id: hex!("30783735366536393666366533313337373033393732376137373665366536363738363336613730333333323735366533393735363733373739363836383761363737343662363837363663333936613636366237333761373436373737333537353638333633393737363136333332373036373733333936383337373736613637").into(),
            version: "ucs01-relay-1".to_owned()
        };

        let value = commit(channel.abi_encode());

        dbg!(value);
        dbg!("0x68361972d5315b7a497e342405661930a6bdb0c17ce58a87227bb676fbcfc3ce");
        dbg!("0x9f4901a9b797640d3d2507111018b5130d209eb7305bdda6ed3163a6ec4d4c9b");
    }

    #[test]
    fn verify_creation_response() {
        let response = VerifyCreationResponse {
            counterparty_chain_id: "chain-id".to_owned(),
            client_state_bytes: None,
            storage_writes: [([].into(), [].into())].into_iter().collect(),
            events: vec![],
        };

        let json = serde_json_wasm::to_string(&response).unwrap();

        let rt = serde_json_wasm::from_str::<VerifyCreationResponse>(&json).unwrap();

        assert_eq!(response, rt);
    }
}
