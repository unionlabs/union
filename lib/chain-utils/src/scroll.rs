use std::{error::Error, sync::Arc};

use contracts::ibc_handler::IBCHandler;
use ethers::providers::{Middleware, Provider, ProviderError, Ws, WsClientError};
use futures::{FutureExt, TryFutureExt};
use scroll_api::ScrollClient;
use serde::{Deserialize, Serialize};
use tracing::debug;
use unionlabs::{
    ethereum::config::Mainnet,
    google::protobuf::any::Any,
    hash::{H160, H256},
    ibc::{core::client::height::Height, lightclients::scroll},
    traits::{Chain, ClientIdOf, ClientState, FromStrExact},
    uint::U256,
};

use crate::{
    ethereum::{
        self, balance_of_signers, Ethereum, EthereumChain, EthereumConsensusChain,
        EthereumInitError, EthereumKeyring, EthereumSignerMiddleware, EthereumSignersConfig,
        ReadWrite, Readonly,
    },
    keyring::{ChainKeyring, ConcurrentKeyring, KeyringConfig, SignerBalance},
    union::Union,
    wasm::Wasm,
};

pub const SCROLL_REVISION_NUMBER: u64 = 0;

#[derive(Debug, Clone)]
pub struct Scroll {
    pub chain_id: U256,

    /// The provider on scroll chain.
    pub provider: Arc<Provider<Ws>>,

    pub keyring: EthereumKeyring,

    /// The address of the `IBCHandler` smart contract deployed on scroll.
    pub ibc_handler_address: H160,
    pub multicall_address: H160,

    pub scroll_api_client: ScrollClient,
    pub scroll_rpc: scroll_rpc::JsonRpcClient,
    pub l1: Ethereum<Mainnet, Readonly>,

    pub rollup_contract_address: H160,
    /// [ScrollChain.finalizedStateRoots](https://github.com/scroll-tech/scroll/blob/71f88b04f5a69196138c8cec63a75cf1f0ba2d99/contracts/src/L1/rollup/ScrollChain.sol#L159)
    pub rollup_finalized_state_roots_slot: U256,
    /// [ScrollChain.lastFinalizedBatchIndex](https://github.com/scroll-tech/scroll/blob/71f88b04f5a69196138c8cec63a75cf1f0ba2d99/contracts/src/L1/rollup/ScrollChain.sol#L153)
    pub rollup_last_finalized_batch_index_slot: U256,
    /// [ScrollChain.messageQueue](https://github.com/scroll-tech/scroll/blob/71f88b04f5a69196138c8cec63a75cf1f0ba2d99/contracts/src/L1/rollup/ScrollChain.sol#L128)
    pub rollup_message_queue_slot: U256,
    /// [ScrollChain.committedBatches](https://github.com/scroll-tech/scroll/blob/71f88b04f5a69196138c8cec63a75cf1f0ba2d99/contracts/src/L1/rollup/ScrollChain.sol#L156)
    pub rollup_committed_batches_slot: U256,

    pub l1_client_id: ClientIdOf<Ethereum<Mainnet>>,
    /// GRPC url of Union, used to query the L1 state with [`Self::l1_client_id`].
    pub union_grpc_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
    pub multicall_address: H160,

    /// The signers that will be used to submit transactions by voyager.
    pub keyring: KeyringConfig,

    /// The RPC endpoint for the execution (scroll) chain.
    pub scroll_eth_rpc_api: String,

    pub rollup_contract_address: H160,
    pub rollup_finalized_state_roots_slot: U256,
    pub rollup_last_finalized_batch_index_slot: U256,
    pub rollup_message_queue_slot: U256,
    pub rollup_committed_batches_slot: U256,

    pub l1_client_id: ClientIdOf<Ethereum<Mainnet>>,
    pub l1: ethereum::Config<Readonly>,
    pub scroll_api: String,
    pub union_grpc_url: String,
}

impl ChainKeyring for Scroll {
    type Address = H160;

    type Signer = IBCHandler<EthereumSignerMiddleware>;

    fn keyring(&self) -> &ConcurrentKeyring<Self::Address, Self::Signer> {
        &self.keyring
    }

    async fn balances(&self) -> Vec<SignerBalance<Self::Address>> {
        balance_of_signers(&self.keyring, &self.provider).await
    }
}

