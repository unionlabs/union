use std::collections::VecDeque;

use alloy::sol_types::SolValue;
use ibc_union_spec::{event::FullEvent, IbcUnion};
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
    DefaultCmd, Plugin, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::{noop, pass::PassResult, BoxDynError, Op};

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
            // TODO: Support IBC classic
            interest_filter: format!(
                r#"
if ."@type" == "data"
    and ."@value"."@type" == "ibc_event"
    and ."@value"."@value".ibc_spec_id == "{ibc_union_id}"
    and ."@value"."@value".event."@type" == "write_ack"
then
    false # interest, but only copy
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
                Op::Data(Data::IbcEvent(ref chain_event)) => match chain_event
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
                    })? {
                    FullEvent::WriteAck(event) => {
                        let noop_ = Ok((vec![idx], noop()));
                        let Ok(ack) = Ack::abi_decode_params(&event.acknowledgement, true) else {
                            return noop_;
                        };

                        if ack.tag != TAG_ACK_SUCCESS {
                            return noop_;
                        }

                        let Ok(ack) =
                            FungibleAssetOrderAck::abi_decode_params(&ack.inner_ack, true)
                        else {
                            return noop_;
                        };

                        if ack.fill_type == FILL_TYPE_PROTOCOL {
                            Ok((vec![], noop()))
                        } else {
                            noop_
                        }
                    }
                    datagram => Err(ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unexpected ibc datagram {}", datagram.name()),
                        Some(json!({
                            "msg": msg,
                        })),
                    )),
                },
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
