// #![warn(clippy::unwrap_used)]

use std::collections::VecDeque;

use alloy::{
    network::{AnyNetwork, AnyRpcBlock},
    providers::{DynProvider, Provider, ProviderBuilder},
};
use ethereum_light_client_types::AccountProof;
use ibc_union_spec::{path::ConsensusStatePath, ClientId, IbcUnion};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use parlia_light_client_types::{ConsensusState, Header};
use parlia_types::ParliaHeader;
use parlia_verifier::EPOCH_LENGTH;
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use unionlabs::{ibc::core::client::height::Height, never::Never, primitives::H160, ErrorReporter};
use voyager_sdk::{
    anyhow,
    hook::UpdateHook,
    into_value,
    message::{
        call::Call,
        data::{Data, DecodedHeaderMeta, OrderedHeaders},
        PluginMessage, VoyagerMessage,
    },
    plugin::Plugin,
    primitives::{ChainId, ClientType, IbcInterface, QueryHeight},
    rpc::{types::PluginInfo, PluginServer},
    vm::{
        self,
        pass::{PassResult, Ready},
        BoxDynError, Op, Visit,
    },
    DefaultCmd, ExtensionsExt, VoyagerClient,
};

use crate::call::{FetchUpdate, ModuleCall};

pub mod call;

#[tokio::main]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub provider: DynProvider<AnyNetwork>,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,

    pub rpc_url: String,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    #[serde(default)]
    pub max_cache_size: u32,
}

fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .connect(&config.rpc_url)
                .await?,
        );

        let chain_id = ChainId::new(provider.get_chain_id().await?.to_string());

        assert_eq!(chain_id, config.chain_id);

        Ok(Self {
            chain_id,
            provider,
            ibc_handler_address: config.ibc_handler_address,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: UpdateHook::filter(
                &config.chain_id,
                &ClientType::new(ClientType::PARLIA),
            ),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
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
        Ok(PassResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .map(|mut op| {
                    UpdateHook::new(
                        &self.chain_id,
                        &ClientType::new(ClientType::PARLIA),
                        |fetch| {
                            Call::Plugin(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(FetchUpdate {
                                    from_height: fetch.update_from,
                                    to_height: fetch.update_to,
                                    counterparty_chain_id: fetch.counterparty_chain_id.clone(),
                                    client_id: fetch
                                        .client_id
                                        .clone()
                                        .decode_spec::<IbcUnion>()
                                        .unwrap(),
                                }),
                            ))
                        },
                    )
                    .visit_op(&mut op);

                    op
                })
                .enumerate()
                .map(|(i, op)| Ready::new(vec![i], op))
                .collect(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(&self, e: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::FetchUpdate(FetchUpdate {
                from_height,
                to_height,
                counterparty_chain_id,
                client_id,
            }) => self
                .fetch_update(
                    e.voyager_client()?,
                    from_height,
                    to_height,
                    counterparty_chain_id,
                    client_id,
                )
                .await
                .map_err(|e| {
                    ErrorObject::owned(
                        -1,
                        format!("error fetching update: {}", ErrorReporter(&*e)),
                        None::<()>,
                    )
                }),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        cb: Never,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}

impl Module {
    async fn fetch_ibc_contract_root_proof(&self, height: u64) -> RpcResult<AccountProof> {
        let proof = self
            .provider
            .get_proof(self.ibc_handler_address.into(), vec![])
            .block_id(height.into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching ibc contract proof: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?;
        Ok(AccountProof {
            storage_root: proof.storage_hash.into(),
            proof: proof.account_proof.into_iter().map(|x| x.into()).collect(),
        })
    }

    #[instrument(
        skip_all,
        fields(
            chain_id = %self.chain_id,
            %counterparty_chain_id,
            %update_from,
            %update_to,
        )
    )]
    async fn fetch_update(
        &self,
        voy_client: &VoyagerClient,
        update_from: Height,
        update_to: Height,
        counterparty_chain_id: ChainId,
        client_id: ClientId,
    ) -> Result<Op<VoyagerMessage>, BoxDynError> {
        // fetch intermediate updates for every epoch

        let mut trusted_valset_size = {
            let trusted_consensus_state_bytes = voy_client
                .query_ibc_state(
                    counterparty_chain_id.clone(),
                    QueryHeight::Latest,
                    ConsensusStatePath {
                        client_id,
                        height: update_from.height(),
                    },
                )
                .await?;

            let trusted_consensus_state = voy_client
                .decode_consensus_state::<IbcUnion, ConsensusState>(
                    ClientType::new(ClientType::PARLIA),
                    IbcInterface::new(IbcInterface::IBC_COSMWASM),
                    trusted_consensus_state_bytes,
                )
                .await?;

            let block = self
                .provider
                .get_block(trusted_consensus_state.valset_epoch_block_number.into())
                .await?
                .unwrap();

            let (_, valset) =
                parlia_verifier::parse_epoch_rotation_header_extra_data(&block.header.extra_data)?;

            valset.len()
        };

        let mut headers = vec![];

        for block in windows(update_from.height(), update_to.height()) {
            info!("fetching update to {block}");

            let source = self.provider.get_block(block.into()).await?.unwrap();
            let target = self.provider.get_block((block + 1).into()).await?.unwrap();
            let attestation = self.provider.get_block((block + 2).into()).await?.unwrap();

            let trusted_valset_epoch_number =
                parlia_verifier::calculate_signing_valset_epoch_block_number(
                    attestation.header.number,
                    trusted_valset_size.try_into().unwrap(),
                );

            info!(%trusted_valset_epoch_number);

            trusted_valset_size = parlia_verifier::parse_epoch_rotation_header_extra_data(
                &self
                    .provider
                    .get_block(trusted_valset_epoch_number.into())
                    .await?
                    .unwrap()
                    .header
                    .extra_data,
            )?
            .1
            .len();

            let ibc_account_proof = self
                .fetch_ibc_contract_root_proof(source.header.number)
                .await?;

            headers.push(Header {
                trusted_valset_epoch_number,
                source: convert_header(source),
                target: convert_header(target),
                attestation: convert_header(attestation),
                ibc_account_proof,
            })
        }

        Ok(vm::data(OrderedHeaders {
            headers: headers
                .into_iter()
                .map(|h| {
                    (
                        DecodedHeaderMeta {
                            height: Height::new(h.source.number.try_into().unwrap()),
                        },
                        into_value(h),
                    )
                })
                .collect(),
        }))
    }
}

fn convert_header(block: AnyRpcBlock) -> ParliaHeader {
    let block = block.0.into_inner();
    ParliaHeader {
        parent_hash: block.header.inner.parent_hash.into(),
        sha3_uncles: block.header.inner.ommers_hash.into(),
        miner: block.header.inner.beneficiary.into(),
        state_root: block.header.inner.state_root.into(),
        transactions_root: block.header.inner.transactions_root.into(),
        receipts_root: block.header.inner.receipts_root.into(),
        logs_bloom: Box::new(block.header.inner.logs_bloom.0.into()),
        difficulty: block.header.inner.difficulty.into(),
        number: block.header.inner.number.into(),
        gas_limit: block.header.inner.gas_limit,
        gas_used: block.header.inner.gas_used,
        timestamp: block.header.inner.timestamp,
        extra_data: block.header.inner.extra_data.into(),
        // always present in a parlia header, contains the timestamp millisecond portion
        mix_hash: block.header.inner.mix_hash.unwrap().into(),
        nonce: block.header.inner.nonce.unwrap().into(),
        base_fee_per_gas: block.header.inner.base_fee_per_gas.unwrap().into(),
        withdrawals_root: block.header.inner.withdrawals_root.unwrap().into(),
        blob_gas_used: block.header.inner.blob_gas_used.unwrap(),
        excess_blob_gas: block.header.inner.excess_blob_gas.unwrap(),
        parent_beacon_block_root: block.header.inner.parent_beacon_block_root.unwrap().into(),
        requests_hash: block.header.inner.requests_hash.map(Into::into).into(),
    }
}

fn windows(from: u64, to: u64) -> impl Iterator<Item = u64> {
    std::iter::successors(Some(from), move |from| {
        if from >= &to {
            None
        } else {
            let next_rotation = from + (EPOCH_LENGTH - (from % EPOCH_LENGTH));

            if next_rotation < to {
                Some(next_rotation)
            } else {
                Some(to)
            }
        }
    })
    // skip initial `from`
    .skip(1)
}

#[test]
fn test_windows() {
    dbg!(windows(1, 9999).collect::<Vec<_>>());
}
