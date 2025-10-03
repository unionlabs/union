use std::collections::VecDeque;

use ibc_union_spec::datagram::{MsgPacketAcknowledgement, MsgPacketRecv};
use jsonrpsee::{
    core::{async_trait, JsonValue as Value, RpcResult},
    types::ErrorObject,
    Extensions, MethodsError,
};
use serde::{Deserialize, Serialize};
use sui_sdk::{
    rpc_types::SuiObjectDataOptions,
    types::{
        base_types::{ObjectID, SequenceNumber, SuiAddress},
        crypto::SuiKeyPair,
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        transaction::ProgrammableTransaction,
    },
    SuiClientBuilder,
};
use ucs03_zkgm::com::{
    OP_BATCH, OP_TOKEN_ORDER, TOKEN_ORDER_KIND_ESCROW, TOKEN_ORDER_KIND_INITIALIZE,
    TOKEN_ORDER_KIND_SOLVE, TOKEN_ORDER_KIND_UNESCROW,
};
use unionlabs::{never::Never, ErrorReporter};
use voyager_sdk::{
    anyhow,
    hook::NEVER_FILTER,
    message::{data::Data, VoyagerMessage},
    plugin::Plugin,
    primitives::ChainId,
    rpc::{types::PluginInfo, PluginServer, FATAL_JSONRPC_ERROR_CODE},
    vm::{pass::PassResult, Op},
    DefaultCmd,
};
use voyager_transaction_plugin_sui::{ModuleInfo, TransactionPluginServer};
use zkgm::register_tokens_if_zkgm;

mod coin;
mod zkgm;

#[derive(Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub ibc_store: SuiAddress,

    pub sui_client: sui_sdk::SuiClient,

    pub ibc_store_initial_seq: SequenceNumber,

    pub zkgm_config: ZkgmConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZkgmConfig {
    vault_object_id: ObjectID,
    /// ID of the `wrapped_token_to_t` mapping
    wrapped_token_to_t: ObjectID,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,
    pub rpc_url: String,
    pub ibc_store: SuiAddress,
    pub zkgm_config: ZkgmConfig,
}

#[tokio::main]
async fn main() {
    Module::run().await
}

impl Plugin for Module {
    type Call = Never;

    type Callback = Never;

    type Config = Config;

    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> voyager_sdk::anyhow::Result<Self> {
        Module::new(config).await
    }

    fn info(_: Self::Config) -> voyager_sdk::rpc::types::PluginInfo {
        PluginInfo {
            name: Module::plugin_name(),
            interest_filter: NEVER_FILTER.into(),
        }
    }

