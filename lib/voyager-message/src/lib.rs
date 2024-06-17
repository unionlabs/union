#![feature(min_exhaustive_patterns)]
#![allow(clippy::large_enum_variant)]
#![warn(clippy::large_futures)]

use std::{collections::VecDeque, fmt::Debug, str::FromStr};

use block_message::BlockMessage;
use chain_utils::{
    arbitrum::Arbitrum, berachain::Berachain, cosmos::Cosmos, ethereum::Ethereum, scroll::Scroll,
    union::Union, wasm::Wasm, Chains,
};
use futures::TryFutureExt;
use queue_msg::{
    event, noop, queue_msg, HandleAggregate, HandleData, HandleEffect, HandleEvent, HandleFetch,
    HandleWait, Op, QueueError, QueueMessage,
};
use relay_message::{AnyLightClientIdentified, RelayMessage};
use tracing::{info_span, Instrument};
use unionlabs::{
    ethereum::config::{Mainnet, Minimal},
    events::{
        AcknowledgePacket, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry,
        ClientMisbehaviour, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
        ConnectionOpenTry, CreateClient, IbcEvent, RecvPacket, SendPacket, SubmitEvidence,
        TimeoutPacket, UpdateClient, WriteAcknowledgement,
    },
    traits::{ChainIdOf, ClientIdOf, ClientTypeOf, HeightOf},
    ClientType, WasmClientType,
};

pub enum VoyagerMessage {}

impl QueueMessage for VoyagerMessage {
    type Event = VoyagerEvent;
    type Data = VoyagerData;
    type Fetch = VoyagerFetch;
    type Effect = VoyagerMsg;
    type Wait = VoyagerWait;
    type Aggregate = VoyagerAggregate;

    type Store = Chains;
}

pub trait FromOp<T: QueueMessage>: QueueMessage + Sized {
    fn from_op(value: Op<T>) -> Op<Self>;
}

impl FromOp<RelayMessage> for VoyagerMessage {
    fn from_op(value: Op<RelayMessage>) -> Op<Self> {
        match value {
            Op::Event(event) => Op::Event(VoyagerEvent::Relay(event)),
            Op::Data(data) => Op::Data(VoyagerData::Relay(data)),
            Op::Fetch(fetch) => Op::Fetch(VoyagerFetch::Relay(fetch)),
            Op::Effect(msg) => Op::Effect(VoyagerMsg::Relay(msg)),
            Op::Wait(wait) => Op::Wait(VoyagerWait::Relay(wait)),
            Op::Defer(defer) => Op::Defer(defer),
            Op::Repeat { times, msg } => Op::Repeat {
                times,
                msg: Box::new(Self::from_op(*msg)),
            },
            Op::Timeout {
                timeout_timestamp,
                msg,
            } => Op::Timeout {
                timeout_timestamp,
                msg: Box::new(Self::from_op(*msg)),
            },
            Op::Seq(seq) => Op::Seq(seq.into_iter().map(Self::from_op).collect()),
            Op::Conc(seq) => Op::Conc(seq.into_iter().map(Self::from_op).collect()),
            Op::Retry { remaining, msg } => Op::Retry {
                remaining,
                msg: Box::new(Self::from_op(*msg)),
            },
            Op::Aggregate {
                queue,
                data,
                receiver,
            } => Op::Aggregate {
                queue: queue.into_iter().map(Self::from_op).collect(),
                data: data.into_iter().map(VoyagerData::Relay).collect(),
                receiver: VoyagerAggregate::Relay(receiver),
            },
            Op::Race(seq) => Op::Race(seq.into_iter().map(Self::from_op).collect()),
            Op::Void(msg) => Op::Void(Box::new(Self::from_op(*msg))),
            Op::Noop => noop(),
        }
    }
}

impl FromOp<BlockMessage> for VoyagerMessage {
    fn from_op(value: Op<BlockMessage>) -> Op<Self> {
        match value {
            Op::Data(data) => Op::Data(VoyagerData::Block(data)),
            Op::Fetch(fetch) => Op::Fetch(VoyagerFetch::Block(fetch)),
            Op::Wait(wait) => Op::Wait(VoyagerWait::Block(wait)),
            Op::Defer(defer) => Op::Defer(defer),
            Op::Repeat { times, msg } => Op::Repeat {
                times,
                msg: Box::new(Self::from_op(*msg)),
            },
            Op::Timeout {
                timeout_timestamp,
                msg,
            } => Op::Timeout {
                timeout_timestamp,
                msg: Box::new(Self::from_op(*msg)),
            },
            Op::Seq(seq) => Op::Seq(seq.into_iter().map(Self::from_op).collect()),
            Op::Conc(seq) => Op::Conc(seq.into_iter().map(Self::from_op).collect()),
            Op::Race(seq) => Op::Race(seq.into_iter().map(Self::from_op).collect()),
            Op::Retry { remaining, msg } => Op::Retry {
                remaining,
                msg: Box::new(Self::from_op(*msg)),
            },
            Op::Aggregate {
                queue,
                data,
                receiver,
            } => Op::Aggregate {
                queue: queue.into_iter().map(Self::from_op).collect(),
                data: data.into_iter().map(VoyagerData::Block).collect(),
                receiver: VoyagerAggregate::Block(receiver),
            },
            Op::Void(msg) => Op::Void(Box::new(Self::from_op(*msg))),
            Op::Noop => noop(),
        }
    }
}

