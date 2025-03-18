#![feature(let_chains)]
extern crate alloc;

pub mod error;
// REVIEW: Unused?
pub mod utils;

use beacon_api_types::{
    altair::{self, SyncAggregateSsz, SyncCommittee},
    chain_spec::ChainSpec,
    consts::{
        CURRENT_SYNC_COMMITTEE_GINDEX, CURRENT_SYNC_COMMITTEE_GINDEX_ELECTRA,
        EXECUTION_PAYLOAD_GINDEX, FINALIZED_ROOT_GINDEX, FINALIZED_ROOT_GINDEX_ELECTRA,
        NEXT_SYNC_COMMITTEE_GINDEX, NEXT_SYNC_COMMITTEE_GINDEX_ELECTRA,
    },
    custom_types::{DomainType, Slot},
    deneb,
};
use ethereum_sync_protocol_types::LightClientHeader;
use fork_schedules::{ForkSchedule, Forks};
use ssz::Ssz;
use typenum::Unsigned;
use unionlabs::{
    ensure,
    primitives::{H256, H384, H768},
};

use crate::{
    error::Error,
    utils::{
        compute_domain, compute_epoch_at_slot, compute_fork_version, compute_signing_root,
        compute_sync_committee_period_at_slot, validate_merkle_branch,
    },
};

pub const GENESIS_SLOT: Slot = Slot::new(0);
pub const DST_POP_G2: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";

pub trait BlsVerify {
    /// Whether to provide non-signers or signers to [`aggregate_verify_signature`]
    const INVERSE: bool;

    /// Aggregate the public keys and verify the signature
    fn aggregate_verify_signature<'pk>(
        &self,
        aggregate_public_key: &'pk H384,
        public_keys: impl IntoIterator<Item = &'pk H384>,
        msg: Vec<u8>,
        signature: H768,
    ) -> Result<(), Error>;
}

