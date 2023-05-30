use crate::primitives::{Bytes32, Domain, DomainType, Epoch, Root, Slot, Version, GENESIS_SLOT};
use crate::{capella::ForkData, ChainConfig, Error, LightClientContext};
use ssz_rs::prelude::*;

// pub use ethereum_consensus::{altair::compute_domain, signing::compute_signing_root};

pub fn compute_fork_version<C: LightClientContext>(ctx: &C, epoch: Epoch) -> Version {
    let fork_parameters = ctx.fork_parameters();
    if epoch >= fork_parameters.eip4844_fork_epoch {
        fork_parameters.eip4844_fork_version
    } else if epoch >= fork_parameters.capella_fork_epoch {
        fork_parameters.capella_fork_version
    } else if epoch >= fork_parameters.bellatrix_fork_epoch {
        fork_parameters.bellatrix_fork_version
    } else if epoch >= fork_parameters.altair_fork_epoch {
        fork_parameters.altair_fork_version
    } else {
        fork_parameters.genesis_fork_version
    }
}

pub fn compute_sync_committee_period_at_slot<C: LightClientContext>(slot: Slot) -> u64 {
    compute_sync_committee_period::<C>(compute_epoch_at_slot::<C>(slot))
}

pub fn compute_epoch_at_slot<C: LightClientContext>(slot: Slot) -> Epoch {
    slot / C::Config::SLOTS_PER_EPOCH
}

pub fn compute_sync_committee_period<C: LightClientContext>(epoch: Epoch) -> Slot {
    epoch / C::Config::EPOCHS_PER_SYNC_COMMITTEE_PERIOD
}

pub fn compute_timestamp_at_slot<C: LightClientContext>(genesis_time: u64, slot: Slot) -> u64 {
    let slots_since_genesis = slot - GENESIS_SLOT;
    genesis_time + slots_since_genesis * C::Config::SECONDS_PER_SLOT
}

pub fn compute_domain(
    domain_type: DomainType,
    fork_version: Option<Version>,
    genesis_validators_root: Option<Root>,
    genesis_fork_version: Version,
) -> Result<Domain, Error> {
    let fork_version = fork_version.unwrap_or(genesis_fork_version);
    let genesis_validators_root = genesis_validators_root.unwrap_or_default();
    let fork_data_root = compute_fork_data_root(fork_version, genesis_validators_root)?;

    let mut domain = Domain::default();
    domain[..4].copy_from_slice(&domain_type.as_bytes());
    domain[4..].copy_from_slice(&fork_data_root.as_ref()[..28]);
    Ok(domain)
}

pub fn compute_fork_data_root(
    current_version: Version,
    genesis_validators_root: Root,
) -> Result<Root, Error> {
    ForkData {
        current_version,
        genesis_validators_root,
    }
    .hash_tree_root()
    .map_err(Error::Merkleization)
}

#[derive(Default, Debug, SimpleSerialize)]
pub struct SigningData {
    pub object_root: Root,
    pub domain: Domain,
}

pub fn compute_signing_root<T: SimpleSerialize>(
    ssz_object: &mut T,
    domain: Domain,
) -> Result<Root, Error> {
    let object_root = ssz_object.hash_tree_root()?;

    let mut s = SigningData {
        object_root,
        domain,
    };
    s.hash_tree_root().map_err(Error::Merkleization)
}

pub fn hash_tree_root<T: SimpleSerialize>(mut object: T) -> Result<Root, Error> {
    object.hash_tree_root().map_err(Into::into)
}

// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#is_valid_merkle_branch
pub fn validate_merkle_branch(
    leaf: &Node,
    branch: &[Bytes32],
    depth: usize,
    index: usize,
    root: &Root,
) -> Result<(), Error> {
    let branch: Vec<_> = branch
        .iter()
        .map(|node| Node::try_from(node.as_ref()).expect("is valid instance"))
        .collect();

    if is_valid_merkle_branch(leaf, branch.iter(), depth, index, root) {
        Ok(())
    } else {
        Err(Error::InvalidMerkleBranch)
    }
}
