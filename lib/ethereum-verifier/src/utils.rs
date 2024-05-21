use sha2::{Digest, Sha256};
use ssz::{types::BitVector, Ssz};
use typenum::Unsigned;
use unionlabs::{
    ethereum::{
        config::{
            EPOCHS_PER_SYNC_COMMITTEE_PERIOD, SECONDS_PER_SLOT, SLOTS_PER_EPOCH,
            SYNC_COMMITTEE_SIZE,
        },
        Domain, DomainType, ForkData, SigningData, Version,
    },
    hash::H256,
    ibc::lightclients::ethereum::fork_parameters::ForkParameters,
};

use crate::{
    error::{Error, InvalidMerkleBranch},
    primitives::GENESIS_SLOT,
};

/// Returns the fork version based on the `epoch` and `fork_parameters`.
/// NOTE: This implementation is based on capella.
///
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/capella/fork.md#modified-compute_fork_version)
pub fn compute_fork_version(fork_parameters: &ForkParameters, epoch: u64) -> Version {
    if epoch >= fork_parameters.deneb.epoch {
        fork_parameters.deneb.version
    } else if epoch >= fork_parameters.capella.epoch {
        fork_parameters.capella.version
    } else if epoch >= fork_parameters.bellatrix.epoch {
        fork_parameters.bellatrix.version
    } else if epoch >= fork_parameters.altair.epoch {
        fork_parameters.altair.version
    } else {
        fork_parameters.genesis_fork_version
    }
}

/// Returns the sync committee period at a given `slot`.
///
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#compute_sync_committee_period_at_slot)
pub fn compute_sync_committee_period_at_slot<
    C: SLOTS_PER_EPOCH + EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
>(
    slot: u64,
) -> u64 {
    compute_sync_committee_period::<C>(compute_epoch_at_slot::<C>(slot))
}

/// Returns the epoch at a given `slot`.
///
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#compute_epoch_at_slot)
pub fn compute_epoch_at_slot<C: SLOTS_PER_EPOCH>(slot: u64) -> u64 {
    slot / C::SLOTS_PER_EPOCH::U64
}

/// Returns the sync committee period at a given `epoch`.
///
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/validator.md#sync-committee)
pub fn compute_sync_committee_period<C: EPOCHS_PER_SYNC_COMMITTEE_PERIOD>(epoch: u64) -> u64 {
    epoch / C::EPOCHS_PER_SYNC_COMMITTEE_PERIOD::U64
}

/// Returns the timestamp at a `slot`, respect to `genesis_time`.
///
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#compute_timestamp_at_slot)
pub fn compute_timestamp_at_slot<C: SECONDS_PER_SLOT>(genesis_time: u64, slot: u64) -> u64 {
    // REVIEW: Should genesis slot be a config param or a constant?
    let slots_since_genesis = slot - GENESIS_SLOT;
    genesis_time + (slots_since_genesis * C::SECONDS_PER_SLOT::U64)
}

/// Return the domain for the `domain_type` and `fork_version`.
///
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#compute_domain)
pub fn compute_domain(
    domain_type: DomainType,
    fork_version: Option<Version>,
    genesis_validators_root: Option<H256>,
    genesis_fork_version: Version,
) -> Domain {
    let fork_version = fork_version.unwrap_or(genesis_fork_version);
    let genesis_validators_root = genesis_validators_root.unwrap_or_default();
    let fork_data_root = compute_fork_data_root(fork_version, genesis_validators_root);

    let mut domain = [0; 32];
    domain[..4].copy_from_slice(&domain_type.0);
    domain[4..].copy_from_slice(&fork_data_root.0[..28]);

    Domain(domain)
}

/// Return the 32-byte fork data root for the `current_version` and `genesis_validators_root`.
/// This is used primarily in signature domains to avoid collisions across forks/chains.
///
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#compute_fork_data_root)
pub fn compute_fork_data_root(current_version: Version, genesis_validators_root: H256) -> H256 {
    ForkData {
        current_version,
        genesis_validators_root,
    }
    .tree_hash_root()
    .into()
}

