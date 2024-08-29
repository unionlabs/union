use std::{
    num::{NonZeroU64, ParseIntError},
    sync::Arc,
};

use cometbft_rpc::AbciQueryResponse;
use contracts::ibc_handler::IBCHandler;
use ethers::providers::{Middleware, Provider, ProviderError, Ws, WsClientError};
use ics23::ibc_api::SDK_SPECS;
use serde::{Deserialize, Serialize};
use unionlabs::{
    berachain::{
        BerachainChainSpec, LATEST_BEACON_BLOCK_HEADER_PREFIX,
        LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX,
    },
    encoding::{DecodeAs, Ssz},
    ethereum::IBC_HANDLER_COMMITMENTS_SLOT,
    google::protobuf::duration::Duration,
    hash::H160,
    ibc::{
        core::client::height::Height,
        lightclients::{
            berachain,
            ethereum::{
                beacon_block_header::BeaconBlockHeader,
                execution_payload_header::ExecutionPayloadHeader, storage_proof::StorageProof,
            },
            tendermint::fraction::Fraction,
        },
    },
    option_unwrap, result_unwrap,
    traits::FromStrExact,
    uint::U256,
};

use crate::{
    ethereum::{
        balance_of_signers, EthereumConsensusChain, EthereumIbcChain, EthereumSignerMiddleware,
        EthereumSignersConfig, ReadWrite,
    },
    keyring::{ChainKeyring, ConcurrentKeyring, SignerBalance},
    BoxDynError,
};

// FLOW:
//
// track cometbft height
// - height of berachain is then the height of the cometbft "beacon" node
//
// update is cometbft update (tm header but with bls12381 sigs) + latest execution header proof
// - can potentially optimize by pre-aggregating the signature
//
// membership proof is eth_getProof

#[derive(Debug, Clone)]
pub struct Berachain {
    /// Consensus layer chain id
    pub consensus_chain_id: String,
    /// Execution layer chain id
    pub execution_chain_id: U256,
    /// The revision of the cometbft consensus layer. This is used by the tendermint light client verification.
    pub consensus_chain_revision: u64,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub multicall_address: H160,

    pub keyring: Arc<<ReadWrite as EthereumSignersConfig>::Out>,

    // tendermint
    pub tm_client: cometbft_rpc::Client,

    // ethereum
    pub provider: Arc<Provider<Ws>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ws_url: String,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
    pub multicall_address: H160,

    pub keyring: <ReadWrite as EthereumSignersConfig>::Config,
}

impl ChainKeyring for Berachain {
    type Address = H160;

    type Signer = IBCHandler<EthereumSignerMiddleware>;

    fn keyring(&self) -> &ConcurrentKeyring<Self::Address, Self::Signer> {
        &self.keyring
    }