#[queue_msg]
pub enum VoyagerMsg {
    Block(<BlockMessage as QueueMessage>::Effect),
    Relay(<RelayMessage as QueueMessage>::Effect),
}

impl HandleEffect<VoyagerMessage> for VoyagerMsg {
    async fn handle(
        self,
        store: &<VoyagerMessage as QueueMessage>::Store,
    ) -> Result<Op<VoyagerMessage>, QueueError> {
        Ok(match self {
            Self::Relay(msg) => {
                Box::pin(msg.handle(store))
                    .map_ok(VoyagerMessage::from_op)
                    .instrument(info_span!("relay"))
                    .await?
            }
        })
    }
}

#[queue_msg]
pub enum VoyagerWait {
    Block(<BlockMessage as QueueMessage>::Wait),
    Relay(<RelayMessage as QueueMessage>::Wait),
}

impl HandleWait<VoyagerMessage> for VoyagerWait {
    async fn handle(
        self,
        store: &<VoyagerMessage as QueueMessage>::Store,
    ) -> Result<Op<VoyagerMessage>, QueueError> {
        Ok(match self {
            Self::Block(msg) => {
                Box::pin(HandleWait::<BlockMessage>::handle(msg, store))
                    .map_ok(VoyagerMessage::from_op)
                    .instrument(info_span!("block"))
                    .await?
            }
            Self::Relay(msg) => {
                Box::pin(HandleWait::<RelayMessage>::handle(msg, store))
                    .map_ok(VoyagerMessage::from_op)
                    .instrument(info_span!("relay"))
                    .await?
            }
        })
    }
}

#[queue_msg]
pub enum VoyagerAggregate {
    Block(<BlockMessage as QueueMessage>::Aggregate),
    Relay(<RelayMessage as QueueMessage>::Aggregate),
}

impl HandleAggregate<VoyagerMessage> for VoyagerAggregate {
    fn handle(
        self,
        data: VecDeque<<VoyagerMessage as QueueMessage>::Data>,
    ) -> Result<Op<VoyagerMessage>, QueueError> {
        match self {
            Self::Block(aggregate) => {
                let _span = info_span!("block").entered();
                aggregate
                    .handle(
                        data.into_iter()
                            .map(|d| match d {
                                VoyagerData::Block(d) => d,
                                VoyagerData::Relay(_) => {
                                    panic!("found relay message in data of block message aggregate")
                                }
                            })
                            .collect(),
                    )
                    .map(VoyagerMessage::from_op)
            }
            Self::Relay(aggregate) => {
                let _span = info_span!("relay").entered();
                aggregate
                    .handle(
                        data.into_iter()
                            .map(|d| match d {
                                VoyagerData::Block(_) => {
                                    panic!("found block message in data of relay message aggregate")
                                }
                                VoyagerData::Relay(d) => d,
                            })
                            .collect(),
                    )
                    .map(VoyagerMessage::from_op)
            }
        }
    }
}

#[queue_msg]
pub enum VoyagerEvent {
    Block(<BlockMessage as QueueMessage>::Event),
    Relay(<RelayMessage as QueueMessage>::Event),
}

impl HandleEvent<VoyagerMessage> for VoyagerEvent {
    fn handle(
        self,
        store: &<VoyagerMessage as QueueMessage>::Store,
    ) -> Result<Op<VoyagerMessage>, QueueError> {
        match self {
            Self::Relay(event) => {
                let _span = info_span!("relay").entered();
                HandleEvent::handle(event, store).map(VoyagerMessage::from_op)
            }
        }
    }
}

#[queue_msg]
pub enum VoyagerData {
    Block(<BlockMessage as QueueMessage>::Data),
    Relay(<RelayMessage as QueueMessage>::Data),
}

