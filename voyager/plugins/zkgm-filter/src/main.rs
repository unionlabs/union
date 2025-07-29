#![feature(let_chains)]

use std::{
    collections::{HashSet, VecDeque},
    time::{SystemTime, UNIX_EPOCH},
};

use alloy::{
    consensus::Transaction,
    hex,
    providers::{DynProvider, Provider, ProviderBuilder},
    sol_types::{SolCall, SolValue},
};
use crc::{Crc, CRC_32_ISO_HDLC};
use futures::TryStreamExt;
use ibc_union_spec::{
    event::{FullEvent, WriteAck},
    IbcUnion,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    prelude::FromRow,
    Executor, PgPool, Row,
};
use tracing::{debug, info, info_span, instrument, trace, warn, Instrument};
use ucs03_zkgm::com::{Ack, BatchAck, TokenOrderAck, FILL_TYPE_PROTOCOL, TAG_ACK_SUCCESS};
use unionlabs::{
    self,
    cosmos::tx::{tx_body::TxBody, tx_raw::TxRaw},
    cosmwasm::wasm::msg_execute_contract::MsgExecuteContract,
    encoding::{DecodeAs, Proto},
    google::protobuf::any::RawAny,
    never::Never,
    primitives::{Bech32, ByteArrayExt, Bytes, H160, H32},
    traits::Member,
    ErrorReporter,
};
use voyager_plugin_transaction_batch::data::{BatchableEvent, EventBatch};
use voyager_sdk::{
    anyhow::{self, bail},
    message::{
        data::{Data, EventProvableHeight},
        PluginMessage, VoyagerMessage,
    },
    plugin::Plugin,
    primitives::{ChainId, IbcSpec},
    rpc::{types::PluginInfo, PluginServer, FATAL_JSONRPC_ERROR_CODE},
    vm::{call, data, noop, pass::PassResult, Op},
};

use crate::{
    call::{CheckSendPacket, ModuleCall},
    IZkgm::sendCall,
};

#[tokio::main]
async fn main() {
    Module::run().await
}

pub mod call {
    use enumorph::Enumorph;
    use ibc_union_spec::event::PacketSend;
    use macros::model;
    use unionlabs::{ibc::core::client::height::Height, primitives::H256};
    use voyager_sdk::primitives::ChainId;

    #[model]
    #[derive(Enumorph)]
    pub enum ModuleCall {
        CheckSendPacket(CheckSendPacket),
    }

    #[model]
    pub struct CheckSendPacket {
        pub event: PacketSend,
        pub chain_id: ChainId,
        pub counterparty_chain_id: ChainId,
        pub tx_hash: H256,
        pub provable_height: Height,
    }
}

pub struct Module {
    drop_protocol_fill_acks: bool,
    drop_invalid_checksum: bool,
    drop_suspicious: bool,
    chain_id: ChainId,
    provider: ChainProvider,
    db: PgPool,
    whitelisted_addresses: HashSet<Bytes>,
    // i64 for ease of use with the sqlx types
    max_invalid_per_address: i64,
}

pub enum ChainProvider {
    Cosmos { client: cometbft_rpc::Client },
    Evm { provider: DynProvider },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct Config {
    drop_protocol_fill_acks: bool,
    drop_invalid_checksum: bool,
    drop_suspicious: bool,
    chain_id: ChainId,
    provider: ChainProviderConfig,
    db_url: String,
    #[serde(default)]
    whitelisted_addresses_evm: HashSet<H160>,
    // NOTE: Prefix is ignored, normalized address is used
    #[serde(default)]
    whitelisted_addresses_cosmos: HashSet<Bech32<Bytes>>,
    max_invalid_per_address: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    deny_unknown_fields,
    rename_all = "snake_case",
    tag = "type",
    content = "config"
)]
pub enum ChainProviderConfig {
    Cosmos { rpc_url: String },
    Evm { rpc_url: String },
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = Cmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        let db = PgPoolOptions::new().connect(&config.db_url).await?;

        db.execute_many(
            r#"
            CREATE TABLE IF NOT EXISTS
              invalid_checksum (
                id BIGSERIAL PRIMARY KEY,
                -- 0x + 32 byte hash
                packet_hash CHAR(66) NOT NULL,
                chain_id TEXT NOT NULL,
                address TEXT NOT NULL
              );

            CREATE UNIQUE INDEX IF NOT EXISTS packet_hash_chain_id_address_unique ON invalid_checksum(packet_hash, chain_id, address);
            "#,
        )
        .try_for_each(|result| async move {
            trace!("rows affected: {}", result.rows_affected());
            Ok(())
        })
        .instrument(info_span!("db_init"))
        .await?;