    async fn cmd(_: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

#[async_trait]
impl TransactionPluginServer for Module {
    async fn on_recv_packet(
        &self,
        pk: SuiKeyPair,
        module_info: ModuleInfo,
        fee_recipient: SuiAddress,
        data: MsgPacketRecv,
    ) -> RpcResult<ProgrammableTransaction> {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let store_initial_seq = self.get_initial_seq(module_info.stores[0].into()).await;
        let vault_store_initial_seq = self.get_initial_seq(self.zkgm_config.vault_object_id).await;

        // If the module is ZKGM, then we register the tokens if needed. Otherwise,
        // the registered tokens are returned.
        let mut coin_ts = vec![];
        for p in &data.packets {
            coin_ts.extend_from_slice(
                &register_tokens_if_zkgm(self, &mut ptb, &pk, p, &module_info, store_initial_seq)
                    .await
                    .map_err(|e| {
                        ErrorObject::owned(FATAL_JSONRPC_ERROR_CODE, e.to_string(), None::<()>)
                    })?,
            );
        }

        // We start the session by calling `begin_recv`. The returned `session` has no drop nor store,
        // which means, we have to consume it within the same PTB via `end_recv`.
        let mut session = zkgm::begin_recv_call(&mut ptb, &module_info, data.clone());

        // SUI code partitions the instructions by the instructions that need coin. And the `recv_packet`
        // endpoint must be called as many times as the partitions. Since the number of coins will be the
        // same as the number of partitions, we are calling `recv_packet` based on the number of coins.
        for coin_t in coin_ts {
            session = zkgm::recv_packet_call(
                &mut ptb,
                self,
                &module_info,
                store_initial_seq,
                self.zkgm_config.vault_object_id,
                vault_store_initial_seq,
                coin_t,
                fee_recipient,
                data.relayer_msgs.clone(),
                session,
            );
        }

        // `end_recv` is done to consume the `session`, and do the recv commitment. Very important thing
        // to note here is that, the fact that `session` have to be consumed makes it s.t. if we don't consume
        // it, this PTB will fail and no partial state will be persisted.
        zkgm::end_recv_call(&mut ptb, self, &module_info, fee_recipient, session, data);

        Ok(ptb.finish())
    }

    async fn on_acknowledge_packet(
        &self,
        _: SuiKeyPair,
        module_info: ModuleInfo,
        fee_recipient: SuiAddress,
        data: MsgPacketAcknowledgement,
    ) -> RpcResult<ProgrammableTransaction> {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let store_initial_seq = self.get_initial_seq(module_info.stores[0].into()).await;
        let vault_store_initial_seq = self.get_initial_seq(self.zkgm_config.vault_object_id).await;

        // If the module is ZKGM, then we register the tokens if needed. Otherwise,
        // the registered tokens are returned.
        let coin_ts =
            zkgm::parse_coin_ts(data.packets.iter().map(|p| p.data.clone()).collect()).unwrap();

        // We start the session by calling `begin_recv`. The returned `session` has no drop nor store,
        // which means, we have to consume it within the same PTB via `end_recv`.
        let mut session = zkgm::begin_ack_call(&mut ptb, &module_info, data.clone());

        for coin_t in coin_ts {
            session = zkgm::acknowledge_packet_call(
                &mut ptb,
                self,
                &module_info,
                store_initial_seq,
                self.zkgm_config.vault_object_id,
                vault_store_initial_seq,
                coin_t,
                fee_recipient,
                session,
            );
        }

        zkgm::end_ack_call(&mut ptb, self, &module_info, fee_recipient, session, data);

        Ok(ptb.finish())
    }
}

impl Module {
    fn plugin_name() -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        PLUGIN_NAME.to_owned()
    }

    async fn new(config: Config) -> anyhow::Result<Self> {
        let sui_client = SuiClientBuilder::default().build(&config.rpc_url).await?;

        let chain_id = sui_client.read_api().get_chain_identifier().await?;

        let ibc_store_initial_seq = sui_client
            .read_api()
            .get_object_with_options(
                ObjectID::new(config.ibc_store.to_inner()),
                SuiObjectDataOptions::default().with_owner(),
            )
            .await
            .map_err(|e| ErrorObject::owned(-1, ErrorReporter(e).to_string(), None::<()>))?
            .data
            .expect("ibc store object exists on chain")
            .owner
            .expect("owner will be present")
            .start_version()
            .expect("ibc store is shared, hence it has a start version");

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            sui_client,
            ibc_store_initial_seq,
            ibc_store: config.ibc_store,
            zkgm_config: config.zkgm_config,
        })
    }

    async fn get_initial_seq(&self, object: ObjectID) -> SequenceNumber {
        self.sui_client
            .read_api()
            .get_object_with_options(object, SuiObjectDataOptions::new().with_owner())
            .await
            .unwrap()
            .data
            .expect("object exists on chain")
            .owner
            .expect("owner will be present")
            .start_version()
            .expect("object is shared, hence it has a start version")
    }
}

#[async_trait]
impl PluginServer<Never, Never> for Module {
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        Ok(PassResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .enumerate()
                .map(|(idx, op)| (vec![idx], op))
                .collect(),
        })
    }

    async fn call(&self, _: &Extensions, msg: Never) -> RpcResult<Op<VoyagerMessage>> {
        match msg {}
    }

    async fn callback(
        &self,
        _: &Extensions,
        cb: Never,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }

    async fn custom(&self, _: &Extensions, method: String, params: Vec<Value>) -> RpcResult<Value> {
        TransactionPluginServer::into_rpc(self.clone())
            .call::<Vec<Value>, Value>(&method, params)
            .await
            .map_err(|e| match e {
                MethodsError::Parse(error) => ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    ErrorReporter(error).with_message("error parsing args"),
                    None::<()>,
                ),
                MethodsError::JsonRpc(error_object) => error_object,
                MethodsError::InvalidSubscriptionId(_) => ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    "subscriptions are not supported",
                    None::<()>,
                ),
            })
    }
}
