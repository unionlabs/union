use std::sync::Arc;

use contracts::ibc_handler::IBCHandler;
use ethers::{
    contract::{EthAbiCodec, EthAbiType, EthDisplay, EthEvent},
    providers::{Middleware, Provider, ProviderError, Ws, WsClientError},
};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};
use unionlabs::{
    bounded::BoundedU32,
    google::protobuf::any::Any,
    hash::{H160, H256},
    ibc::{
        core::client::height::Height,
        lightclients::{
            arbitrum,
            ethereum::{self, storage_proof::StorageProof},
        },
    },
    uint::U256,
    validated::ValidateT,
};

use crate::{
    ethereum::{
        balance_of_signers, AnyEthereum, AnyEthereumError, EthereumConsensusChain,
        EthereumIbcChain, EthereumKeyring, EthereumSignerMiddleware, EthereumSignersConfig,
        ReadWrite, Readonly,
    },
    keyring::{ChainKeyring, ConcurrentKeyring, KeyringConfig, SignerBalance},
};

pub const ARBITRUM_REVISION_NUMBER: u64 = 0;

#[derive(Debug, Clone)]
pub struct Arbitrum {
    chain_id: U256,

    pub keyring: EthereumKeyring,

    pub provider: Arc<Provider<Ws>>,
    pub ibc_handler_address: H160,
    pub ibc_commitment_slot: U256,
    pub multicall_address: H160,

    pub l1: AnyEthereum<Readonly>,
    pub l1_contract_address: H160,
    pub l1_next_node_num_slot: U256,
    pub l1_next_node_num_slot_offset_bytes: BoundedU32<0, 24>,
    pub l1_nodes_slot: U256,
    pub l1_nodes_confirm_data_offset: U256,

    pub l1_client_id: U256,
    /// GRPC url of Union, used to query the L1 state with [`Self::l1_client_id`].
    pub union_grpc_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
    pub ibc_commitment_slot: U256,
    pub multicall_address: H160,

    /// The signer that will be used to submit transactions by voyager.
    pub keyring: KeyringConfig,

    /// The RPC endpoint for the execution (scroll) chain.
    pub l2_eth_rpc_api: String,

    pub l1_contract_address: H160,
    pub l1_latest_confirmed_slot: U256,
    pub l1_nodes_slot: U256,
    pub l1_nodes_confirm_data_offset: U256,
    pub l1_next_node_num_slot_offset_bytes: BoundedU32<0, 24>,

    pub l1_client_id: U256,
    pub l1: crate::ethereum::Config<Readonly>,

    pub union_grpc_url: String,
}

impl ChainKeyring for Arbitrum {
    type Address = H160;

    type Signer = IBCHandler<EthereumSignerMiddleware>;

    fn keyring(&self) -> &ConcurrentKeyring<Self::Address, Self::Signer> {
        &self.keyring
    }