        info!("registering chain {}", config.chain_id);

        let provider = match config.provider {
            ChainProviderConfig::Cosmos { rpc_url } => {
                let client = cometbft_rpc::Client::new(rpc_url.clone()).await?;

                let expected_chain_id = client.status().await?.node_info.network;

                if config.chain_id.as_str() != expected_chain_id {
                    bail!(
                        "expected chain id {} for rpc endpoint \
                        {rpc_url} but found {expected_chain_id}",
                        config.chain_id
                    );
                }

                ChainProvider::Cosmos { client }
            }
            ChainProviderConfig::Evm { rpc_url } => {
                let provider = DynProvider::new(ProviderBuilder::new().connect(&rpc_url).await?);

                let raw_chain_id = provider.get_chain_id().await?;

                if config.chain_id.as_str() != raw_chain_id.to_string() {
                    bail!(
                        "expected chain id {} for rpc endpoint \
                        {rpc_url} but found {raw_chain_id}",
                        config.chain_id
                    );
                }

                ChainProvider::Evm { provider }
            }
        };

        Ok(Self {
            chain_id: config.chain_id,
            provider,
            drop_protocol_fill_acks: config.drop_protocol_fill_acks,
            drop_invalid_checksum: config.drop_invalid_checksum,
            drop_suspicious: config.drop_suspicious,
            db,
            whitelisted_addresses: config
                .whitelisted_addresses_evm
                .into_iter()
                .map(Into::into)
                .chain(
                    config
                        .whitelisted_addresses_cosmos
                        .into_iter()
                        .map(|a| a.into_data()),
                )
                .collect(),
            max_invalid_per_address: config.max_invalid_per_address.try_into().unwrap(),
        })
    }

    fn info(Config { chain_id, .. }: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&chain_id),
            interest_filter: format!(
                r#"
if ."@type" == "data"
    and ."@value"."@type" == "ibc_event"
    and ."@value"."@value".ibc_spec_id == "{ibc_union_id}"
    and ."@value"."@value".chain_id == "{chain_id}"
    and (
        ."@value"."@value".event."@type" == "write_ack"
        or ."@value"."@value".event."@type" == "packet_send"
    )
    and ."@value"."@value".event."@value".packet.source_channel.version == "ucs03-zkgm-0"
then
    true
else
    null
end
"#,
                ibc_union_id = IbcUnion::ID,
            ),
        }
    }

    async fn cmd(config: Self::Config, cmd: Self::Cmd) {
        match cmd {
            Cmd::InvalidBySender {
                sender,
                count,
                limit,
            } => {
                let db = PgPoolOptions::new().connect(&config.db_url).await.unwrap();

                let sender = sender
                    .parse::<Bytes>()
                    .unwrap_or_else(|_| sender.parse::<Bech32<Bytes>>().unwrap().into_data());

                if count {
                    let res = sqlx::query(
                        r#"
                        SELECT
                            count(*)
                        FROM
                            invalid_checksum
                        WHERE
                            address = $1
                        "#,
                    )
                    .bind(sender.to_string())
                    .map::<_, i64>(|r| r.get("count"))
                    .fetch_one(&db)
                    .await
                    .unwrap();

                    println!(
                        "{}",
                        serde_json::to_string(&res).expect("serialization is infallible; qed;")
                    );
                } else {
                    let res = sqlx::query(
                        r#"
                        SELECT
                            packet_hash, chain_id
                        FROM
                            invalid_checksum
                        WHERE
                            address = $1
                        LIMIT
                            $2
                        "#,
                    )
                    .bind(sender.to_string())
                    .bind(limit)
                    .map::<_, (String, String)>(|r| (r.get("packet_hash"), r.get("chain_id")))
                    .fetch_all(&db)
                    .await
                    .unwrap();

                    println!(
                        "{}",
                        serde_json::to_string(&res).expect("serialization is infallible; qed;")
                    );
                }
            }
            Cmd::InvalidSenders { limit, count } => {
                let db = PgPoolOptions::new().connect(&config.db_url).await.unwrap();

                if count {
                    let res = sqlx::query(
                        r#"
                        SELECT
                            count(distinct address)
                        FROM
                            invalid_checksum
                    "#,
                    )
                    .map::<_, i64>(|r: PgRow| r.get("count"))
                    .fetch_one(&db)
                    .await
                    .unwrap();

                    println!(
                        "{}",
                        serde_json::to_string(&res).expect("serialization is infallible; qed;")
                    );
                } else {
                    let res = sqlx::query(
                        r#"
                        SELECT
                            count(*), address
                        FROM
                            invalid_checksum
                        GROUP BY
                            address
                        ORDER BY
                            -count(*)
                        LIMIT
                            $1
                    "#,
                    )
                    .bind(limit)
                    .map::<_, (String, i64)>(|r: PgRow| (r.get("address"), r.get("count")))
                    .fetch_all(&db)
                    .await
                    .unwrap();

                    println!(
                        "{}",
                        serde_json::to_string(&res).expect("serialization is infallible; qed;")
                    );
                }
            }
        }
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum Cmd {
    InvalidBySender {
        sender: String,
        #[arg(long, short = 'c')]
        count: bool,
        #[arg(long, short = 'l', conflicts_with("count"))]
        limit: Option<i64>,
    },
    InvalidSenders {
        #[arg(long, short = 'c')]
        count: bool,
        #[arg(long, short = 'l', conflicts_with("count"))]
        limit: Option<i64>,
    },
}

fn plugin_name(chain_id: &ChainId) -> String {
    const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");
    format!("{PLUGIN_NAME}/{chain_id}")
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }
}