/// Return the signing root for the corresponding signing data
///
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#compute_signing_root)
pub fn compute_signing_root<T: Ssz>(ssz_object: &T, domain: Domain) -> ssz::tree_hash::Hash256 {
    SigningData {
        object_root: ssz_object.tree_hash_root().into(),
        domain,
    }
    .tree_hash_root()
}

/// Return the slot at timestamp with respect to the genesis time
pub fn compute_slot_at_timestamp<C: SECONDS_PER_SLOT>(
    genesis_time: u64,
    timestamp_seconds: u64,
) -> Option<u64> {
    timestamp_seconds
        .checked_sub(genesis_time)?
        .checked_div(C::SECONDS_PER_SLOT::U64)?
        .checked_add(GENESIS_SLOT)
}

// Returns if at least 2/3 of the sync committee signed
//
// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#process_light_client_update
pub fn validate_signature_supermajority<C: SYNC_COMMITTEE_SIZE>(
    sync_committee_bits: &BitVector<C::SYNC_COMMITTEE_SIZE>,
) -> bool {
    sync_committee_bits.num_set_bits() * 3 >= sync_committee_bits.len() * 2
}

/// Check if `leaf` at `index` verifies against the Merkle `root` and `branch`.
///
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#is_valid_merkle_branch)
pub fn validate_merkle_branch<'a>(
    leaf: &H256,
    branch: impl IntoIterator<Item = &'a H256>,
    depth: usize,
    index: u64,
    root: &H256,
) -> Result<(), Error> {
    let branch = branch.into_iter().cloned().collect::<Vec<_>>();

    'block: {
        let mut value = *leaf;

        // TODO: This is just a fold
        // NB: zip ends when either iterator ends
        for (b, i) in branch.iter().zip(0..depth) {
            if let Some(v) = 2u64.checked_pow(i as u32) {
                let val = index / v % 2;
                value = Sha256::digest(
                    if val == 1 {
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
        leaf: *leaf,
        branch,
        depth,
        index,
        root: *root,
    }))
}

#[cfg(test)]
#[allow(clippy::redundant_clone)]
mod tests {

    use unionlabs::ethereum::config::{Minimal, SEPOLIA};

    use super::*;

    pub const SAMPLE_SLOT: u64 = 1235232;
    pub const SAMPLE_EPOCH: u64 = 10000;

    #[test]
    fn compute_fork_version_works() {
        let fork_parameters = SEPOLIA.fork_parameters;
        assert_eq!(
            compute_fork_version(&fork_parameters, fork_parameters.deneb.epoch),
            fork_parameters.deneb.version
        );
        assert_eq!(
            compute_fork_version(&fork_parameters, fork_parameters.capella.epoch),
            fork_parameters.capella.version
        );
        assert_eq!(
            compute_fork_version(&fork_parameters, fork_parameters.bellatrix.epoch),
            fork_parameters.bellatrix.version
        );
        assert_eq!(
            compute_fork_version(&fork_parameters, fork_parameters.altair.epoch),
            fork_parameters.altair.version
        );
        assert_eq!(
            compute_fork_version(&fork_parameters, 0),
            fork_parameters.genesis_fork_version
        );
    }

    #[test]
    fn compute_sync_committee_period_at_slot_works() {
        assert_eq!(
            compute_sync_committee_period_at_slot::<Minimal>(SAMPLE_SLOT),
            SAMPLE_SLOT / <Minimal as SLOTS_PER_EPOCH>::SLOTS_PER_EPOCH::U64
                / <Minimal as EPOCHS_PER_SYNC_COMMITTEE_PERIOD>::EPOCHS_PER_SYNC_COMMITTEE_PERIOD::U64
        )
    }

    #[test]
    fn compute_epoch_at_slot_works() {
        assert_eq!(
            compute_epoch_at_slot::<Minimal>(SAMPLE_SLOT),
            SAMPLE_SLOT / <Minimal as SLOTS_PER_EPOCH>::SLOTS_PER_EPOCH::U64
        );
    }

    #[test]
    fn compute_sync_committee_period_works() {
        assert_eq!(
            compute_sync_committee_period::<Minimal>(SAMPLE_EPOCH),
            SAMPLE_EPOCH / <Minimal as EPOCHS_PER_SYNC_COMMITTEE_PERIOD>::EPOCHS_PER_SYNC_COMMITTEE_PERIOD::U64,
        );
    }

    #[test]
    fn compute_timestamp_at_slot_works() {
        assert_eq!(
            compute_timestamp_at_slot::<Minimal>(0, 150),
            150 * <Minimal as SECONDS_PER_SLOT>::SECONDS_PER_SLOT::U64
        );
    }

    #[test]
    fn compute_domain_works() {
        let domain_type = DomainType([1, 2, 3, 4]);
        let current_version = Version([5, 6, 7, 8]);
        let genesis_validators_root = H256([1; 32]);
        let fork_data_root = compute_fork_data_root(current_version, genesis_validators_root);
        let genesis_version = Version([0, 0, 0, 0]);

        let mut domain = Domain::default();
        domain.0[..4].copy_from_slice(&domain_type.0);
        domain.0[4..].copy_from_slice(&fork_data_root.0[..28]);

        // Uses the values instead of the default ones when `current_version` and
        // `genesis_validators_root` is provided.
        assert_eq!(
            domain,
            compute_domain(
                domain_type,
                Some(current_version),
                Some(genesis_validators_root),
                genesis_version,
            )
        );

        let fork_data_root = compute_fork_data_root(genesis_version, Default::default());
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
        let fork_data = ForkData {
            current_version: Version(Default::default()),
            genesis_validators_root: Default::default(),
        };

        let domain = Domain([3; 32]);

        let signing_data = SigningData {
            object_root: fork_data.tree_hash_root().into(),
            domain,
        };

        assert_eq!(
            signing_data.tree_hash_root(),
            compute_signing_root(&fork_data, domain)
        )
    }

    // #[test]
    // fn valid_merkle_branch_works() {
    //     // TODO(aeryz): move test data to ibc types
    //     let header = <Header<Minimal>>::try_from_proto(
    //         serde_json::from_str(include_str!(
    //             "../../../light-clients/ethereum-light-client/src/test/finality_update_1.json"
    //         ))
    //         .unwrap(),
    //     )
    //     .unwrap();

    //     let header = header.consensus_update.attested_header;

    //     let valid_leaf = H256::from(header.execution.tree_hash_root());
    //     let valid_branch = header.execution_branch.clone();
    //     let valid_root = header.beacon.body_root.clone();

    //     // Works for valid data
    //     assert_eq!(
    //         validate_merkle_branch(
    //             &valid_leaf,
    //             &valid_branch,
    //             floorlog2(EXECUTION_PAYLOAD_INDEX),
    //             EXECUTION_PAYLOAD_INDEX,
    //             &valid_root,
    //         ),
    //         Ok(())
    //     );

    //     // Fails when index is wrong
    //     assert!(validate_merkle_branch(
    //         &valid_leaf,
    //         &valid_branch,
    //         floorlog2(EXECUTION_PAYLOAD_INDEX),
    //         EXECUTION_PAYLOAD_INDEX + 1,
    //         &valid_root,
    //     )
    //     .is_err());

    //     let invalid_leaf = {
    //         let mut header = header.clone();
    //         header.execution.gas_limit += 1;
    //         H256::from(header.execution.tree_hash_root())
    //     };

    //     // Fails when root is wrong
    //     assert!(validate_merkle_branch(
    //         &invalid_leaf,
    //         &valid_branch,
    //         floorlog2(EXECUTION_PAYLOAD_INDEX),
    //         EXECUTION_PAYLOAD_INDEX,
    //         &valid_root,
    //     )
    //     .is_err());

    //     let invalid_branch = {
    //         let mut header = header.clone();
    //         header.execution_branch[0] = Default::default();
    //         header.execution_branch
    //     };

    //     // Fails when branch is wrong
    //     assert!(validate_merkle_branch(
    //         &valid_leaf,
    //         &invalid_branch,
    //         floorlog2(EXECUTION_PAYLOAD_INDEX),
    //         EXECUTION_PAYLOAD_INDEX,
    //         &valid_root,
    //     )
    //     .is_err());
    // }
}
