use std::sync::Arc;

use bip32::secp256k1::ecdsa;
use contracts::ibc_handler::IBCHandler;
use ethers::{
    contract::EthEvent,
    providers::{Middleware, Provider, ProviderError, Ws, WsClientError},
};
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::EthAbi,
    ethereum::config::Mainnet,
    google::protobuf::any::Any,
    hash::{H160, H256},
    ibc::{
        core::client::height::Height,
        lightclients::{arbitrum, ethereum::storage_proof::StorageProof},
    },
    id::ClientId,
    traits::{Chain, ChainIdOf, ClientIdOf, FromStrExact},
    uint::U256,
    ByteArrayExt,
};

use crate::{
    ethereum::{
        self, get_proof, Ethereum, EthereumChain, EthereumConsensusChain, EthereumInitError,
        EthereumSignerMiddleware, EthereumSignersConfig, ReadWrite, Readonly,
    },
    private_key::PrivateKey,
    union::Union,
    wasm::Wasm,
    Pool,
};

pub const ARBITRUM_REVISION_NUMBER: u64 = 0;

#[derive(Debug, Clone)]
pub struct Arbitrum {
    chain_id: U256,

    pub ibc_handlers: Pool<IBCHandler<EthereumSignerMiddleware>>,

    pub provider: Arc<Provider<Ws>>,
    pub ibc_handler_address: H160,
    pub ibc_commitment_slot: U256,

    pub l1: Ethereum<Mainnet, Readonly>,
    pub l1_contract_address: H160,
    pub l1_latest_confirmed_slot: U256,
    pub l1_nodes_slot: U256,
    pub l1_nodes_confirm_data_offset: U256,

    pub l1_client_id: ClientIdOf<Ethereum<Mainnet>>,
    /// GRPC url of Union, used to query the L1 state with [`Self::l1_client_id`].
    pub union_grpc_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
    pub ibc_commitment_slot: U256,

    /// The signer that will be used to submit transactions by voyager.
    pub signers: Vec<PrivateKey<ecdsa::SigningKey>>,

    /// The RPC endpoint for the execution (scroll) chain.
    pub l2_eth_rpc_api: String,

    pub l1_contract_address: H160,
    pub l1_latest_confirmed_slot: U256,
    pub l1_nodes_slot: U256,
    pub l1_nodes_confirm_data_offset: U256,

    pub l1_client_id: ClientIdOf<Ethereum<Mainnet>>,
    pub l1: ethereum::Config<Readonly>,

