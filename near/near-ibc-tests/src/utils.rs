use near_primitives::{
    merkle::merklize,
    types::BlockHeight,
    views::{
        validator_stake_view::ValidatorStakeView, BlockHeaderInnerLiteView, LightClientBlockView,
    },
};
use near_primitives_core::hash::CryptoHash;
use near_sdk::AccountId;
use near_workspaces::{network::Sandbox, Worker};
use unionlabs::near::types::{self, MerklePathItem};

pub async fn state_proof(
    sandbox: &Worker<Sandbox>,
    ibc_contract_id: &AccountId,
    block_height: BlockHeight,
    commitment_key: &str,
) -> Vec<u8> {
    #[derive(serde::Serialize)]
    pub struct RawStateProof {
        state_proof: Vec<Vec<u8>>,
    }

    let mut proof_key = b"commitments".to_vec();
    proof_key.extend(borsh::to_vec(commitment_key).unwrap());

    let state = sandbox
        .view_state(ibc_contract_id)
        .prefix(&proof_key)
        .block_height(block_height)
        .await
        .unwrap();
    let state_proof: Vec<Vec<u8>> = state
        .proof
        .clone()
        .into_iter()
        .map(|item| item.to_vec())
        .collect();

    serde_json::to_vec(&RawStateProof { state_proof }).unwrap()
}

pub async fn chunk_proof(
    sandbox: &Worker<Sandbox>,
    block_height: BlockHeight,
) -> (CryptoHash, types::MerklePath) {
    let chunks = sandbox
        .view_block()
        .block_height(block_height)
        .await
        .unwrap()
        .chunks()
        .to_vec();

    let prev_state_root = CryptoHash(chunks[0].prev_state_root.clone().0);

    let (_, merkle_path) = merklize(
        &chunks
            .into_iter()
            .map(|chunk| CryptoHash(chunk.prev_state_root.0))
            .collect::<Vec<CryptoHash>>(),
    );

    let prev_state_root_proof = merkle_path[0]
        .clone()
        .into_iter()
        .map(|item| MerklePathItem {
            hash: near_primitives_core::hash::CryptoHash(item.hash.0),
            direction: match item.direction {
                near_primitives::merkle::Direction::Left => types::Direction::Left,
                near_primitives::merkle::Direction::Right => types::Direction::Right,
            },
        })
        .collect();

    (prev_state_root, prev_state_root_proof)
}

pub fn convert_block_producers(bps: Vec<ValidatorStakeView>) -> Vec<types::ValidatorStakeView> {
    bps.into_iter()
        .map(|stake| {
            let ValidatorStakeView::V1(stake) = stake;
            let stake = types::ValidatorStakeView::V1(types::ValidatorStakeViewV1 {
                account_id: stake.account_id,
                public_key: types::PublicKey::Ed25519(
                    stake.public_key.key_data().try_into().unwrap(),
                ),
                stake: stake.stake,
            });
            stake
        })
        .collect()
}

pub fn convert_block_header_inner(
    block_view: BlockHeaderInnerLiteView,
) -> types::BlockHeaderInnerLiteView {
    types::BlockHeaderInnerLiteView {
        height: block_view.height,
        epoch_id: CryptoHash(block_view.epoch_id.0),
        next_epoch_id: CryptoHash(block_view.next_epoch_id.0),
        prev_state_root: CryptoHash(block_view.prev_state_root.0),
        outcome_root: CryptoHash(block_view.outcome_root.0),
        timestamp: block_view.timestamp,
        timestamp_nanosec: block_view.timestamp_nanosec,
        next_bp_hash: CryptoHash(block_view.next_bp_hash.0),
        block_merkle_root: CryptoHash(block_view.block_merkle_root.0),
    }
}

pub fn convert_light_client_block_view(
    light_client_block: LightClientBlockView,
) -> types::LightClientBlockView {
    types::LightClientBlockView {
        inner_lite: convert_block_header_inner(light_client_block.inner_lite),
        prev_block_hash: near_primitives_core::hash::CryptoHash(
            light_client_block.prev_block_hash.0,
        ),
        next_block_inner_hash: near_primitives_core::hash::CryptoHash(
            light_client_block.next_block_inner_hash.0,
        ),
        inner_rest_hash: near_primitives_core::hash::CryptoHash(
            light_client_block.inner_rest_hash.0,
        ),
        next_bps: light_client_block.next_bps.map(|bps| {
            bps.into_iter()
                .map(|stake| {
                    let ValidatorStakeView::V1(stake) = stake;
                    types::ValidatorStakeView::V1(types::ValidatorStakeViewV1 {
                        account_id: stake.account_id,
                        public_key: types::PublicKey::Ed25519(
                            stake.public_key.key_data().try_into().unwrap(),
                        ),
                        stake: stake.stake,
                    })
                })
                .collect()
        }),
        approvals_after_next: light_client_block
            .approvals_after_next
            .into_iter()
            .map(|sig| {
                sig.map(|s| match s.as_ref() {
                    near_crypto::Signature::ED25519(sig) => {
                        Box::new(types::Signature::Ed25519(sig.to_bytes().to_vec()))
                    }
                    near_crypto::Signature::SECP256K1(_) => {
                        Box::new(types::Signature::Secp256k1(Vec::new()))
                    }
                })
            })
            .collect(),
    }
}