    async fn balances(&self) -> Vec<SignerBalance<Self::Address>> {
        balance_of_signers(&self.keyring, &self.provider).await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BerachainChainType;

impl FromStrExact for BerachainChainType {
    const EXPECTING: &'static str = "berachain";
}

#[allow(dead_code)]
impl Berachain {
    async fn query_latest_height(&self) -> Result<Height, BoxDynError> {
        Ok(Height {
            revision_number: self.consensus_chain_revision,
            revision_height: self
                .tm_client
                .block(None)
                .await
                .unwrap()
                .block
                .header
                .height
                .inner()
                .try_into()
                .expect("value is bounded; qed;"),
        })
    }

    async fn query_latest_timestamp(&self) -> Result<i64, BoxDynError> {
        Ok(self
            .provider
            .get_block(self.provider.get_block_number().await?)
            .await?
            .unwrap()
            .timestamp
            .try_into()
            .unwrap())
    }

    async fn self_client_state(&self, height: Height) -> berachain::client_state::ClientState {
        let commit = self
            .tm_client
            .commit(Some(height.revision_height.try_into().unwrap()))
            .await
            .unwrap();

        let height = commit.signed_header.header.height;

        // 256 epochs @ 3 seconds per block
        // 1/4 eth mainnet's ~28hrs
        const UNBONDING_PERIOD: i64 = 60 * 60 * 7;

        berachain::client_state::ClientState {
            consensus_chain_id: self.consensus_chain_id.clone(),
            execution_chain_id: self.execution_chain_id,
            // https://github.com/cometbft/cometbft/blob/da0e55604b075bac9e1d5866cb2e62eaae386dd9/light/verifier.go#L16
            trust_level: const {
                Fraction {
                    numerator: 1,
                    denominator: option_unwrap!(NonZeroU64::new(3)),
                }
            },
            trusting_period: const { result_unwrap!(Duration::new(UNBONDING_PERIOD * 85 / 100, 0)) },
            max_clock_drift: const { result_unwrap!(Duration::new(60 * 10, 0)) },
            frozen_height: None,
            latest_height: Height {
                revision_number: self.consensus_chain_revision,
                revision_height: height.inner().try_into().expect("value is >= 0; qed"),
            },
            proof_specs: SDK_SPECS.into(),
            upgrade_path: vec!["upgrade".into(), "upgradedIBCState".into()],
            ibc_commitment_slot: IBC_HANDLER_COMMITMENTS_SLOT,
            ibc_contract_address: self.ibc_handler_address,
        }
    }

    async fn self_consensus_state(
        &self,
        height: Height,
    ) -> berachain::consensus_state::ConsensusState {
        let execution_header = self
            .execution_header_at_beacon_slot(height.revision_height)
            .await;

        let commit = self
            .tm_client
            .commit(Some(height.revision_height.try_into().unwrap()))
            .await
            .unwrap();

        berachain::consensus_state::ConsensusState {
            eth_timestamp: execution_header.timestamp,
            comet_timestamp: commit.signed_header.header.time,
            eth_storage_root: self
                .provider
                .get_proof(
                    ethers::types::H160::from(self.ibc_handler_address.0),
                    vec![],
                    Some(execution_header.block_number.into()),
                )
                .await
                .unwrap()
                .storage_hash
                .0
                .into(),
            comet_next_validators_hash: commit.signed_header.header.next_validators_hash,
        }
    }
}

impl EthereumIbcChain for Berachain {
    fn provider(&self) -> Arc<Provider<Ws>> {
        self.provider.clone()
    }

    fn ibc_handler_address(&self) -> H160 {
        self.ibc_handler_address
    }
}

impl EthereumConsensusChain for Berachain {
    /// Fetches the execution block number of the provided beacon slot. Note that berachain has two notions of consensus heights - the cometbft height and the beacon slot, and they are *almost* 1:1. Beacon slots can be missed (in the same way as normal ethereum), however the cometbft block will still be produced. If a slot is missed, then the cometbft state at that height will be the same as the previous height.
    ///
    /// ```plaintext
    /// CL 10, EL 10
    /// CL 11, EL 10 <- missed slot
    /// CL 12, EL 11
    /// ```
    async fn execution_height_of_beacon_slot(&self, slot: u64) -> u64 {
        self.execution_header_at_beacon_slot(slot)
            .await
            .block_number
    }

    async fn get_proof(&self, _address: H160, _location: U256, _block: u64) -> StorageProof {
        todo!()
        // ethereum::get_proof(self, address, location, block).await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BerachainInitError {
    #[error("tendermint rpc error")]
    Tendermint(#[from] cometbft_rpc::JsonRpcError),
    #[error("provider error")]
    Provider(#[from] ProviderError),
    #[error("eth rpc ws client error")]
    WsClient(#[from] WsClientError),
    #[error(
        "unable to parse chain id: expected format `<chain>-<revision-number>`, found `{found}`"
    )]
    ChainIdParse {
        found: String,
        #[source]
        source: Option<ParseIntError>,
    },
}

impl Berachain {
    pub async fn new(config: Config) -> Result<Self, BerachainInitError> {
        let tm_client = cometbft_rpc::Client::new(config.ws_url).await?;

        let provider = Provider::new(Ws::connect(config.eth_rpc_api).await?);

        let execution_chain_id = provider.get_chainid().await?;

        let consensus_chain_id = tm_client.status().await?.node_info.network.to_string();

        let consensus_chain_revision = consensus_chain_id
            .split('-')
            .last()
            .ok_or_else(|| BerachainInitError::ChainIdParse {
                found: consensus_chain_id.clone(),
                source: None,
            })?
            .parse()
            .map_err(|err| BerachainInitError::ChainIdParse {
                found: consensus_chain_id.clone(),
                source: Some(err),
            })?;

        Ok(Self {
            tm_client,
            consensus_chain_id,
            execution_chain_id: execution_chain_id.into(),
            ibc_handler_address: config.ibc_handler_address,
            multicall_address: config.multicall_address,
            keyring: Arc::new(ReadWrite::new(
                config.keyring,
                config.ibc_handler_address,
                execution_chain_id.as_u64(),
                provider.clone(),
            )),
            provider: Arc::new(provider),
            consensus_chain_revision,
        })
    }

    /// A thin wrapper around abci_query to fetch the latest execution payload header.
    pub async fn execution_header_at_beacon_slot(
        &self,
        slot: u64,
    ) -> ExecutionPayloadHeader<BerachainChainSpec> {
        ExecutionPayloadHeader::<BerachainChainSpec>::decode_as::<Ssz>(
            &self
                .beacon_store_abci_query([LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX], slot, false)
                .await
                .response
                .value,
        )
        .unwrap()
    }

    /// A thin wrapper around abci_query to fetch the latest execution payload header.
    pub async fn beacon_block_header_at_beacon_slot(&self, slot: u64) -> BeaconBlockHeader {
        BeaconBlockHeader::decode_as::<Ssz>(
            &self
                .beacon_store_abci_query([LATEST_BEACON_BLOCK_HEADER_PREFIX], slot, false)
                .await
                .response
                .value,
        )
        .unwrap()
    }

    /// Perform an abci query on the `beacon` store ("store/beacon/key").
    pub async fn beacon_store_abci_query(
        &self,
        data: impl AsRef<[u8]>,
        slot: u64,
        prove: bool,
    ) -> AbciQueryResponse {
        self.tm_client
            .abci_query(
                "store/beacon/key",
                data,
                Some((slot as i64 - 1).try_into().unwrap()),
                prove,
            )
            .await
            .unwrap()
    }
}