impl HandleData<VoyagerMessage> for VoyagerData {
    fn handle(
        self,
        store: &<VoyagerMessage as QueueMessage>::Store,
    ) -> Result<Op<VoyagerMessage>, QueueError> {
        Ok(match self {
            Self::Block(data) => {
                macro_rules! block_data_to_relay_event {
                    ($($Variant:ident => {$(
                        $ClientType:pat => ($Hc:ty, $BlockHc:ty, $Tr:ty),
                    )*})*) => {
                        match data.handle(store)? {
                            $(Op::Data(block_message::AnyChainIdentified::$Variant(
                                block_message::Identified {
                                    chain_id,
                                    t: block_message::data::Data::IbcEvent(ibc_event),
                                },
                            )) => <VoyagerMessage as FromOp<RelayMessage>>::from_op(
                                match ibc_event.client_type {
                                    $($ClientType => {
                                        mk_relay_event::<$Hc, $BlockHc, $Tr>(chain_id, ibc_event)
                                    })*
                                    _ => unimplemented!(),
                                },
                            ),)*
                            msg => VoyagerMessage::from_op(msg),
                        }
                    };
                }

                let _span = info_span!("block").entered();

                block_data_to_relay_event!(
                    Cosmos => {
                        ClientType::Wasm(WasmClientType::Cometbls)        => (Wasm<Cosmos>, Cosmos, Union),
                        ClientType::Tendermint                            => (Cosmos, Cosmos, Cosmos),
                    }
                    Union => {
                        ClientType::Wasm(WasmClientType::EthereumMinimal) => (Wasm<Union>, Union, Ethereum<Minimal>),
                        ClientType::Wasm(WasmClientType::EthereumMainnet) => (Wasm<Union>, Union, Ethereum<Mainnet>),
                        ClientType::Wasm(WasmClientType::Scroll)          => (Wasm<Union>, Union, Scroll),
                        ClientType::Wasm(WasmClientType::Arbitrum)        => (Wasm<Union>, Union, Arbitrum),
                        ClientType::Wasm(WasmClientType::Berachain)       => (Wasm<Union>, Union, Berachain),
                        ClientType::Tendermint                            => (Union, Union, Wasm<Cosmos>),
                    }
                    EthMainnet => {
                        ClientType::Cometbls                              => (Ethereum<Mainnet>, Ethereum<Mainnet>, Wasm<Union>),
                    }
                    EthMinimal => {
                        ClientType::Cometbls                              => (Ethereum<Minimal>, Ethereum<Minimal>, Wasm<Union>),
                    }
                    Scroll => {
                        ClientType::Cometbls                              => (Scroll, Scroll, Wasm<Union>),
                    }
                    Arbitrum => {
                        ClientType::Cometbls                              => (Arbitrum, Arbitrum, Wasm<Union>),
                    }
                    Berachain => {
                        ClientType::Cometbls                              => (Berachain, Berachain, Wasm<Union>),
                    }
                )
            }
            Self::Relay(data) => {
                let _span = info_span!("relay").entered();
                VoyagerMessage::from_op(data.handle(store)?)
            }
        })
    }
}

#[queue_msg]
pub enum VoyagerFetch {
    Block(<BlockMessage as QueueMessage>::Fetch),
    Relay(<RelayMessage as QueueMessage>::Fetch),
}

impl HandleFetch<VoyagerMessage> for VoyagerFetch {
    async fn handle(
        self,
        store: &<VoyagerMessage as QueueMessage>::Store,
    ) -> Result<Op<VoyagerMessage>, QueueError> {
        match self {
            Self::Block(fetch) => {
                fetch
                    .handle(store)
                    .map_ok(VoyagerMessage::from_op)
                    .instrument(info_span!("block"))
                    .await
            }
            Self::Relay(fetch) => {
                Box::pin(fetch.handle(store))
                    .map_ok(VoyagerMessage::from_op)
                    .instrument(info_span!("relay"))
                    .await
            }
        }
    }
}

fn mk_relay_event<Hc, BlockHc, Tr>(
    chain_id: ChainIdOf<Hc>,
    ibc_event: block_message::data::ChainEvent<BlockHc>,
) -> Op<RelayMessage>
where
    Hc: relay_message::ChainExt<
        Height = HeightOf<BlockHc>,
        ClientId = ClientIdOf<BlockHc>,
        ClientType = ClientTypeOf<BlockHc>,
    >,
    BlockHc: block_message::ChainExt,
    Tr: relay_message::ChainExt<ClientId: FromStr<Err: Debug>>,

    AnyLightClientIdentified<relay_message::event::AnyEvent>:
        From<relay_message::Identified<Hc, Tr, relay_message::event::Event<Hc, Tr>>>,
{
    event::<RelayMessage>(relay_message::id::<Hc, Tr, _>(
        chain_id,
        relay_message::event::IbcEvent {
            tx_hash: ibc_event.tx_hash,
            height: ibc_event.height,
            event: chain_event_to_lc_event::<Hc, Tr>(ibc_event.event),
        },
    ))
}