    async fn balances(&self) -> Vec<SignerBalance<Self::Address>> {
        balance_of_signers(&self.keyring, &self.provider).await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ArbitrumInitError {
    #[error("unable to initialize L1")]
    Ethereum(#[from] AnyEthereumError),
    #[error("unable to connect to websocket")]
    Ws(#[from] WsClientError),
    #[error("provider error")]
    Provider(#[from] ProviderError),
}

impl Arbitrum {
    pub async fn new(config: Config) -> Result<Self, ArbitrumInitError> {
        let provider = Provider::new(Ws::connect(config.l2_eth_rpc_api.clone()).await?);

        let chain_id = provider.get_chainid().await?;

        Ok(Self {
            chain_id: U256::from(chain_id),
            keyring: ReadWrite::new(
                config.keyring,
                config.ibc_handler_address,
                chain_id.as_u64(),
                provider.clone(),
            ),
            ibc_handler_address: config.ibc_handler_address,
            multicall_address: config.multicall_address,
            provider: Arc::new(provider),
            l1: AnyEthereum::new(config.l1).await?,
            l1_client_id: config.l1_client_id,
            union_grpc_url: config.union_grpc_url,
            l1_contract_address: config.l1_contract_address,
            l1_next_node_num_slot: config.l1_latest_confirmed_slot,
            ibc_commitment_slot: config.ibc_commitment_slot,
            l1_nodes_slot: config.l1_nodes_slot,
            l1_next_node_num_slot_offset_bytes: config.l1_next_node_num_slot_offset_bytes,
            l1_nodes_confirm_data_offset: config.l1_nodes_confirm_data_offset,
        })
    }

    #[instrument(
        skip_all,
        level = "trace",
        fields(
            %slot,
            %self.l1_next_node_num_slot_offset_bytes,
            %self.l1_contract_address,
            %self.l1_next_node_num_slot,
        )
    )]
    pub async fn next_node_num_at_beacon_slot(&self, slot: u64) -> u64 {
        let l1_height = self.l1.execution_height_of_beacon_slot(slot).await;

        let slot_offset_bytes = self.l1_next_node_num_slot_offset_bytes.inner() as usize;

        let raw_slot = self
            .l1
            .provider()
            .get_storage_at(
                ethers::types::H160::from(self.l1_contract_address),
                ethers::types::H256(self.l1_next_node_num_slot.to_be_bytes()),
                Some(ethers::types::BlockNumber::Number(l1_height.into()).into()),
            )
            .await
            .unwrap();

        debug!(raw_slot = %H256::from(raw_slot));

        let latest_confirmed = u64::from_be_bytes(
            raw_slot.0[slot_offset_bytes..slot_offset_bytes + 8]
                .try_into()
                .expect("size is correct; qed;"),
        );

        debug!("l1_height {l1_height} is next node num {latest_confirmed}");

        latest_confirmed
    }
}

impl EthereumIbcChain for Arbitrum {
    fn provider(&self) -> Arc<Provider<Ws>> {
        self.provider.clone()
    }

    fn ibc_handler_address(&self) -> H160 {
        self.ibc_handler_address
    }
}

impl EthereumConsensusChain for Arbitrum {
    // NOTE: any modifications made here should also be made to the same fn on the hubble Arb consensus height indexer
    async fn execution_height_of_beacon_slot(&self, slot: u64) -> u64 {
        // read the next_node_num at l1.execution_height(beacon_slot), then from there filter for `NodeCreated`
        let next_node_num = self.next_node_num_at_beacon_slot(slot).await;

        let [event] = self
            .l1
            .provider()
            .get_logs(
                &ethers::types::Filter::new()
                    .select(
                        ethers::types::BlockNumber::Earliest..ethers::types::BlockNumber::Latest,
                    )
                    .address(ethers::types::H160(self.l1_contract_address.0))
                    .topic0(NodeCreated::signature())
                    .topic1(ethers::types::H256(U256::from(next_node_num).to_be_bytes())),
            )
            .await
            .unwrap()
            .try_into()
            .unwrap();

        let event: NodeCreated =
            NodeCreated::decode_log(&ethers::abi::RawLog::from(event)).unwrap();

        debug!("next node num: {next_node_num}: {event:?}");

        let block = self
            .provider
            .get_block(ethers::types::H256(
                event.assertion.after_state.global_state.bytes32_vals[0].0,
            ))
            .await
            .unwrap()
            .unwrap();

        block.number.unwrap().0[0]
    }

    async fn get_proof(&self, _address: H160, _location: U256, _block: u64) -> StorageProof {
        todo!()
        // get_proof(self, address, location, block).await
    }
}

// #[derive(Debug, ethers::contract::EthEvent, ethers::contract::EthDisplay)]
// #[ethevent(
//     name = "NodeConfirmed",
//     abi = "NodeConfirmed(uint64 indexed, bytes32, bytes32)"
// )]
// struct NodeConfirmed {
//     node_num: u64,
//     block_hash: H256,
//     send_root: H256,
// }

#[derive(Debug, ethers::contract::EthEvent, EthDisplay)]
#[ethevent(
    name = "NodeCreated",
    abi = "NodeCreated(
        uint64 indexed,
        bytes32 indexed,
        bytes32 indexed,
        bytes32,
        (((bytes32[2],uint64[2]),uint8),((bytes32[2],uint64[2]),uint8),uint64),
        bytes32,
        bytes32,
        uint256
    )"
)]
pub struct NodeCreated {
    pub node_num: u64,
    pub parent_node_hash: H256,
    pub node_hash: H256,
    pub execution_hash: H256,
    pub assertion: Assertion,
    pub after_inbox_batch_acc: H256,
    pub wasm_module_root: H256,
    pub inbox_max_count: U256,
}

