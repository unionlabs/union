use ibc_types::{
    ethereum::{Domain, DomainType, ForkData, Version, H256},
    ethereum_consts_traits::{EPOCHS_PER_SYNC_COMMITTEE_PERIOD, SECONDS_PER_SLOT, SLOTS_PER_EPOCH},
};
use sha2::{Digest, Sha256};
use ssz::Encode;
use tree_hash::TreeHash;
use typenum::Unsigned;

use crate::{
    primitives::{Epoch, Root, Slot, GENESIS_SLOT},
    Error, InvalidMerkleBranch, LightClientContext,
};

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
    let branch = branch.into_iter().cloned().collect::<Vec<_>>();

    'block: {
        let mut value = leaf.clone();

        // TODO: This is just a fold
        // NB: zip ends when either iterator ends
        for (b, i) in branch.iter().zip(0..depth) {
            if let Some(v) = 2u64.checked_pow(i as u32) {
                value = Sha256::digest(
                    if index / v % 2 == 1 {
                        [b.0, value.0]
                    } else {
                        [value.0, b.0]
                    }
                    .concat(),
                )
                .into();
            } else {
                break 'block false;
            }
        }

        value == *root
    }
    .then_some(())
    .ok_or(Error::InvalidMerkleBranch(InvalidMerkleBranch {
        leaf: leaf.clone(),
        branch,
        depth,
        index,
        root: root.clone(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ibc_types::{
        ethereum_consts_traits::{
            consts::{floorlog2, EXECUTION_PAYLOAD_INDEX},
            Minimal,
        },
        ibc::lightclients::ethereum::header::Header,
        TryFromProto,
    };

    #[test]
    fn compute_domain_works() {
        let domain_type = DomainType([1, 2, 3, 4]);
        let current_version = Version([5, 6, 7, 8]);
        let genesis_validators_root = H256([1; 32]);
        let fork_data_root =
            compute_fork_data_root(current_version.clone(), genesis_validators_root.clone());
        let genesis_version = Version([0, 0, 0, 0]);

        let mut domain = Domain::default();
        domain.0[..4].copy_from_slice(&domain_type.0);
        domain.0[4..].copy_from_slice(&fork_data_root.0[..28]);

        // Uses the values instead of the default ones when `current_version` and
        // `genesis_validators_root` is provided.
        assert_eq!(
            domain,
            compute_domain(
                domain_type.clone(),
                Some(current_version),
                Some(genesis_validators_root),
                genesis_version.clone(),
            )
        );

        let fork_data_root = compute_fork_data_root(genesis_version.clone(), Default::default());
        let mut domain = Domain::default();
        domain.0[..4].copy_from_slice(&domain_type.0);
        domain.0[4..].copy_from_slice(&fork_data_root.0[..28]);

        // Uses default values when version and validators root is None
        assert_eq!(
            domain,
            compute_domain(domain_type, None, None, genesis_version)
        );
    }

    #[test]
    fn compute_fork_data_root_works() {
        let fork_data_root: H256 = ForkData {
            current_version: Version(Default::default()),
            genesis_validators_root: Default::default(),
        }
        .tree_hash_root()
        .into();

        assert_eq!(
            fork_data_root,
            compute_fork_data_root(Version(Default::default()), Default::default())
        )
    }

    #[test]
    fn compute_signing_root_works() {
        // TODO(aeryz): implement using a dummy bls verifier
    }

    #[test]
    fn valid_merkle_branch_works() {
        // TODO(aeryz): move test data to ibc types
        let header = <Header<Minimal>>::try_from_proto(
            serde_json::from_str(include_str!(
                "../../../light-clients/ethereum-light-client/src/test/finality_update_1.json"
            ))
            .unwrap(),
        )
        .unwrap();

        let header = header.consensus_update.attested_header;

        let valid_leaf = H256::from(header.execution.tree_hash_root());
        let valid_branch = header.execution_branch.clone();
        let valid_root = header.beacon.body_root.clone();

        // Works for valid data
        assert_eq!(
            validate_merkle_branch(
                &valid_leaf,
                &valid_branch,
                floorlog2(EXECUTION_PAYLOAD_INDEX),
                EXECUTION_PAYLOAD_INDEX,
                &valid_root,
            ),
            Ok(())
        );

        // Fails when index is wrong
        assert!(validate_merkle_branch(
            &valid_leaf,
            &valid_branch,
            floorlog2(EXECUTION_PAYLOAD_INDEX),
            EXECUTION_PAYLOAD_INDEX + 1,
            &valid_root,
        )
        .is_err());

        let invalid_leaf = {
            let mut header = header.clone();
            header.execution.gas_limit += 1;
            H256::from(header.execution.tree_hash_root())
        };

        // Fails when root is wrong
        assert!(validate_merkle_branch(
            &invalid_leaf,
            &valid_branch,
            floorlog2(EXECUTION_PAYLOAD_INDEX),
            EXECUTION_PAYLOAD_INDEX,
            &valid_root,
        )
        .is_err());

        let invalid_branch = {
            let mut header = header.clone();
            header.execution_branch[0] = Default::default();
            header.execution_branch
        };

        // Fails when branch is wrong
        assert!(validate_merkle_branch(
            &valid_leaf,
            &invalid_branch,
            floorlog2(EXECUTION_PAYLOAD_INDEX),
            EXECUTION_PAYLOAD_INDEX,
            &valid_root,
        )
        .is_err());
    }
}