#[async_trait]
impl PluginServer<ModuleCall, Never> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        let ready = msgs
            .into_iter()
            .enumerate()
            .map(|(idx, msg)| match msg {
                Op::Data(Data::IbcEvent(ref chain_event)) => {
                    let full_event = chain_event
                        .decode_event::<IbcUnion>()
                        .ok_or_else(|| {
                            ErrorObject::owned(
                                FATAL_JSONRPC_ERROR_CODE,
                                "unexpected data message in queue",
                                Some(json!({
                                    "msg": msg.clone(),
                                })),
                            )
                        })?
                        .map_err(|err| {
                            ErrorObject::owned(
                                FATAL_JSONRPC_ERROR_CODE,
                                "unable to parse ibc datagram",
                                Some(json!({
                                    "err": ErrorReporter(err).to_string(),
                                    "msg": msg,
                                })),
                            )
                        })?;

                    let ready = || {
                        let first_seen_at: u64 = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis()
                            .try_into()
                            .expect("how many milliseconds can there be man");
                        let batchable_event = BatchableEvent {
                            first_seen_at,
                            provable_height: chain_event.provable_height,
                            event: full_event.clone().try_into().unwrap(),
                        };
                        Ok((
                            vec![idx],
                            data(PluginMessage::new(
                                voyager_plugin_transaction_batch::plugin_name(&chain_event.counterparty_chain_id),
                                voyager_plugin_transaction_batch::data::ModuleData::BatchEventsUnion(EventBatch {
                                    client_id: full_event.counterparty_client_id().unwrap(),
                                    events: vec![batchable_event],
                                }),
                            )),
                        ))
                    };

                    match &full_event {
                        FullEvent::PacketSend(packet_send) => {
                            info!(
                                packet_hash = %packet_send.packet().hash(),
                                chain_id = %chain_event.chain_id,
                                "found zkgm packet"
                            );

                            Ok((
                                vec![idx],
                                call(PluginMessage::new(
                                    self.plugin_name(),
                                    ModuleCall::CheckSendPacket(CheckSendPacket {
                                        event: packet_send.clone(),
                                        chain_id: chain_event.chain_id.clone(),
                                        tx_hash: chain_event.tx_hash,
                                        counterparty_chain_id: chain_event
                                            .counterparty_chain_id
                                            .clone(),
                                        provable_height: *chain_event.provable_height.height(),
                                    }),
                                )),
                            ))
                        }
                        FullEvent::WriteAck(write_ack) => {
                            if self.drop_protocol_fill_acks && is_successful_protocol_fill(write_ack) {
                                info!(
                                    packet_hash = %write_ack.packet().hash(),
                                    "not acknowledging protocol filled packet"
                                );
                                return Ok((vec![], noop()));
                            }

                            ready()
                        }
                        datagram => Err(ErrorObject::owned(
                            FATAL_JSONRPC_ERROR_CODE,
                            format!("unexpected ibc datagram {}", datagram.name()),
                            Some(json!({
                                "msg": msg,
                            })),
                        )),
                    }
                }
                _ => Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    "unexpected message in queue",
                    Some(json!({
                        "msg": msg,
                    })),
                )),
            })
            .collect::<RpcResult<Vec<_>>>()?;

        Ok(PassResult {
            optimize_further: vec![],
            ready,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(&self, _: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::CheckSendPacket(p) => self.check_send_packet(p).await,
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        cb: Never,
        _datas: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}

impl Module {
    #[instrument(
        skip_all,
        fields(
            %chain_id,
            %counterparty_chain_id,
            %tx_hash,
            %provable_height,
            packet_hash = %event.packet().hash(),
            chain_id = %self.chain_id
        )
    )]
    async fn check_send_packet(
        &self,
        CheckSendPacket {
            event,
            chain_id,
            counterparty_chain_id,
            tx_hash,
            provable_height,
        }: CheckSendPacket,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let packet_hash = event.packet().hash();

        let (valid, address) = match &self.provider {
            ChainProvider::Cosmos { client } => {
                let tx = client.tx(tx_hash, false).await.map_err(|err| {
                    ErrorObject::owned(
                        -1,
                        ErrorReporter(err).with_message("error fetching source tx for packet"),
                        Some(json!({
                            "packet_hash": packet_hash,
                            "tx_hash": tx_hash,
                        })),
                    )
                })?;

                let (valid, address) = valid_checksum_cosmos(&tx.tx);

                (valid, address.map(Bech32::into_data))
            }
            ChainProvider::Evm { provider } => {
                let tx = provider
                    .get_transaction_by_hash(tx_hash.into())
                    .await
                    .map_err(|err| {
                        ErrorObject::owned(
                            -1,
                            ErrorReporter(err).with_message("error fetching source tx for packet"),
                            Some(json!({
                                "packet_hash": packet_hash,
                                "tx_hash": tx_hash,
                            })),
                        )
                    })?
                    .expect("tx exists");

                (
                    valid_checksum_eth(tx.input()),
                    Some(tx.inner.signer().into()),
                )
            }
        };

        let continuation = || {
            let first_seen_at: u64 = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .try_into()
                .expect("how many milliseconds can there be man");

            let client_id = event.packet.destination_channel.connection.client_id;

            let batchable_event = BatchableEvent::<IbcUnion> {
                first_seen_at,
                provable_height: EventProvableHeight::Min(provable_height),
                event: event.into(),
            };

            Ok(data(PluginMessage::new(
                voyager_plugin_transaction_batch::plugin_name(&counterparty_chain_id),
                voyager_plugin_transaction_batch::data::ModuleData::BatchEventsUnion(EventBatch {
                    client_id,
                    events: vec![batchable_event],
                }),
            )))
        };

        if valid {
            info!("valid checksum");
            continuation()
        } else if let Some(ref address) = address
            && self.whitelisted_addresses.contains(address)
        {
            info!(%address, "invalid checksum from whitelisted address");
            continuation()
        } else {
            warn!("invalid checksum");

            #[derive(Debug, FromRow)]
            struct Count {
                count: i64,
            }

            if let Some(address) = address {
                let res = sqlx::query(
                    r#"
                    WITH res AS (
                        INSERT INTO invalid_checksum(chain_id, address, packet_hash)
                        VALUES($1, $2, $3)
                        ON CONFLICT(packet_hash, chain_id, address) DO NOTHING
                    )
                    SELECT count(*) FROM invalid_checksum WHERE invalid_checksum.address = $2
                    "#,
                )
                .bind(chain_id.to_string())
                .bind(address.to_string())
                .bind(packet_hash.to_string())
                .try_map(|r| Count::from_row(&r))
                .fetch_one(&self.db)
                .await
                .map_err(|err| {
                    ErrorObject::owned(
                        -1,
                        ErrorReporter(err).with_message("error inserting into db"),
                        Some(json!({
                            "packet_hash": packet_hash,
                            "tx_hash": tx_hash,
                        })),
                    )
                })?;

                info!(total_invalid = %res.count, %address, "invalid checksum for address");

                if res.count > self.max_invalid_per_address {
                    info!(total_invalid = %res.count, %address, "suspicious address");

                    if self.drop_suspicious || self.drop_invalid_checksum {
                        Ok(noop())
                    } else {
                        continuation()
                    }
                } else {
                    continuation()
                }
            } else if self.drop_invalid_checksum {
                Ok(noop())
            } else {
                continuation()
            }
        }
    }
}

