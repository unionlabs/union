use near_jsonrpc_client::methods::{self, status};
use near_primitives::types::{AccountId, BlockId, BlockReference, Finality};
use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::Borsh,
    ibc::{core::client::height::Height, lightclients::near},
    id::ClientId,
    near::raw_state_proof::RawStateProof,
    traits::{Chain, ChainIdOf, FromStrExact},
};

use crate::keyring::{ChainKeyring, ConcurrentKeyring};

pub const NEAR_REVISION_NUMBER: u64 = 0;

#[derive(Debug, Clone)]
pub struct Near {
    // ???
    rpc: near_jsonrpc_client::JsonRpcClient,
    chain_id: String,
    ibc_account_id: AccountId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    rpc_url: String,
    ibc_account_id: AccountId,
}

impl ChainKeyring for Near {
    type Address = String;

    // TODO(aeryz): temporary hack for near
    type Signer = unionlabs::signer::CosmosSigner;

    fn keyring(&self) -> &ConcurrentKeyring<Self::Address, Self::Signer> {
        unimplemented!()
    }

    async fn balances(&self) -> Vec<crate::keyring::SignerBalance<Self::Address>> {
        unimplemented!()
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum NearInitError {
    // TODO(aeryz): add error context?
    #[error("rpc error")]
    RpcError,
}

impl Near {
    pub async fn new(config: Config) -> Result<Self, NearInitError> {
        let rpc = near_jsonrpc_client::JsonRpcClient::connect(config.rpc_url);
        let chain_id = rpc
            .call(status::RpcStatusRequest)
            .await
            .map_err(|_| NearInitError::RpcError)?
            .chain_id;

        Ok(Self {
            rpc,
            chain_id,
            ibc_account_id: config.ibc_account_id,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NearChainType;

impl FromStrExact for NearChainType {
    const EXPECTING: &'static str = "near";
}

impl Chain for Near {
    type ChainType = NearChainType;

    type SelfClientState = near::client_state::ClientState;
    type SelfConsensusState = near::consensus_state::ConsensusState;
    type Header = near::header::Header;

    type StoredClientState<Tr: Chain> = Tr::SelfClientState;
    type StoredConsensusState<Tr: Chain> = Tr::SelfConsensusState;

    type Height = Height;
    type ClientId = ClientId;
    type IbcStateEncoding = Borsh;
    type StateProof = RawStateProof;
    type ClientType = String;

    type Error = Box<dyn std::error::Error>;

    fn chain_id(&self) -> ChainIdOf<Self> {
        self.chain_id.clone()
    }

    async fn query_latest_height(&self) -> Result<Self::Height, Self::Error> {
        self.rpc
            .call(methods::block::RpcBlockRequest {
                block_reference: BlockReference::Finality(Finality::Final),
            })
            .await
            .map(|x| Height {
                revision_number: NEAR_REVISION_NUMBER,
                revision_height: x.header.height,
            })
            .map_err(|x| Box::new(x) as _)
    }

    async fn query_latest_height_as_destination(&self) -> Result<Self::Height, Self::Error> {
        self.query_latest_height().await
    }

    async fn query_latest_timestamp(&self) -> Result<i64, Self::Error> {
        self.rpc
            .call(methods::block::RpcBlockRequest {
                block_reference: BlockReference::Finality(Finality::Final),
            })
            .await
            .map(|x| x.header.timestamp_nanosec.try_into().expect("idk bro"))
            .map_err(|x| Box::new(x) as _)
    }

    async fn self_client_state(&self, height: Self::Height) -> Self::SelfClientState {
        let block = self
            .rpc
            .call(methods::block::RpcBlockRequest {
                block_reference: BlockReference::BlockId(BlockId::Height(height.revision_height)),
            })
            .await
            .unwrap();

        let validators = self
            .rpc
            .call(
                methods::EXPERIMENTAL_validators_ordered::RpcValidatorsOrderedRequest {
                    block_id: Some(BlockId::Height(block.header.height)),
                },
            )
            .await
            .unwrap();

        near::client_state::ClientState {
            chain_id: self.chain_id.clone(),
            latest_height: block.header.height - 1,
            ibc_account_id: self.ibc_account_id.clone(),
            initial_block_producers: convert_block_producers(validators),
            frozen_height: 0,
        }
    }

    async fn self_consensus_state(&self, height: Self::Height) -> Self::SelfConsensusState {
        let block = self
            .rpc
            .call(methods::block::RpcBlockRequest {
                block_reference: BlockReference::BlockId(BlockId::Height(height.revision_height)),
            })
            .await
            .unwrap();

        let chunk_prev_state_root = block.header.prev_state_root;
        let timestamp = block.header.timestamp_nanosec;

        near::consensus_state::ConsensusState {
            state: block_header_to_inner_lite(block.header),
            chunk_prev_state_root,
            timestamp,
        }
    }
}

pub fn convert_block_producers(
    bps: Vec<near_primitives::views::validator_stake_view::ValidatorStakeView>,
) -> Vec<near::validator_stake_view::ValidatorStakeView> {
    bps.into_iter()
        .map(|stake| {
            let near_primitives::views::validator_stake_view::ValidatorStakeView::V1(stake) = stake;
            let stake = near::validator_stake_view::ValidatorStakeView::V1(
                near::validator_stake_view::ValidatorStakeViewV1 {
                    account_id: stake.account_id,
                    public_key: unionlabs::near::types::PublicKey::Ed25519(
                        stake.public_key.key_data().try_into().unwrap(),
                    ),
                    stake: stake.stake,
                },
            );
            stake
        })
        .collect()
}

// pub fn convert_block_header_inner(
//     block_view: near_primitives::views::BlockHeaderInnerLiteView,
// ) -> near::block_header_inner::BlockHeaderInnerLiteView {
//     near::block_header_inner::BlockHeaderInnerLiteView {
//         height: block_view.height,
//         epoch_id: near_primitives_core::CryptoHash(block_view.epoch_id.0),
//         next_epoch_id: near_primitives_core::CryptoHash(block_view.next_epoch_id.0),
//         prev_state_root: near_primitives_core::CryptoHash(block_view.prev_state_root.0),
//         outcome_root: near_primitives_core::CryptoHash(block_view.outcome_root.0),
//         timestamp: block_view.timestamp,
//         timestamp_nanosec: block_view.timestamp_nanosec,
//         next_bp_hash: block_view.next_bp_hash.0.into(),
//         block_merkle_root: near_primitives_core::CryptoHash(block_view.block_merkle_root.0),
//     }
// }

pub fn block_header_to_inner_lite(
    header: near_primitives::views::BlockHeaderView,
) -> near::block_header_inner::BlockHeaderInnerLiteView {
    use near_primitives_core::hash::CryptoHash;
    near::block_header_inner::BlockHeaderInnerLiteView {
        height: header.height,
        epoch_id: CryptoHash(header.epoch_id.0),
        next_epoch_id: CryptoHash(header.next_epoch_id.0),
        prev_state_root: CryptoHash(header.prev_state_root.0),
        outcome_root: CryptoHash(header.outcome_root.0),
        timestamp: header.timestamp,
        timestamp_nanosec: header.timestamp_nanosec,
        next_bp_hash: CryptoHash(header.next_bp_hash.0),
        block_merkle_root: CryptoHash(header.block_merkle_root.0),
    }
}