impl EthereumChain for Scroll {
    fn provider(&self) -> Arc<Provider<Ws>> {
        self.provider.clone()
    }

    fn ibc_handler_address(&self) -> H160 {
        self.ibc_handler_address
    }
}

impl EthereumConsensusChain for Scroll {
    async fn execution_height_of_beacon_slot(&self, slot: u64) -> u64 {
        self.batch_index_of_beacon_slot(slot)
            .then(|bi| self.scroll_height_of_batch_index(bi))
            .await
    }

    async fn get_proof(
        &self,
        address: H160,
        location: U256,
        block: u64,
    ) -> unionlabs::ibc::lightclients::ethereum::storage_proof::StorageProof {
        let proof = self
            .scroll_rpc
            .get_proof(address, [location], scroll_rpc::BlockId::Number(block))
            .await
            .unwrap();

        let proof = match <[_; 1]>::try_from(proof.storage_proof) {
            Ok([proof]) => proof,
            Err(invalid) => {
                panic!("received invalid response from eth_getProof, expected length of 1 but got `{invalid:#?}`");
            }
        };

        unionlabs::ibc::lightclients::ethereum::storage_proof::StorageProof {
            key: proof.key,
            value: proof.value,
            proof: proof
                .proof
                .into_iter()
                .map(|bytes| bytes.to_vec())
                .collect(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ScrollInitError {
    #[error("unable to initialize L1")]
    Ethereum(#[from] EthereumInitError),
    #[error("unable to connect to websocket")]
    Ws(#[from] WsClientError),
    #[error("provider error")]
    Provider(#[from] ProviderError),
    #[error("jsonrpc error")]
    JsonRpc(#[from] scroll_rpc::JsonRpcError),
}

impl Scroll {
    pub async fn new(config: Config) -> Result<Self, ScrollInitError> {
        let provider = Provider::new(Ws::connect(config.scroll_eth_rpc_api.clone()).await?);

        let chain_id = provider.get_chainid().await?;

        Ok(Self {
            chain_id: U256(chain_id),
            keyring: ReadWrite::new(
                config.keyring,
                config.ibc_handler_address,
                chain_id.as_u64(),
                provider.clone(),
            ),
            ibc_handler_address: config.ibc_handler_address,
            multicall_address: config.multicall_address,
            provider: Arc::new(provider),
            scroll_api_client: ScrollClient::new(config.scroll_api),
            l1: Ethereum::new(config.l1).await?,
            rollup_contract_address: config.rollup_contract_address,
            rollup_finalized_state_roots_slot: config.rollup_finalized_state_roots_slot,
            rollup_last_finalized_batch_index_slot: config.rollup_last_finalized_batch_index_slot,
            l1_client_id: config.l1_client_id,
            union_grpc_url: config.union_grpc_url,
            scroll_rpc: scroll_rpc::JsonRpcClient::new(config.scroll_eth_rpc_api).await?,
            rollup_message_queue_slot: config.rollup_message_queue_slot,
            rollup_committed_batches_slot: config.rollup_committed_batches_slot,
        })
    }

    pub async fn batch_index_of_beacon_slot(&self, slot: u64) -> u64 {
        let execution_height = self
            .l1
            .beacon_api_client
            .execution_height(beacon_api::client::BlockId::Slot(slot))
            .await
            .unwrap();

        let storage = self
            .l1
            .provider
            .get_storage_at(
                ethers::types::H160(self.rollup_contract_address.0),
                H256::from(self.rollup_last_finalized_batch_index_slot.to_be_bytes()).into(),
                Some(ethers::types::BlockId::Number(
                    ethers::types::BlockNumber::Number(execution_height.into()),
                )),
            )
            .await
            .unwrap();

        let batch_index = U256::from_be_bytes(storage.to_fixed_bytes())
            .try_into()
            .unwrap();

        debug!("execution height {execution_height} is batch index {batch_index}");

        batch_index
    }

    pub async fn scroll_height_of_batch_index(&self, batch_index: u64) -> u64 {
        let batch = self.scroll_api_client.batch(batch_index).await.batch;

        debug!(
            "batch index {batch_index} is scroll height range {}..={}",
            batch.start_block_number, batch.end_block_number
        );

        batch.end_block_number
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScrollChainType;
impl FromStrExact for ScrollChainType {
    const EXPECTING: &'static str = "scroll";
}

type BoxDynError = Box<dyn Error + Send + Sync + 'static>;

impl Chain for Scroll {
    type ChainType = ScrollChainType;

    type SelfClientState = scroll::client_state::ClientState;
    type SelfConsensusState = scroll::consensus_state::ConsensusState;

    type StoredClientState<Tr: Chain> = <Ethereum<Mainnet> as Chain>::StoredClientState<Tr>;
    type StoredConsensusState<Tr: Chain> = <Ethereum<Mainnet> as Chain>::StoredConsensusState<Tr>;

    type Header = scroll::header::Header;

    type Height = <Ethereum<Mainnet> as Chain>::Height;

    type ClientId = <Ethereum<Mainnet> as Chain>::ClientId;

    type IbcStateEncoding = <Ethereum<Mainnet> as Chain>::IbcStateEncoding;

    type StateProof = <Ethereum<Mainnet> as Chain>::StateProof;

    type ClientType = String;

    type Error = BoxDynError;

    fn chain_id(&self) -> <Self::SelfClientState as ClientState>::ChainId {
        self.chain_id
    }

    async fn query_latest_height(&self) -> Result<Self::Height, Self::Error> {
        // the latest height of scroll is the latest height of the l1 light client on union
        let l1_client_state = protos::ibc::core::client::v1::query_client::QueryClient::connect(
            self.union_grpc_url.clone(),
        )
        .await?
        .client_state(protos::ibc::core::client::v1::QueryClientStateRequest {
            client_id: self.l1_client_id.to_string(),
        })
        .await?
        .into_inner()
        .client_state
        .ok_or("client state missing???")?;

        // don't worry about it
        let Any(l1_client_state) =
            <<Wasm<Union> as Chain>::StoredClientState<Ethereum<Mainnet>>>::try_from(
                l1_client_state,
            )
            .unwrap();

        Ok(self.l1.make_height(l1_client_state.data.latest_slot))
    }

    async fn query_latest_height_as_destination(&self) -> Result<Self::Height, Self::Error> {
        // the height of scroll (as destination) is the beacon height of the l1
        self.l1
            .query_latest_height_as_destination()
            .await
            .map_err(Into::into)
    }

    // FIXME: must be scroll timestamp, not L1
    async fn query_latest_timestamp(&self) -> Result<i64, Self::Error> {
        self.l1.query_latest_timestamp().map_err(Into::into).await
    }

    async fn self_client_state(&self, height: Self::Height) -> Self::SelfClientState {
        scroll::client_state::ClientState {
            l1_client_id: self.l1_client_id.to_string(),
            chain_id: self.chain_id(),
            // REVIEW: Should we query the l1 latest slot here?
            latest_slot: height.revision_height,
            latest_batch_index_slot: self.rollup_last_finalized_batch_index_slot,
            frozen_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
            rollup_contract_address: self.rollup_contract_address,
            rollup_finalized_state_roots_slot: self.rollup_finalized_state_roots_slot,
            ibc_contract_address: self.ibc_handler_address,
            ibc_commitment_slot: U256::from(0),
            rollup_committed_batches_slot: self.rollup_committed_batches_slot,
        }
    }

    async fn self_consensus_state(&self, height: Self::Height) -> Self::SelfConsensusState {
        let batch_index = self
            .batch_index_of_beacon_slot(height.revision_height)
            .await;
        let scroll_height = self.scroll_height_of_batch_index(batch_index).await;

        let storage_root = self
            .provider
            .get_storage_at(
                ethers::types::H160(self.rollup_contract_address.0),
                H256::from(self.rollup_last_finalized_batch_index_slot.to_be_bytes()).into(),
                Some(ethers::types::BlockId::Number(
                    ethers::types::BlockNumber::Number(scroll_height.into()),
                )),
            )
            .await
            .unwrap();

        scroll::consensus_state::ConsensusState {
            ibc_storage_root: storage_root.into(),
            // Normalize to nanoseconds to be ibc-go compliant
            // FIXME: must be scroll timestamp, not L1
            timestamp: self
                .l1
                .beacon_api_client
                .bootstrap_for_slot(height.revision_height)
                .await
                .unwrap()
                .data
                .header
                .execution
                .timestamp
                * 1_000_000_000,
        }
    }
}