    pub union_grpc_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ArbitrumInitError {
    #[error("unable to initialize L1")]
    Ethereum(#[from] EthereumInitError),
    #[error("unable to connect to websocket")]
    Ws(#[from] WsClientError),
    #[error("provider error")]
    Provider(#[from] ProviderError),
}

impl Arbitrum {
    pub async fn new(config: Config) -> Result<Self, ArbitrumInitError> {
        let provider = Provider::new(Ws::connect(config.l2_eth_rpc_api.clone()).await?);

        let chain_id = provider.get_chainid().await?;
        tracing::info!(?chain_id);

        Ok(Self {
            chain_id: U256(chain_id),
            ibc_handlers: ReadWrite::new(
                config.signers,
                config.ibc_handler_address,
                chain_id.as_u64(),
                provider.clone(),
            ),
            ibc_handler_address: config.ibc_handler_address,
            provider: Arc::new(provider),
            l1: Ethereum::new(config.l1).await?,
            l1_client_id: config.l1_client_id,
            union_grpc_url: config.union_grpc_url,
            l1_contract_address: config.l1_contract_address,
            l1_latest_confirmed_slot: config.l1_latest_confirmed_slot,
            ibc_commitment_slot: config.ibc_commitment_slot,
            l1_nodes_slot: config.l1_nodes_slot,
            l1_nodes_confirm_data_offset: config.l1_nodes_confirm_data_offset,
        })
    }

    pub async fn latest_confirmed_at_beacon_slot(&self, slot: u64) -> u64 {
        let l1_height = self.l1.execution_height_of_beacon_slot(slot).await;

        let latest_confirmed = u64::from_be_bytes(
            self.l1
                .provider()
                .get_storage_at(
                    ethers::types::H160::from(self.l1_contract_address),
                    ethers::types::H256(self.l1_latest_confirmed_slot.to_be_bytes()),
                    Some(ethers::types::BlockNumber::Number(l1_height.into()).into()),
                )
                .await
                .unwrap()
                .0
                .array_slice::<24, 8>(),
        );

        tracing::debug!("l1_height {l1_height} is _latestConfirmed {latest_confirmed}");

        latest_confirmed
    }
}

impl EthereumChain for Arbitrum {
    fn provider(&self) -> Arc<Provider<Ws>> {
        self.provider.clone()
    }

    fn ibc_handler_address(&self) -> H160 {
        self.ibc_handler_address
    }
}

impl EthereumConsensusChain for Arbitrum {
    async fn execution_height_of_beacon_slot(&self, slot: u64) -> u64 {
        // read `_latestConfirmed` at l1.execution_height(beacon_slot), then from there filter for `NodeConfirmed`
        let latest_confirmed = self.latest_confirmed_at_beacon_slot(slot).await;

        let [event] = self
            .l1
            .provider()
            .get_logs(
                &ethers::types::Filter::new()
                    .select(
                        ethers::types::BlockNumber::Earliest..ethers::types::BlockNumber::Latest,
                    )
                    .address(ethers::types::H160(self.l1_contract_address.0))
                    .topic0(NodeConfirmed::signature())
                    .topic1(ethers::types::H256(
                        U256::from(latest_confirmed).to_be_bytes(),
                    )),
            )
            .await
            .unwrap()
            .try_into()
            .unwrap();

        let event: NodeConfirmed =
            NodeConfirmed::decode_log(&ethers::abi::RawLog::from(event)).unwrap();

        tracing::debug!("_latestConfirmed {latest_confirmed}: {event}");

        let block = self
            .provider
            .get_block(ethers::types::H256(event.block_hash.0))
            .await
            .unwrap()
            .unwrap();

        block.number.unwrap().0[0]
    }

    async fn get_proof(&self, address: H160, location: U256, block: u64) -> StorageProof {
        get_proof(self, address, location, block).await
    }
}

#[derive(Debug, ethers::contract::EthEvent, ethers::contract::EthDisplay)]
#[ethevent(
    name = "NodeConfirmed",
    abi = "NodeConfirmed(uint64 indexed, bytes32, bytes32)"
)]
struct NodeConfirmed {
    node_num: u64,
    block_hash: H256,
    send_root: H256,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArbitrumChainType;
impl FromStrExact for ArbitrumChainType {
    const EXPECTING: &'static str = "arbitrum";
}

impl Chain for Arbitrum {
    type ChainType = ArbitrumChainType;

    type SelfClientState = arbitrum::client_state::ClientState;
    type SelfConsensusState = arbitrum::consensus_state::ConsensusState;
    type Header = arbitrum::header::Header;

    type StoredClientState<Tr: Chain> = Tr::SelfClientState;
    type StoredConsensusState<Tr: Chain> = Tr::SelfConsensusState;

    type Height = Height;

    type ClientId = ClientId;

    type IbcStateEncoding = EthAbi;

    type StateProof = StorageProof;

    type ClientType = String;

    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn chain_id(&self) -> ChainIdOf<Self> {
        self.chain_id
    }

    async fn query_latest_height(&self) -> Result<Self::Height, Self::Error> {
        // the latest height of arbitrum is the latest height of the l1 light client on union

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
        todo!()
    }

    async fn query_latest_timestamp(&self) -> Result<i64, Self::Error> {
        self.l1.query_latest_timestamp().map_err(Into::into).await
    }

    async fn self_client_state(&self, height: Self::Height) -> Self::SelfClientState {
        arbitrum::client_state::ClientState {
            l1_client_id: self.l1_client_id.clone(),
            chain_id: self.chain_id,
            l1_latest_slot: height.revision_height,
            l1_contract_address: self.l1_contract_address,
            l1_latest_confirmed_slot: self.l1_latest_confirmed_slot,
            l1_nodes_slot: self.l1_nodes_slot,
            l1_nodes_confirm_data_offset: self.l1_nodes_confirm_data_offset,
            frozen_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
            l2_ibc_contract_address: self.ibc_handler_address,
            l2_ibc_commitment_slot: self.ibc_commitment_slot,
        }
    }

    async fn self_consensus_state(&self, height: Self::Height) -> Self::SelfConsensusState {
        let arbitrum_height = ethers::types::BlockId::Number(ethers::types::BlockNumber::Number(
            self.execution_height_of_beacon_slot(height.revision_height)
                .await
                .into(),
        ));

        let storage_root = self
            .provider
            .get_storage_at(
                ethers::types::H160(self.l1_contract_address.0),
                H256::from(self.ibc_commitment_slot.to_be_bytes()).into(),
                Some(arbitrum_height),
            )
            .await
            .unwrap();

        arbitrum::consensus_state::ConsensusState {
            ibc_storage_root: storage_root.0.into(),
            timestamp: self
                .provider
                .get_block(arbitrum_height)
                .await
                .unwrap()
                .unwrap()
                .timestamp
                .as_u64(),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use std::sync::Arc;

//     use hex_literal::hex;

//     use super::*;
//     use crate::ethereum;

//     #[tokio::test]
//     async fn fetch_block() {
//         // tracing_subscriber::fmt::init();

//         // let l1 = Ethereum::new(ethereum::Config {
//         //     ibc_handler_address: H160(hex!("6b6b60a68b8dcbb170f25045974d10098917f816")),
//         //     eth_rpc_api: "wss://eth-sepolia.g.alchemy.com/v2/6PCr1n8dJeYbE2Z9LrXScs05hLTYiVFl"
//         //         .to_string(),
//         //     eth_beacon_rpc_api: "https://lodestar-sepolia.chainsafe.io".to_string(),
//         //     signers: (),
//         // })
//         // .await
//         // .unwrap();

//         // let slot = l1.query_latest_height().await.unwrap();

//         // dbg!(slot);

//         // let arbitrum = Arbitrum {
//         //     chain_id: U256::from(421614),
//         //     provider: Arc::new(Provider::new(
//         //         Ws::connect("wss://arbitrum-sepolia.drpc.org")
//         //             .await
//         //             .unwrap(),
//         //     )),
//         //     ibc_handler_address: H160(hex!("61ba1780ecce6513872beb7ce698b49168010416")),
//         //     l1,
//         //     l1_contract_address: H160(hex!("d80810638dbDF9081b72C1B33c65375e807281C8")),
//         //     l1_latest_confirmed_slot: U256::from(0x75),
//         //     l1_client_id: "08-wasm-0".parse().unwrap(),
//         //     union_grpc_url: "https://grpc.testnet.bonlulu.uno:443".to_string(),
//         //     ibc_handlers: Pool::new(),
//         // };

//         // let block = arbitrum
//         //     .execution_height_of_beacon_slot(slot.revision_height)
//         //     .await;

//         // dbg!(block);
//     }
// }
