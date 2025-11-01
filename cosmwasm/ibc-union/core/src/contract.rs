use core::slice;
use std::{collections::BTreeSet, num::NonZeroU32};

use access_managed::{EnsureCanCallResult, handle_consume_scheduled_op_reply, state::Authority};
use alloy_sol_types::SolValue;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Addr, Attribute, Binary, Deps, DepsMut, Env, Event, MessageInfo, OverflowError,
    OverflowOperation, Reply, Response, StdError, StdResult, to_json_binary, wasm_execute,
};
use depolama::{RawStore, StorageExt};
use frissitheto::{UpgradeError, UpgradeMsg};
use ibc_union_msg::{
    lightclient::{
        QueryMsg as LightClientQuery, UpdateStateResponse, VerifyCreationResponse,
        VerifyCreationResponseEvent,
    },
    module::{ExecuteMsg as ModuleMsg, IbcUnionMsg},
    msg::{
        ExecuteMsg, InitMsg, MsgBatchAcks, MsgBatchSend, MsgChannelCloseConfirm,
        MsgChannelCloseInit, MsgChannelOpenAck, MsgChannelOpenConfirm, MsgChannelOpenInit,
        MsgChannelOpenTry, MsgCommitMembershipProof, MsgCommitNonMembershipProof,
        MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
        MsgConnectionOpenTry, MsgCreateClient, MsgForceUpdateClient, MsgIntentPacketRecv,
        MsgMigrateState, MsgPacketAcknowledgement, MsgPacketRecv, MsgPacketTimeout,
        MsgRegisterClient, MsgSendPacket, MsgUpdateClient, MsgWriteAcknowledgement,
        RestrictedExecuteMsg,
    },
    query::QueryMsg,
};
use ibc_union_spec::{
    Channel, ChannelId, ChannelState, ClientId, Connection, ConnectionId, ConnectionState,
    MustBeZero, Packet, Status, Timestamp,
    path::{
        BatchPacketsPath, BatchReceiptsPath, COMMITMENT_MAGIC, COMMITMENT_MAGIC_ACK, ChannelPath,
        ClientStatePath, ConnectionPath, ConsensusStatePath, MembershipProofPath,
        NON_MEMBERSHIP_COMMITMENT_VALUE, NonMembershipProofPath, commit_packets,
    },
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use unionlabs::{
    ethereum::keccak256,
    primitives::{Bytes, H256},
};

use crate::{
    ContractError,
    state::{
        ChannelOwner, Channels, ClientConsensusStates, ClientImpls, ClientRegistry, ClientStates,
        ClientStore, ClientTypes, Commitments, Connections, ContractChannels, NextChannelId,
        NextClientId, NextConnectionId, QueryStore, WhitelistedRelayers, WhitelistedRelayersAdmin,
    },
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
    pub mod proof {
        pub const COMMIT_MEMBERSHIP: &str = "commit_membership_proof";
        pub const COMMIT_NON_MEMBERSHIP: &str = "commit_non_membership_proof";
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
        pub const PATH: &str = "path";
        pub const VALUE: &str = "value";
        pub const PROOF_HEIGHT: &str = "proof_height";
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
        // emitted as a legacy attribute
        (events::attribute::PACKET_TIMEOUT_HEIGHT, 0.to_string()),
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
            commit_packets(slice::from_ref(packet)).to_string(),
        ),
    ]
    .map(Into::into)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
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
            timeout_timestamp,
            data,
        }) => send_packet(
            deps.branch(),
            info.sender,
            source_channel_id,
            timeout_timestamp,
            data.into_vec(),
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
        ExecuteMsg::AccessManaged(msg) => {
            access_managed::execute(deps, env, info, msg).map_err(Into::into)
        }
        ExecuteMsg::Restricted(msg) => {
            let msg = match msg.ensure_can_call::<Authority>(deps.branch(), &env, &info)? {
                EnsureCanCallResult::Msg(msg) => msg,
                EnsureCanCallResult::Scheduled(sub_msgs) => {
                    return Ok(Response::new().add_submessages(sub_msgs));
                }
            };

            match msg {
                RestrictedExecuteMsg::RegisterClient(MsgRegisterClient {
                    client_type,
                    client_address,
                }) => {
                    let address = deps.api.addr_validate(&client_address)?;
                    register_client(deps.branch(), client_type, address)
                }
                RestrictedExecuteMsg::CreateClient(MsgCreateClient {
                    client_type,
                    client_state_bytes,
                    consensus_state_bytes,
                    relayer,
                }) => {
                    let relayer = deps.api.addr_validate(&relayer)?;
                    create_client(
                        deps,
                        info,
                        client_type,
                        client_state_bytes.to_vec(),
                        consensus_state_bytes.to_vec(),
                        relayer,
                    )
                }
                RestrictedExecuteMsg::UpdateClient(MsgUpdateClient {
                    client_id,
                    client_message,
                    relayer,
                }) => {
                    let relayer = deps.api.addr_validate(&relayer)?;
                    update_client(deps, info, client_id, client_message.to_vec(), relayer)
                }
                RestrictedExecuteMsg::ForceUpdateClient(MsgForceUpdateClient {
                    client_id,
                    client_state_bytes,
                    consensus_state_bytes,
                }) => {
                    let client_impl = client_impl(deps.as_ref(), client_id)?;

                    let (_, height) = init_client(
                        deps,
                        info.clone(),
                        client_state_bytes.into(),
                        consensus_state_bytes.into(),
                        info.sender,
                        client_impl,
                        client_id,
                    )?;

                    Ok(Response::new().add_event(
                        Event::new("force_update_client")
                            .add_attribute(events::attribute::CLIENT_ID, client_id.to_string())
                            .add_attribute(
                                events::attribute::COUNTERPARTY_HEIGHT,
                                height.to_string(),
                            ),
                    ))
                }
                RestrictedExecuteMsg::ConnectionOpenInit(MsgConnectionOpenInit {
                    client_id,
                    counterparty_client_id,
                }) => connection_open_init(deps, client_id, counterparty_client_id),
                RestrictedExecuteMsg::ConnectionOpenTry(MsgConnectionOpenTry {
                    counterparty_client_id,
                    counterparty_connection_id,
                    client_id,
                    proof_init,
                    proof_height,
                }) => connection_open_try(
                    deps,
                    counterparty_client_id,
                    counterparty_connection_id,
                    client_id,
                    proof_init.to_vec(),
                    proof_height,
                    true,
                ),
                RestrictedExecuteMsg::ForceConnectionOpenTry(MsgConnectionOpenTry {
                    counterparty_client_id,
                    counterparty_connection_id,
                    client_id,
                    proof_init,
                    proof_height,
                }) => connection_open_try(
                    deps,
                    counterparty_client_id,
                    counterparty_connection_id,
                    client_id,
                    proof_init.to_vec(),
                    proof_height,
                    false,
                ),
                RestrictedExecuteMsg::ConnectionOpenAck(MsgConnectionOpenAck {
                    connection_id,
                    counterparty_connection_id,
                    proof_try,
                    proof_height,
                }) => connection_open_ack(
                    deps,
                    connection_id,
                    counterparty_connection_id,
                    proof_try.to_vec(),
                    proof_height,
                    true,
                ),
                RestrictedExecuteMsg::ForceConnectionOpenAck(MsgConnectionOpenAck {
                    connection_id,
                    counterparty_connection_id,
                    proof_try,
                    proof_height,
                }) => connection_open_ack(
                    deps,
                    connection_id,
                    counterparty_connection_id,
                    proof_try.to_vec(),
                    proof_height,
                    false,
                ),
                RestrictedExecuteMsg::ConnectionOpenConfirm(MsgConnectionOpenConfirm {
                    connection_id,
                    proof_ack,
                    proof_height,
                }) => connection_open_confirm(
                    deps,
                    connection_id,
                    proof_ack.to_vec(),
                    proof_height,
                    true,
                ),
                RestrictedExecuteMsg::ForceConnectionOpenConfirm(MsgConnectionOpenConfirm {
                    connection_id,
                    proof_ack,
                    proof_height,
                }) => connection_open_confirm(
                    deps,
                    connection_id,
                    proof_ack.to_vec(),
                    proof_height,
                    false,
                ),
                RestrictedExecuteMsg::ChannelOpenInit(MsgChannelOpenInit {
                    port_id,
                    counterparty_port_id,
                    connection_id,
                    version,
                    relayer,
                }) => {
                    let relayer = deps.api.addr_validate(&relayer)?;
                    channel_open_init(
                        deps,
                        info,
                        port_id,
                        counterparty_port_id,
                        connection_id,
                        version,
                        relayer,
                    )
                }
                RestrictedExecuteMsg::ChannelOpenTry(MsgChannelOpenTry {
                    port_id,
                    channel,
                    counterparty_version,
                    proof_init,
                    proof_height,
                    relayer,
                }) => {
                    let relayer = deps.api.addr_validate(&relayer)?;
                    channel_open_try(
                        deps,
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
                RestrictedExecuteMsg::ForceChannelOpenTry(MsgChannelOpenTry {
                    port_id,
                    channel,
                    counterparty_version,
                    proof_init,
                    proof_height,
                    relayer,
                }) => {
                    let relayer = deps.api.addr_validate(&relayer)?;
                    channel_open_try(
                        deps,
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
                RestrictedExecuteMsg::ChannelOpenAck(MsgChannelOpenAck {
                    channel_id,
                    counterparty_version,
                    counterparty_channel_id,
                    proof_try,
                    proof_height,
                    relayer,
                }) => {
                    let relayer = deps.api.addr_validate(&relayer)?;
                    channel_open_ack(
                        deps,
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
                RestrictedExecuteMsg::ForceChannelOpenAck(MsgChannelOpenAck {
                    channel_id,
                    counterparty_version,
                    counterparty_channel_id,
                    proof_try,
                    proof_height,
                    relayer,
                }) => {
                    let relayer = deps.api.addr_validate(&relayer)?;
                    channel_open_ack(
                        deps,
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
                RestrictedExecuteMsg::ChannelOpenConfirm(MsgChannelOpenConfirm {
                    channel_id,
                    proof_ack,
                    proof_height,
                    relayer,
                }) => {
                    let relayer = deps.api.addr_validate(&relayer)?;
                    channel_open_confirm(
                        deps,
                        info,
                        channel_id,
                        proof_ack.to_vec(),
                        proof_height,
                        relayer,
                        true,
                    )
                }
                RestrictedExecuteMsg::ForceChannelOpenConfirm(MsgChannelOpenConfirm {
                    channel_id,
                    proof_ack,
                    proof_height,
                    relayer,
                }) => {
                    let relayer = deps.api.addr_validate(&relayer)?;
                    channel_open_confirm(
                        deps,
                        info,
                        channel_id,
                        proof_ack.to_vec(),
                        proof_height,
                        relayer,
                        false,
                    )
                }
                RestrictedExecuteMsg::ChannelCloseInit(MsgChannelCloseInit {
                    channel_id,
                    relayer,
                }) => {
                    let relayer = deps.api.addr_validate(&relayer)?;
                    channel_close_init(deps, info, channel_id, relayer)
                }
                RestrictedExecuteMsg::ChannelCloseConfirm(MsgChannelCloseConfirm {
                    channel_id,
                    proof_init,
                    proof_height,
                    relayer,
                }) => {
                    let relayer = deps.api.addr_validate(&relayer)?;
                    channel_close_confirm(
                        deps,
                        info,
                        channel_id,
                        proof_init.to_vec(),
                        proof_height,
                        relayer,
                    )
                }
                RestrictedExecuteMsg::PacketRecv(MsgPacketRecv {
                    packets,
                    relayer_msgs,
                    relayer,
                    proof,
                    proof_height,
                }) => process_receive(
                    deps,
                    env,
                    info,
                    packets,
                    relayer_msgs.into_iter().collect(),
                    relayer,
                    proof,
                    proof_height,
                    false,
                ),
                RestrictedExecuteMsg::PacketAck(MsgPacketAcknowledgement {
                    packets,
                    acknowledgements,
                    proof,
                    proof_height,
                    relayer,
                }) => {
                    let relayer = deps.api.addr_validate(&relayer)?;
                    acknowledge_packet(
                        deps,
                        info,
                        packets,
                        acknowledgements.into_iter().collect(),
                        proof.to_vec(),
                        proof_height,
                        relayer,
                    )
                }
                RestrictedExecuteMsg::PacketTimeout(MsgPacketTimeout {
                    packet,
                    proof,
                    proof_height,
                    relayer,
                }) => {
                    let relayer = deps.api.addr_validate(&relayer)?;
                    timeout_packet(deps, info, packet, proof.to_vec(), proof_height, relayer)
                }
                RestrictedExecuteMsg::IntentPacketRecv(MsgIntentPacketRecv {
                    packets,
                    market_maker_msgs,
                    market_maker,
                }) => process_receive(
                    deps,
                    env,
                    info,
                    packets,
                    market_maker_msgs.into_iter().collect(),
                    market_maker,
                    Bytes::new_static(&[]),
                    0,
                    true,
                ),
                RestrictedExecuteMsg::BatchSend(MsgBatchSend { packets }) => {
                    batch_send(deps, packets)
                }
                RestrictedExecuteMsg::BatchAcks(MsgBatchAcks { packets, acks }) => {
                    batch_acks(deps, packets, acks.into_iter().collect())
                }
                RestrictedExecuteMsg::CommitMembershipProof(MsgCommitMembershipProof {
                    client_id,
                    proof_height,
                    proof,
                    path,
                    value,
                }) => commit_membership_proof(deps, client_id, proof_height, proof, path, value),
                RestrictedExecuteMsg::CommitNonMembershipProof(MsgCommitNonMembershipProof {
                    client_id,
                    proof_height,
                    proof,
                    path,
                }) => commit_non_membership_proof(deps, client_id, proof_height, proof, path),
                RestrictedExecuteMsg::Upgradable(msg) => {
                    upgradable::execute(deps, env, info, msg).map_err(Into::into)
                }
            }
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
    panic!(
        "this contract cannot be instantiated directly, but must be migrated from an existing instantiated contract."
    );
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct IbcUnionMigrateMsg {
    pub access_managed_init_msg: access_managed::InitMsg,
}

/// Major state versions of this contract, used in the [`frissitheto`] migrations.
pub mod version {
    use std::num::NonZeroU32;

    /// Initial state of the contract. Access management is handled internally in this contract for specific endpoints.
    pub const INIT: NonZeroU32 = NonZeroU32::new(1).unwrap();

    /// Same as [`INIT`], except that access management is handled externally via [`access_managed`]. All storage in this contract relating to internally handled access management has been removed, and additional storages for [`access_managed`] have been added.
    ///
    /// This is the current latest state version of this contract.
    pub const MANAGED: NonZeroU32 = NonZeroU32::new(2).unwrap();

    /// The latest state version of this contract. Any new deployments will be init'd with this version and the corresponding state.
    pub const LATEST: NonZeroU32 = MANAGED;
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    msg: UpgradeMsg<InitMsg, IbcUnionMigrateMsg>,
) -> Result<Response, ContractError> {
    msg.run(deps, init, |mut deps, msg, version| match version {
        version::INIT => {
            access_managed::init(deps.branch(), msg.access_managed_init_msg)?;

            deps.storage.delete_item::<WhitelistedRelayersAdmin>();

            for (k, ()) in deps
                .storage
                .iter::<WhitelistedRelayers>(cosmwasm_std::Order::Ascending)
                .collect::<Result<Vec<_>, _>>()?
            {
                deps.storage.delete::<WhitelistedRelayers>(&k);
            }

            Ok((Response::new(), Some(version::MANAGED)))
        }
        version::MANAGED => Ok((Response::new(), None)),
        _ => Err(UpgradeError::UnknownStateVersion(version).into()),
    })
}

pub(crate) fn init(
    mut deps: DepsMut<'_>,
    msg: InitMsg,
) -> Result<(Response, Option<NonZeroU32>), ContractError> {
    access_managed::init(deps.branch(), msg.access_managed_init_msg)?;

    // init all id storages to 1 (ids are non-zero, and for simplicity in the rest of the contract we assume the storages exist)
    deps.storage.write_item::<NextChannelId>(&ChannelId!(1));
    deps.storage
        .write_item::<NextConnectionId>(&ConnectionId!(1));
    deps.storage.write_item::<NextClientId>(&ClientId!(1));

    Ok((Response::default(), Some(version::LATEST)))
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

    let commitment_key = BatchReceiptsPath::from_packets(slice::from_ref(&packet)).key();

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

    if packet.timeout_timestamp.is_zero() {
        return Err(ContractError::TimeoutMustBeSet);
    }

    if packet.timeout_timestamp > proof_timestamp {
        return Err(ContractError::TimeoutTimestampNotReached);
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
    let commitment_key = BatchPacketsPath::from_packets(slice::from_ref(packet)).key();
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

    let (verify_creation_response, _) = init_client(
        deps,
        info,
        client_state_bytes,
        consensus_state_bytes,
        relayer,
        client_impl,
        client_id,
    )?;

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

/// Shared functionality between CreateClient and ForceUpdateClient.
fn init_client(
    mut deps: DepsMut<'_>,
    info: MessageInfo,
    client_state_bytes: Vec<u8>,
    consensus_state_bytes: Vec<u8>,
    relayer: Addr,
    client_impl: Addr,
    client_id: ClientId,
) -> Result<(VerifyCreationResponse, u64), ContractError> {
    // Ugly hack to allow for >64K messages (not configurable) to be threaded for the query.
    // See https://github.com/CosmWasm/cosmwasm/blob/e17ecc44cdebc84de1caae648c7a4f4b56846f8f/packages/vm/src/imports.rs#L47

    // 1. write these states first, so they can be read by the light client contract during VerifyCreation
    deps.storage
        .write::<ClientStates>(&client_id, &client_state_bytes.to_vec().into());

    // 2. once the client state is saved, query the light client impl for the height of that client state (the state we just saved is the latest height)
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
    if let Some(client_state_bytes) = verify_creation_response.client_state_bytes.as_ref() {
        // if VerifyCreation returns a new client state to save, overwrite the state we just wrote.
        deps.storage
            .write::<ClientStates>(&client_id, client_state_bytes);
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

    for (k, v) in &verify_creation_response.storage_writes {
        deps.storage
            .write::<ClientStore<RawStore>>(&(client_id, k.clone()), v);
    }

    Ok((verify_creation_response, latest_height))
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

        let current_timestamp = Timestamp::from_nanos(env.block.time.nanos());
        if current_timestamp >= packet.timeout_timestamp {
            return Err(ContractError::ReceivedTimedOutPacketTimestamp {
                timeout_timestamp: packet.timeout_timestamp,
                current_timestamp,
            });
        }

        let commitment_key = BatchReceiptsPath::from_packets(slice::from_ref(&packet)).key();
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

    let commitment_key = BatchReceiptsPath::from_packets(slice::from_ref(&packet)).key();

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
    timeout_timestamp: Timestamp,
    data: Vec<u8>,
) -> ContractResult {
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
        timeout_height: MustBeZero,
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
    commit_acks(slice::from_ref(ack))
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
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
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
        QueryMsg::GetCommittedMembershipProof {
            client_id,
            proof_height,
            path,
        } => {
            let commit = read_commit(
                deps,
                &MembershipProofPath {
                    client_id,
                    proof_height,
                    path,
                }
                .key(),
            );
            Ok(to_json_binary(&commit)?)
        }
        QueryMsg::GetCommittedNonMembershipProof {
            client_id,
            proof_height,
            path,
        } => {
            let commit = read_commit(
                deps,
                &NonMembershipProofPath {
                    client_id,
                    proof_height,
                    path,
                }
                .key(),
            );

            if commit.is_none() {
                Ok(to_json_binary(&false)?)
            } else if commit == Some(NON_MEMBERSHIP_COMMITMENT_VALUE) {
                Ok(to_json_binary(&true)?)
            } else {
                unreachable!()
            }
        }
        QueryMsg::AccessManaged(msg) => access_managed::query(deps, env, msg).map_err(Into::into),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, _: Env, reply: Reply) -> Result<Response, ContractError> {
    if let Some(reply) = handle_consume_scheduled_op_reply(deps, reply)? {
        Err(StdError::generic_err(format!("unknown reply: {reply:?}")).into())
    } else {
        Ok(Response::new())
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

fn commit_membership_proof(
    deps: DepsMut,
    client_id: ClientId,
    proof_height: u64,
    proof: Bytes,
    path: Bytes,
    value: Bytes,
) -> Result<Response, ContractError> {
    let client_impl = client_impl(deps.as_ref(), client_id)?;

    query_light_client::<()>(
        deps.as_ref(),
        client_impl,
        LightClientQuery::VerifyMembership {
            client_id,
            height: proof_height,
            proof: proof.to_vec().into(),
            path: path.clone(),
            value: value.clone(),
        },
    )?;

    store_commit(
        deps,
        &MembershipProofPath {
            client_id,
            proof_height,
            path: path.clone(),
        }
        .key(),
        &commit(&value),
    );

    Ok(Response::new().add_event(
        Event::new(events::proof::COMMIT_MEMBERSHIP)
            .add_attribute(events::attribute::CLIENT_ID, client_id.to_string())
            .add_attribute(events::attribute::PROOF_HEIGHT, proof_height.to_string())
            .add_attribute(events::attribute::PATH, path.to_string())
            .add_attribute(events::attribute::VALUE, value.to_string()),
    ))
}

fn commit_non_membership_proof(
    deps: DepsMut,
    client_id: ClientId,
    proof_height: u64,
    proof: Bytes,
    path: Bytes,
) -> Result<Response, ContractError> {
    let client_impl = client_impl(deps.as_ref(), client_id)?;

    query_light_client::<()>(
        deps.as_ref(),
        client_impl,
        LightClientQuery::VerifyNonMembership {
            client_id,
            height: proof_height,
            proof: proof.to_vec().into(),
            path: path.clone(),
        },
    )?;

    store_commit(
        deps,
        &NonMembershipProofPath {
            client_id,
            proof_height,
            path: path.clone(),
        }
        .key(),
        &commit(NON_MEMBERSHIP_COMMITMENT_VALUE),
    );

    Ok(Response::new().add_event(
        Event::new(events::proof::COMMIT_NON_MEMBERSHIP)
            .add_attribute(events::attribute::CLIENT_ID, client_id.to_string())
            .add_attribute(events::attribute::PROOF_HEIGHT, proof_height.to_string())
            .add_attribute(events::attribute::PATH, path.to_string()),
    ))
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
