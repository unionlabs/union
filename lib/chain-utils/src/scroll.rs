use std::{num::NonZeroU64, sync::Arc};

use bip32::secp256k1::ecdsa;
use contracts::{devnet_ownable_ibc_handler::DevnetOwnableIBCHandler, ibc_handler::IBCHandler};
use ethers::{
    middleware::{NonceManagerMiddleware, SignerMiddleware},
    providers::{Middleware, Provider, ProviderError, Ws, WsClientError},
    signers::LocalWallet,
    utils::secret_key_to_address,
};
use scroll_api::ScrollClient;
use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::EthAbi,
    ethereum::config::Mainnet,
    hash::{H160, H256},
    ibc::{core::client::height::Height, lightclients::scroll},
    id::{ChannelId, ClientId, PortId},
    traits::{Chain, ClientState, FromStrExact, HeightOf},
    uint::U256,
};

use crate::{
    evm::{self, Ethereum, EvmInitError, EvmSignerMiddleware, HasIbcHandler, Readonly},
    private_key::PrivateKey,
    Pool,
};

pub const SCROLL_REVISION_NUMBER: u64 = 0;

#[derive(Debug, Clone)]
pub struct Scroll {
    pub chain_id: U256,

    // The provider on scroll chain.
    pub provider: Arc<Provider<Ws>>,

    pub ibc_handlers: Pool<IBCHandler<EvmSignerMiddleware>>,

    /// The address of the `IBCHandler` smart contract on scroll chain.
    pub ibc_handler_address: H160,
    pub scroll_api_client: ScrollClient,

    pub l1: Ethereum<Mainnet, Readonly>,

    pub rollup_contract_address: H160,
    pub rollup_finalized_state_roots_slot: U256,
    pub rollup_last_finalized_batch_index_slot: U256,
    pub latest_batch_index_slot: U256,
    pub l1_client_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The signer that will be used to submit transactions by voyager.
    pub signers: Vec<PrivateKey<ecdsa::SigningKey>>,

    /// The RPC endpoint for the execution (scroll) chain.
    pub scroll_eth_rpc_api: String,

    pub rollup_contract_address: H160,
    pub rollup_finalized_state_roots_slot: U256,
    pub rollup_last_finalized_batch_index_slot: U256,
    pub latest_batch_index_slot: U256,
    pub l1_client_id: String,

