use std::{fmt::Debug, num::NonZeroU64, time::Duration};

use gno_light_client_types::{ClientState, ConsensusState, Fraction};
use ics23::ibc_api::SDK_SPECS;
use jsonrpsee::{Extensions, core::async_trait};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;
use unionlabs::{
    ibc::core::{client::height::Height, commitment::merkle_root::MerkleRoot},
    result_unwrap,
};
use voyager_sdk::{
    anyhow, ensure_null,
    plugin::ClientBootstrapModule,
    primitives::{ChainId, ClientType},
    rpc::{ClientBootstrapModuleServer, RpcError, RpcResult, types::ClientBootstrapModuleInfo},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub gno_client: gno_rpc::Client,

    pub ibc_core_realm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
    pub ibc_core_realm: String,
}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ClientBootstrapModuleInfo) -> anyhow::Result<Self> {
        let gno_client = gno_rpc::Client::new(config.rpc_url).await?;

        let chain_id = gno_client.status(None).await?.node_info.network.to_string();

        info.ensure_chain_id(&chain_id)?;
        info.ensure_client_type(ClientType::GNO)?;

        Ok(Self {
            gno_client,
            chain_id: ChainId::new(chain_id),
            ibc_core_realm: config.ibc_core_realm,
        })
    }
}

impl Module {
    async fn fetch_unbonding_period(&self, _height: Height) -> Duration {
        // let params = self
        //     .gno_client
        //     .grpc_abci_query::<_, protos::cosmos::staking::v1beta1::QueryParamsResponse>(
        //         "/cosmos.staking.v1beta1.Query/Params",
        //         &protos::cosmos::staking::v1beta1::QueryParamsRequest {},
        //         Some(i64::try_from(height.height()).unwrap().try_into().unwrap()),
        //         false,
        //     )
        //     .await
        //     .unwrap()
        //     .value
        //     .unwrap()
        //     .params
        //     .unwrap();

        // let unbonding_period = params.unbonding_time.clone().unwrap();

        // TODO: Query from the chain properly, see: https://github.com/gnolang/gno/issues/4829

        Duration::from_hours(24 * 3)
    }
}

#[async_trait]
impl ClientBootstrapModuleServer for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_client_state(
        &self,
        _: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        ensure_null(config)?;

        let unbonding_period = self.fetch_unbonding_period(height).await;

        let commit = self
            .gno_client
            .commit((height.height() as i64).try_into().unwrap())
            .await
            .unwrap();

        let height = commit.signed_header.header.height;

        Ok(serde_json::to_value(ClientState {
            chain_id: __self.chain_id.to_string(),
            trust_level: Fraction {
                numerator: 1,
                denominator: const { NonZeroU64::new(3).unwrap() },
            },
            trusting_period: unionlabs::google::protobuf::duration::Duration::new(
                (unbonding_period * 85 / 100).as_secs().try_into().unwrap(),
                (unbonding_period * 85 / 100)
                    .subsec_nanos()
                    .try_into()
                    .unwrap(),
            )
            .unwrap(),
            unbonding_period: unionlabs::google::protobuf::duration::Duration::new(
                unbonding_period.as_secs().try_into().unwrap(),
                unbonding_period.subsec_nanos().try_into().unwrap(),
            )
            .unwrap(),
            max_clock_drift: const {
                result_unwrap!(unionlabs::google::protobuf::duration::Duration::new(
                    60 * 10,
                    0
                ))
            },
            frozen_height: None,
            latest_height: Height::new_with_revision(
                0,
                height.inner().try_into().expect("is within bounds; qed;"),
            ),
            proof_specs: SDK_SPECS.into(),
            upgrade_path: vec!["upgrade".into(), "upgradedIBCState".into()],
            realm: self.ibc_core_realm.clone(),
        })
        .unwrap())
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_consensus_state(
        &self,
        _: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        ensure_null(config)?;

        let commit = self
            .gno_client
            .commit((height.height() as i64).try_into().unwrap())
            .await
            .map_err(RpcError::retryable("error fetching commit"))?;

        Ok(serde_json::to_value(&ConsensusState {
            root: MerkleRoot {
                hash: commit
                    .signed_header
                    .header
                    .app_hash
                    .ok_or(RpcError::fatal_from_message("no app_hash in commit"))?
                    .into_encoding(),
            },
            next_validators_hash: commit.signed_header.header.next_validators_hash,
            timestamp: commit.signed_header.header.time,
        })
        .unwrap())
    }
}
