use alloy::sol_types::SolValue;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    wasm_execute, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdResult,
};
use ibc_solidity::ibc::{
    Channel, ChannelOrder, ChannelState, Connection, ConnectionState, MsgChannelCloseConfirm,
    MsgChannelCloseInit, MsgChannelOpenAck, MsgChannelOpenConfirm, MsgChannelOpenInit,
    MsgChannelOpenTry, MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
    MsgConnectionOpenTry, MsgCreateClient, MsgIntentPacketRecv, MsgPacketRecv, MsgUpdateClient,
    Packet,
};
use sha2::Digest as _;
use sha3::Keccak256;
use unionlabs::{ethereum::keccak256, hash::H256, ics24::ethabi::COMMITMENT_MAGIC, uint::U256};

use crate::{
    lightclient::query::{QueryMsg as LightClientQuery, VerifyClientMessageUpdate},
    module::msg::ExecuteMsg as ModuleMsg,
    msg::{ExecuteMsg, InitMsg, MsgSendPacket, MsgWriteAcknowledgement},
    query::QueryMsg,
    state::{
        CHANNELS, CHANNEL_OWNER, CLIENT_CONSENSUS_STATES, CLIENT_IMPLS, CLIENT_REGISTRY,
        CLIENT_STATES, CLIENT_TYPES, CONNECTIONS, NEXT_CHANNEL_ID, NEXT_CLIENT_ID,
        NEXT_CONNECTION_ID,
    },
    ContractError,
};

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
    let response = match msg {
        ExecuteMsg::RegisterClient(msg_register_client) => {
            if CLIENT_REGISTRY
                .may_load(deps.storage, &msg_register_client.client_type)?
                .is_some()
            {
                return Err(ContractError::ClientTypeAlreadyExists);
            }
            CLIENT_REGISTRY.save(
                deps.storage,
                &msg_register_client.client_type,
                &msg_register_client.client_address,
            )?;

            Response::new().add_event(
                Event::new("client_registered")
                    .add_attribute("client_type", msg_register_client.client_type)
                    .add_attribute("client_address", msg_register_client.client_address),
            )
        }
        ExecuteMsg::CreateClient(MsgCreateClient {
            clientType,
            clientStateBytes,
            consensusStateBytes,
            relayer: _,
        }) => {
            let client_impl = CLIENT_REGISTRY.load(deps.storage, &clientType)?;
            let client_id = next_client_id(deps.branch())?;
            CLIENT_TYPES.save(deps.storage, client_id, &clientType)?;
            CLIENT_IMPLS.save(deps.storage, client_id, &client_impl)?;
            let latest_height = deps.querier.query_wasm_smart(
                &client_impl,
                &LightClientQuery::VerifyCreation {
                    client_id,
                    client_state: clientStateBytes.to_vec().into(),
                    consensus_state: consensusStateBytes.to_vec().into(),
                },
            )?;
            CLIENT_STATES.save(deps.storage, client_id, &clientStateBytes.to_vec())?;
            CLIENT_CONSENSUS_STATES.save(
                deps.storage,
                (client_id, latest_height),
                &consensusStateBytes.to_vec(),
            )?;
            store_commit(
                deps.branch(),
                &unionlabs::ics24::ethabi::client_state_key(client_id),
                &commit(clientStateBytes),
            )?;
            store_commit(
                deps.branch(),
                &unionlabs::ics24::ethabi::consensus_state_key(client_id, latest_height),
                &commit(consensusStateBytes),
            )?;
            Response::new().add_event(Event::new("client_created").add_attributes([
                ("client_type", clientType),
                ("client_id", client_id.to_string()),
            ]))
        }
        ExecuteMsg::UpdateClient(MsgUpdateClient {
            clientId,
            clientMessage,
            relayer: _,
        }) => {
            let client_impl = client_impl(deps.as_ref(), clientId)?;
            let update = deps.querier.query_wasm_smart::<VerifyClientMessageUpdate>(
                &client_impl,
                &LightClientQuery::VerifyClientMessage {
                    client_id: clientId,
                    message: clientMessage.to_vec().into(),
                },
            )?;
            store_commit(
                deps.branch(),
                &unionlabs::ics24::ethabi::client_state_key(clientId),
                &commit(update.client_state),
            )?;
            store_commit(
                deps.branch(),
                &unionlabs::ics24::ethabi::consensus_state_key(clientId, update.height),
                &commit(update.consensus_state),
            )?;
            Response::new().add_event(Event::new("client_updated").add_attributes([
                ("client_id", clientId.to_string()),
                ("height", update.height.to_string()),
            ]))
        }
        ExecuteMsg::ConnectionOpenInit(MsgConnectionOpenInit {
            clientId,
            counterpartyClientId,
            relayer: _,
        }) => {
            let connection_id = next_connection_id(deps.branch())?;
            let connection = Connection {
                state: ConnectionState::Init,
                clientId,
                counterpartyClientId,
                counterpartyConnectionId: 0,
            };
            save_connection(deps.branch(), connection_id, &connection)?;
            Response::new().add_event(Event::new("connection_open_init").add_attributes([
                ("connection_id", connection_id.to_string()),
                ("client_id", clientId.to_string()),
                ("counterparty_client_id", counterpartyClientId.to_string()),
            ]))
        }
        ExecuteMsg::ConnectionOpenTry(MsgConnectionOpenTry {
            counterpartyClientId,
            counterpartyConnectionId,
            clientId,
            proofInit,
            proofHeight,
            relayer: _,
        }) => {
            let connection_id = next_connection_id(deps.branch())?;
            let connection = Connection {
                state: ConnectionState::TryOpen,
                clientId,
                counterpartyClientId,
                counterpartyConnectionId,
            };
            let expected_connection = Connection {
                state: ConnectionState::Init,
                clientId: counterpartyClientId,
                counterpartyClientId: clientId,
                counterpartyConnectionId: 0,
            };
            let client_impl = client_impl(deps.as_ref(), clientId)?;
            deps.querier.query_wasm_smart(
                &client_impl,
                &LightClientQuery::VerifyMembership {
                    client_id: clientId,
                    height: proofHeight,
                    proof: proofInit.to_vec().into(),
                    path: unionlabs::ics24::ethabi::connection_key(counterpartyConnectionId)
                        .into_bytes()
                        .into(),
                    value: commit(expected_connection.abi_encode()).into_bytes().into(),
                },
            )?;
            save_connection(deps.branch(), connection_id, &connection)?;
            Response::new().add_event(Event::new("connection_open_try").add_attributes([
                ("connection_id", connection_id.to_string()),
                ("client_id", clientId.to_string()),
                ("counterparty_client_id", counterpartyClientId.to_string()),
                (
                    "counterparty_connection_id",
                    counterpartyConnectionId.to_string(),
                ),
            ]))
        }
        ExecuteMsg::ConnectionOpenAck(MsgConnectionOpenAck {
            connectionId,
            counterpartyConnectionId,
            proofTry,
            proofHeight,
            relayer: _,
        }) => {
            let mut connection = CONNECTIONS.load(deps.storage, connectionId)?;
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
                counterpartyConnectionId: connectionId,
            };
            let client_impl = client_impl(deps.as_ref(), connection.clientId)?;
            deps.querier.query_wasm_smart(
                &client_impl,
                &LightClientQuery::VerifyMembership {
                    client_id: connection.clientId,
                    height: proofHeight,
                    proof: proofTry.to_vec().into(),
                    path: unionlabs::ics24::ethabi::connection_key(counterpartyConnectionId)
                        .into_bytes()
                        .into(),
                    value: commit(expected_connection.abi_encode()).into_bytes().into(),
                },
            )?;
            connection.state = ConnectionState::Open;
            connection.counterpartyConnectionId = counterpartyConnectionId;
            save_connection(deps.branch(), connectionId, &connection)?;
            Response::new().add_event(Event::new("connection_open_ack").add_attributes([
                ("connection_id", connectionId.to_string()),
                ("client_id", connection.clientId.to_string()),
                (
                    "counterparty_client_id",
                    connection.counterpartyClientId.to_string(),
                ),
                (
                    "counterparty_connection_id",
                    connection.counterpartyConnectionId.to_string(),
                ),
            ]))
        }
        ExecuteMsg::ConnectionOpenConfirm(MsgConnectionOpenConfirm {
            connectionId,
            proofAck,
            proofHeight,
            relayer: _,
        }) => {
            let mut connection = CONNECTIONS.load(deps.storage, connectionId)?;
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
                counterpartyConnectionId: connectionId,
            };
            let client_impl = client_impl(deps.as_ref(), connection.clientId)?;
            deps.querier.query_wasm_smart(
                &client_impl,
                &LightClientQuery::VerifyMembership {
                    client_id: connection.clientId,
                    height: proofHeight,
                    proof: proofAck.to_vec().into(),
                    path: unionlabs::ics24::ethabi::connection_key(
                        connection.counterpartyConnectionId,
                    )
                    .into_bytes()
                    .into(),
                    value: commit(expected_connection.abi_encode()).into_bytes().into(),
                },
            )?;
            connection.state = ConnectionState::Open;
            save_connection(deps.branch(), connectionId, &connection)?;
            Response::new().add_event(Event::new("connection_open_confirm").add_attributes([
                ("connection_id", connectionId.to_string()),
                ("client_id", connection.clientId.to_string()),
                (
                    "counterparty_client_id",
                    connection.counterpartyClientId.to_string(),
                ),
                (
                    "counterparty_connection_id",
                    connection.counterpartyConnectionId.to_string(),
                ),
            ]))
        }
        ExecuteMsg::ChannelOpenInit(MsgChannelOpenInit {
            portId,
            counterpartyPortId,
            connectionId,
            ordering,
            version,
            relayer,
        }) => {
            let relayer = deps.api.addr_validate(&relayer)?;
            let port_id = deps.api.addr_validate(&portId)?;
            if ordering != ChannelOrder::Ordered && ordering != ChannelOrder::Unordered {
                return Err(ContractError::ChannelInvalidOrdering { got: ordering });
            }
            ensure_connection_state(deps.as_ref(), connectionId)?;
            let channel_id = next_channel_id(deps.branch())?;
            let channel = Channel {
                state: ChannelState::Init,
                ordering,
                connectionId,
                counterpartyChannelId: 0,
                counterpartyPortId: counterpartyPortId.clone(),
                version: version.clone(),
            };
            save_channel(deps.branch(), channel_id, &channel)?;
            CHANNEL_OWNER.save(deps.storage, channel_id, &port_id)?;
            initialize_channel_sequences(deps.branch(), channel_id)?;
            Response::new()
                .add_event(Event::new("channel_open_init").add_attributes([
                    ("port_id", port_id.to_string()),
                    ("channel_id", channel_id.to_string()),
                    ("counterparty_port_id", counterpartyPortId),
                    ("connection_id", connectionId.to_string()),
                    ("version", version.clone()),
                ]))
                .add_message(wasm_execute(
                    port_id,
                    &ModuleMsg::OnChannelOpenInit {
                        order: channel.ordering,
                        connection_id: connectionId,
                        channel_id,
                        version,
                        relayer,
                    },
                    vec![],
                )?)
        }
        ExecuteMsg::ChannelOpenTry(MsgChannelOpenTry {
            portId,
            channel,
            counterpartyVersion,
            proofInit,
            proofHeight,
            relayer,
        }) => {
            if channel.ordering != ChannelOrder::Ordered
                && channel.ordering != ChannelOrder::Unordered
            {
                return Err(ContractError::ChannelInvalidOrdering {
                    got: channel.ordering,
                });
            }
            if channel.state != ChannelState::TryOpen {
                return Err(ContractError::ChannelInvalidState {
                    got: channel.state,
                    expected: ChannelState::TryOpen,
                });
            }
            let connection = ensure_connection_state(deps.as_ref(), channel.connectionId)?;
            let expected_channel = Channel {
                state: ChannelState::Init,
                ordering: channel.ordering,
                connectionId: connection.counterpartyConnectionId,
                counterpartyChannelId: 0,
                counterpartyPortId: portId.clone(),
                version: counterpartyVersion.clone(),
            };
            let client_impl = client_impl(deps.as_ref(), connection.clientId)?;
            deps.querier.query_wasm_smart(
                &client_impl,
                &LightClientQuery::VerifyMembership {
                    client_id: connection.clientId,
                    height: proofHeight,
                    proof: proofInit.to_vec().into(),
                    path: unionlabs::ics24::ethabi::channel_key(channel.counterpartyChannelId)
                        .into_bytes()
                        .into(),
                    value: commit(expected_channel.abi_encode()).into_bytes().into(),
                },
            )?;
            let channel_id = next_channel_id(deps.branch())?;
            let port_id = deps.api.addr_validate(&portId)?;
            let relayer = deps.api.addr_validate(&relayer)?;
            save_channel(deps.branch(), channel_id, &channel)?;
            CHANNEL_OWNER.save(deps.storage, channel_id, &port_id)?;
            initialize_channel_sequences(deps.branch(), channel_id)?;
            Response::new()
                .add_event(Event::new("channel_open_try").add_attributes([
                    ("port_id", port_id.to_string()),
                    ("channel_id", channel_id.to_string()),
                    ("counterparty_port_id", channel.counterpartyPortId),
                    (
                        "counterparty_channel_id",
                        channel.counterpartyChannelId.to_string(),
                    ),
                    ("connection_id", channel.connectionId.to_string()),
                    ("counterparty_version", counterpartyVersion.clone()),
                ]))
                .add_message(wasm_execute(
                    port_id,
                    &ModuleMsg::OnChannelOpenTry {
                        order: channel.ordering,
                        connection_id: channel.connectionId,
                        channel_id,
                        version: channel.version,
                        counterparty_version: counterpartyVersion,
                        relayer,
                    },
                    vec![],
                )?)
        }
        ExecuteMsg::ChannelOpenAck(MsgChannelOpenAck {
            channelId,
            counterpartyVersion,
            counterpartyChannelId,
            proofTry,
            proofHeight,
            relayer,
        }) => {
            let mut channel = CHANNELS.load(deps.storage, channelId)?;
            if channel.state != ChannelState::Init {
                return Err(ContractError::ChannelInvalidState {
                    got: channel.state,
                    expected: ChannelState::Init,
                });
            }
            let connection = ensure_connection_state(deps.as_ref(), channel.connectionId)?;
            let port_id = CHANNEL_OWNER.load(deps.storage, channelId)?;
            let expected_channel = Channel {
                state: ChannelState::TryOpen,
                ordering: channel.ordering,
                connectionId: connection.counterpartyConnectionId,
                counterpartyChannelId: channelId,
                counterpartyPortId: port_id.to_string(),
                version: counterpartyVersion.clone(),
            };
            let client_impl = client_impl(deps.as_ref(), connection.clientId)?;
            deps.querier.query_wasm_smart(
                &client_impl,
                &LightClientQuery::VerifyMembership {
                    client_id: connection.clientId,
                    height: proofHeight,
                    proof: proofTry.to_vec().into(),
                    path: unionlabs::ics24::ethabi::channel_key(counterpartyChannelId)
                        .into_bytes()
                        .into(),
                    value: commit(expected_channel.abi_encode()).into_bytes().into(),
                },
            )?;
            channel.state = ChannelState::Open;
            channel.version = counterpartyVersion.clone();
            channel.counterpartyChannelId = counterpartyChannelId;
            save_channel(deps.branch(), channelId, &channel)?;
            let relayer = deps.api.addr_validate(&relayer)?;
            Response::new()
                .add_event(Event::new("channel_open_ack").add_attributes([
                    ("port_id", port_id.to_string()),
                    ("channel_id", channelId.to_string()),
                    ("counterparty_port_id", channel.counterpartyPortId),
                    (
                        "counterparty_channel_id",
                        channel.counterpartyChannelId.to_string(),
                    ),
                    ("connection_id", channel.connectionId.to_string()),
                ]))
                .add_message(wasm_execute(
                    port_id,
                    &ModuleMsg::OnChannelOpenAck {
                        channel_id: channelId,
                        counterparty_channel_id: counterpartyChannelId,
                        counterparty_version: counterpartyVersion,
                        relayer,
                    },
                    vec![],
                )?)
        }
        ExecuteMsg::ChannelOpenConfirm(MsgChannelOpenConfirm {
            channelId,
            proofAck,
            proofHeight,
            relayer,
        }) => {
            let mut channel = CHANNELS.load(deps.storage, channelId)?;
            if channel.state != ChannelState::TryOpen {
                return Err(ContractError::ChannelInvalidState {
                    got: channel.state,
                    expected: ChannelState::TryOpen,
                });
            }
            let connection = ensure_connection_state(deps.as_ref(), channel.connectionId)?;
            let port_id = CHANNEL_OWNER.load(deps.storage, channelId)?;
            let expected_channel = Channel {
                state: ChannelState::Open,
                ordering: channel.ordering,
                connectionId: connection.counterpartyConnectionId,
                counterpartyChannelId: channelId,
                counterpartyPortId: port_id.to_string(),
                version: channel.version.clone(),
            };
            let client_impl = client_impl(deps.as_ref(), connection.clientId)?;
            deps.querier.query_wasm_smart(
                &client_impl,
                &LightClientQuery::VerifyMembership {
                    client_id: connection.clientId,
                    height: proofHeight,
                    proof: proofAck.to_vec().into(),
                    path: unionlabs::ics24::ethabi::channel_key(channel.counterpartyChannelId)
                        .into_bytes()
                        .into(),
                    value: commit(expected_channel.abi_encode()).into_bytes().into(),
                },
            )?;
            channel.state = ChannelState::Open;
            save_channel(deps.branch(), channelId, &channel)?;
            let relayer = deps.api.addr_validate(&relayer)?;
            Response::new()
                .add_event(Event::new("channel_open_confirm").add_attributes([
                    ("port_id", port_id.to_string()),
                    ("channel_id", channelId.to_string()),
                    ("counterparty_port_id", channel.counterpartyPortId),
                    (
                        "counterparty_channel_id",
                        channel.counterpartyChannelId.to_string(),
                    ),
                    ("connection_id", channel.connectionId.to_string()),
                ]))
                .add_message(wasm_execute(
                    port_id,
                    &ModuleMsg::OnChannelOpenConfirm {
                        channel_id: channelId,
                        relayer,
                    },
                    vec![],
                )?)
        }
        ExecuteMsg::ChannelCloseInit(MsgChannelCloseInit { channelId, relayer }) => {
            let mut channel = CHANNELS.load(deps.storage, channelId)?;
            if channel.state != ChannelState::Open {
                return Err(ContractError::ChannelInvalidState {
                    got: channel.state,
                    expected: ChannelState::Open,
                });
            }
            ensure_connection_state(deps.as_ref(), channel.connectionId)?;
            channel.state = ChannelState::Closed;
            save_channel(deps.branch(), channelId, &channel)?;
            let port_id = CHANNEL_OWNER.load(deps.storage, channelId)?;
            let relayer = deps.api.addr_validate(&relayer)?;
            Response::new()
                .add_event(Event::new("channel_close_init").add_attributes([
                    ("port_id", port_id.to_string()),
                    ("channel_id", channelId.to_string()),
                    ("counterparty_port_id", channel.counterpartyPortId),
                    (
                        "counterparty_channel_id",
                        channel.counterpartyChannelId.to_string(),
                    ),
                ]))
                .add_message(wasm_execute(
                    port_id,
                    &ModuleMsg::OnChannelCloseInit {
                        channel_id: channelId,
                        relayer,
                    },
                    vec![],
                )?)
        }
        ExecuteMsg::ChannelCloseConfirm(MsgChannelCloseConfirm {
            channelId,
            proofInit,
            proofHeight,
            relayer,
        }) => {
            let mut channel = CHANNELS.load(deps.storage, channelId)?;
            if channel.state != ChannelState::Open {
                return Err(ContractError::ChannelInvalidState {
                    got: channel.state,
                    expected: ChannelState::Open,
                });
            }
            let connection = ensure_connection_state(deps.as_ref(), channel.connectionId)?;
            let port_id = CHANNEL_OWNER.load(deps.storage, channelId)?;
            let expected_channel = Channel {
                state: ChannelState::Closed,
                ordering: channel.ordering,
                connectionId: connection.counterpartyConnectionId,
                counterpartyChannelId: channelId,
                counterpartyPortId: port_id.to_string(),
                version: channel.version.clone(),
            };
            let client_impl = client_impl(deps.as_ref(), connection.clientId)?;
            deps.querier.query_wasm_smart(
                &client_impl,
                &LightClientQuery::VerifyMembership {
                    client_id: connection.clientId,
                    height: proofHeight,
                    proof: proofInit.to_vec().into(),
                    path: unionlabs::ics24::ethabi::channel_key(channel.counterpartyChannelId)
                        .into_bytes()
                        .into(),
                    value: commit(expected_channel.abi_encode()).into_bytes().into(),
                },
            )?;
            channel.state = ChannelState::Closed;
            CHANNELS.save(deps.storage, channelId, &channel)?;
            store_commit(
                deps.branch(),
                &unionlabs::ics24::ethabi::channel_key(channelId),
                &commit(channel.abi_encode()),
            )?;
            let relayer = deps.api.addr_validate(&relayer)?;
            Response::new()
                .add_event(Event::new("channel_close_confirm").add_attributes([
                    ("port_id", port_id.to_string()),
                    ("channel_id", channelId.to_string()),
                    ("counterparty_port_id", channel.counterpartyPortId),
                    (
                        "counterparty_channel_id",
                        channel.counterpartyChannelId.to_string(),
                    ),
                ]))
                .add_message(wasm_execute(
                    port_id,
                    &ModuleMsg::OnChannelOpenConfirm {
                        channel_id: channelId,
                        relayer,
                    },
                    vec![],
                )?)
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
        )?,
        ExecuteMsg::PacketAck(_) => todo!(),
        ExecuteMsg::PacketTimeout(_msg_packet_timeout) => todo!(),
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
        )?,
        ExecuteMsg::BatchSend(_msg_batch_send) => todo!(),
        ExecuteMsg::BatchAcks(_msg_batch_acks) => todo!(),
        ExecuteMsg::WriteAcknowledgement(MsgWriteAcknowledgement {
            channel_id,
            packet,
            acknowledgement,
        }) => {
            // make sure the caller owns the channel
            let port_id = CHANNEL_OWNER.load(deps.storage, channel_id)?;
            if port_id != info.sender {
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

            Response::new().add_event(Event::new("write_acknowledgement").add_attributes([
                ("packet", serde_json::to_string(&packet).unwrap()),
                ("acknowledgement", hex::encode(acknowledgement)),
            ]))
        }
        ExecuteMsg::PacketSend(MsgSendPacket {
            source_channel,
            timeout_height,
            timeout_timestamp,
            data,
        }) => {
            if timeout_timestamp == 0 && timeout_height == 0 {
                return Err(ContractError::TimeoutMustBeSet);
            }

            let port_id = CHANNEL_OWNER.load(deps.storage, source_channel)?;
            if port_id != info.sender {
                return Err(ContractError::Unauthorized);
            }

            let channel = ensure_channel_state(deps.as_ref(), source_channel)?;
            let sequence = generate_packet_sequence(deps.branch(), source_channel)?;
            let packet = Packet {
                sequence,
                sourceChannel: source_channel,
                destinationChannel: channel.counterpartyChannelId,
                data: data.into(),
                timeoutHeight: timeout_height,
                timeoutTimestamp: timeout_timestamp,
            };

            store_commit(
                deps.branch(),
                &unionlabs::ics24::ethabi::batch_packets_key(
                    source_channel,
                    commit_packet(&packet),
                ),
                &COMMITMENT_MAGIC,
            )?;

            Response::new().add_event(
                Event::new("send_packet")
                    .add_attribute("packet", serde_json::to_string(&packet).unwrap()),
            )
        }
    };

    Ok(response)
}

