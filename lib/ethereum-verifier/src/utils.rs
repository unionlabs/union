use sha2::{Digest, Sha256};
use tree_hash::TreeHash;
use typenum::Unsigned;
use unionlabs::{
    ethereum::{
        config::{EPOCHS_PER_SYNC_COMMITTEE_PERIOD, SECONDS_PER_SLOT, SLOTS_PER_EPOCH},
        Domain, DomainType, ForkData, SigningData, Version,
    },
    hash::H256,
    ibc::lightclients::ethereum::{
        beacon_block_header::BeaconBlockHeader, fork_parameters::ForkParameters,
    },
};

use crate::{primitives::GENESIS_SLOT, Error, InvalidMerkleBranch};

/// Returns the fork version based on the `epoch` and `fork_parameters`.
/// NOTE: This implementation is based on capella.
///
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/capella/fork.md#modified-compute_fork_version)
pub fn compute_fork_version(fork_parameters: &ForkParameters, epoch: u64) -> Version {
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
pub fn compute_signing_root<T: TreeHash>(ssz_object: &T, domain: Domain) -> primitive_types::H256 {
    SigningData {
        object_root: ssz_object.tree_hash_root().into(),
        domain,
    }
    .tree_hash_root()
}

/*
 leaf:
    H256(0x95a70d24f0a388301cf1659d9e59583b045d65fa1465c51314719609e115d40c),

branch: [
    H256(0x0400000000000000000000000000000000000000000000000000000000000000),
    H256(0x86220a2d72000ffb901cf75bf2918181ffea3c6567a573566d8c826e9e567493),
    H256(0x5bd50ecad03f16305866677a772260ea3275922a98d389a566be479549c6ada8),
    H256(0x3836bf436572e4493691f3f7c531a1a1fc7ad4a817c9dc6565b9c75cf7a09784),
    H256(0x56fb6ae1e15a48b0f9e534a60ffc8a176589ddf1063e63f112417831a05f25d8),
    H256(0x46b19a7a9d2c6c071d157555008d46336542841760103342cfb5df284319b1f1)],

depth: 6,

index: 39,

root: H256(0x6f6b3f64a5d4ffad6471af8042cae34a4b0c521a9c5d7a43932ceea648c8e5c2) }
*/

#[test]
fn hello() {
    let finalized_beacon = BeaconBlockHeader {
        slot: 16,
        proposer_index: 1,
        parent_root: H256(hex_literal::hex!(
            "0ca4610c4306e90bc21f42b10064e9fdeca5f7cbcf7a6fc0fb02ec26c1c09459"
        )),
        state_root: H256(hex_literal::hex!(
            "5f083d9e0c9199837b0343b55de0a3334f03bed6608173cf8f983f457b77077d"
        )),
        body_root: H256(hex_literal::hex!(
            "799fa16009926af2c7180bdfec121ca91df6626a349e1a478be116ce2cf45cfc"
        )),
    };

    let leaf = finalized_beacon.tree_hash_root();

    // let leaf = H256(hex_literal::hex!(
    //     "b483b79911a026cca1fcbe98ac6e4854391534e94950f0eac034bee1a6d2831c"
    // ));
    let branch = [
        &H256(hex_literal::hex!(
            "0200000000000000000000000000000000000000000000000000000000000000"
        )),
        &H256(hex_literal::hex!(
            "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
        )),
        &H256(hex_literal::hex!(
            "f469b98c708f31794d25050805cc0d836085a75378ccf77fd692c93e2d596247"
        )),
        &H256(hex_literal::hex!(
            "d393050d80ba6b8d66a7680f0a3f9e1fd29e21464645c6cc59ad23b57b400ee4"
        )),
        &H256(hex_literal::hex!(
            "0d67602c387b095c6d5e333459d1fd0dff74b7a02159dc630589a834f1b99a09"
        )),
        &H256(hex_literal::hex!(
            "b9e54fc7d4f4da163aceb5b40461c4946e1646e5560bfd07eed8af8a112ea55a"
        )),
    ];

    let depth = 6;
    let index = 39;

    let root = H256(hex_literal::hex!(
        "cbc3893f0c54cb36df81812125387cef6a6c387a7f912bfe02f2611977f51868"
    ));

    validate_merkle_branch(&leaf.into(), branch, depth, index, &root).unwrap();

    // for i in 1..100 {
    //     for j in 1..10 {
    //         if validate_merkle_branch(&leaf.into(), branch, j, i, &root).is_ok() {
    //             panic!("FOUND AT: {}", i)
    //         }
    //     }
    // }

    /*
    // (InvalidMerkleBranch { leaf:
    H256(0x73365fd766ee2242d0c545b5746c227892a3a02af4f27c7f82f4afe8daf3c22c),

    branch: [
    H256(0x5700000000000000000000000000000000000000000000000000000000000000),
    H256(0x86220a2d72000ffb901cf75bf2918181ffea3c6567a573566d8c826e9e567493),
    H256(0x81e0a0e65af09080b943564f20d0e412a3de3a5b15b470295045033cd8875d85),
    H256(0xb59a711c0b74397ff5a8e4260b73fc1490b859b9b99ffc5fd0c3dc7389129770),
    H256(0x8d94c4d59b64af48d88df52bd8fb0d1f4bfe69efa6a5d52b2a3729dda94fe1ee),
    H256(0x70130aa70d38170b19005f2e0fa4a59fe7408fa9805941e83e6583a0a8b6d2f6)],

    depth: 6, index: 39, root: H256(0xe97ebb82b9994112033b12f8e33586ba490104a337e2a6b8205f89b442a40529)
     */
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
        let mut value = leaf.clone();

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
        leaf: leaf.clone(),
        branch,
        depth,
        index,
        root: root.clone(),
    }))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::redundant_clone)]

    use unionlabs::ethereum::config::{Minimal, SECONDS_PER_SLOT, SEPOLIA};

    use super::*;

    pub const SAMPLE_SLOT: u64 = 1235232;
    pub const SAMPLE_EPOCH: u64 = 10000;

    #[test]
    fn compute_fork_version_works() {
        let fork_parameters = SEPOLIA.fork_parameters;
        assert_eq!(
            compute_fork_version(&fork_parameters, fork_parameters.eip4844.epoch),
            fork_parameters.eip4844.version
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
        let fork_data = ForkData {
            current_version: Version(Default::default()),
            genesis_validators_root: Default::default(),
        };

        let domain = Domain([3; 32]);

        let signing_data = SigningData {
            object_root: fork_data.tree_hash_root().into(),
            domain: domain.clone(),
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
