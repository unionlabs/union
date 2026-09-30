use core::slice;
use std::cmp::Ordering;

use enumorph::Enumorph;
use ibc_classic_spec::IbcClassic;
use ibc_union_spec::{IbcUnion, query::PacketsByBatchHash};
use itertools::Itertools;
use macros::model;
use serde_json::json;
use tracing::{debug, info, instrument, warn};
use unionlabs::{ibc::core::client::height::Height, primitives::Bytes};
use voyager_sdk::{
    VoyagerClient,
    message::{
        PluginMessage, VoyagerMessage,
        call::FetchUpdateHeaders,
        data::{EventProvableHeight, IbcDatagram},
    },
    primitives::{ChainId, QueryHeight},
    rpc::{RpcError, RpcResult},
    types::RawClientId,
    vm::{Op, conc, data, noop, now, promise},
};

use crate::{
    IbcSpecExt, Module, call,
    callback::{MakeBatchTransaction, MakeIbcMessagesFromUpdate, ModuleCallback, make_msgs},
    data::{BatchableEvent, EventClassic, EventUnion},
};

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    MakeTransactionBatchesWithUpdateClassic(MakeTransactionBatchesWithUpdate<IbcClassic>),
    MakeTransactionBatchesWithUpdateUnion(MakeTransactionBatchesWithUpdate<IbcUnion>),

    MakeMsgClassic(MakeMsg<IbcClassic>),
    MakeMsgUnion(MakeMsg<IbcUnion>),
}

/// Constructs multiple batch transactions, where all of the batches are provable at the new consensus height.
#[model]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MakeTransactionBatchesWithUpdate<V: IbcSpecExt> {
    pub client_id: V::ClientId,
    pub batches: Vec<Vec<BatchableEvent<V>>>,
}

impl<V: IbcSpecExt> MakeTransactionBatchesWithUpdate<V>
where
    ModuleCall: From<MakeMsg<V>> + From<MakeTransactionBatchesWithUpdate<V>>,
    ModuleCallback: From<MakeBatchTransaction<V>> + From<MakeIbcMessagesFromUpdate<V>>,
{
    #[instrument(skip_all, fields(client_id = %self.client_id, batches = self.batches.len()))]
    pub async fn call(
        self,
        module: &Module,
        voyager_client: &VoyagerClient,
    ) -> RpcResult<Op<VoyagerMessage>> {
        #[derive(Debug)]
        enum TargetHeights {
            None,
            Min(Height),
            Exact(Vec<Height>),
        }

        let client_state_meta = voyager_client
            .client_state_meta::<V>(
                module.chain_id.clone(),
                QueryHeight::Latest,
                self.client_id.clone(),
            )
            .await?;

        let client_info = voyager_client
            .client_info::<V>(module.chain_id.clone(), self.client_id.clone())
            .await?;

        let latest_height = voyager_client
            .query_latest_height(client_state_meta.counterparty_chain_id.clone(), true)
            .await?;

        let target_heights = self
            .batches
            .iter()
            .flatten()
            .map(|e| e.provable_height)
            .try_fold(TargetHeights::None, |acc, elem| match (elem, acc) {
                (EventProvableHeight::Min(elem), TargetHeights::None) => {
                    Ok(TargetHeights::Min(elem))
                }
                (EventProvableHeight::Exactly(elem), TargetHeights::None) => {
                    Ok(TargetHeights::Exact(vec![elem]))
                }

                (EventProvableHeight::Min(elem), TargetHeights::Min(acc)) => {
                    Ok(TargetHeights::Min(elem.max(acc)))
                }

                (EventProvableHeight::Exactly(elem), TargetHeights::Exact(acc)) => Ok(
                    TargetHeights::Exact(acc.into_iter().chain([elem]).collect()),
                ),

                (elem, acc) => Err(RpcError::fatal_from_message(format!(
                    "cannot mix exact and min update heights in a \
                    single instance of this plugin: {elem:?}, {acc:?}"
                ))),
            })?;

        let mut ops = vec![];

        // TODO: Check the same for exact heights?
        match target_heights {
            TargetHeights::Min(target_height) => {
                // at this point we assume that a valid update exists - we only ever enqueue this message behind the relevant WaitForHeight on the counterparty chain. to prevent explosions, we do a sanity check here.
                if latest_height < target_height {
                    // we treat this as a missing state error, since this message assumes the state exists.
                    return Err(RpcError::missing_state(format!(
                        "the latest height of the counterparty chain ({counterparty_chain_id}) \
                        is {latest_height} and the latest trusted height on the client tracking \
                        it ({client_id}) on this chain ({self_chain_id}) is {trusted_height}. \
                        in order to create an update for this client, we need to wait for the \
                        counterparty chain to progress to the next consensus checkpoint greater \
                        than the required target height {target_height}",
                        counterparty_chain_id = client_state_meta.counterparty_chain_id,
                        trusted_height = client_state_meta.counterparty_height,
                        client_id = self.client_id,
                        self_chain_id = module.chain_id,
                    ))
                    .with_data(json!({
                        "current_timestamp": now(),
                    })));
                }
                if client_state_meta.counterparty_height >= target_height {
                    info!(
                        "client {client_id} has already been updated to a height \
                        >= the desired target height ({} >= {target_height})",
                        client_state_meta.counterparty_height,
                        client_id = self.client_id,
                    );

                    ops.push(make_msgs(
                        module,
                        self.client_id,
                        self.batches,
                        None,
                        client_state_meta.clone(),
                        client_state_meta.counterparty_height,
                    )?);
                } else {
                    ops.push(promise(
                        [call(FetchUpdateHeaders {
                            client_type: client_info.client_type.clone(),
                            counterparty_chain_id: module.chain_id.clone(),
                            chain_id: client_state_meta.counterparty_chain_id.clone(),
                            client_id: RawClientId::new(self.client_id.clone()),
                            update_from: client_state_meta.counterparty_height,
                            update_to: if latest_height.height() < target_height.height() {
                                warn!(
                                    "latest height {latest_height} is less than the target \
                                    height {target_height}, there may be something wrong \
                                    with the rpc for {} - client {} will be updated to the \
                                    target height instead of the latest height",
                                    module.chain_id, self.client_id
                                );
                                target_height
                            } else {
                                latest_height
                            },
                        })],
                        [],
                        PluginMessage::new(
                            module.plugin_name(),
                            ModuleCallback::from(MakeIbcMessagesFromUpdate::<V> {
                                client_id: self.client_id.clone(),
                                batches: self.batches.clone(),
                            }),
                        ),
                    ));
                }
            }
            TargetHeights::None => todo!(),
            #[allow(unstable_name_collisions)]
            TargetHeights::Exact(target_heights) => {
                info!(
                    "found exact heights: [{}]",
                    target_heights
                        .iter()
                        .map(|h| h.to_string())
                        .intersperse(",".to_string())
                        .collect::<String>(),
                );

                for events in self.batches {
                    let EventProvableHeight::Exactly(target_height) = events[0].provable_height
                    else {
                        panic!("???")
                    };

                    match client_state_meta.counterparty_height.cmp(&target_height) {
                        Ordering::Equal => {
                            info!(
                                "client {client_id} has already been updated to \
                                the desired target height ({} == {target_height})",
                                client_state_meta.counterparty_height,
                                client_id = self.client_id,
                            );

                            ops.push(make_msgs(
                                module,
                                self.client_id.clone(),
                                vec![events],
                                None,
                                client_state_meta.clone(),
                                client_state_meta.counterparty_height,
                            )?);
                        }
                        Ordering::Less => ops.push(promise(
                            [call(FetchUpdateHeaders {
                                client_type: client_info.client_type.clone(),
                                counterparty_chain_id: module.chain_id.clone(),
                                chain_id: client_state_meta.counterparty_chain_id.clone(),
                                client_id: RawClientId::new(self.client_id.clone()),
                                update_from: client_state_meta.counterparty_height,
                                update_to: target_height,
                            })],
                            [],
                            PluginMessage::new(
                                module.plugin_name(),
                                ModuleCallback::from(MakeIbcMessagesFromUpdate::<V> {
                                    client_id: self.client_id.clone(),
                                    batches: vec![events],
                                }),
                            ),
                        )),
                        // update backwards
                        // currently this is only supported in sui, and as such has some baked-in assumptions about the semantics of when this branch is hit
                        Ordering::Greater => {
                            info!(
                                "updating client to an earlier height ({} -> {target_height})",
                                client_state_meta.counterparty_height
                            );

                            ops.push(promise(
                                [call(FetchUpdateHeaders {
                                    client_type: client_info.client_type.clone(),
                                    counterparty_chain_id: module.chain_id.clone(),
                                    chain_id: client_state_meta.counterparty_chain_id.clone(),
                                    client_id: RawClientId::new(self.client_id.clone()),
                                    update_from: client_state_meta.counterparty_height,
                                    update_to: target_height,
                                })],
                                [],
                                PluginMessage::new(
                                    module.plugin_name(),
                                    ModuleCallback::from(MakeIbcMessagesFromUpdate::<V> {
                                        client_id: self.client_id.clone(),
                                        batches: vec![events],
                                    }),
                                ),
                            ));
                        }
                    }
                }
            }
        }

        Ok(conc(ops))
    }
}