/// Verifies if the light client `update` is valid.
///
/// * `update`: The light client update we want to verify.
/// * `current_slot`: The slot number computed based on the current timestamp.
/// * `genesis_validators_root`: The latest `genesis_validators_root` that is saved by the light client.
/// * `bls_verifier`: BLS verification implementation.
///
/// ## Important Notes
/// * This verification does not assume that the updated header is greater (in terms of height) than the
///   light client state. When the updated header is in the next signature period, the light client uses
///   the next sync committee to verify the signature, then it saves the next sync committee as the current
///   sync committee. However, it's not mandatory for light clients to expect the next sync committee to be given
///   during these updates. So if it's not given, the light client still can validate updates until the next signature
///   period arrives. In a situation like this, the update can be any header within the same signature period. And
///   this function only allows a non-existent next sync committee to be set in that case. It doesn't allow a sync committee
///   to be changed or removed.
///
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#validate_light_client_update)
#[allow(clippy::too_many_arguments)]
pub fn validate_light_client_update<C: ChainSpec, V: BlsVerify>(
    chain_id: u64,
    update: &ethereum_sync_protocol_types::LightClientUpdate,
    current_sync_committee: Option<&SyncCommittee>,
    next_sync_committee: Option<&SyncCommittee>,
    current_slot: Slot,
    finalized_slot: Slot,
    genesis_validators_root: H256,
    bls_verifier: V,
) -> Result<(), Error> {
    // verify that the sync committee has sufficient participants
    let sync_aggregate: SyncAggregateSsz<C> = update.sync_aggregate.clone().try_into()?;

    let set_bits = sync_aggregate.sync_committee_bits.num_set_bits();
    ensure(
        set_bits >= C::MIN_SYNC_COMMITTEE_PARTICIPANTS::USIZE,
        Error::InsufficientSyncCommitteeParticipants(set_bits),
    )?;

    is_valid_light_client_header::<C>(chain_id, &update.attested_header)?;

    // verify that the update does not skip a sync committee period
    let update_attested_slot = update.attested_header.beacon.slot;
    let update_finalized_slot = update.finalized_header.beacon.slot;

    ensure(
        update_finalized_slot != GENESIS_SLOT,
        Error::FinalizedSlotIsGenesis,
    )?;

    ensure(
        current_slot >= update.signature_slot,
        Error::UpdateMoreRecentThanCurrentSlot {
            current_slot,
            update_signature_slot: update.signature_slot,
        },
    )?;

    ensure(
        update.signature_slot > update_attested_slot
            && update_attested_slot >= update_finalized_slot,
        Error::InvalidSlots {
            update_signature_slot: update.signature_slot,
            update_attested_slot,
            update_finalized_slot,
        },
    )?;

    // Let's say N is the signature period of the header we store, we can only do updates with
    // the following settings:
    // 1. stored_period = N, signature_period = N:
    //     - the light client must have the `current_sync_committee` and use it to verify the new header.
    // 2. stored_period = N, signature_period = N + 1:
    //     - the light client must have the `next_sync_committee` and use it to verify the new header.
    let stored_period = compute_sync_committee_period_at_slot::<C>(finalized_slot);
    let signature_period = compute_sync_committee_period_at_slot::<C>(update.signature_slot);

    if next_sync_committee.is_some() {
        ensure(
            signature_period == stored_period || signature_period == stored_period + 1,
            Error::InvalidSignaturePeriodWhenNextSyncCommitteeExists {
                signature_period,
                stored_period,
            },
        )?;
    } else {
        ensure(
            signature_period == stored_period,
            Error::InvalidSignaturePeriodWhenNextSyncCommitteeDoesNotExist {
                signature_period,
                stored_period,
            },
        )?;
    }

    // verify that the update is relevant
    let update_attested_period = compute_sync_committee_period_at_slot::<C>(update_attested_slot);

    // There are two options to do a light client update:
    // 1. We are updating the header with a newer one.
    // 2. We haven't set the next sync committee yet and we can use any attested header within the same
    // signature period to set the next sync committee. This means that the stored header could be larger.
    // The light client implementation needs to take care of it.
    ensure(
        update_attested_slot > finalized_slot
            || (update_attested_period == stored_period
                && update.next_sync_committee.is_some()
                && next_sync_committee.is_none()),
        Error::IrrelevantUpdate {
            update_attested_slot,
            trusted_finalized_slot: finalized_slot,
            update_attested_period,
            stored_period,
            update_sync_committee_is_set: update.next_sync_committee.is_some(),
            trusted_next_sync_committee_is_set: next_sync_committee.is_some(),
        },
    )?;

    // Verify that the `finality_branch`, if present, confirms `finalized_header`
    // to match the finalized checkpoint root saved in the state of `attested_header`.
    // NOTE(aeryz): We always expect to get `finalized_header` and it's embedded into the type definition.
    is_valid_light_client_header::<C>(chain_id, &update.finalized_header)?;

    // This confirms that the `finalized_header` is really finalized.
    validate_merkle_branch(
        &update.finalized_header.beacon.tree_hash_root(),
        &update.finality_branch,
        finalized_root_gindex_at_slot::<C>(chain_id, update_attested_slot),
        &update.attested_header.beacon.state_root,
    )?;

    // Verify that if the update contains the next sync committee, and the signature periods do match,
    // next sync committees match too.
    if let (Some(next_sync_committee), Some(stored_next_sync_committee)) =
        (&update.next_sync_committee, &next_sync_committee)
    {
        if update_attested_period == stored_period {
            ensure(
                &next_sync_committee == stored_next_sync_committee,
                Error::NextSyncCommitteeMismatch {
                    expected: stored_next_sync_committee.aggregate_pubkey,
                    found: next_sync_committee.aggregate_pubkey,
                },
            )?;
        }
        // This validates the given next sync committee against the attested header's state root.
        validate_merkle_branch(
            &TryInto::<altair::SyncCommitteeSsz<C>>::try_into(next_sync_committee.clone())
                .unwrap()
                .tree_hash_root(),
            update
                .next_sync_committee_branch
                .clone()
                .unwrap_or_default()
                .iter(),
            next_sync_committee_gindex_at_slot::<C>(chain_id, update_attested_slot),
            &update.attested_header.beacon.state_root,
        )?;
    }

    // Verify sync committee aggregate signature
    let sync_committee = if signature_period == stored_period {
        current_sync_committee.ok_or(Error::ExpectedCurrentSyncCommittee)?
    } else {
        next_sync_committee.ok_or(Error::ExpectedNextSyncCommittee)?
    };

    verify_signature::<C, V>(
        chain_id,
        update,
        sync_aggregate,
        genesis_validators_root,
        sync_committee,
        bls_verifier,
    )?;

    // It's not mandatory for all of the members of the sync committee to participate. So we are extracting the
    // public keys of the ones who did not participate.
    Ok(())
}