    pub evm: evm::Config<Readonly>,
    pub scroll_api: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ScrollInitError {
    #[error("unable to initialize L1")]
    Evm(#[from] EvmInitError),
    #[error("unable to connect to websocket")]
    Ws(#[from] WsClientError),
    #[error("provider error")]
    Provider(#[from] ProviderError),
}

impl Scroll {
    pub async fn new(config: Config) -> Result<Self, ScrollInitError> {
        let provider = Provider::new(Ws::connect(config.scroll_eth_rpc_api).await?);

        let chain_id = provider.get_chainid().await?;
        tracing::info!(?chain_id);

        let ibc_handlers = config.signers.into_iter().map(|signer| {
            let signing_key: ecdsa::SigningKey = signer.value();
            let address = secret_key_to_address(&signing_key);

            let wallet = LocalWallet::new_with_signer(signing_key, address, chain_id.as_u64());

            let signer_middleware = Arc::new(SignerMiddleware::new(
                NonceManagerMiddleware::new(provider.clone(), address),
                wallet.clone(),
            ));

            IBCHandler::new(
                config.ibc_handler_address.clone(),
                signer_middleware.clone(),
            )
        });

        Ok(Self {
            chain_id: U256(chain_id),
            ibc_handlers: Pool::new(ibc_handlers),
            provider: Arc::new(provider),
            scroll_api_client: ScrollClient::new(config.scroll_api),
            ibc_handler_address: config.ibc_handler_address,
            l1: Ethereum::new(config.evm).await?,
            rollup_contract_address: config.rollup_contract_address,
            rollup_finalized_state_roots_slot: config.rollup_finalized_state_roots_slot,
            rollup_last_finalized_batch_index_slot: config.rollup_last_finalized_batch_index_slot,
            latest_batch_index_slot: config.latest_batch_index_slot,
            l1_client_id: config.l1_client_id,
        })
    }

    pub async fn batch_index_of_beacon_height(&self, height: HeightOf<Self>) -> u64 {
        let execution_height = self
            .l1
            .beacon_api_client
            .execution_height(beacon_api::client::BlockId::Slot(height.revision_height))
            .await
            .unwrap();

        let storage = self
            .l1
            .provider
            .get_storage_at(
                ethers::types::H160(self.rollup_contract_address.0),
                H256::from(self.rollup_last_finalized_batch_index_slot.to_big_endian()).into(),
                Some(ethers::types::BlockId::Number(
                    ethers::types::BlockNumber::Number(execution_height.into()),
                )),
            )
            .await
            .unwrap();

        let batch_index = U256::from_big_endian(storage.to_fixed_bytes())
            .try_into()
            .unwrap();

        tracing::debug!("execution height {execution_height} is batch index {batch_index}");

        batch_index
    }

    pub async fn scroll_height_of_batch_index(&self, batch_index: u64) -> u64 {
        let batch = self.scroll_api_client.batch(batch_index).await.batch;

        tracing::debug!(
            "batch index {batch_index} is scroll height range {}..={}",
            batch.start_block_number,
            batch.end_block_number
        );

        batch.end_block_number
    }
}

impl HasIbcHandler for Scroll {
    fn ibc_handler(&self) -> IBCHandler<Provider<Ws>> {
        IBCHandler::new(self.ibc_handler_address.clone(), self.provider.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScrollChainType;
impl FromStrExact for ScrollChainType {
    const EXPECTING: &'static str = "scroll";
}

impl Chain for Scroll {
    type ChainType = ScrollChainType;

    type SelfClientState = scroll::client_state::ClientState;
    type SelfConsensusState = scroll::consensus_state::ConsensusState;

    type StoredClientState<Tr: Chain> =
        <Ethereum<Mainnet, Readonly> as Chain>::StoredClientState<Tr>;
    type StoredConsensusState<Tr: Chain> =
        <Ethereum<Mainnet, Readonly> as Chain>::StoredConsensusState<Tr>;

    type Header = scroll::header::Header;

    type Height = Height;

    type ClientId = ClientId;

    type IbcStateEncoding = EthAbi;

    type StateProof = unionlabs::ibc::lightclients::ethereum::storage_proof::StorageProof;

    type ClientType = String;

    type Error = <Ethereum<Mainnet, Readonly> as Chain>::Error;

    fn chain_id(&self) -> <Self::SelfClientState as ClientState>::ChainId {
        self.chain_id
    }

    async fn query_latest_height(&self) -> Result<Self::Height, Self::Error> {
        // the height of scroll is the beacon height of the l1
        self.l1.query_latest_height().await
    }

    async fn query_latest_height_as_destination(&self) -> Result<Self::Height, Self::Error> {
        // the height of scroll is the beacon height of the l1
        self.l1.query_latest_height_as_destination().await
    }

    fn query_latest_timestamp(
        &self,
    ) -> impl futures::prelude::Future<Output = Result<i64, Self::Error>> + '_ {
        self.l1.query_latest_timestamp()
    }

    async fn self_client_state(&self, height: Self::Height) -> Self::SelfClientState {
        scroll::client_state::ClientState {
            l1_client_id: self.l1_client_id.clone(),
            chain_id: self.chain_id(),
            latest_batch_index: self.batch_index_of_beacon_height(height).await,
            latest_batch_index_slot: self.latest_batch_index_slot,
            frozen_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
            rollup_contract_address: self.rollup_contract_address.clone(),
            rollup_finalized_state_roots_slot: self.rollup_finalized_state_roots_slot,
            ibc_contract_address: self.l1.ibc_handler_address.clone(),
            ibc_commitment_slot: U256::from(0),
        }
    }

    async fn self_consensus_state(&self, height: Self::Height) -> Self::SelfConsensusState {
        let trusted_header = self
            .l1
            .beacon_api_client
            .header(beacon_api::client::BlockId::Slot(height.revision_height))
            .await
            .unwrap()
            .data;

        let batch_index = self.batch_index_of_beacon_height(height).await;
        let scroll_height = self.scroll_height_of_batch_index(batch_index).await;

        let storage_root = self
            .provider
            .get_storage_at(
                ethers::types::H160(self.rollup_contract_address.0),
                H256::from(self.rollup_last_finalized_batch_index_slot.to_big_endian()).into(),
                Some(ethers::types::BlockId::Number(
                    ethers::types::BlockNumber::Number(scroll_height.into()),
                )),
            )
            .await
            .unwrap();

        scroll::consensus_state::ConsensusState {
            batch_index,
            ibc_storage_root: storage_root.into(),
            timestamp: self
                .l1
                .beacon_api_client
                .bootstrap(trusted_header.root)
                .await
                .unwrap()
                .data
                .header
                .execution
                .timestamp,
        }
    }

    async fn read_ack(
        &self,
        _tx_hash: H256,
        _destination_channel_id: ChannelId,
        _destination_port_id: PortId,
        _sequence: NonZeroU64,
    ) -> Vec<u8> {
        // This should be the same logic but using the scroll provider
        // self.evm
        //     .read_ack(
        //         tx_hash,
        //         destination_channel_id,
        //         destination_port_id,
        //         sequence,
        //     )
        //     .await
        todo!()
    }
}
