use std::cmp::Ordering;

use enumorph::Enumorph;
use ibc_classic_spec::IbcClassic;
use ibc_union_spec::{query::PacketsByBatchHash, IbcUnion};
use jsonrpsee::{core::RpcResult, types::ErrorObject};
use macros::model;
use serde_json::json;
use tracing::{debug, info, instrument, warn};
use unionlabs::{ibc::core::client::height::Height, primitives::Bytes};
use voyager_sdk::{
    message::{
        call::FetchUpdateHeaders,
        data::{EventProvableHeight, IbcDatagram},
        PluginMessage, VoyagerMessage,
    },
    primitives::{ChainId, QueryHeight},
    rpc::MISSING_STATE_ERROR_CODE,
    types::RawClientId,
    vm::{data, now, promise, Op},
    VoyagerClient,
};

use crate::{
    call,
    callback::{make_msgs, MakeBatchTransaction, MakeIbcMessagesFromUpdate, ModuleCallback},
    data::{BatchableEvent, EventClassic, EventUnion},
    IbcSpecExt, Module,
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
    pub async fn call(
        self,
        module: &Module,
        voyager_client: &VoyagerClient,
    ) -> RpcResult<Op<VoyagerMessage>> {
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

        let target_height = self
            .batches
            .iter()
            .flatten()
            .map(|e| e.provable_height)
            .reduce(|acc, elem| match (elem, acc) {
                (EventProvableHeight::Min(elem), EventProvableHeight::Min(acc)) => {
                    // the min target height of a batch of `Min` events is the highest min height
                    // given the batch [10, 11, 12]
                    // the min height that all events are provable at is 12
                    EventProvableHeight::Min(elem.max(acc))
                }
                (EventProvableHeight::Exactly(elem), EventProvableHeight::Exactly(acc)) => {
                    assert_eq!(elem, acc, "multiple exact heights in the batch");
                    EventProvableHeight::Exactly(elem)
                }
                tuple => {
                    panic!("cannot mix exact and min provable heights currently (found {tuple:?})");
                }
            })
            .expect("batch has at least one event; qed;");

        // at this point we assume that a valid update exists - we only ever enqueue this message behind the relevant WaitForHeight on the counterparty chain. to prevent explosions, we do a sanity check here.
        {
            let (EventProvableHeight::Min(target_height)
            | EventProvableHeight::Exactly(target_height)) = target_height;

            if latest_height < target_height {
                return Err(ErrorObject::owned(
                    // we treat this as a missing state error, since this message assumes the state exists.
                    MISSING_STATE_ERROR_CODE,
                    format!(
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
                    ),
                    Some(json!({
                        "current_timestamp": now(),
                    })),
                ));
            }
        }

        match target_height {
            EventProvableHeight::Min(target_height) => {
                if client_state_meta.counterparty_height >= target_height {
                    info!(
                        "client {client_id} has already been updated to a height \
                        >= the desired target height ({} >= {target_height})",
                        client_state_meta.counterparty_height,
                        client_id = self.client_id,
                    );

                    make_msgs(
                        module,
                        self.client_id,
                        self.batches,
                        None,
                        client_state_meta.clone(),
                        client_state_meta.counterparty_height,
                    )
                } else {
                    Ok(promise(
                        [call(FetchUpdateHeaders {
                            client_type: client_info.client_type,
                            counterparty_chain_id: module.chain_id.clone(),
                            chain_id: client_state_meta.counterparty_chain_id,
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
                                batches: self.batches,
                            }),
                        ),
                    ))
                }
            }
            EventProvableHeight::Exactly(target_height) => {
                match client_state_meta.counterparty_height.cmp(&target_height) {
                    Ordering::Equal => {
                        info!(
                            "client {client_id} has already been updated to \
                            the desired target height ({} == {target_height})",
                            client_state_meta.counterparty_height,
                            client_id = self.client_id,
                        );
                        make_msgs(
                            module,
                            self.client_id,
                            self.batches,
                            None,
                            client_state_meta.clone(),
                            client_state_meta.counterparty_height,
                        )
                    }
                    Ordering::Less => Ok(promise(
                        [call(FetchUpdateHeaders {
                            client_type: client_info.client_type,
                            counterparty_chain_id: module.chain_id.clone(),
                            chain_id: client_state_meta.counterparty_chain_id,
                            client_id: RawClientId::new(self.client_id.clone()),
                            update_from: client_state_meta.counterparty_height,
                            update_to: target_height,
                        })],
                        [],
                        PluginMessage::new(
                            module.plugin_name(),
                            ModuleCallback::from(MakeIbcMessagesFromUpdate::<V> {
                                client_id: self.client_id.clone(),
                                batches: self.batches,
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

                        Ok(promise(
                            [call(FetchUpdateHeaders {
                                client_type: client_info.client_type,
                                counterparty_chain_id: module.chain_id.clone(),
                                chain_id: client_state_meta.counterparty_chain_id,
                                client_id: RawClientId::new(self.client_id.clone()),
                                update_from: client_state_meta.counterparty_height,
                                update_to: target_height,
                            })],
                            [],
                            PluginMessage::new(
                                module.plugin_name(),
                                ModuleCallback::from(MakeIbcMessagesFromUpdate::<V> {
                                    client_id: self.client_id.clone(),
                                    batches: self.batches,
                                }),
                            ),
                        ))
                    }
                }
            }
        }
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
    pub async fn call(self, voyager_client: &VoyagerClient) -> RpcResult<Op<VoyagerMessage>> {
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
                        ibc_union_spec::path::BatchPacketsPath::from_packets(&[packet.clone()]),
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
                    .await?;

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
                        ibc_union_spec::path::BatchReceiptsPath::from_packets(&[packet.clone()]),
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
                let ConnectionHandshakeStateAndProof {
                    connection_state,
                    encoded_connection_state_proof,
                } = mk_connection_handshake_state_and_proofs(
                    voyager_client,
                    origin_chain_id,
                    target_chain_id,
                    connection_open_init_event.client_id.clone(),
                    connection_open_init_event.counterparty_client_id.clone(),
                    connection_open_init_event.connection_id.clone(),
                    origin_chain_proof_height,
                )
                .await?;

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
) -> RpcResult<ConnectionHandshakeStateAndProof> {
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

    Ok(ConnectionHandshakeStateAndProof {
        connection_state,
        encoded_connection_state_proof,
    })
}

struct ConnectionHandshakeStateAndProof {
    connection_state: unionlabs::ibc::core::connection::connection_end::ConnectionEnd,
    encoded_connection_state_proof: Bytes,
}
