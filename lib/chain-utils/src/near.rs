use near_jsonrpc_client::methods::{self, status};
use near_primitives::types::{AccountId, BlockId, BlockReference, Finality};
use unionlabs::{
    encoding::Borsh,
    ibc::{core::client::height::Height, lightclients::near},
    id::ClientId,
    traits::{Chain, ChainIdOf},
};

pub struct Near {
    // ???
    rpc: near_jsonrpc_client::JsonRpcClient,
    chain_id: String,
    ibc_account_id: AccountId,
}

pub struct Config {
    rpc_url: String,
    ibc_account_id: AccountId,
}

impl Near {
    pub fn new(config: Config) -> Self {
        let rpc = near_jsonrpc_client::JsonRpcClient::connect(config.rpc_url);
        Self {
            rpc,
            chain_id: rpc.call(status::RpcStatusRequest).await.unwrap().chain_id,
            ibc_account_id: config.ibc_account_id,
        }
    }
}

impl Chain for Near {
    type ChainType;

    type SelfClientState = near::client_state::ClientState;
    type SelfConsensusState = near::consensus_state::ConsensusState;
    type Header = near::header::Header;

    type StoredClientState<Tr: Chain> = Tr::SelfClientState;
    type StoredConsensusState<Tr: Chain> = Tr::SelfConsensusState;

    type Height = Height;
    type ClientId = ClientId;
    type IbcStateEncoding = Borsh;
    type StateProof = RawStateProof;
    type ClientType;

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
                block_reference: BlockReference::Finality(Finality::Final),
            })
            .await
            .map_err(|x| Box::new(x))?;

        let lc_block = self
            .rpc
            .call(
                methods::next_light_client_block::RpcLightClientNextBlockRequest {
                    last_block_hash: block.header.last_final_block,
                },
            )
            .await
            .map_err(|x| Box::new(x) as _)?
            .expect("needs a light client block");

        Ok(near::client_state::ClientState {
            latest_height: lc_block.inner_lite.height - 1,
            ibc_account_id: self.ibc_account_id.clone(),
            initial_block_producers: lc_block.next_bps.map(convert_block_producers),
            frozen_height: 0,
        })
    }

    async fn self_consensus_state(&self, height: Self::Height) -> Self::SelfConsensusState {
        let block = self
            .rpc
            .call(methods::block::RpcBlockRequest {
                block_reference: BlockReference::Finality(Finality::Final),
            })
            .await
            .map_err(|x| Box::new(x))?;

        let lc_block = self
            .rpc
            .call(
                methods::next_light_client_block::RpcLightClientNextBlockRequest {
                    last_block_hash: block.header.last_final_block,
                },
            )
            .await
            .map_err(|x| Box::new(x) as _)?
            .expect("needs a light client block");

        let prev_state_root = self
            .rpc
            .call(methods::block::RpcBlockRequest {
                block_reference: BlockReference::BlockId(BlockId::Height(
                    lc_block.inner_lite.height,
                )),
            })
            .await
            .map(|x| x.chunks[0].prev_state_root)
            .map_err(|x| Box::new(x) as _)?;

        Ok(near::consensus_state::ConsensusState {
            state: convert_block_header_inner(lc_block.inner_lite),
            chunk_prev_state_root: prev_state_root,
            timestamp: self
                .rpc
                .call(methods::block::RpcBlockRequest {
                    block_reference: BlockReference::Finality(Finality::Final),
                })
                .await
                .map(|x| x.header.timestamp_nanosec.try_into().expect("idk bro"))
                .map_err(|x| Box::new(x))?,
        })
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
                    public_key: ::near::types::PublicKey::Ed25519(
                        stake.public_key.key_data().try_into().unwrap(),
                    ),
                    stake: stake.stake,
                },
            );
            stake
        })
        .collect()
}

pub fn convert_block_header_inner(
    block_view: near_primitives::views::BlockHeaderInnerLiteView,
) -> near::block_header_inner::BlockHeaderInnerLiteView {
    near::block_header_inner::BlockHeaderInnerLiteView {
        height: block_view.height,
        epoch_id: near_primitives_core::CryptoHash(block_view.epoch_id.0),
        next_epoch_id: near_primitives_core::CryptoHash(block_view.next_epoch_id.0),
        prev_state_root: near_primitives_core::CryptoHash(block_view.prev_state_root.0),
        outcome_root: near_primitives_core::CryptoHash(block_view.outcome_root.0),
        timestamp: block_view.timestamp,
        timestamp_nanosec: block_view.timestamp_nanosec,
        next_bp_hash: block_view.next_bp_hash.0.into(),
        block_merkle_root: near_primitives_core::CryptoHash(block_view.block_merkle_root.0),
    }
}
