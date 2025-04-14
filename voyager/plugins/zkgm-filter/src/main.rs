use std::{
    collections::{BTreeMap, VecDeque},
    time::{SystemTime, UNIX_EPOCH},
};

use alloy::{
    consensus::Transaction,
    providers::{DynProvider, Provider, ProviderBuilder},
    sol_types::{SolCall, SolValue},
};
use crc::{Crc, CRC_32_ISO_HDLC};
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
use tracing::{info, instrument, warn};
use ucs03_zkgm::com::{Ack, FungibleAssetOrderAck, FILL_TYPE_PROTOCOL, TAG_ACK_SUCCESS};
use unionlabs::{
    self,
    cosmos::tx::{tx_body::TxBody, tx_raw::TxRaw},
    cosmwasm::wasm::msg_execute_contract::MsgExecuteContract,
    encoding::{DecodeAs, Proto},
    google::protobuf::any::Any,
    never::Never,
    primitives::{ByteArrayExt, H32},
    traits::Member,
    ErrorReporter,
};
use voyager_message::{
    data::Data,
    module::{PluginInfo, PluginServer},
    primitives::{ChainId, IbcSpec},
    DefaultCmd, Plugin, PluginMessage, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_plugin_transaction_batch::data::{BatchableEvent, EventBatch};
use voyager_vm::{call, data, noop, pass::PassResult, BoxDynError, Op};

use crate::{
    call::{CheckSendPacket, ModuleCall},
    IZkgm::sendCall,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

pub mod call {
    use enumorph::Enumorph;
    use ibc_union_spec::event::PacketSend;
    use macros::model;
    use unionlabs::{ibc::core::client::height::Height, primitives::H256};
    use voyager_message::primitives::ChainId;

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
    /// chain id -> provider
    providers: BTreeMap<ChainId, ChainProvider>,
}

pub enum ChainProvider {
    Cosmos { client: cometbft_rpc::Client },
    Evm { provider: DynProvider },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct Config {
    /// chain id -> rpc url
    providers: BTreeMap<ChainId, ChainProviderConfig>,
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
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let mut providers = BTreeMap::new();

        for (chain_id, provider_config) in config.providers {
            info!("registering chain {chain_id}");

            match provider_config {
                ChainProviderConfig::Cosmos { rpc_url } => {
                    let client = cometbft_rpc::Client::new(rpc_url.clone()).await?;

                    let expected_chain_id = client.status().await?.node_info.network;

                    if chain_id.as_str() != expected_chain_id {
                        return Err(format!(
                            "expected chain id {chain_id} for rpc endpoint \
                            {rpc_url} but found {expected_chain_id}"
                        )
                        .into());
                    }

                    providers.insert(chain_id, ChainProvider::Cosmos { client });
                }
                ChainProviderConfig::Evm { rpc_url } => {
                    let provider =
                        DynProvider::new(ProviderBuilder::new().connect(&rpc_url).await?);

                    let raw_chain_id = provider.get_chain_id().await?;

                    if chain_id.as_str() != raw_chain_id.to_string() {
                        return Err(format!(
                            "expected chain id {chain_id} for rpc endpoint \
                            {rpc_url} but found {raw_chain_id}"
                        )
                        .into());
                    }

                    providers.insert(chain_id, ChainProvider::Evm { provider });
                }
            }
        }

        Ok(Self { providers })
    }

    fn info(Config { .. }: Self::Config) -> PluginInfo {
        PluginInfo {
            name: PLUGIN_NAME.to_owned(),
            interest_filter: format!(
                r#"
if ."@type" == "data"
    and ."@value"."@type" == "ibc_event"
    and ."@value"."@value".ibc_spec_id == "{ibc_union_id}"
    and (
        ."@value"."@value".event."@type" == "write_ack"
        or ."@value"."@value".event."@type" == "packet_send"
    )
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

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

impl Module {
    fn plugin_name(&self) -> String {
        PLUGIN_NAME.to_string()
    }
}

#[async_trait]
impl PluginServer<ModuleCall, Never> for Module {
    #[instrument(skip_all, fields())]
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
                            if packet_send.packet.source_channel.version
                                == ucs03_zkgm::contract::PROTOCOL_VERSION
                                && self.providers.contains_key(&chain_event.chain_id)
                            {
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
                                            provable_height: chain_event.provable_height,
                                        }),
                                    )),
                                ))
                            } else {
                                ready()
                            }
                        }
                        FullEvent::WriteAck(write_ack) => {
                            if !self.filter_ack(write_ack) {
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

    #[instrument(skip_all, fields())]
    async fn call(&self, _: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::CheckSendPacket(p) => self.check_send_packet(p).await,
        }
    }

    #[instrument(skip_all, fields())]
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
    /// Drop the message if it is `WriteAck`, ACK is success and the fill type is PROTOCOL
    pub fn filter_ack(&self, event: &WriteAck) -> bool {
        let Ok(ack) = Ack::abi_decode_params(&event.acknowledgement, true) else {
            return false;
        };

        if ack.tag != TAG_ACK_SUCCESS {
            return false;
        }

        let Ok(ack) = FungibleAssetOrderAck::abi_decode_params(&ack.inner_ack, true) else {
            return false;
        };

        ack.fill_type == FILL_TYPE_PROTOCOL
    }

    #[instrument(
        skip_all,
        fields(
            %chain_id,
            %counterparty_chain_id,
            %tx_hash,
            %provable_height,
            packet_hash = %event.packet().hash()
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
        let provider = self.providers.get(&chain_id).ok_or_else(|| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!("unknown chain {chain_id}"),
                None::<()>,
            )
        })?;

        let valid = match provider {
            ChainProvider::Cosmos { client } => {
                let tx = client.tx(tx_hash, false).await.map_err(|err| {
                    ErrorObject::owned(
                        -1,
                        ErrorReporter(err).with_message("error fetching source tx for packet"),
                        Some(json!({
                            "packet_hash": event.packet().hash(),
                            "tx_hash": tx_hash,
                        })),
                    )
                })?;

                valid_checksum_cosmos(&tx.tx)
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
                                "packet_hash": event.packet().hash(),
                                "tx_hash": tx_hash,
                            })),
                        )
                    })?
                    .expect("tx exists");

                valid_checksum_eth(tx.input())
            }
        };

        if valid {
            info!("valid checksum");

            let first_seen_at: u64 = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .try_into()
                .expect("how many milliseconds can there be man");

            let client_id = event.packet.destination_channel.connection.client_id;

            let batchable_event = BatchableEvent::<IbcUnion> {
                first_seen_at,
                provable_height,
                event: event.into(),
            };

            Ok(data(PluginMessage::new(
                voyager_plugin_transaction_batch::plugin_name(&counterparty_chain_id),
                voyager_plugin_transaction_batch::data::ModuleData::BatchEventsUnion(EventBatch {
                    client_id,
                    events: vec![batchable_event],
                }),
            )))
        } else {
            warn!("invalid checksum");

            Ok(noop())
        }
    }
}

fn valid_checksum_cosmos(tx_bytes: &[u8]) -> bool {
    let tx_raw = TxRaw::decode_as::<Proto>(tx_bytes).expect("invalid transaction?");
    let mut tx_body = <TxBody<Any<MsgExecuteContract>>>::decode_as::<Proto>(&tx_raw.body_bytes)
        .expect("invalid auth info?");

    let msg = tx_body
        .messages
        .pop()
        .expect("must contain at least one message");

    let execute_msg = serde_json::from_slice::<ucs03_zkgm::msg::ExecuteMsg>(&msg.0.msg)
        .expect("invalid execute wasm contract message?");

    match execute_msg {
        ucs03_zkgm::msg::ExecuteMsg::Send { salt, .. } => valid_checksum(salt),
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
    let send_call = sendCall::abi_decode(tx_input, true).expect("invalid transaction?");

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

        let ok = valid_checksum_cosmos(&tx_bytes);

        assert!(ok);
    }
}