#[model]
pub struct MakeMsg<V: IbcSpecExt> {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId,
    /// The original event that was emitted on the origin chain.
    pub event: V::BatchableEvent,
}

impl MakeMsg<IbcUnion> {
    #[instrument(
        skip_all,
        fields(
            origin_chain_id = %self.origin_chain_id,
            origin_chain_proof_height = %self.origin_chain_proof_height,
            target_chain_id = %self.target_chain_id,
            msg = IbcUnion::event_name(&self.event)
        )
    )]
    pub async fn call<C: jsonrpsee::core::client::ClientT + Send + Sync>(
        self,
        voyager_client: &voyager_sdk::client::VoyagerClient<C>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let MakeMsg {
            origin_chain_id,
            origin_chain_proof_height,
            target_chain_id,
            event,
        } = self;

        match event {
            EventUnion::ConnectionOpenInit(connection_open_init_event) => {
                let counterparty_client_id = connection_open_init_event.counterparty_client_id;
                let connection_id = connection_open_init_event.connection_id;

                // info of the client on the target chain that will verify the storage
                // proofs
                let target_client_info = voyager_client
                    // counterparty_client_id from open_init/try is the client on the target chain
                    .client_info::<IbcUnion>(target_chain_id.clone(), counterparty_client_id)
                    .await?;

                debug!(
                    %counterparty_client_id,
                    %target_client_info.client_type,
                    %target_client_info.ibc_interface,
                    %target_client_info.metadata,
                    "counterparty client info"
                );

                // the connection end as stored by the origin chain after open_init/try
                let connection_state = voyager_client
                    .query_ibc_state(
                        origin_chain_id.clone(),
                        QueryHeight::Specific(origin_chain_proof_height),
                        ibc_union_spec::path::ConnectionPath { connection_id },
                    )
                    .await?;
                debug!(
                    connection_state = %serde_json::to_string(&connection_state).unwrap(),
                    "connection"
                );

                if !handshake_state_matches(
                    connection_state.state,
                    ibc_union_spec::ConnectionState::Init,
                ) {
                    return Ok(noop());
                }

                // proof of connection_state, encoded for the client on the target chain
                let connection_proof = voyager_client
                    .query_ibc_proof(
                        origin_chain_id.clone(),
                        QueryHeight::Specific(origin_chain_proof_height),
                        ibc_union_spec::path::ConnectionPath { connection_id },
                    )
                    .await?
                    .into_result()?
                    .proof;
                debug!(%connection_proof, "connection proof");

                let encoded_connection_state_proof = voyager_client
                    .encode_proof::<IbcUnion>(
                        target_client_info.client_type.clone(),
                        target_client_info.ibc_interface.clone(),
                        connection_proof,
                    )
                    .await?;
                debug!(%encoded_connection_state_proof, "encoded connection proof");

                Ok(data(IbcDatagram::new::<IbcUnion>(
                    ibc_union_spec::datagram::Datagram::from(
                        ibc_union_spec::datagram::MsgConnectionOpenTry {
                            client_id: connection_open_init_event.counterparty_client_id,
                            counterparty_client_id: connection_open_init_event.client_id,
                            counterparty_connection_id: connection_open_init_event.connection_id,
                            proof_height: origin_chain_proof_height.height(),
                            proof_init: encoded_connection_state_proof,
                        },
                    ),
                )))
            }

            EventUnion::ConnectionOpenTry(connection_open_try_event) => {
                let counterparty_client_id = connection_open_try_event.counterparty_client_id;
                let connection_id = connection_open_try_event.connection_id;

                // info of the client on the target chain that will verify the storage
                // proofs
                let target_client_info = voyager_client
                    // counterparty_client_id from open_init/try is the client on the target chain
                    .client_info::<IbcUnion>(target_chain_id.clone(), counterparty_client_id)
                    .await?;

                debug!(
                    %counterparty_client_id,
                    %target_client_info.client_type,
                    %target_client_info.ibc_interface,
                    %target_client_info.metadata,
                    "counterparty client info"
                );

                // the connection end as stored by the origin chain after open_init/try
                let connection_state = voyager_client
                    .query_ibc_state(
                        origin_chain_id.clone(),
                        QueryHeight::Specific(origin_chain_proof_height),
                        ibc_union_spec::path::ConnectionPath { connection_id },
                    )
                    .await?;
                debug!(
                    connection_state = %serde_json::to_string(&connection_state).unwrap(),
                    "connection"
                );

                if !handshake_state_matches(
                    connection_state.state,
                    ibc_union_spec::ConnectionState::TryOpen,
                ) {
                    return Ok(noop());
                }

                // proof of connection_state, encoded for the client on the target chain
                let connection_proof = voyager_client
                    .query_ibc_proof(
                        origin_chain_id.clone(),
                        QueryHeight::Specific(origin_chain_proof_height),
                        ibc_union_spec::path::ConnectionPath { connection_id },
                    )
                    .await?
                    .into_result()?
                    .proof;
                debug!(%connection_proof, "connection proof");

                let encoded_connection_state_proof = voyager_client
                    .encode_proof::<IbcUnion>(
                        target_client_info.client_type.clone(),
                        target_client_info.ibc_interface.clone(),
                        connection_proof,
                    )
                    .await?;
                debug!(%encoded_connection_state_proof, "encoded connection proof");

                Ok(data(IbcDatagram::new::<IbcUnion>(
                    ibc_union_spec::datagram::Datagram::from(
                        ibc_union_spec::datagram::MsgConnectionOpenAck {
                            connection_id: connection_open_try_event.counterparty_connection_id,
                            counterparty_connection_id: connection_open_try_event.connection_id,
                            proof_height: origin_chain_proof_height.height(),
                            proof_try: encoded_connection_state_proof,
                        },
                    ),
                )))
            }

            EventUnion::ConnectionOpenAck(connection_open_ack_event) => {
                let counterparty_client_id = connection_open_ack_event.counterparty_client_id;
                let connection_id = connection_open_ack_event.connection_id;

                // info of the client on the target chain that will verify the storage
                // proofs
                let target_client_info = voyager_client
                    // counterparty_client_id from open_init/ack is the client on the target chain
                    .client_info::<IbcUnion>(target_chain_id.clone(), counterparty_client_id)
                    .await?;

                debug!(
                    %counterparty_client_id,
                    %target_client_info.client_type,
                    %target_client_info.ibc_interface,
                    %target_client_info.metadata,
                );

                // the connection end as stored by the origin chain after open_init/ack
                let connection_state = voyager_client
                    .query_ibc_state(
                        origin_chain_id.clone(),
                        QueryHeight::Specific(origin_chain_proof_height),
                        ibc_union_spec::path::ConnectionPath { connection_id },
                    )
                    .await?;
                debug!(
                    connection_state = %serde_json::to_string(&connection_state).unwrap(),
                    "connection"
                );

                if !handshake_state_matches(
                    connection_state.state,
                    ibc_union_spec::ConnectionState::Open,
                ) {
                    return Ok(noop());
                }

                // proof of connection_state, encoded for the client on the target chain
                let connection_proof = voyager_client
                    .query_ibc_proof(
                        origin_chain_id.clone(),
                        QueryHeight::Specific(origin_chain_proof_height),
                        ibc_union_spec::path::ConnectionPath { connection_id },
                    )
                    .await?
                    .into_result()?
                    .proof;
                debug!(%connection_proof, "connection proof");

                let encoded_connection_state_proof = voyager_client
                    .encode_proof::<IbcUnion>(
                        target_client_info.client_type.clone(),
                        target_client_info.ibc_interface.clone(),
                        connection_proof,
                    )
                    .await?;
                debug!(%encoded_connection_state_proof, "encoded connection proof");

                Ok(data(IbcDatagram::new::<IbcUnion>(
                    ibc_union_spec::datagram::Datagram::from(
                        ibc_union_spec::datagram::MsgConnectionOpenConfirm {
                            connection_id: connection_open_ack_event.counterparty_connection_id,
                            proof_height: origin_chain_proof_height.height(),
                            proof_ack: encoded_connection_state_proof,
                        },
                    ),
                )))
            }

            EventUnion::ChannelOpenInit(event) => {
                let channel = voyager_client
                    .query_ibc_state(
                        origin_chain_id.clone(),
                        QueryHeight::Specific(origin_chain_proof_height),
                        ibc_union_spec::path::ChannelPath {
                            channel_id: event.channel_id,
                        },
                    )
                    .await?;
                if !handshake_state_matches(channel.state, ibc_union_spec::ChannelState::Init) {
                    return Ok(noop());
                }

                let proof = voyager_client
                    .query_ibc_proof(
                        origin_chain_id,
                        QueryHeight::Specific(origin_chain_proof_height),
                        ibc_union_spec::path::ChannelPath {
                            channel_id: event.channel_id,
                        },
                    )
                    .await?
                    .into_result()?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(
                        target_chain_id,
                        event.connection.counterparty_client_id,
                    )
                    .await?;

                let encoded_proof = voyager_client
                    .encode_proof::<IbcUnion>(
                        client_info.client_type,
                        client_info.ibc_interface,
                        proof.proof,
                    )
                    .await?;

                Ok(data(IbcDatagram::new::<IbcUnion>(
                    ibc_union_spec::datagram::Datagram::from(
                        ibc_union_spec::datagram::MsgChannelOpenTry {
                            port_id: event.counterparty_port_id,
                            channel: ibc_union_spec::Channel {
                                state: ibc_union_spec::ChannelState::TryOpen,
                                counterparty_channel_id: Some(event.channel_id),
                                counterparty_port_id: event.port_id,
                                connection_id: event.connection.counterparty_connection_id.unwrap(),
                                version: event.version.clone(),
                            },
                            counterparty_version: event.version,
                            proof_init: encoded_proof,
                            proof_height: origin_chain_proof_height.height(),
                        },
                    ),
                )))
            }

            EventUnion::ChannelOpenTry(event) => {
                let channel = voyager_client
                    .query_ibc_state(
                        origin_chain_id.clone(),
                        QueryHeight::Specific(origin_chain_proof_height),
                        ibc_union_spec::path::ChannelPath {
                            channel_id: event.channel_id,
                        },
                    )
                    .await?;
                if !handshake_state_matches(channel.state, ibc_union_spec::ChannelState::TryOpen) {
                    return Ok(noop());
                }

                let proof = voyager_client
                    .query_ibc_proof(
                        origin_chain_id,
                        QueryHeight::Specific(origin_chain_proof_height),
                        ibc_union_spec::path::ChannelPath {
                            channel_id: event.channel_id,
                        },
                    )
                    .await?
                    .into_result()?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(
                        target_chain_id,
                        event.connection.counterparty_client_id,
                    )
                    .await?;

                let encoded_proof = voyager_client
                    .encode_proof::<IbcUnion>(
                        client_info.client_type,
                        client_info.ibc_interface,
                        proof.proof,
                    )
                    .await?;

                Ok(data(IbcDatagram::new::<IbcUnion>(
                    ibc_union_spec::datagram::Datagram::from(
                        ibc_union_spec::datagram::MsgChannelOpenAck {
                            channel_id: event.counterparty_channel_id,
                            counterparty_channel_id: event.channel_id,
                            counterparty_version: event.version,
                            proof_try: encoded_proof,
                            proof_height: origin_chain_proof_height.height(),
                        },
                    ),
                )))
            }

            EventUnion::ChannelOpenAck(event) => {
                let channel = voyager_client
                    .query_ibc_state(
                        origin_chain_id.clone(),
                        QueryHeight::Specific(origin_chain_proof_height),
                        ibc_union_spec::path::ChannelPath {
                            channel_id: event.channel_id,
                        },
                    )
                    .await?;
                if !handshake_state_matches(channel.state, ibc_union_spec::ChannelState::Open) {
                    return Ok(noop());
                }

                let proof = voyager_client
                    .query_ibc_proof(
                        origin_chain_id,
                        QueryHeight::Specific(origin_chain_proof_height),
                        ibc_union_spec::path::ChannelPath {
                            channel_id: event.channel_id,
                        },
                    )
                    .await?
                    .into_result()?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(
                        target_chain_id,
                        event.connection.counterparty_client_id,
                    )
                    .await?;

                let encoded_proof = voyager_client
                    .encode_proof::<IbcUnion>(
                        client_info.client_type,
                        client_info.ibc_interface,
                        proof.proof,
                    )
                    .await?;

                Ok(data(IbcDatagram::new::<IbcUnion>(
                    ibc_union_spec::datagram::Datagram::from(
                        ibc_union_spec::datagram::MsgChannelOpenConfirm {
                            channel_id: event.counterparty_channel_id,
                            proof_ack: encoded_proof,
                            proof_height: origin_chain_proof_height.height(),
                        },
                    ),
                )))
            }

            EventUnion::PacketSend(event) => {
                let packet = event.packet();

                let proof = voyager_client
                    .query_ibc_proof(
                        origin_chain_id,
                        QueryHeight::Specific(origin_chain_proof_height),
                        ibc_union_spec::path::BatchPacketsPath::from_packets(slice::from_ref(
                            &packet,
                        )),
                    )
                    .await?
                    .into_result()?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(
                        target_chain_id,
                        event.packet.destination_channel.connection.client_id,
                    )
                    .await?;

                let encoded_proof = voyager_client
                    .encode_proof::<IbcUnion>(
                        client_info.client_type,
                        client_info.ibc_interface,
                        proof.proof,
                    )
                    .await?;

                Ok(data(IbcDatagram::new::<IbcUnion>(
                    ibc_union_spec::datagram::Datagram::from(
                        ibc_union_spec::datagram::MsgPacketRecv {
                            packets: vec![packet],
                            relayer_msgs: vec![vec![].into()],
                            proof: encoded_proof,
                            proof_height: origin_chain_proof_height.height(),
                        },
                    ),
                )))
            }

            EventUnion::BatchSend(event) => {
                let mut packets = voyager_client
                    .query(
                        origin_chain_id.clone(),
                        PacketsByBatchHash {
                            channel_id: event.source_channel.channel_id,
                            batch_hash: event.batch_hash,
                        },
                    )
                    .await?
                    .packets;

                packets.sort_by_cached_key(|packet| packet.hash());

                let proof = voyager_client
                    .query_ibc_proof(
                        origin_chain_id,
                        QueryHeight::Specific(origin_chain_proof_height),
                        ibc_union_spec::path::BatchPacketsPath {
                            batch_hash: event.batch_hash,
                        },
                    )
                    .await?
                    .into_result()?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(
                        target_chain_id,
                        event.destination_channel.connection.client_id,
                    )
                    .await?;

                let encoded_proof = voyager_client
                    .encode_proof::<IbcUnion>(
                        client_info.client_type,
                        client_info.ibc_interface,
                        proof.proof,
                    )
                    .await?;

                Ok(data(IbcDatagram::new::<IbcUnion>(
                    ibc_union_spec::datagram::Datagram::from(
                        ibc_union_spec::datagram::MsgPacketRecv {
                            relayer_msgs: vec![vec![].into(); packets.len()],
                            packets,
                            proof: encoded_proof,
                            proof_height: origin_chain_proof_height.height(),
                        },
                    ),
                )))
            }

            EventUnion::WriteAck(event) => {
                let packet = event.packet();

                let proof = voyager_client
                    .query_ibc_proof(
                        origin_chain_id,
                        QueryHeight::Specific(origin_chain_proof_height),
                        ibc_union_spec::path::BatchReceiptsPath::from_packets(slice::from_ref(
                            &packet,
                        )),
                    )
                    .await?
                    .into_result()?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(
                        target_chain_id,
                        event.packet.source_channel.connection.client_id,
                    )
                    .await?;

                let encoded_proof = voyager_client
                    .encode_proof::<IbcUnion>(
                        client_info.client_type,
                        client_info.ibc_interface,
                        proof.proof,
                    )
                    .await?;

                Ok(data(IbcDatagram::new::<IbcUnion>(
                    ibc_union_spec::datagram::Datagram::from(
                        ibc_union_spec::datagram::MsgPacketAcknowledgement {
                            packets: vec![packet],
                            acknowledgements: vec![event.acknowledgement],
                            proof: encoded_proof,
                            proof_height: origin_chain_proof_height.height(),
                        },
                    ),
                )))
            }
        }
    }
}