pub fn verify_signature<C: ChainSpec, V: BlsVerify>(
    chain_id: u64,
    update: &ethereum_sync_protocol_types::LightClientUpdate,
    sync_aggregate: SyncAggregateSsz<C>,
    genesis_validators_root: H256,
    sync_committee: &SyncCommittee,
    bls_verifier: V,
) -> Result<(), Error> {
    let fork_version_slot = Slot::new(std::cmp::max(update.signature_slot.get(), 1) - 1);
    let fork_version =
        compute_fork_version(chain_id, compute_epoch_at_slot::<C>(fork_version_slot));

    let domain = compute_domain(
        DomainType::SYNC_COMMITTEE,
        Some(fork_version),
        Some(genesis_validators_root),
        ForkSchedule::for_chain_id(chain_id)
            .genesis()
            .current_version,
    );
    let signing_root = compute_signing_root(&update.attested_header.beacon, domain);

    let participant_pubkeys = sync_aggregate
        .sync_committee_bits
        .iter()
        .zip(sync_committee.pubkeys.iter());

    bls_verifier.aggregate_verify_signature(
        &sync_committee.aggregate_pubkey,
        participant_pubkeys
            .filter_map(|(included, pubkey)| {
                if (V::INVERSE && included) || (!V::INVERSE && !included) {
                    None
                } else {
                    Some(pubkey)
                }
            })
            .collect::<Vec<_>>(),
        signing_root.as_ref().to_owned(),
        update.sync_aggregate.sync_committee_signature,
    )?;

    Ok(())
}

/// Computes the execution block root hash.
///
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/deneb/light-client/sync-protocol.md#modified-get_lc_execution_root)
pub fn get_lc_execution_root<C: ChainSpec>(chain_id: u64, header: &LightClientHeader) -> H256 {
    let fs = ForkSchedule::for_chain_id(chain_id);

    let epoch = compute_epoch_at_slot::<C>(header.beacon.slot);

    // No new field in electra
    if let Some(fork) = fs.fork(Forks::Electra)
        && epoch >= fork.epoch
    {
        return TryInto::<deneb::ExecutionPayloadHeaderSsz<C>>::try_into(header.execution.clone())
            .unwrap()
            .tree_hash_root();
    }

    if let Some(fork) = fs.fork(Forks::Deneb)
        && epoch >= fork.epoch
    {
        return TryInto::<deneb::ExecutionPayloadHeaderSsz<C>>::try_into(header.execution.clone())
            .unwrap()
            .tree_hash_root();
    }

    // TODO: Figure out what to do here
    // if epoch >= fork_parameters.deneb.epoch {
    //     return denebExecutionPayloadHeader::from(header.execution.clone())
    //         .tree_hash_root()
    //         .into();
    // }

    H256::default()
}