// https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/Node.sol#L15
#[derive(Debug, EthAbiType, EthAbiCodec, EthDisplay)]
pub struct Assertion {
    pub before_state: ExecutionState,
    pub after_state: ExecutionState,
    pub num_blocks: u64,
}

// https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/Node.sol#L10
#[derive(Debug, EthAbiType, EthAbiCodec, EthDisplay)]
pub struct ExecutionState {
    pub global_state: GlobalState,

    // https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/state/Machine.sol
    pub machine_status: u8,
}

// ethers doesn't support solidity enums (?????)
// #[derive(Debug, EthAbiType, EthAbiCodec)]
// enum MachineStatus {
//     RUNNING,
//     FINISHED,
//     ERRORED,
//     TOO_FAR,
// }

// https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/state/GlobalState.sol
#[derive(Debug, EthAbiType, EthAbiCodec, EthDisplay)]
pub struct GlobalState {
    pub bytes32_vals: [H256; 2],
    pub u64_vals: [u64; 2],
}

impl Arbitrum {
    pub async fn query_latest_height(
        &self,
    ) -> Result<Height, Box<dyn core::error::Error + Send + Sync>> {
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
        let Any(_l1_client_state) = <Any<
            unionlabs::ibc::lightclients::wasm::client_state::ClientState<
                ethereum::client_state::ClientState,
            >,
        >>::try_from(l1_client_state)
        .unwrap();

        // Ok(match &self.l1 {
        //     AnyEthereum::Mainnet(eth) => eth.make_height(l1_client_state.data.latest_slot),
        //     AnyEthereum::Minimal(eth) => eth.make_height(l1_client_state.data.latest_slot),
        // })

        todo!()
    }

    // async fn query_latest_height_as_destination(&self) -> Result<Self::Height, Self::Error> {
    //     todo!()
    // }

    // pub async fn query_latest_timestamp(
    //     &self,
    // ) -> Result<i64, Box<dyn core::error::Error + Send + Sync>> {
    //     match &self.l1 {
    //         AnyEthereum::Mainnet(eth) => eth.query_latest_timestamp().map_err(Into::into).await,
    //         AnyEthereum::Minimal(eth) => eth.query_latest_timestamp().map_err(Into::into).await,
    //     }
    // }

    pub async fn self_client_state(&self, height: Height) -> arbitrum::client_state::ClientState {
        arbitrum::client_state::ClientState {
            l1_client_id: self.l1_client_id.clone().to_string().validate().unwrap(),
            chain_id: self.chain_id,
            l1_latest_slot: height.revision_height,
            l1_contract_address: self.l1_contract_address,
            l1_next_node_num_slot: self.l1_next_node_num_slot,
            l1_nodes_slot: self.l1_nodes_slot,
            l1_next_node_num_slot_offset_bytes: self.l1_next_node_num_slot_offset_bytes,
            l1_nodes_confirm_data_offset: self.l1_nodes_confirm_data_offset,
            frozen_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
            l2_ibc_contract_address: self.ibc_handler_address,
            l2_ibc_commitment_slot: self.ibc_commitment_slot,
        }
    }

    pub async fn self_consensus_state(
        &self,
        height: Height,
    ) -> arbitrum::consensus_state::ConsensusState {
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
