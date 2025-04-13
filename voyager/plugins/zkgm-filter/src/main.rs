use std::{
    collections::VecDeque,
    time::{SystemTime, UNIX_EPOCH},
};

use alloy::sol_types::SolValue;
use ibc_union_spec::{
    event::{FullEvent, WriteAck},
    ClientId, IbcUnion,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::instrument;
use ucs03_zkgm::com::{Ack, FungibleAssetOrderAck, FILL_TYPE_PROTOCOL, TAG_ACK_SUCCESS};
use unionlabs::{self, never::Never, traits::Member, ErrorReporter};
use voyager_message::{
    data::Data,
    module::{PluginInfo, PluginServer},
    primitives::IbcSpec,
    DefaultCmd, Plugin, PluginMessage, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_plugin_transaction_batch::data::{BatchableEvent, EventBatch, ModuleData};
use voyager_vm::{data, noop, pass::PassResult, BoxDynError, Op};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

pub struct Module {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {}

impl Plugin for Module {
    type Call = Never;
    type Callback = Never;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        Ok(Module::new(config))
    }

    fn info(config: Self::Config) -> PluginInfo {
        let module = Module::new(config);

        PluginInfo {
            name: module.plugin_name(),
            interest_filter: format!(
                r#"
if ."@type" == "data"
    and ."@value"."@type" == "ibc_event"
    and ."@value"."@value".ibc_spec_id == "{ibc_union_id}"
    and ."@value"."@value".event."@type" == "write_ack"
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

    pub fn new(Config {}: Config) -> Self {
        Self {}
    }
}

#[async_trait]
impl PluginServer<Never, Never> for Module {
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

                    let ready = |client_id: ClientId| {
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
                                voyager_plugin_transaction_batch::PLUGIN_NAME,
                                ModuleData::BatchEventsUnion(EventBatch {
                                    client_id,
                                    events: vec![batchable_event],
                                }),
                            )),
                        ))
                    };

                    match &full_event {
                        FullEvent::WriteAck(event) => {
                            if !self.filter_ack(event) {
                                return Ok((vec![], noop()));
                            }

                            ready(full_event.counterparty_client_id().unwrap())
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
    async fn call(&self, _: &Extensions, msg: Never) -> RpcResult<Op<VoyagerMessage>> {
        match msg {}
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
}