fn commit_packet(packet: &Packet) -> H256 {
    Keccak256::new()
        .chain_update(packet.abi_encode())
        .finalize()
        .into()
}

fn commit_packets(packets: &[Packet]) -> H256 {
    Keccak256::new()
        .chain_update(packets.abi_encode())
        .finalize()
        .into()
}

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
    if packets.is_empty() {
        return Err(ContractError::NoPacketsReceived);
    }

    let source_channel = packets[0].sourceChannel;
    let destination_channel = packets[0].destinationChannel;

    let channel = ensure_channel_state(deps.as_ref(), destination_channel)?;
    let connection = ensure_connection_state(deps.as_ref(), channel.connectionId)?;

    if !intent {
        let proof_commitment_key = match packets.len() {
            1 => unionlabs::ics24::ethabi::batch_receipts_key(
                source_channel,
                commit_packet(&packets[0]),
            ),
            _ => unionlabs::ics24::ethabi::batch_receipts_key(
                source_channel,
                commit_packets(&packets),
            ),
        };

        let client_impl = client_impl(deps.as_ref(), connection.clientId)?;
        #[allow(dependency_on_unit_never_type_fallback)]
        deps.querier.query_wasm_smart(
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
                events.push(Event::new("recv_intent_packet").add_attributes([
                    ("packet", serde_json::to_string(&packet).unwrap()),
                    ("maker", relayer.clone()),
                    // TODO(aeryz): should this be hex?
                    ("maker_msg", hex::encode(&relayer_msg)),
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
                events.push(Event::new("recv_packet").add_attributes([
                    ("packet", serde_json::to_string(&packet).unwrap()),
                    ("relayer", relayer.clone()),
                    // TODO(aeryz): should this be hex?
                    ("relayer_msg", hex::encode(&relayer_msg)),
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    todo!()
}

fn next_channel_id(deps: DepsMut) -> Result<u32, ContractError> {
    let channel_id = NEXT_CHANNEL_ID.may_load(deps.storage)?.unwrap_or_default();
    NEXT_CHANNEL_ID.save(
        deps.storage,
        &channel_id
            .checked_add(1)
            .ok_or(ContractError::ArithmeticOverflow)?,
    )?;
    Ok(channel_id)
}

fn next_connection_id(deps: DepsMut) -> Result<u32, ContractError> {
    let connection_id = NEXT_CONNECTION_ID
        .may_load(deps.storage)?
        .unwrap_or_default();
    NEXT_CONNECTION_ID.save(
        deps.storage,
        &connection_id
            .checked_add(1)
            .ok_or(ContractError::ArithmeticOverflow)?,
    )?;
    Ok(connection_id)
}

fn next_client_id(deps: DepsMut) -> Result<u32, ContractError> {
    let client_id = NEXT_CLIENT_ID.may_load(deps.storage)?.unwrap_or_default();
    NEXT_CLIENT_ID.save(
        deps.storage,
        &client_id
            .checked_add(1)
            .ok_or(ContractError::ArithmeticOverflow)?,
    )?;
    Ok(client_id)
}

fn client_impl(deps: Deps, client_id: u32) -> Result<Addr, ContractError> {
    Ok(CLIENT_IMPLS.load(deps.storage, client_id)?)
}

fn commit(bytes: impl AsRef<[u8]>) -> H256 {
    keccak256(bytes)
}

fn commit_ack(ack: Vec<u8>) -> H256 {
    merge_ack(Keccak256::new().chain_update(ack).finalize().into())
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
    CONNECTIONS.save(deps.storage, connection_id, &connection)?;
    store_commit(
        deps,
        &unionlabs::ics24::ethabi::connection_key(connection_id),
        &commit(connection.abi_encode()),
    )?;
    Ok(())
}

fn save_channel(deps: DepsMut, channel_id: u32, channel: &Channel) -> Result<(), ContractError> {
    CHANNELS.save(deps.storage, channel_id, &channel)?;
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

fn generate_packet_sequence(deps: DepsMut, channel_id: u32) -> Result<u64, ContractError> {
    let commitment_key = unionlabs::ics24::ethabi::next_seq_send_key(channel_id);
    let seq = U256::from_be_bytes(
        deps.storage
            .get(commitment_key.as_ref())
            .ok_or(ContractError::ChannelNotExist(channel_id))?
            .try_into()
            .unwrap(),
    );

    store_commit(
        deps,
        &commitment_key,
        &H256::from((seq + 1.into()).to_be_bytes()),
    )?;

    Ok(seq.0.as_u128() as u64)
}

fn initialize_channel_sequences(mut deps: DepsMut, channel_id: u32) -> Result<(), ContractError> {
    store_commit(
        deps.branch(),
        &unionlabs::ics24::ethabi::next_seq_recv_key(channel_id),
        &H256::from(U256::from(1u64).to_be_bytes()),
    )?;
    store_commit(
        deps.branch(),
        &unionlabs::ics24::ethabi::next_seq_send_key(channel_id),
        &H256::from(U256::from(1u64).to_be_bytes()),
    )?;
    store_commit(
        deps.branch(),
        &unionlabs::ics24::ethabi::next_seq_ack_key(channel_id),
        &H256::from(U256::from(1u64).to_be_bytes()),
    )?;
    Ok(())
}