/// The `ucs03-zkgm` protocol-filled packets do nothing on acknowledgement, so there's no need to relay those messages.
#[instrument(
    skip_all,
    fields(
        packet_hash = %event.packet().hash()
    )
)]
pub fn is_successful_protocol_fill(event: &WriteAck) -> bool {
    is_successful_protocol_fill_ack(&event.acknowledgement)
}

fn is_successful_protocol_fill_ack(acknowledgement: &[u8]) -> bool {
    let Ok(ack) = Ack::abi_decode_params_validate(acknowledgement) else {
        // not a zkgm ack
        return false;
    };

    info!(%ack.tag, %ack.inner_ack, "zkgm ack");

    if ack.tag != TAG_ACK_SUCCESS {
        debug!("ack is not {TAG_ACK_SUCCESS}");
        return false;
    }

    match TokenOrderAck::abi_decode_params_validate(&ack.inner_ack) {
        Ok(ack) => ack.fill_type == FILL_TYPE_PROTOCOL,
        Err(_) => match BatchAck::abi_decode_params_validate(&ack.inner_ack) {
            Ok(BatchAck { acknowledgements }) => {
                is_successful_protocol_fill_batch_ack(acknowledgements)
            }
            _ => false,
        },
    }
}

fn is_successful_protocol_fill_batch_ack(acknowledgements: Vec<alloy::primitives::Bytes>) -> bool {
    for (idx, ack) in acknowledgements.iter().enumerate() {
        if let Ok(ack) = TokenOrderAck::abi_decode_params_validate(ack) {
            info!(%idx, %ack.fill_type, %ack.market_maker, "fungible asset order ack");

            if ack.fill_type == FILL_TYPE_PROTOCOL {
                continue;
            } else {
                return false;
            }
        } else if let Ok(batch_ack) = BatchAck::abi_decode_params_validate(ack) {
            info!("batch ack");

            is_successful_protocol_fill_batch_ack(batch_ack.acknowledgements);
        } else {
            return false;
        };
    }

    true
}