/// Validates a light client header.
///
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/deneb/light-client/sync-protocol.md#modified-is_valid_light_client_header)
pub fn is_valid_light_client_header<C: ChainSpec>(
    chain_id: u64,
    header: &LightClientHeader,
) -> Result<(), Error> {
    let fs = ForkSchedule::for_chain_id(chain_id);

    let epoch = compute_epoch_at_slot::<C>(header.beacon.slot);

    if let Some(fork) = fs.fork(Forks::Deneb) {
        if epoch < fork.epoch {
            ensure(
                header.execution.blob_gas_used.is_zero()
                    && header.execution.excess_blob_gas.is_zero(),
                Error::MustBeDeneb,
            )?;
        }

        ensure(epoch >= fork.epoch, Error::InvalidChainVersion)?;
    } else {
        unreachable!("all known chains support deneb");
    }

    Ok(validate_merkle_branch(
        &get_lc_execution_root::<C>(chain_id, header),
        &header.execution_branch,
        EXECUTION_PAYLOAD_GINDEX,
        &header.beacon.body_root,
    )?)
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#finalized_root_gindex_at_slot>
pub fn finalized_root_gindex_at_slot<C: ChainSpec>(chain_id: u64, slot: Slot) -> u64 {
    let fs = ForkSchedule::for_chain_id(chain_id);

    let epoch = compute_epoch_at_slot::<C>(slot);

    if let Some(fork) = fs.fork(Forks::Electra)
        && epoch >= fork.epoch
    {
        return FINALIZED_ROOT_GINDEX_ELECTRA;
    }

    FINALIZED_ROOT_GINDEX
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#current_sync_committee_gindex_at_slot>
pub fn current_sync_committee_gindex_at_slot<C: ChainSpec>(chain_id: u64, slot: Slot) -> u64 {
    let fs = ForkSchedule::for_chain_id(chain_id);

    let epoch = compute_epoch_at_slot::<C>(slot);

    if let Some(fork) = fs.fork(Forks::Electra)
        && epoch >= fork.epoch
    {
        return CURRENT_SYNC_COMMITTEE_GINDEX_ELECTRA;
    }

    CURRENT_SYNC_COMMITTEE_GINDEX
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#next_sync_committee_gindex_at_slot>
pub fn next_sync_committee_gindex_at_slot<C: ChainSpec>(chain_id: u64, slot: Slot) -> u64 {
    let fs = ForkSchedule::for_chain_id(chain_id);

    let epoch = compute_epoch_at_slot::<C>(slot);

    if let Some(fork) = fs.fork(Forks::Electra)
        && epoch >= fork.epoch
    {
        return NEXT_SYNC_COMMITTEE_GINDEX_ELECTRA;
    }

    NEXT_SYNC_COMMITTEE_GINDEX
}

#[cfg(test)]
mod tests {
    use beacon_api_types::chain_spec::Mainnet;
    use hex_literal::hex;

    use super::*;

    pub struct AlwaysSuccessBlsVerifier;

    impl BlsVerify for AlwaysSuccessBlsVerifier {
        const INVERSE: bool = false;

        fn aggregate_verify_signature<'pk>(
            &self,
            _aggregate_public_key: &'pk H384,
            _public_keys: impl IntoIterator<Item = &'pk H384>,
            _msg: Vec<u8>,
            _signature: H768,
        ) -> Result<(), Error> {
            Ok(())
        }
    }

    const SEPOLIA_CHAIN_ID: u64 = 11155111;

    mod data_6553725 {
        use std::sync::LazyLock;

        use super::*;

        pub static UPDATE: LazyLock<ethereum_sync_protocol_types::LightClientUpdate> =
            LazyLock::new(|| {
                serde_json::from_str(include_str!("./test/light_client_update_6553725.json"))
                    .unwrap()
            });

        pub static SYNC_COMMITTEE: LazyLock<SyncCommittee> = LazyLock::new(|| {
            serde_json::from_str(include_str!("./test/sync_committee_6553725.json")).unwrap()
        });

        pub const GENESIS_VALIDATORS_ROOT: H256 = H256::new(hex!(
            "d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b8078"
        ));
    }

    fn update_6553725<V: BlsVerify>(
        update: &ethereum_sync_protocol_types::LightClientUpdate,
        bls_verifier: V,
    ) -> Result<(), Error> {
        validate_light_client_update::<Mainnet, V>(
            SEPOLIA_CHAIN_ID,
            update,
            Some(&data_6553725::SYNC_COMMITTEE),
            None,
            Slot::new(6553726),
            Slot::new(6553718),
            data_6553725::GENESIS_VALIDATORS_ROOT,
            bls_verifier,
        )
    }

    #[test]
    fn validate_update_works() {
        assert_eq!(
            update_6553725(&data_6553725::UPDATE, AlwaysSuccessBlsVerifier),
            Ok(())
        );
    }

    #[test]
    fn validate_update_fails_when_not_enough_participants() {
        let mut update = data_6553725::UPDATE.clone();
        update.sync_aggregate.sync_committee_bits = [0u8; 64].into();

        assert!(matches!(
            update_6553725(&update, AlwaysSuccessBlsVerifier),
            Err(Error::InsufficientSyncCommitteeParticipants(..))
        ));

        let _ = update.sync_aggregate.sync_committee_bits.pop();

        assert!(matches!(
            update_6553725(&update, AlwaysSuccessBlsVerifier),
            Err(Error::InvalidSyncCommitteeBits(..))
        ));
    }

    #[test]
    fn validate_update_fails_when_altered_next_sync_committee() {
        let mut update = data_6553725::UPDATE.clone();
        update.finalized_header.beacon.slot = update.finalized_header.beacon.slot + Slot::new(1);

        let mut next_sync_committee = update.next_sync_committee.as_ref().unwrap().clone();
        next_sync_committee.aggregate_pubkey = Default::default();

        assert!(matches!(
            validate_light_client_update::<Mainnet, AlwaysSuccessBlsVerifier>(
                SEPOLIA_CHAIN_ID,
                &update,
                Some(&data_6553725::SYNC_COMMITTEE),
                Some(&next_sync_committee),
                Slot::new(6553726),
                Slot::new(6553718),
                data_6553725::GENESIS_VALIDATORS_ROOT,
                AlwaysSuccessBlsVerifier,
            ),
            Err(Error::InvalidMerkleBranch(..))
        ));
    }

    #[test]
    fn validate_update_fails_when_altered_finalized_header() {
        let mut update = data_6553725::UPDATE.clone();
        update.finalized_header.beacon.slot = update.finalized_header.beacon.slot + Slot::new(1);

        assert!(matches!(
            update_6553725(&update, AlwaysSuccessBlsVerifier),
            Err(Error::InvalidMerkleBranch(..))
        ));
    }

    /// This test also ensures that any `attested_header.beacon` alteration would cause the signature verification
    /// to fail.
    #[test]
    fn validate_update_signature_data_correct() {
        pub struct AssertDataCorrectVerifier;

        impl BlsVerify for AssertDataCorrectVerifier {
            const INVERSE: bool = false;

            fn aggregate_verify_signature<'pk>(
                &self,
                aggregate_public_key: &'pk H384,
                public_keys: impl IntoIterator<Item = &'pk H384>,
                msg: Vec<u8>,
                signature: H768,
            ) -> Result<(), Error> {
                let fork_version_slot =
                    Slot::new(std::cmp::max(data_6553725::UPDATE.signature_slot.get(), 1) - 1);
                let fork_version = compute_fork_version(
                    SEPOLIA_CHAIN_ID,
                    compute_epoch_at_slot::<Mainnet>(fork_version_slot),
                );

                let domain = compute_domain(
                    DomainType::SYNC_COMMITTEE,
                    Some(fork_version),
                    Some(data_6553725::GENESIS_VALIDATORS_ROOT),
                    ForkSchedule::for_chain_id(SEPOLIA_CHAIN_ID)
                        .genesis()
                        .current_version,
                );
                let signing_root =
                    compute_signing_root(&data_6553725::UPDATE.attested_header.beacon, domain);

                assert_eq!(
                    aggregate_public_key,
                    &data_6553725::SYNC_COMMITTEE.aggregate_pubkey
                );

                let sync_aggregate: SyncAggregateSsz<Mainnet> = data_6553725::UPDATE
                    .sync_aggregate
                    .clone()
                    .try_into()
                    .unwrap();

                assert_eq!(
                    public_keys.into_iter().copied().collect::<Vec<_>>(),
                    sync_aggregate
                        .sync_committee_bits
                        .iter()
                        .zip(data_6553725::SYNC_COMMITTEE.pubkeys.iter())
                        .filter(|(included, _)| *included)
                        .map(|(_, pubkey)| *pubkey)
                        .collect::<Vec<_>>()
                );
                assert_eq!(msg, signing_root.as_ref().to_vec());
                assert_eq!(
                    signature,
                    data_6553725::UPDATE.sync_aggregate.sync_committee_signature
                );
                Ok(())
            }
        }

        update_6553725(&data_6553725::UPDATE, AssertDataCorrectVerifier).unwrap();
    }

    #[test]
    fn validate_update_slot_ordering_checks() {
        let mut update = data_6553725::UPDATE.clone();
        let original_attested_slot = update.attested_header.beacon.slot;

        // attested header can't be smaller than the finalized
        update.attested_header.beacon.slot = update.finalized_header.beacon.slot - Slot::new(1);

        assert!(matches!(
            update_6553725(&update, AlwaysSuccessBlsVerifier),
            Err(Error::InvalidSlots { .. })
        ));

        // but it can be equal
        update.attested_header.beacon.slot = update.finalized_header.beacon.slot;
        assert_eq!(update_6553725(&update, AlwaysSuccessBlsVerifier), Ok(()));

        update.attested_header.beacon.slot = original_attested_slot;
        // signature slot always must be greater than the attested slot
        update.signature_slot = update.attested_header.beacon.slot;

        assert!(matches!(
            update_6553725(&update, AlwaysSuccessBlsVerifier),
            Err(Error::InvalidSlots { .. })
        ));
    }

    #[test]
    fn validate_update_signature_period_checks() {
        let mut update = data_6553725::UPDATE.clone();
        let original_signature_slot = update.signature_slot;
        update.signature_slot = update.signature_slot + Slot::new(1000000);

        assert!(matches!(
            validate_light_client_update::<Mainnet, AlwaysSuccessBlsVerifier>(
                SEPOLIA_CHAIN_ID,
                &update,
                Some(&data_6553725::SYNC_COMMITTEE),
                None,
                update.signature_slot + Slot::new(1),
                Slot::new(6553718),
                data_6553725::GENESIS_VALIDATORS_ROOT,
                AlwaysSuccessBlsVerifier,
            ),
            Err(Error::InvalidSignaturePeriodWhenNextSyncCommitteeDoesNotExist { .. })
        ));

        assert!(matches!(
            validate_light_client_update::<Mainnet, AlwaysSuccessBlsVerifier>(
                SEPOLIA_CHAIN_ID,
                &update,
                Some(&data_6553725::SYNC_COMMITTEE),
                Some(&data_6553725::SYNC_COMMITTEE),
                update.signature_slot + Slot::new(1),
                Slot::new(6553718),
                data_6553725::GENESIS_VALIDATORS_ROOT,
                AlwaysSuccessBlsVerifier,
            ),
            Err(Error::InvalidSignaturePeriodWhenNextSyncCommitteeExists { .. })
        ));

        // we allow signature slot to be in the trusted period + 1
        update.signature_slot = original_signature_slot + Slot::new(256 * 32);
        assert_eq!(
            validate_light_client_update::<Mainnet, AlwaysSuccessBlsVerifier>(
                SEPOLIA_CHAIN_ID,
                &update,
                Some(&data_6553725::SYNC_COMMITTEE),
                update.next_sync_committee.as_ref(),
                update.signature_slot + Slot::new(1),
                Slot::new(6553718),
                data_6553725::GENESIS_VALIDATORS_ROOT,
                AlwaysSuccessBlsVerifier,
            ),
            Ok(())
        );
    }

    #[test]
    fn validate_update_fails_irrelevant_update() {
        let mut update = data_6553725::UPDATE.clone();

        // if the `update.attested.slot > finalized_slot`, then there must be no next sync committee given
        assert!(matches!(
            validate_light_client_update::<Mainnet, AlwaysSuccessBlsVerifier>(
                SEPOLIA_CHAIN_ID,
                &update,
                Some(&data_6553725::SYNC_COMMITTEE),
                update.next_sync_committee.as_ref(),
                update.signature_slot + Slot::new(1),
                update.attested_header.beacon.slot + Slot::new(100),
                data_6553725::GENESIS_VALIDATORS_ROOT,
                AlwaysSuccessBlsVerifier,
            ),
            Err(Error::IrrelevantUpdate { .. })
        ));

        update.next_sync_committee = None;

        // if the `update.attested.slot > finalized_slot`, then there must be no next sync committee in the update
        assert!(matches!(
            validate_light_client_update::<Mainnet, AlwaysSuccessBlsVerifier>(
                SEPOLIA_CHAIN_ID,
                &update,
                Some(&data_6553725::SYNC_COMMITTEE),
                None,
                update.signature_slot + Slot::new(1),
                update.attested_header.beacon.slot + Slot::new(100),
                data_6553725::GENESIS_VALIDATORS_ROOT,
                AlwaysSuccessBlsVerifier,
            ),
            Err(Error::IrrelevantUpdate { .. })
        ));
    }

    #[test]
    fn finalized_groot_index_correct() {
        assert_eq!(
            finalized_root_gindex_at_slot::<Mainnet>(SEPOLIA_CHAIN_ID, Slot::new(100)),
            FINALIZED_ROOT_GINDEX
        );

        assert_eq!(
            finalized_root_gindex_at_slot::<Mainnet>(SEPOLIA_CHAIN_ID, Slot::new(222465 * 32)),
            FINALIZED_ROOT_GINDEX_ELECTRA
        );
    }

    #[test]
    fn current_sync_committee_gindex_correct() {
        assert_eq!(
            current_sync_committee_gindex_at_slot::<Mainnet>(SEPOLIA_CHAIN_ID, Slot::new(100)),
            CURRENT_SYNC_COMMITTEE_GINDEX
        );

        assert_eq!(
            current_sync_committee_gindex_at_slot::<Mainnet>(
                SEPOLIA_CHAIN_ID,
                Slot::new(222465 * 32)
            ),
            CURRENT_SYNC_COMMITTEE_GINDEX_ELECTRA
        );
    }
}