// poor man's monad
fn chain_event_to_lc_event<Hc, Tr>(
    event: IbcEvent<Hc::ClientId, Hc::ClientType, String>,
) -> IbcEvent<Hc::ClientId, Hc::ClientType, Tr::ClientId>
where
    Hc: relay_message::ChainExt,
    Tr: relay_message::ChainExt<ClientId: FromStr<Err: Debug>>,
{
    match event {
        IbcEvent::CreateClient(CreateClient {
            client_id,
            client_type,
            consensus_height,
        }) => IbcEvent::CreateClient(CreateClient {
            client_id,
            client_type,
            consensus_height,
        }),
        IbcEvent::UpdateClient(UpdateClient {
            client_id,
            client_type,
            consensus_heights,
        }) => IbcEvent::UpdateClient(UpdateClient {
            client_id,
            client_type,
            consensus_heights,
        }),
        IbcEvent::ClientMisbehaviour(ClientMisbehaviour {
            client_id,
            client_type,
            consensus_height,
        }) => IbcEvent::ClientMisbehaviour(ClientMisbehaviour {
            client_id,
            client_type,
            consensus_height,
        }),
        IbcEvent::SubmitEvidence(SubmitEvidence { evidence_hash }) => {
            IbcEvent::SubmitEvidence(SubmitEvidence { evidence_hash })
        }
        IbcEvent::ConnectionOpenInit(ConnectionOpenInit {
            connection_id,
            client_id,
            counterparty_client_id,
        }) => IbcEvent::ConnectionOpenInit(ConnectionOpenInit {
            connection_id,
            client_id,
            counterparty_client_id: counterparty_client_id.parse().unwrap(),
        }),
        IbcEvent::ConnectionOpenTry(ConnectionOpenTry {
            connection_id,
            client_id,
            counterparty_client_id,
            counterparty_connection_id,
        }) => IbcEvent::ConnectionOpenTry(ConnectionOpenTry {
            connection_id,
            client_id,
            counterparty_client_id: counterparty_client_id.parse().unwrap(),
            counterparty_connection_id,
        }),
        IbcEvent::ConnectionOpenAck(ConnectionOpenAck {
            connection_id,
            client_id,
            counterparty_client_id,
            counterparty_connection_id,
        }) => IbcEvent::ConnectionOpenAck(ConnectionOpenAck {
            connection_id,
            client_id,
            counterparty_client_id: counterparty_client_id.parse().unwrap(),
            counterparty_connection_id,
        }),
        IbcEvent::ConnectionOpenConfirm(ConnectionOpenConfirm {
            connection_id,
            client_id,
            counterparty_client_id,
            counterparty_connection_id,
        }) => IbcEvent::ConnectionOpenConfirm(ConnectionOpenConfirm {
            connection_id,
            client_id,
            counterparty_client_id: counterparty_client_id.parse().unwrap(),
            counterparty_connection_id,
        }),
        IbcEvent::ChannelOpenInit(ChannelOpenInit {
            port_id,
            channel_id,
            counterparty_port_id,
            connection_id,
            version,
        }) => IbcEvent::ChannelOpenInit(ChannelOpenInit {
            port_id,
            channel_id,
            counterparty_port_id,
            connection_id,
            version,
        }),
        IbcEvent::ChannelOpenTry(ChannelOpenTry {
            port_id,
            channel_id,
            counterparty_port_id,
            counterparty_channel_id,
            connection_id,
            version,
        }) => IbcEvent::ChannelOpenTry(ChannelOpenTry {
            port_id,
            channel_id,
            counterparty_port_id,
            counterparty_channel_id,
            connection_id,
            version,
        }),
        IbcEvent::ChannelOpenAck(ChannelOpenAck {
            port_id,
            channel_id,
            counterparty_port_id,
            counterparty_channel_id,
            connection_id,
        }) => IbcEvent::ChannelOpenAck(ChannelOpenAck {
            port_id,
            channel_id,
            counterparty_port_id,
            counterparty_channel_id,
            connection_id,
        }),
        IbcEvent::ChannelOpenConfirm(ChannelOpenConfirm {
            port_id,
            channel_id,
            counterparty_port_id,
            counterparty_channel_id,
            connection_id,
        }) => IbcEvent::ChannelOpenConfirm(ChannelOpenConfirm {
            port_id,
            channel_id,
            counterparty_port_id,
            counterparty_channel_id,
            connection_id,
        }),
        IbcEvent::WriteAcknowledgement(WriteAcknowledgement {
            packet_data_hex,
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_ack_hex,
            connection_id,
        }) => IbcEvent::WriteAcknowledgement(WriteAcknowledgement {
            packet_data_hex,
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_ack_hex,
            connection_id,
        }),
        IbcEvent::RecvPacket(RecvPacket {
            packet_data_hex,
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_channel_ordering,
            connection_id,
        }) => IbcEvent::RecvPacket(RecvPacket {
            packet_data_hex,
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_channel_ordering,
            connection_id,
        }),
        IbcEvent::SendPacket(SendPacket {
            packet_data_hex,
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_channel_ordering,
            connection_id,
        }) => IbcEvent::SendPacket(SendPacket {
            packet_data_hex,
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_channel_ordering,
            connection_id,
        }),
        IbcEvent::AcknowledgePacket(AcknowledgePacket {
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_channel_ordering,
            connection_id,
        }) => IbcEvent::AcknowledgePacket(AcknowledgePacket {
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_channel_ordering,
            connection_id,
        }),
        IbcEvent::TimeoutPacket(TimeoutPacket {
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_channel_ordering,
            connection_id,
        }) => IbcEvent::TimeoutPacket(TimeoutPacket {
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_channel_ordering,
            connection_id,
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use block_message::BlockMessage;
    use chain_utils::{
        cosmos::Cosmos, ethereum::Ethereum, scroll::Scroll, union::Union, wasm::Wasm,
    };
    use hex_literal::hex;
    use queue_msg::{
        aggregate, defer_relative, effect, event, fetch, repeat, seq, Op, QueueMessage,
    };
    use relay_message::{
        aggregate::AggregateMsgCreateClient,
        chain::{
            cosmos_sdk::{
                fetch::{AbciQueryType, FetchAbciQuery},
                wasm::WasmConfig,
            },
            ethereum::EthereumConfig,
        },
        effect::{MsgChannelOpenInitData, MsgConnectionOpenInitData},
        event::IbcEvent,
        fetch::{FetchSelfClientState, FetchSelfConsensusState},
        RelayMessage,
    };
    use unionlabs::{
        ethereum::config::Minimal,
        events::ConnectionOpenTry,
        hash::{H160, H256},
        ibc::core::{
            channel::{
                self, channel::Channel, msg_channel_open_init::MsgChannelOpenInit, order::Order,
            },
            commitment::merkle_prefix::MerklePrefix,
            connection::{self, msg_connection_open_init::MsgConnectionOpenInit, version::Version},
        },
        ics24,
        uint::U256,
        QueryHeight, DELAY_PERIOD,
    };

    use crate::{FromOp, VoyagerMessage};

    macro_rules! parse {
        ($expr:expr) => {
            $expr.parse().unwrap()
        };
    }

    #[test]
    fn msg_serde() {
        let union_chain_id: String = parse!("union-devnet-1");
        let eth_chain_id: U256 = parse!("32382");
        let simd_chain_id: String = parse!("simd-devnet-1");
        let scroll_chain_id: U256 = parse!("534351");
        let stargaze_chain_id: String = parse!("stargaze-devnet-1");
        let osmosis_chain_id: String = parse!("osmosis-devnet-1");

        println!("---------------------------------------");
        println!("Union - Eth (Sending to Union) Connection Open: ");
        println!("---------------------------------------");
        print_json::<RelayMessage>(effect(
            relay_message::id::<Wasm<Union>, Ethereum<Minimal>, _>(
                union_chain_id.clone(),
                MsgConnectionOpenInitData(MsgConnectionOpenInit {
                    client_id: parse!("08-wasm-0"),
                    counterparty: connection::counterparty::Counterparty {
                        client_id: parse!("cometbls-0"),
                        connection_id: parse!(""),
                        prefix: MerklePrefix {
                            key_prefix: b"ibc".to_vec(),
                        },
                    },
                    version: Version {
                        identifier: "1".into(),
                        features: [Order::Ordered, Order::Unordered].into_iter().collect(),
                    },
                    delay_period: DELAY_PERIOD,
                }),
            ),
        ));

        println!("---------------------------------------");
        println!("Fetch Client State: ");
        println!("---------------------------------------");
        print_json::<RelayMessage>(fetch(
            relay_message::id::<Wasm<Union>, Ethereum<Minimal>, _>(
                union_chain_id.clone(),
                relay_message::fetch::Fetch::specific(FetchAbciQuery {
                    path: ics24::Path::ClientState(ics24::ClientStatePath {
                        client_id: parse!("client-id"),
                    }),
                    height: parse!("123-456"),
                    ty: AbciQueryType::State,
                }),
            ),
        ));

        println!("---------------------------------------");
        println!("Eth - Union (Sending to Union) Channel Open: ");
        println!("---------------------------------------");
        print_json::<RelayMessage>(effect(
            relay_message::id::<Wasm<Union>, Ethereum<Minimal>, _>(
                union_chain_id.clone(),
                MsgChannelOpenInitData {
                    msg: MsgChannelOpenInit {
                        port_id: parse!("WASM_PORT_ID"),
                        channel: Channel {
                            state: channel::state::State::Init,
                            ordering: channel::order::Order::Unordered,
                            counterparty: channel::counterparty::Counterparty {
                                port_id: parse!("ucs01-relay"),
                                channel_id: parse!(""),
                            },
                            connection_hops: vec![parse!("connection-8")],
                            version: "ucs01-0".to_string(),
                        },
                    },
                    __marker: PhantomData,
                },
            ),
        ));

        println!("---------------------------------------");
        println!("Eth - Union (Starting on Union) Channel Open: ");
        println!("---------------------------------------");
        print_json::<RelayMessage>(effect(
            relay_message::id::<Ethereum<Minimal>, Wasm<Union>, _>(
                eth_chain_id,
                MsgChannelOpenInitData {
                    msg: MsgChannelOpenInit {
                        port_id: parse!("ucs01-relay"),
                        channel: Channel {
                            state: channel::state::State::Init,
                            ordering: channel::order::Order::Ordered,
                            counterparty: channel::counterparty::Counterparty {
                                port_id: parse!("ucs01-relay"),
                                channel_id: parse!(""),
                            },
                            connection_hops: vec![parse!("connection-8")],
                            version: "ucs001-pingpong".to_string(),
                        },
                    },
                    __marker: PhantomData,
                },
            ),
        ));

        println!("---------------------------------------");
        println!("Eth - Union (Sending to Eth) Connection Open: ");
        println!("---------------------------------------");
        print_json::<RelayMessage>(effect(
            relay_message::id::<Ethereum<Minimal>, Wasm<Union>, _>(
                eth_chain_id,
                MsgConnectionOpenInitData(MsgConnectionOpenInit {
                    client_id: parse!("cometbls-0"),
                    counterparty: connection::counterparty::Counterparty {
                        client_id: parse!("08-wasm-0"),
                        connection_id: parse!(""),
                        prefix: MerklePrefix {
                            key_prefix: b"ibc".to_vec(),
                        },
                    },
                    version: Version {
                        identifier: "1".into(),
                        features: [Order::Ordered, Order::Unordered].into_iter().collect(),
                    },
                    delay_period: DELAY_PERIOD,
                }),
            ),
        ));

        println!("---------------------------------------");
        println!("Eth - Union (Sending to Eth) Connection Try: ");
        println!("---------------------------------------");
        print_json::<RelayMessage>(event(
            relay_message::id::<Ethereum<Minimal>, Wasm<Union>, _>(
                eth_chain_id,
                IbcEvent {
                    tx_hash: H256([0; 32]),
                    height: parse!("0-2941"),
                    event: unionlabs::events::IbcEvent::ConnectionOpenTry(ConnectionOpenTry {
                        connection_id: parse!("connection-0"),
                        client_id: parse!("cometbls-0"),
                        counterparty_client_id: parse!("08-wasm-1"),
                        counterparty_connection_id: parse!("connection-14"),
                    }),
                },
            ),
        ));

        println!("---------------------------------------");
        println!("Eth - Union (Sending to Eth) Update Client: ");
        println!("---------------------------------------");
        print_json::<RelayMessage>(repeat(
            None,
            seq([
                event(relay_message::id::<Ethereum<Minimal>, Wasm<Union>, _>(
                    eth_chain_id,
                    relay_message::event::Command::UpdateClient {
                        client_id: parse!("cometbls-0"),
                        __marker: PhantomData,
                    },
                )),
                defer_relative(10),
            ]),
        ));

        println!("---------------------------------------");
        println!("Eth - Union (Sending to Union) Update Client: ");
        println!("---------------------------------------");
        print_json::<RelayMessage>(repeat(
            None,
            seq([
                event(relay_message::id::<Wasm<Union>, Ethereum<Minimal>, _>(
                    union_chain_id.clone(),
                    relay_message::event::Command::UpdateClient {
                        client_id: parse!("08-wasm-0"),
                        __marker: PhantomData,
                    },
                )),
                defer_relative(10),
            ]),
        ));

        println!("---------------------------------------");
        println!("Cosmos - Union (Sending to Cosmos) Update Client: ");
        println!("---------------------------------------");
        print_json::<RelayMessage>(repeat(
            None,
            seq([
                event(relay_message::id::<Wasm<Cosmos>, Union, _>(
                    simd_chain_id.clone(),
                    relay_message::event::Command::UpdateClient {
                        client_id: parse!("08-wasm-0"),
                        __marker: PhantomData,
                    },
                )),
                defer_relative(10),
            ]),
        ));

        println!("---------------------------------------");
        println!("Cosmos - Union (Sending to Union) Update Client: ");
        println!("---------------------------------------");
        print_json::<RelayMessage>(repeat(
            None,
            seq([
                event(relay_message::id::<Union, Wasm<Cosmos>, _>(
                    union_chain_id.clone(),
                    relay_message::event::Command::UpdateClient {
                        client_id: parse!("07-tendermint-0"),
                        __marker: PhantomData,
                    },
                )),
                defer_relative(10),
            ]),
        ));

        println!("---------------------------------------");
        println!("Scroll - Union (Sending to Union) Create Scroll lightclient on Union: ");
        println!("---------------------------------------");
        print_json::<RelayMessage>(aggregate(
            [
                fetch(relay_message::id::<Scroll, Wasm<Union>, _>(
                    scroll_chain_id,
                    FetchSelfClientState {
                        at: QueryHeight::Latest,
                        __marker: PhantomData,
                    },
                )),
                fetch(relay_message::id::<Scroll, Wasm<Union>, _>(
                    scroll_chain_id,
                    FetchSelfConsensusState {
                        at: QueryHeight::Latest,
                        __marker: PhantomData,
                    },
                )),
            ],
            [],
            relay_message::id::<Wasm<Union>, Scroll, _>(
                union_chain_id.clone(),
                AggregateMsgCreateClient {
                    config: WasmConfig {
                        checksum: H256(hex!(
                            "c4c38c95b12a03dabe366dab1a19671193b5f8de7abf53eb3ecabbb946a4ac88"
                        )),
                    },
                    __marker: PhantomData,
                },
            ),
        ));

        println!("---------------------------------------");
        println!("Scroll - single update client");
        println!("---------------------------------------");
        print_json::<RelayMessage>(event(relay_message::id::<Scroll, Wasm<Union>, _>(
            scroll_chain_id,
            relay_message::event::Command::UpdateClient {
                client_id: parse!("cometbls-0"),
                __marker: PhantomData,
            },
        )));

        println!("---------------------------------------");
        println!("Union - Eth Create Both Clients: ");
        println!("---------------------------------------");
        print_json::<RelayMessage>(seq([
            aggregate(
                [
                    fetch(relay_message::id::<Wasm<Union>, Ethereum<Minimal>, _>(
                        union_chain_id.clone(),
                        FetchSelfClientState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                    fetch(relay_message::id::<Wasm<Union>, Ethereum<Minimal>, _>(
                        union_chain_id.clone(),
                        FetchSelfConsensusState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                ],
                [],
                relay_message::id::<Ethereum<Minimal>, Wasm<Union>, _>(
                    eth_chain_id,
                    AggregateMsgCreateClient {
                        config: EthereumConfig {
                            client_type: "cometbls".to_string(),
                            client_address: H160(hex!("83428c7db9815f482a39a1715684dcf755021997")),
                        },
                        __marker: PhantomData,
                    },
                ),
            ),
            aggregate(
                [
                    fetch(relay_message::id::<Ethereum<Minimal>, Wasm<Union>, _>(
                        eth_chain_id,
                        FetchSelfClientState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                    fetch(relay_message::id::<Ethereum<Minimal>, Wasm<Union>, _>(
                        eth_chain_id,
                        FetchSelfConsensusState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                ],
                [],
                relay_message::id::<Wasm<Union>, Ethereum<Minimal>, _>(
                    union_chain_id.clone(),
                    AggregateMsgCreateClient {
                        config: WasmConfig {
                            checksum: H256(hex!(
                                "78266014ea77f3b785e45a33d1f8d3709444a076b3b38b2aeef265b39ad1e494"
                            )),
                        },
                        __marker: PhantomData,
                    },
                ),
            ),
        ]));

        println!("---------------------------------------");
        println!("Union - Cosmos Create Both Client: ");
        println!("---------------------------------------");
        print_json::<RelayMessage>(seq([
            aggregate(
                [
                    fetch(relay_message::id::<Wasm<Cosmos>, Union, _>(
                        simd_chain_id.clone(),
                        FetchSelfClientState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                    fetch(relay_message::id::<Wasm<Cosmos>, Union, _>(
                        simd_chain_id.clone(),
                        FetchSelfConsensusState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                ],
                [],
                relay_message::id::<Union, Wasm<Cosmos>, _>(
                    union_chain_id.clone(),
                    AggregateMsgCreateClient {
                        config: (),
                        __marker: PhantomData,
                    },
                ),
            ),
            aggregate(
                [
                    fetch(relay_message::id::<Union, Wasm<Cosmos>, _>(
                        union_chain_id.clone(),
                        FetchSelfClientState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                    fetch(relay_message::id::<Union, Wasm<Cosmos>, _>(
                        union_chain_id.clone(),
                        FetchSelfConsensusState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                ],
                [],
                relay_message::id::<Wasm<Cosmos>, Union, _>(
                    simd_chain_id,
                    AggregateMsgCreateClient {
                        config: WasmConfig {
                            checksum: H256(hex!(
                                "78266014ea77f3b785e45a33d1f8d3709444a076b3b38b2aeef265b39ad1e494"
                            )),
                        },
                        __marker: PhantomData,
                    },
                ),
            ),
        ]));

        println!("---------------------------------------");
        println!("Cosmos - Cosmos Create Both Client: ");
        println!("---------------------------------------");
        print_json::<RelayMessage>(seq([
            aggregate(
                [
                    fetch(relay_message::id::<Cosmos, Cosmos, _>(
                        stargaze_chain_id.clone(),
                        FetchSelfClientState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                    fetch(relay_message::id::<Cosmos, Cosmos, _>(
                        stargaze_chain_id.clone(),
                        FetchSelfConsensusState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                ],
                [],
                relay_message::id::<Cosmos, Cosmos, _>(
                    osmosis_chain_id.clone(),
                    AggregateMsgCreateClient {
                        config: (),
                        __marker: PhantomData,
                    },
                ),
            ),
            aggregate(
                [
                    fetch(relay_message::id::<Cosmos, Cosmos, _>(
                        osmosis_chain_id.clone(),
                        FetchSelfClientState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                    fetch(relay_message::id::<Cosmos, Cosmos, _>(
                        osmosis_chain_id.clone(),
                        FetchSelfConsensusState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                ],
                [],
                relay_message::id::<Cosmos, Cosmos, _>(
                    stargaze_chain_id.clone(),
                    AggregateMsgCreateClient {
                        config: (),
                        __marker: PhantomData,
                    },
                ),
            ),
        ]));

        println!("---------------------------------------");
        println!("Scroll - single update client");
        println!("---------------------------------------");
        print_json::<RelayMessage>(event(relay_message::id::<Scroll, Wasm<Union>, _>(
            scroll_chain_id,
            relay_message::event::Command::UpdateClient {
                client_id: parse!("cometbls-0"),
                __marker: PhantomData,
            },
        )));

        println!("---------------------------------------");
        println!("Scroll - fetch update header");
        println!("---------------------------------------");
        print_json::<RelayMessage>(fetch(relay_message::id::<Scroll, Wasm<Union>, _>(
            scroll_chain_id,
            relay_message::fetch::Fetch::UpdateHeaders(relay_message::fetch::FetchUpdateHeaders {
                counterparty_chain_id: union_chain_id.clone(),
                counterparty_client_id: parse!("08-wasm-0"),
                update_from: parse!("0-1"),
                update_to: parse!("0-4846816"),
            }),
        )));

        print_json::<BlockMessage>(fetch(block_message::id::<Cosmos, _>(
            "simd-devnet-1".parse().unwrap(),
            block_message::fetch::FetchBlock {
                height: unionlabs::ibc::core::client::height::Height {
                    revision_number: 1,
                    revision_height: 1,
                },
            },
        )));

        print_json::<BlockMessage>(fetch(block_message::id::<Union, _>(
            "union-devnet-1".parse().unwrap(),
            block_message::fetch::FetchBlock {
                height: unionlabs::ibc::core::client::height::Height {
                    revision_number: 1,
                    revision_height: 1,
                },
            },
        )));
    }

    fn print_json<T: QueueMessage>(msg: Op<T>)
    where
        VoyagerMessage: FromOp<T>,
    {
        let msg = VoyagerMessage::from_op(msg);

        let json = serde_json::to_string(&msg).unwrap();

        println!("{json}\n");

        let from_json = serde_json::from_str(&json).unwrap();

        assert_eq!(&msg, &from_json, "json roundtrip failed");
    }
}