impl MakeMsg<IbcClassic> {
    pub async fn call(self, voyager_client: &VoyagerClient) -> RpcResult<Op<VoyagerMessage>> {
        let MakeMsg {
            origin_chain_id,
            origin_chain_proof_height,
            target_chain_id,
            event,
        } = self;

        match event {
            EventClassic::ConnectionOpenInit(connection_open_init_event) => {
                let Some(ConnectionHandshakeStateAndProof {
                    connection_state,
                    encoded_connection_state_proof,
                }) = mk_connection_handshake_state_and_proofs(
                    voyager_client,
                    origin_chain_id,
                    target_chain_id,
                    connection_open_init_event.client_id.clone(),
                    connection_open_init_event.counterparty_client_id.clone(),
                    connection_open_init_event.connection_id.clone(),
                    origin_chain_proof_height,
                )
                .await?
                else {
                    return Ok(noop());
                };

                Ok(data(IbcDatagram::new::<IbcClassic>(
                    ibc_classic_spec::Datagram::from(unionlabs::ibc::core::connection::msg_connection_open_try::MsgConnectionOpenTry {
                        client_id: connection_open_init_event.counterparty_client_id,
                        counterparty:
                            unionlabs::ibc::core::connection::counterparty::Counterparty {
                                client_id: connection_open_init_event.client_id,
                                connection_id: Some(connection_open_init_event.connection_id),
                                prefix: unionlabs::ibc::core::commitment::merkle_prefix::MerklePrefix {
                                    // TODO: Make configurable
                                    key_prefix: b"ibc".into(),
                                },
                            },
                        // TODO: Make configurable
                        delay_period: unionlabs::DELAY_PERIOD,
                        counterparty_versions: connection_state.versions,
                        proof_height: origin_chain_proof_height,
                        proof_init: encoded_connection_state_proof,
                    }),
                )))
            }

            // MakeMsgV1::MakeMsgConnectionOpenAck(MakeMsgConnectionOpenAck {
            //     origin_chain_id,
            //     origin_chain_proof_height,
            //     target_chain_id,
            //     connection_open_try_event,
            // }) => {
            //     let ConnectionHandshakeStateAndProofs {
            //         connection_state,
            //         encoded_connection_state_proof,
            //         consensus_height,
            //     } = mk_connection_handshake_state_and_proofs(
            //         &voyager_client,
            //         origin_chain_id,
            //         target_chain_id,
            //         connection_open_try_event.client_id,
            //         connection_open_try_event.counterparty_client_id,
            //         connection_open_try_event.connection_id.clone(),
            //         origin_chain_proof_height,
            //     )
            //     .await?;

            //     Ok(voyager_vm::data(IbcMessage::from(MsgConnectionOpenAck {
            //         connection_id: connection_open_try_event.counterparty_connection_id,
            //         counterparty_connection_id: connection_open_try_event.connection_id,
            //         client_state: encoded_client_state,
            //         version: connection_state.versions[0].clone(),
            //         proof_height: origin_chain_proof_height,
            //         proof_try: encoded_connection_state_proof,
            //         proof_client: encoded_client_state_proof,
            //         proof_consensus: encoded_consensus_state_proof,
            //         consensus_height,
            //     })))
            // }

            // MakeMsgV1::MakeMsgConnectionOpenConfirm(MakeMsgConnectionOpenConfirm {
            //     origin_chain_id,
            //     origin_chain_proof_height,
            //     target_chain_id,
            //     connection_open_ack_event,
            // }) => {
            //     // info of the client on the target chain that will verify the storage
            //     // proofs
            //     let target_client_info = &voyager_client
            //         .rpc_server
            //         // counterparty_client_id from open_try is the client on the target chain
            //         .client_info(
            //             &target_chain_id,
            //             connection_open_ack_event.counterparty_client_id.clone(),
            //         )
            //         .await
            //         ?;

            //     // proof of connection_state, encoded for the client on the target chain
            //     // this is encoded via the client module for the client on the origin chain
            //     // (the chain the event was emitted on)
            //     let connection_proof = &voyager_client
            //         .rpc_server
            //         .encode_proof(
            //             &target_client_info.client_type,
            //             &target_client_info.ibc_interface,
            //             &voyager_client
            //                 .rpc_server
            //                 .query_ibc_proof(
            //                     &origin_chain_id,
            //                     origin_chain_proof_height,
            //                     ConnectionPath {
            //                         connection_id: connection_open_ack_event.connection_id.clone(),
            //                     }
            //                     .into(),
            //                 )
            //                 .await
            //                 ?
            //                 .proof,
            //         )
            //         .await
            //         ?;

            //     Ok(voyager_vm::data(IbcMessage::from(
            //         MsgConnectionOpenConfirm {
            //             connection_id: connection_open_ack_event.counterparty_connection_id,
            //             proof_height: origin_chain_proof_height,
            //             proof_ack: connection_proof,
            //         },
            //     )))
            // }

            // MakeMsgV1::MakeMsgChannelOpenTry(MakeMsgChannelOpenTry {
            //     origin_chain_id,
            //     origin_chain_proof_height,
            //     target_chain_id,
            //     channel_open_init_event: event,
            // }) => {
            //     let origin_channel = voyager_client
            //         .query_channel(
            //             origin_chain_id.clone(),
            //             QueryHeight::Specific(origin_chain_proof_height),
            //             event.port_id.clone(),
            //             event.channel_id.clone(),
            //         )
            //         .await
            //         ?;

            //     let proof_init = voyager_client
            //         .query_ibc_proof(
            //             &origin_chain_id,
            //             origin_chain_proof_height,
            //             ChannelEndPath {
            //                 port_id: event.port_id.clone(),
            //                 channel_id: event.channel_id.clone(),
            //             }
            //             .into(),
            //         )
            //         .await
            //         ?;

            //     let client_info = voyager_client
            //         .client_info(&target_chain_id, event.connection.counterparty.client_id)
            //         .await
            //         ?;

            //     let encoded_proof_init = voyager_client
            //         .encode_proof(
            //             &client_info.client_type,
            //             &client_info.ibc_interface,
            //             proof_init.proof,
            //         )
            //         .await
            //         ?;

            //     Ok(data(IbcMessage::from(MsgChannelOpenTry {
            //         port_id: event.counterparty_port_id,
            //         channel: Channel {
            //             state: channel::state::State::Tryopen,
            //             ordering: origin_channel
            //                 .state
            //                 .ok_or(QueueError::Fatal("channel must exist".into()))?
            //                 .ordering,
            //             counterparty: channel::counterparty::Counterparty {
            //                 port_id: event.port_id,
            //                 channel_id: Some(event.channel_id),
            //             },
            //             connection_hops: vec![event.connection.counterparty.connection_id.unwrap()],
            //             version: event.version.clone(),
            //             upgrade_sequence: 0,
            //         },
            //         counterparty_version: event.version,
            //         proof_init: encoded_proof_init,
            //         proof_height: origin_chain_proof_height,
            //     })))
            // }

            // MakeMsgV1::MakeMsgChannelOpenAck(MakeMsgChannelOpenAck {
            //     origin_chain_id,
            //     origin_chain_proof_height,
            //     target_chain_id,
            //     channel_open_try_event,
            // }) => {
            //     let origin_channel_path = ChannelEndPath {
            //         port_id: channel_open_try_event.port_id.clone(),
            //         channel_id: channel_open_try_event.channel_id.clone(),
            //     };

            //     let proof_try = voyager_client
            //         .query_ibc_proof(
            //             &origin_chain_id,
            //             origin_chain_proof_height,
            //             origin_channel_path.into(),
            //         )
            //         .await
            //         ?;

            //     let client_info = voyager_client
            //         .client_info(
            //             &target_chain_id,
            //             channel_open_try_event.connection.counterparty.client_id,
            //         )
            //         .await
            //         ?;

            //     let encoded_proof_try = voyager_client
            //         .encode_proof(
            //             &client_info.client_type,
            //             &client_info.ibc_interface,
            //             proof_try.proof,
            //         )
            //         .await
            //         ?;

            //     Ok(data(IbcMessage::from(MsgChannelOpenAck {
            //         port_id: channel_open_try_event.counterparty_port_id,
            //         channel_id: channel_open_try_event.counterparty_channel_id,
            //         counterparty_channel_id: channel_open_try_event.channel_id,
            //         counterparty_version: channel_open_try_event.version,
            //         proof_try: encoded_proof_try,
            //         proof_height: origin_chain_proof_height,
            //     })))
            // }

            // MakeMsgV1::MakeMsgChannelOpenConfirm(MakeMsgChannelOpenConfirm {
            //     origin_chain_id,
            //     origin_chain_proof_height,
            //     target_chain_id,
            //     channel_open_ack_event,
            // }) => {
            //     let origin_channel_path = ChannelEndPath {
            //         port_id: channel_open_ack_event.port_id.clone(),
            //         channel_id: channel_open_ack_event.channel_id.clone(),
            //     };

            //     let proof_ack = voyager_client
            //         .query_ibc_proof(
            //             &origin_chain_id,
            //             origin_chain_proof_height,
            //             origin_channel_path.into(),
            //         )
            //         .await
            //         ?;

            //     let client_info = voyager_client
            //         .client_info(
            //             &target_chain_id,
            //             channel_open_ack_event.connection.counterparty.client_id,
            //         )
            //         .await
            //         ?;

            //     let encoded_proof_ack = voyager_client
            //         .encode_proof(
            //             &client_info.client_type,
            //             &client_info.ibc_interface,
            //             proof_ack.proof,
            //         )
            //         .await
            //         ?;

            //     Ok(voyager_vm::data(IbcMessage::from(MsgChannelOpenConfirm {
            //         port_id: channel_open_ack_event.counterparty_port_id,
            //         channel_id: channel_open_ack_event.counterparty_channel_id,
            //         proof_ack: encoded_proof_ack,
            //         proof_height: origin_chain_proof_height,
            //     })))
            // }

            // MakeMsgV1::MakeMsgRecvPacket(msg) => make_msg_recv_packet(ctx, msg).await,
            _ => todo!(),
        }
    }
}

