use error::Error;
use near_primitives_core::{
    borsh::{self, BorshSerialize},
    hash::CryptoHash,
    types::MerkleHash,
};
use near_sdk::AccountId;
use sha2::{Digest, Sha256};
use state_proof::StateProof;
use unionlabs::{
    ibc::lightclients::near::{
        approval::ApprovalInner,
        block_header_inner::{BlockHeaderInnerLite, BlockHeaderInnerLiteView},
        light_client_block::LightClientBlockView,
        validator_stake_view::ValidatorStakeView,
    },
    near::{
        raw_state_proof::RawStateProof,
        types::{Direction, MerklePath, PublicKey, Signature},
    },
};

pub mod error;
mod nibble_slice;
pub mod state_proof;

pub trait NearVerifierCtx {
    fn get_epoch_block_producers(&self, epoch_id: CryptoHash) -> Option<Vec<ValidatorStakeView>>;

    fn ed25519_verify(
        &self,
        public_key: &[u8],
        signature: &[u8],
        message: &[u8],
    ) -> Result<(), Error>;
}

pub fn verify_header<Ctx: NearVerifierCtx>(
    ctx: &Ctx,
    head: BlockHeaderInnerLiteView,
    block_view: LightClientBlockView,
) -> Result<(), Error> {
    let (_current_block_hash, _next_block_hash, approval_message) =
        reconstruct_light_client_block_view_fields(block_view.clone());

    if block_view.inner_lite.height <= head.height {
        return Err(Error::UpdateHeightMustBeGreater(
            block_view.inner_lite.height,
            head.height,
        ));
    }

    if ![&head.epoch_id, &head.next_epoch_id].contains(&&block_view.inner_lite.epoch_id) {
        return Err(Error::InvalidEpochId(block_view.inner_lite.epoch_id));
    }

    if block_view.inner_lite.epoch_id == head.next_epoch_id && block_view.next_bps.is_none() {
        return Err(Error::MustHaveNextEpochId);
    }

    let mut total_stake = 0;
    let mut approved_stake = 0;

    let epoch_block_producers = ctx
        .get_epoch_block_producers(block_view.inner_lite.epoch_id)
        .ok_or(Error::EpochBlockProducersNotFound(
            block_view.inner_lite.epoch_id,
        ))?;

    for (maybe_signature, block_producer) in block_view
        .approvals_after_next
        .iter()
        .zip(epoch_block_producers.iter())
    {
        let ValidatorStakeView::V1(block_producer) = block_producer.clone();
        total_stake += block_producer.stake;

        if maybe_signature.is_none() {
            continue;
        }

        match maybe_signature {
            Some(signature) => {
                approved_stake += block_producer.stake;

                let PublicKey::Ed25519(pubkey) = block_producer.public_key else {
                    return Err(Error::UnsupportedPublicKey);
                };

                let Signature::Ed25519(sig) = signature.as_ref() else {
                    return Err(Error::UnsupportedSignature);
                };

                ctx.ed25519_verify(&pubkey[..], &sig, &approval_message)?;
            }
            None => continue,
        }
    }

    let threshold = total_stake.checked_mul(2).unwrap().checked_div(3).unwrap();
    if approved_stake <= threshold {
        return Err(Error::ApprovedStakeBelowThreshold(
            approved_stake,
            threshold,
        ));
    }

    if let Some(next_bps) = &block_view.next_bps {
        let next_bp_hash = hash_borsh(next_bps);
        if next_bp_hash != block_view.inner_lite.next_bp_hash {
            return Err(Error::NextBpsHashMismatch(
                next_bp_hash,
                block_view.inner_lite.next_bp_hash,
            ));
        }
    }

    Ok(())
}

pub fn verify_state(
    raw_state_proof: RawStateProof,
    state_root: &CryptoHash,
    account_id: &AccountId,
    key: &[u8],
    value: Option<&[u8]>,
) -> Result<(), Error> {
    let state_proof = StateProof::parse(raw_state_proof);

    if !state_proof.verify(state_root, account_id, key, value) {
        return Err(Error::StateVerificationFailure);
    }

    Ok(())
}

fn reconstruct_light_client_block_view_fields(
    block_view: LightClientBlockView,
) -> (CryptoHash, CryptoHash, Vec<u8>) {
    let current_block_hash = combine_hash(
        &combine_hash(
            &CryptoHash(
                Sha256::new()
                    .chain_update(
                        &borsh::to_vec(&Into::<BlockHeaderInnerLite>::into(
                            block_view.inner_lite.clone(),
                        ))
                        .unwrap(),
                    )
                    .finalize()
                    .try_into()
                    .unwrap(),
            ),
            &block_view.inner_rest_hash,
        ),
        &block_view.prev_block_hash,
    );

    let next_block_hash = combine_hash(&block_view.next_block_inner_hash, &current_block_hash);

    let mut approval_message = borsh::to_vec(&ApprovalInner::Endorsement(next_block_hash)).unwrap();
    approval_message.extend_from_slice(&(block_view.inner_lite.height + 2).to_le_bytes());

    (current_block_hash, next_block_hash, approval_message)
}

pub fn combine_hash(hash1: &MerkleHash, hash2: &MerkleHash) -> MerkleHash {
    hash_borsh((hash1, hash2))
}

pub fn hash_borsh<T: BorshSerialize>(value: T) -> CryptoHash {
    CryptoHash(
        Sha256::new()
            .chain_update(&borsh::to_vec(&value).expect("serialize will work"))
            .finalize()
            .try_into()
            .expect("sha256 output size is always 32 bytes"),
    )
}

/// Verify merkle path for given item and corresponding path.
pub fn verify_path<T: BorshSerialize>(
    root: MerkleHash,
    path: &MerklePath,
    item: T,
) -> Result<(), Error> {
    if !verify_hash(root, path, CryptoHash::hash_borsh(item)) {
        return Err(Error::MerkleVerificationFailure);
    }

    Ok(())
}

pub fn verify_hash(root: MerkleHash, path: &MerklePath, item_hash: MerkleHash) -> bool {
    compute_root_from_path(path, item_hash) == root
}

pub fn compute_root_from_path(path: &MerklePath, item_hash: MerkleHash) -> MerkleHash {
    let mut res = item_hash;
    for item in path {
        match item.direction {
            Direction::Left => {
                res = combine_hash(&item.hash, &res);
            }
            Direction::Right => {
                res = combine_hash(&res, &item.hash);
            }
        }
    }
    res
}
