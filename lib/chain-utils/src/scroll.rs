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
    ethereum::config::Mainnet,
    hash::{H160, H256},
    ibc::{core::client::height::Height, lightclients::scroll},
    id::{ChannelId, PortId},
    traits::{Chain, ClientState, FromStrExact, HeightOf},
    uint::U256,
};

use crate::{
    evm::{self, Evm, EvmInitError, EvmSignerMiddleware},
    private_key::PrivateKey,
    Pool,
};

pub const SCROLL_REVISION_NUMBER: u64 = 0;

#[derive(Debug, Clone)]
pub struct Scroll {
    pub chain_id: U256,

    // The provider on scroll chain.
    pub provider: Provider<Ws>,

    pub ibc_handlers: Pool<IBCHandler<EvmSignerMiddleware>>,

    // The IBCHandler contract deployed on scroll chain.
    pub readonly_ibc_handler: DevnetOwnableIBCHandler<Provider<Ws>>,
    pub scroll_api_client: ScrollClient,
    pub evm: Evm<Mainnet>,

    pub rollup_contract_address: H160,
    pub rollup_finalized_state_roots_slot: U256,
    pub rollup_last_finalized_batch_index_slot: U256,
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
    pub l1_client_id: String,
    pub evm: evm::Config,
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
            readonly_ibc_handler: DevnetOwnableIBCHandler::new(
                config.ibc_handler_address.clone(),
                provider.clone().into(),
            ),
            provider,
            scroll_api_client: ScrollClient::new(config.scroll_api),
            evm: Evm::new(config.evm).await?,
            rollup_contract_address: config.rollup_contract_address,
            rollup_finalized_state_roots_slot: config.rollup_finalized_state_roots_slot,
            rollup_last_finalized_batch_index_slot: config.rollup_last_finalized_batch_index_slot,
            l1_client_id: config.l1_client_id,
        })
    }

    pub async fn batch_index_of_beacon_height(&self, height: HeightOf<Self>) -> u64 {
        let execution_height = self.evm.execution_height(height).await;

        let storage = self
            .evm
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScrollChainType;
impl FromStrExact for ScrollChainType {
    const EXPECTING: &'static str = "scroll";
}

impl Chain for Scroll {
    type ChainType = ScrollChainType;

    type SelfClientState = scroll::client_state::ClientState;
    type SelfConsensusState = scroll::consensus_state::ConsensusState;

    type StoredClientState<Tr: Chain> = <Evm<Mainnet> as Chain>::StoredClientState<Tr>;
    type StoredConsensusState<Tr: Chain> = <Evm<Mainnet> as Chain>::StoredConsensusState<Tr>;

    type Header = scroll::header::Header;

    type Height = <Evm<Mainnet> as Chain>::Height;

    type ClientId = <Evm<Mainnet> as Chain>::ClientId;

    type IbcStateEncoding = <Evm<Mainnet> as Chain>::IbcStateEncoding;

    type StateProof = <Evm<Mainnet> as Chain>::StateProof;

    type ClientType = String;

    type Error = <Evm<Mainnet> as Chain>::Error;

    fn chain_id(&self) -> <Self::SelfClientState as ClientState>::ChainId {
        self.chain_id
    }

    async fn query_latest_height(&self) -> Result<Self::Height, Self::Error> {
        self.evm.query_latest_height().await
    }

    async fn query_latest_height_as_destination(&self) -> Result<Self::Height, Self::Error> {
        self.evm.query_latest_height_as_destination().await
    }

    fn query_latest_timestamp(
        &self,
    ) -> impl futures::prelude::Future<Output = Result<i64, Self::Error>> + '_ {
        self.evm.query_latest_timestamp()
    }

    async fn self_client_state(&self, height: Self::Height) -> Self::SelfClientState {
        scroll::client_state::ClientState {
            l1_client_id: self.l1_client_id.clone(),
            chain_id: self.chain_id(),
            latest_batch_index: self.batch_index_of_beacon_height(height).await,
            frozen_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
            rollup_contract_address: self.rollup_contract_address.clone(),
            rollup_finalized_state_roots_slot: self.rollup_finalized_state_roots_slot,
            ibc_contract_address: self.evm.readonly_ibc_handler.address().into(),
            ibc_commitment_slot: U256::from(0),
        }
    }

    async fn self_consensus_state(&self, height: Self::Height) -> Self::SelfConsensusState {
        let trusted_header = self
            .evm
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
                .evm
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