/// Used to fetch and construct the state and proofs for
/// MsgConnectionOpenTry/Ack.
#[instrument(
    skip_all,
    fields(
        %origin_chain_id,
        %target_chain_id,
        %client_id,
        %counterparty_client_id,
        %connection_id,
        %origin_chain_proof_height,
    )
)]
async fn mk_connection_handshake_state_and_proofs(
    voyager_client: &VoyagerClient,
    origin_chain_id: ChainId,
    target_chain_id: ChainId,
    client_id: unionlabs::id::ClientId,
    counterparty_client_id: unionlabs::id::ClientId,
    connection_id: unionlabs::id::ConnectionId,
    origin_chain_proof_height: Height,
) -> RpcResult<Option<ConnectionHandshakeStateAndProof>> {
    // info of the client on the target chain that will verify the storage
    // proofs
    let target_client_info = voyager_client
        // counterparty_client_id from open_init/try is the client on the target chain
        .client_info::<IbcClassic>(target_chain_id.clone(), counterparty_client_id.clone())
        .await?;

    debug!(
        %counterparty_client_id,
        %target_client_info.client_type,
        %target_client_info.ibc_interface,
        %target_client_info.metadata,
    );

    // info of the client on the origin chain, this is used to decode the stored
    // client state
    let origin_client_info = voyager_client
        // client_id from open_init/try is the client on the origin chain
        .client_info::<IbcClassic>(origin_chain_id.clone(), client_id.clone())
        .await?;

    debug!(
        %client_id,
        %origin_client_info.client_type,
        %origin_client_info.ibc_interface,
        %origin_client_info.metadata,
    );

    // the connection end as stored by the origin chain after open_init/try
    let connection_state = voyager_client
        .query_ibc_state(
            origin_chain_id.clone(),
            QueryHeight::Specific(origin_chain_proof_height),
            ibc_classic_spec::ConnectionPath {
                connection_id: connection_id.clone(),
            },
        )
        .await?;
    debug!(
        connection_state = %serde_json::to_string(&connection_state).unwrap(),
    );

    if !handshake_state_matches(
        connection_state.state,
        unionlabs::ibc::core::connection::state::State::Init,
    ) {
        return Ok(None);
    }

    // proof of connection_state, encoded for the client on the target chain
    let connection_proof = voyager_client
        .query_ibc_proof(
            origin_chain_id.clone(),
            QueryHeight::Specific(origin_chain_proof_height),
            ibc_classic_spec::ConnectionPath {
                connection_id: connection_id.clone(),
            },
        )
        .await?
        .into_result()?
        .proof;
    debug!(%connection_proof);

    let encoded_connection_state_proof = voyager_client
        .encode_proof::<IbcClassic>(
            target_client_info.client_type.clone(),
            target_client_info.ibc_interface.clone(),
            connection_proof,
        )
        .await?;
    debug!(%encoded_connection_state_proof);

    Ok(Some(ConnectionHandshakeStateAndProof {
        connection_state,
        encoded_connection_state_proof,
    }))
}