/// NOTE: Assumes the tx contains only one MsgExecuteContract.
fn valid_checksum_cosmos(tx_bytes: &[u8]) -> (bool, Option<Bech32<Bytes>>) {
    let tx_raw = TxRaw::decode_as::<Proto>(tx_bytes).expect("invalid transaction?");

    let tx_body = match <TxBody<RawAny>>::decode_as::<Proto>(&tx_raw.body_bytes) {
        Ok(ok) => ok,
        Err(err) => {
            warn!(
                err = %ErrorReporter(err),
                tx_input = %hex::encode(&tx_raw.body_bytes),
                "unable to decode transaction, crc will not be checked"
            );

            return (true, None);
        }
    };

    let msg = match tx_body
        .messages
        .first()
        .expect("must contain at least one message")
        .decode::<MsgExecuteContract>()
    {
        Ok(ok) => ok,
        Err(err) => {
            warn!(
                err = %ErrorReporter(err),
                messages = ?tx_body.messages,
                "unable to decode MsgExecuteContract from transaction, crc will not be checked"
            );

            return (true, None);
        }
    };

    let execute_msg = match serde_json::from_slice::<ucs03_zkgm::msg::ExecuteMsg>(&msg.msg) {
        Ok(ok) => ok,
        Err(err) => {
            warn!(
                err = %ErrorReporter(err),
                msg = %msg.msg,
                "unable to decode ExecuteMsg, crc will not be checked"
            );

            return (true, None);
        }
    };

    match execute_msg {
        ucs03_zkgm::msg::ExecuteMsg::Send { salt, .. } => (valid_checksum(salt), Some(msg.sender)),
        _ => panic!("????? {execute_msg:?}"),
    }
}

fn valid_checksum(salt: unionlabs::primitives::FixedBytes<32>) -> bool {
    const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

    let expected_checksum = CRC.checksum(&salt.get().array_slice::<0, 28>());

    let found_checksum = u32::from_be_bytes(salt.get().array_slice::<28, 4>());

    info!(
        found_checksum,
        expected_checksum,
        found_checksum_hex = %<H32>::new(found_checksum.to_be_bytes()),
        expected_checksum_hex = %<H32>::new(expected_checksum.to_be_bytes()),
        "crc checksum"
    );

    found_checksum == expected_checksum
}

fn valid_checksum_eth(tx_input: &[u8]) -> bool {
    let send_call = match sendCall::abi_decode_validate(tx_input) {
        Ok(ok) => ok,
        Err(err) => {
            warn!(
                err = %ErrorReporter(err),
                tx_input = %hex::encode(tx_input),
                "unable to decode calldata, crc will not be checked"
            );

            return true;
        }
    };

    valid_checksum(send_call.salt.into())
}

