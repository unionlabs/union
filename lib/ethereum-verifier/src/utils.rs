use ibc_types::ethereum::{Domain, DomainType, ForkData, Version, H256};
use ibc_types::ethereum_consts_traits::{
    EPOCHS_PER_SYNC_COMMITTEE_PERIOD, SECONDS_PER_SLOT, SLOTS_PER_EPOCH,
};
use sha2::{Digest, Sha256};
use ssz::Encode;
use tree_hash::TreeHash;
use typenum::Unsigned;

use crate::primitives::{Epoch, Root, Slot, GENESIS_SLOT};
use crate::{Error, LightClientContext};

pub fn compute_fork_version<Ctx: LightClientContext>(ctx: &Ctx, epoch: Epoch) -> Version {
    let fork_parameters = ctx.fork_parameters();
    if epoch >= fork_parameters.eip4844.epoch {
        fork_parameters.eip4844.version.clone()
    } else if epoch >= fork_parameters.capella.epoch {
        fork_parameters.capella.version.clone()
    } else if epoch >= fork_parameters.bellatrix.epoch {
        fork_parameters.bellatrix.version.clone()
    } else if epoch >= fork_parameters.altair.epoch {
        fork_parameters.altair.version.clone()
    } else {
        fork_parameters.genesis_fork_version.clone()
    }
}

pub fn compute_sync_committee_period_at_slot<
    C: SLOTS_PER_EPOCH + EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
>(
    slot: Slot,
) -> u64 {
    compute_sync_committee_period::<C>(compute_epoch_at_slot::<C>(slot))
}

pub fn compute_epoch_at_slot<C: SLOTS_PER_EPOCH>(slot: Slot) -> Epoch {
    slot / C::SLOTS_PER_EPOCH::U64
}

pub fn compute_sync_committee_period<C: EPOCHS_PER_SYNC_COMMITTEE_PERIOD>(epoch: Epoch) -> Slot {
    epoch / C::EPOCHS_PER_SYNC_COMMITTEE_PERIOD::U64
}

pub fn compute_timestamp_at_slot<C: SECONDS_PER_SLOT>(genesis_time: u64, slot: Slot) -> u64 {
    // REVIEW: Should genesis slot be a config param or a constant?
    let slots_since_genesis = slot - GENESIS_SLOT;
    genesis_time + (slots_since_genesis * C::SECONDS_PER_SLOT::U64)
}

// REVIEW: should these fields be optional?
pub fn compute_domain(
    domain_type: DomainType,
    fork_version: Option<Version>,
    genesis_validators_root: Option<H256>,
    genesis_fork_version: Version,
) -> Domain {
    let fork_version = fork_version.unwrap_or(genesis_fork_version);
    let genesis_validators_root = genesis_validators_root.unwrap_or_default();
    let fork_data_root = compute_fork_data_root(fork_version, genesis_validators_root);

    let mut domain = Domain::default();
    domain.0[..4].copy_from_slice(&domain_type.0);
    domain.0[4..].copy_from_slice(&fork_data_root.0[..28]);

    domain
}

pub fn compute_fork_data_root(current_version: Version, genesis_validators_root: H256) -> H256 {
    ForkData {
        current_version,
        genesis_validators_root,
    }
    .tree_hash_root()
    .into()
}

#[derive(Debug, Encode, TreeHash)]
pub struct SigningData {
    pub object_root: Root,
    pub domain: Domain,
}

pub fn compute_signing_root<T: TreeHash>(ssz_object: &T, domain: Domain) -> primitive_types::H256 {
    SigningData {
        object_root: ssz_object.tree_hash_root().into(),
        domain,
    }
    .tree_hash_root()
}

// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#is_valid_merkle_branch
pub fn validate_merkle_branch<'a>(
    leaf: &H256,
    branch: impl IntoIterator<Item = &'a H256>,
    depth: usize,
    index: u64,
    root: &H256,
) -> Result<(), Error> {
    is_valid_merkle_branch(leaf, branch, depth, index, root)
        .then_some(())
        .ok_or(Error::InvalidMerkleBranch)
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#is_valid_merkle_branch
pub fn is_valid_merkle_branch<'a>(
    leaf: &H256,
    branch: impl IntoIterator<Item = &'a H256>,
    depth: usize,
    index: u64,
    root: &H256,
) -> bool {
    let mut value = leaf.clone();

    // TODO: This is just a fold
    // NB: zip ends when either iterator end
    for (b, i) in branch.into_iter().zip(0..depth) {
        if let Some(v) = 2u64.checked_pow(i as u32) {
            if index / v % 2 == 1 {
                value = hash([b.0, value.0].concat());
            } else {
                value = hash([value.0, b.0].concat());
            }
        } else {
            return false;
        }
    }

    &value == root
}

// NB: This is used because Sha256::digest returns a GenericArray
fn hash(bz: Vec<u8>) -> H256 {
    let mut output = H256::default();
    output.0.copy_from_slice(Sha256::digest(bz).as_slice());
    output
}