struct ConnectionHandshakeStateAndProof {
    connection_state: unionlabs::ibc::core::connection::connection_end::ConnectionEnd,
    encoded_connection_state_proof: Bytes,
}

/// Ignore events whose source state has changed by the height used for the proof.
/// RPC and missing-state errors still propagate from the state query.
fn handshake_state_matches<S: PartialEq + std::fmt::Display>(actual: S, expected: S) -> bool {
    if actual == expected {
        true
    } else {
        info!(%actual, %expected, "ignoring handshake event with unexpected source state");
        false
    }
}

#[cfg(test)]
mod tests {
    use ibc_union_spec::{ChannelState, ConnectionState};
    use unionlabs::ibc::core::connection::state::State;

    use super::handshake_state_matches;

    #[test]
    fn union_connection_handshakes_require_exact_source_state() {
        let states = [
            ConnectionState::Init,
            ConnectionState::TryOpen,
            ConnectionState::Open,
        ];
        for expected in states {
            for actual in states {
                assert_eq!(
                    handshake_state_matches(actual, expected),
                    actual == expected
                );
            }
        }
    }

    #[test]
    fn union_channel_handshakes_reject_stale_and_closed_states() {
        let states = [
            ChannelState::Init,
            ChannelState::TryOpen,
            ChannelState::Open,
            ChannelState::Closed,
        ];
        for expected in [
            ChannelState::Init,
            ChannelState::TryOpen,
            ChannelState::Open,
        ] {
            for actual in states {
                assert_eq!(
                    handshake_state_matches(actual, expected),
                    actual == expected
                );
            }
        }
    }