alloy::sol! {
    interface IZkgm {
        function send(
            uint32 channelId,
            uint64 timeoutHeight,
            uint64 timeoutTimestamp,
            bytes32 salt,
            Instruction calldata instruction
        ) external;
    }

    struct Instruction {
        uint8 version;
        uint8 opcode;
        bytes operand;
    }
}

#[cfg(test)]
mod tests {
    use unionlabs::primitives::{encoding::Base64, Bytes};

    use super::*;

    #[test]
    pub(crate) fn evm_checksum() {
        let tx_input = alloy::hex!("0xff0d7c2f000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001836f0fb1c9aa40037cef61c9badad19d7027619c79ff874b48642083f0156a074f5ad885b680e0b00000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002e00000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e00000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000022000000000000000000000000000000000000000000000000000000000000002600000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000002a0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000142c96e52fce14baa13868ca8182f8a7903e4e76e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a62626e31396c6e7063733070767a39687463766d35386a6b7036616b35356d343978356e3633726e666e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014e53dcec07d16d88e386ae0710e86d9a400f83c31000000000000000000000000000000000000000000000000000000000000000000000000000000000000000442414259000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000007426162796c6f6e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000047562626e00000000000000000000000000000000000000000000000000000000");

        let ok = valid_checksum_eth(&tx_input);

        assert!(ok);
    }

    #[test]
    pub(crate) fn cosmos_checksum() {
        let tx_bytes = "Cp4QCpsQCiQvY29zbXdhc20ud2FzbS52MS5Nc2dFeGVjdXRlQ29udHJhY3QS8g8KKmJibjE5bG5wY3MwcHZ6OWh0Y3ZtNThqa3A2YWs1NW00OXg1bjYzcm5mbhI+YmJuMTMzNmpqOGVydGw4aDdyZHZuejRkaDVycWFoZDA5Y3kweDQzZ3Voc3h4Nnh5cnp0eDI5MnE3Nzk0NWga9w57InNlbmQiOnsiY2hhbm5lbF9pZCI6MSwidGltZW91dF9oZWlnaHQiOiIwIiwidGltZW91dF90aW1lc3RhbXAiOiIxNzQ0ODQ1NTQyNDQ1MDAwMDAwIiwic2FsdCI6IjB4NGU0N2Y5NmVkNDBiZGY2YjIxMmE1YWYxZWRkNzA3MjkxNTg1ZmE4ZDdiMTk0MzY2OTM1MmNiODgxNmExOGQ5NSIsImluc3RydWN0aW9uIjoiMHgwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAxMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMzAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwNjAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMmUwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDE0MDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAxYTAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMWUwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwYTAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAyMjAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMjYwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMmEwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwYTAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMmE2MjYyNmUzMTM5NmM2ZTcwNjM3MzMwNzA3NjdhMzk2ODc0NjM3NjZkMzUzODZhNmI3MDM2NjE2YjM1MzU2ZDM0Mzk3ODM1NmUzNjMzNzI2ZTY2NmUwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMTQyYzk2ZTUyZmNlMTRiYWExMzg2OGNhODE4MmY4YTc5MDNlNGU3NmUwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwNDc1NjI2MjZlMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDA0NzU2MjYyNmUwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDQ3NTYyNjI2ZTAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAxNGU1M2RDZWMwN2QxNkQ4OGUzODZBRTA3MTBFODZkOWE0MDBmODNjMzEwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAifX0qCgoEdWJibhICMTASZgpQCkYKHy9jb3Ntb3MuY3J5cHRvLnNlY3AyNTZrMS5QdWJLZXkSIwohAu6hSq9M46334S1bIA4kRXOc0AQlDOemU4GnuAfsFmtgEgQKAggBGBASEgoMCgR1YmJuEgQzMzc2EKK3HRpA5/p8D7nvFFWmTHvG08hjVNRFy5M7O7KxUPovXKLxk0BPa7utQEYZp9dVl1DNwHskCQNfGkRr0AW//HtERChn+w==".parse::<Bytes<Base64>>().unwrap();

        let (ok, _address) = valid_checksum_cosmos(&tx_bytes);

        assert!(ok);
    }

    #[test]
    fn successful_protocol_fill() {
        let ack = hex!("0x0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000b0cad000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000");

        assert!(is_successful_protocol_fill_ack(&ack));

        let ack = hex!("0x00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000b0cad00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000b0cad000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000");

        assert!(is_successful_protocol_fill_ack(&ack));
    }
}