    #[test]
    fn classic_connection_open_init_requires_init_state() {
        for actual in [
            State::UninitializedUnspecified,
            State::Init,
            State::Tryopen,
            State::Open,
        ] {
            assert_eq!(
                handshake_state_matches(actual, State::Init),
                actual == State::Init
            );
        }
    }

    async fn run_union_handshake(event_name: &str, state: Option<&str>, rpc_error: bool) {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };

        use ibc_union_spec::IbcUnion;
        use jsonrpsee::{
            RpcModule, http_client::HttpClientBuilder, server::ServerBuilder,
            types::ErrorObjectOwned,
        };
        use serde_json::{Value, json};
        use unionlabs::ibc::core::client::height::Height;
        use voyager_sdk::{client::VoyagerClient, primitives::ChainId, vm::Op};

        use super::{EventUnion, MakeMsg};

        let height = Height::new(42);
        let is_channel = event_name.starts_with("channel");
        let expected = if event_name.ends_with("init") {
            "init"
        } else if event_name.ends_with("try") {
            "try_open"
        } else {
            "open"
        };
        let state = state.map(str::to_owned);
        let returned_state = state.clone();
        let proofs = Arc::new(AtomicUsize::new(0));
        let mut rpc = RpcModule::new(());
        rpc.register_method(
            "voyager_clientInfo",
            |_, _, _| json!({"client_type": "test", "ibc_interface": "test"}),
        )
        .unwrap();
        rpc.register_method("voyager_queryIbcState", move |params, _, _| {
            let params: Vec<Value> = params.parse().unwrap();
            assert_eq!(params[0], "origin");
            assert_eq!(params[2], json!(height));
            let path = if is_channel { "channel" } else { "connection" };
            assert_eq!(params[3][path][if is_channel { "channel_id" } else { "connection_id" }], 1);
            if rpc_error {
                return Err(ErrorObjectOwned::owned(-1, "state RPC failed", None::<()>));
            }
            let stored = returned_state.as_ref().map(|state| if is_channel {
                json!({"state": state, "connection_id": 1, "counterparty_channel_id": 2, "counterparty_port_id": "0x01", "version": "test"})
            } else {
                json!({"state": state, "client_id": 1, "counterparty_client_id": 2, "counterparty_connection_id": 2})
            });
            Ok::<_, ErrorObjectOwned>(json!({"height": height, "state": stored}))
        }).unwrap();
        let proof_count = proofs.clone();
        rpc.register_method("voyager_queryIbcProof", move |params, _, _| {
            let params: Vec<Value> = params.parse().unwrap();
            assert_eq!(params[0], "origin");
            assert_eq!(params[2], json!(height));
            proof_count.fetch_add(1, Ordering::SeqCst);
            json!({"Proof": {"proof_type": "membership", "height": height, "proof": {}}})
        })
        .unwrap();
        rpc.register_method("voyager_encodeProof", |_, _, _| "0x01")
            .unwrap();
        let server = ServerBuilder::default().build("127.0.0.1:0").await.unwrap();
        let address = server.local_addr().unwrap();
        let handle = server.start(rpc);
        let client = VoyagerClient::new(
            HttpClientBuilder::default()
                .build(format!("http://{address}"))
                .unwrap(),
        );
        let connection = json!({"state": "open", "client_id": 1, "counterparty_client_id": 2, "counterparty_connection_id": 2});
        let mut value = if is_channel {
            json!({"port_id": "0x01", "channel_id": 1, "counterparty_port_id": "0x02", "connection": connection, "version": "test"})
        } else {
            json!({"connection_id": 1, "client_id": 1, "counterparty_client_id": 2})
        };
        if !event_name.ends_with("init") {
            value[if is_channel {
                "counterparty_channel_id"
            } else {
                "counterparty_connection_id"
            }] = json!(2);
        }
        let event: EventUnion =
            serde_json::from_value(json!({"@type": event_name, "@value": value})).unwrap();
        let result = MakeMsg::<IbcUnion> {
            origin_chain_id: ChainId::new("origin"),
            target_chain_id: ChainId::new("target"),
            origin_chain_proof_height: height,
            event,
        }
        .call(&client)
        .await;
        if rpc_error || state.is_none() {
            assert!(result.is_err(), "{event_name}: query errors must propagate");
            assert_eq!(proofs.load(Ordering::SeqCst), 0);
        } else if state.as_deref() == Some(expected) {
            assert!(
                matches!(result.unwrap(), Op::Data(_)),
                "{event_name}: matching state must emit a datagram"
            );
            assert_eq!(proofs.load(Ordering::SeqCst), 1);
        } else {
            assert!(
                matches!(result.unwrap(), Op::Noop),
                "{event_name}: mismatching state must skip the datagram"
            );
            assert_eq!(proofs.load(Ordering::SeqCst), 0);
        }
        handle.stop().unwrap();
        handle.stopped().await;
    }

    #[tokio::test]
    async fn union_handshake_calls_check_state_before_querying_proofs() {
        for event in [
            "connection_open_init",
            "connection_open_try",
            "connection_open_ack",
            "channel_open_init",
            "channel_open_try",
            "channel_open_ack",
        ] {
            for state in ["init", "try_open", "open"] {
                run_union_handshake(event, Some(state), false).await;
            }
            if event.starts_with("channel") {
                run_union_handshake(event, Some("closed"), false).await;
            }
            run_union_handshake(event, None, false).await;
            run_union_handshake(event, None, true).await;
        }
    }
}
