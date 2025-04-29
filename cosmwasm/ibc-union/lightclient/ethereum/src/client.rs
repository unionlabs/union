use beacon_api_types::{
    chain_spec::{ChainSpec, Mainnet, Minimal, PresetBaseKind},
    custom_types::Period,
};
use cosmwasm_std::{Addr, Empty, StdError, StdResult};
use depolama::{KeyCodec, Prefix, Store, ValueCodec};
use ethereum_light_client_types::{
    ClientState, ClientStateV1, ConsensusState, Header, LightClientUpdate, Misbehaviour,
    StorageProof,
};
use ethereum_sync_protocol::{
    utils::{
        compute_slot_at_timestamp, compute_sync_committee_period_at_slot,
        validate_signature_supermajority,
    },
    validate_light_client_update,
};
use evm_storage_verifier::{
    verify_account_storage_root, verify_storage_absence, verify_storage_proof,
};
use ibc_union_light_client::{
    spec::Timestamp, ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate,
};
use ibc_union_msg::lightclient::Status;
use unionlabs::{
    encoding::{Bincode, DecodeAs, EncodeAs},
    ensure,
    ethereum::ibc_commitment_key,
    ibc::core::client::height::Height,
    primitives::{Bytes, H256, U256},
};

use crate::{
    errors::Error, inverse_sync_committee::InverseSyncCommittee, verification::VerificationContext,
};

pub enum EthereumLightClient {}

impl IbcClient for EthereumLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    type Misbehaviour = Misbehaviour;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type StorageProof = StorageProof;

    type Encoding = Bincode;

    fn verify_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        Ok(verify_membership(
            key,
            consensus_state.storage_root,
            storage_proof,
            value,
        )?)
    }

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        Ok(verify_non_membership(
            key,
            consensus_state.storage_root,
            storage_proof,
        )?)
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> Timestamp {
        consensus_state.timestamp
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        match client_state {
            ClientState::V1(cs) => cs.latest_height,
        }
    }

    fn get_counterparty_chain_id(client_state: &Self::ClientState) -> String {
        match client_state {
            ClientState::V1(cs) => cs.chain_id.to_string(),
        }
    }

    fn status(ctx: IbcClientCtx<Self>, client_state: &Self::ClientState) -> Status {
        let _ = ctx;

        if match client_state {
            ClientState::V1(cs) => cs.frozen_height,
        }
        .height()
            != 0
        {
            Status::Frozen
        } else {
            Status::Active
        }
    }

    fn verify_creation(
        _caller: Addr,
        client_state: &Self::ClientState,
        consensus_state: &Self::ConsensusState,
        _relayer: Addr,
    ) -> Result<ClientCreationResult<Self>, IbcClientError<Self>> {
        let ClientState::V1(client_state) = client_state;
        let Some(initial_sync_committee) = client_state.initial_sync_committee.as_ref() else {
            return Err(Error::NoInitialSyncCommittee.into());
        };
        let mut client_state = client_state.clone();
        // We only require this at the creation phase. The client then manages the committees in a separate storage.
        client_state.initial_sync_committee = None;

        let current_sync_period = if client_state.chain_spec == PresetBaseKind::Minimal {
            compute_sync_committee_period_at_slot::<Minimal>(consensus_state.slot)
        } else {
            compute_sync_committee_period_at_slot::<Mainnet>(consensus_state.slot)
        };

        // Set the client state so that it overwrites the one that is passed.
        // Also save the current and next sync committees with the corresponding epoch numbers.
        Ok(ClientCreationResult::new()
            .overwrite_client_state(ClientState::V1(client_state))
            .add_storage_write::<SyncCommitteeStore>(
                current_sync_period,
                InverseSyncCommittee::take_inverse(&initial_sync_committee.current_sync_committee),
            )
            .add_storage_write::<SyncCommitteeStore>(
                current_sync_period + 1,
                InverseSyncCommittee::take_inverse(&initial_sync_committee.next_sync_committee),
            ))
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        _caller: Addr,
        header: Header,
        _relayer: Addr,
    ) -> Result<StateUpdate<Self>, IbcClientError<Self>> {
        let ClientState::V1(client_state) = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(header.trusted_height.height())?;

        let finalized_slot = header
            .consensus_update
            .update_data()
            .finalized_header
            .beacon
            .slot;

        match client_state.chain_spec {
            PresetBaseKind::Minimal => verify_header::<Minimal>(
                &ctx,
                client_state,
                consensus_state,
                ctx.read_self_storage::<SyncCommitteeStore>(
                    compute_sync_committee_period_at_slot::<Minimal>(finalized_slot),
                )?,
                header,
            ),
            PresetBaseKind::Mainnet => verify_header::<Mainnet>(
                &ctx,
                client_state,
                consensus_state,
                ctx.read_self_storage::<SyncCommitteeStore>(
                    compute_sync_committee_period_at_slot::<Mainnet>(finalized_slot),
                )?,
                header,
            ),
        }
    }

    fn misbehaviour(
        ctx: IbcClientCtx<Self>,
        _caller: Addr,
        misbehaviour: Self::Misbehaviour,
        _relayer: Addr,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        let consensus_state =
            ctx.read_self_consensus_state(misbehaviour.trusted_height.height())?;

        let ClientState::V1(mut client_state) = ctx.read_self_client_state()?;

        match client_state.chain_spec {
            PresetBaseKind::Minimal => {
                verify_misbehaviour::<Minimal>(&ctx, &client_state, consensus_state, misbehaviour)?
            }
            PresetBaseKind::Mainnet => {
                verify_misbehaviour::<Mainnet>(&ctx, &client_state, consensus_state, misbehaviour)?
            }
        }

        client_state.frozen_height = Height::new(1);

        Ok(ClientState::V1(client_state))
    }
}

pub fn verify_membership(
    key: Vec<u8>,
    storage_root: H256,
    storage_proof: StorageProof,
    value: Vec<u8>,
) -> Result<(), Error> {
    check_commitment_key(
        H256::try_from(&key).map_err(|_| Error::InvalidCommitmentKeyLength(key))?,
        storage_proof.key,
    )?;

    let value = H256::try_from(&value).map_err(|_| Error::InvalidCommitmentValueLength(value))?;

    let proof_value = H256::from(storage_proof.value.to_be_bytes());

    if value != proof_value {
        return Err(Error::StoredValueMismatch {
            expected: value,
            stored: proof_value,
        });
    }

    verify_storage_proof(
        storage_root,
        storage_proof.key,
        &rlp::encode(&storage_proof.value),
        storage_proof.proof,
    )
    .map_err(Error::VerifyStorageProof)
}

/// Verifies that no value is committed at `path` in the counterparty light client's storage.
pub fn verify_non_membership(
    key: Vec<u8>,
    storage_root: H256,
    storage_proof: StorageProof,
) -> Result<(), Error> {
    check_commitment_key(
        H256::try_from(&key).map_err(|_| Error::InvalidCommitmentKeyLength(key))?,
        storage_proof.key,
    )?;

    if verify_storage_absence(storage_root, storage_proof.key, &storage_proof.proof)
        .map_err(Error::VerifyStorageAbsence)?
    {
        Ok(())
    } else {
        Err(Error::CounterpartyStorageNotNil)
    }
}

pub fn check_commitment_key(path: H256, key: U256) -> Result<(), Error> {
    let expected_commitment_key = ibc_commitment_key(path);

    if expected_commitment_key != key {
        Err(Error::InvalidCommitmentKey {
            expected: expected_commitment_key,
            found: key,
        })
    } else {
        Ok(())
    }
}

pub fn verify_header<C: ChainSpec>(
    ctx: &IbcClientCtx<EthereumLightClient>,
    client_state: ClientStateV1,
    consensus_state: ConsensusState,
    sync_committee: InverseSyncCommittee,
    header: Header,
) -> Result<StateUpdate<EthereumLightClient>, IbcClientError<EthereumLightClient>> {
    // NOTE(aeryz): Ethereum consensus-spec says that we should use the slot
    // at the current timestamp.
    let current_slot =
        compute_slot_at_timestamp::<C>(client_state.genesis_time, ctx.env.block.time.seconds())
            .ok_or(Error::IntegerOverflow)?;

    let sync_committee = sync_committee.as_sync_committee();
    let (current_sync_committee, next_sync_committee) = match header.consensus_update {
        LightClientUpdate::SyncCommitteePeriodChange(_) => (None, Some(&sync_committee)),
        LightClientUpdate::WithinSyncCommitteePeriod(_) => (Some(&sync_committee), None),
    };

    validate_light_client_update::<C, _>(
        client_state.chain_id,
        &header.consensus_update.clone().into_light_client_update(),
        current_sync_committee,
        next_sync_committee,
        current_slot,
        consensus_state.slot,
        client_state.genesis_validators_root,
        VerificationContext { deps: ctx.deps },
    )
    .map_err(Error::ValidateLightClient)?;

    let update_data = header.consensus_update.update_data();

    // check whether at least 2/3 of the sync committee signed
    ensure(
        validate_signature_supermajority(&update_data.sync_aggregate.sync_committee_bits),
        Error::NotEnoughSignatures,
    )?;

    let proof_data = &header.ibc_account_proof;

    verify_account_storage_root(
        update_data.finalized_header.execution.state_root,
        &client_state.ibc_contract_address,
        &proof_data.proof,
        &proof_data.storage_root,
    )
    .map_err(Error::VerifyAccountStorageRoot)?;

    update_state::<C>(client_state, consensus_state, header)
}

fn update_state<C: ChainSpec>(
    mut client_state: ClientStateV1,
    mut consensus_state: ConsensusState,
    mut header: Header,
) -> Result<StateUpdate<EthereumLightClient>, IbcClientError<EthereumLightClient>> {
    let trusted_height = header.trusted_height;
    let consensus_update = header.consensus_update.update_data();

    // TODO(aeryz): we should ditch this functionality as it complicates the light client and we don't use it
    // Some updates can be only for updating the sync committee, therefore the slot number can be
    // smaller. We don't want to save a new state if this is the case.
    let updated_height = core::cmp::max(
        trusted_height.height(),
        consensus_update.finalized_header.execution.block_number,
    );

    if consensus_update.finalized_header.beacon.slot > consensus_state.slot {
        consensus_state.slot = consensus_update.finalized_header.beacon.slot;

        consensus_state.state_root = consensus_update.finalized_header.execution.state_root;
        consensus_state.storage_root = header.ibc_account_proof.storage_root;

        consensus_state.timestamp =
            Timestamp::from_secs(consensus_update.finalized_header.execution.timestamp);

        if client_state.latest_height < consensus_update.finalized_header.execution.block_number {
            client_state.latest_height = consensus_update.finalized_header.execution.block_number;
        }
    }

    let mut state_update = StateUpdate::new(updated_height, consensus_state);
    if client_state.latest_height == consensus_update.finalized_header.execution.block_number {
        state_update = state_update.overwrite_client_state(ClientState::V1(client_state));
    }

    if let LightClientUpdate::SyncCommitteePeriodChange(update) = &mut header.consensus_update {
        let period = compute_sync_committee_period_at_slot::<C>(
            update.update_data.finalized_header.beacon.slot,
        );
        state_update = state_update.add_storage_write::<SyncCommitteeStore>(
            period + 1,
            InverseSyncCommittee::take_inverse(&update.next_sync_committee),
        );
    }

    Ok(state_update)
}

pub fn verify_misbehaviour<C: ChainSpec>(
    ctx: &IbcClientCtx<EthereumLightClient>,
    client_state: &ClientStateV1,
    consensus_state: ConsensusState,
    misbehaviour: Misbehaviour,
) -> Result<(), IbcClientError<EthereumLightClient>> {
    // There is no point to check for misbehaviour when the headers are not for the same height
    let (slot_1, slot_2) = (
        misbehaviour
            .update_1
            .update_data()
            .finalized_header
            .beacon
            .slot,
        misbehaviour
            .update_2
            .update_data()
            .finalized_header
            .beacon
            .slot,
    );
    ensure(
        slot_1 == slot_2,
        Error::MisbehaviourCannotExist(slot_1, slot_2),
    )?;

    ensure(
        misbehaviour.update_1.update_data().finalized_header
            != misbehaviour.update_2.update_data().finalized_header,
        Error::IdenticalMisbehaviourHeaders,
    )?;

    let current_slot =
        compute_slot_at_timestamp::<C>(client_state.genesis_time, ctx.env.block.time.seconds())
            .ok_or(Error::IntegerOverflow)?;

    let period = compute_sync_committee_period_at_slot::<C>(slot_1);

    let sync_committee = ctx
        .read_self_storage::<SyncCommitteeStore>(period)?
        .as_sync_committee();
    let (current_sync_committee, next_sync_committee) = match misbehaviour.update_1 {
        LightClientUpdate::SyncCommitteePeriodChange(_) => (None, Some(&sync_committee)),
        LightClientUpdate::WithinSyncCommitteePeriod(_) => (Some(&sync_committee), None),
    };

    // Make sure both headers would have been accepted by the light client
    validate_light_client_update::<C, VerificationContext>(
        client_state.chain_id,
        &misbehaviour.update_1.clone().into_light_client_update(),
        current_sync_committee,
        next_sync_committee,
        current_slot,
        consensus_state.slot,
        client_state.genesis_validators_root,
        VerificationContext { deps: ctx.deps },
    )
    .map_err(Error::ValidateLightClient)?;

    // check whether at least 2/3 of the sync committee signed
    ensure(
        validate_signature_supermajority(
            &misbehaviour
                .update_1
                .update_data()
                .sync_aggregate
                .sync_committee_bits,
        ),
        Error::NotEnoughSignatures,
    )?;

    validate_light_client_update::<C, VerificationContext>(
        client_state.chain_id,
        &misbehaviour.update_1.into_light_client_update(),
        current_sync_committee,
        next_sync_committee,
        current_slot,
        consensus_state.slot,
        client_state.genesis_validators_root,
        VerificationContext { deps: ctx.deps },
    )
    .map_err(Error::ValidateLightClient)?;

    ensure(
        validate_signature_supermajority(
            &misbehaviour
                .update_2
                .update_data()
                .sync_aggregate
                .sync_committee_bits,
        ),
        Error::NotEnoughSignatures,
    )?;

    Ok(())
}

pub enum SyncCommitteeStore {}
impl Store for SyncCommitteeStore {
    const PREFIX: Prefix = Prefix::new(b"sync_committee");

    type Key = Period;
    type Value = InverseSyncCommittee;
}
impl KeyCodec<Period> for SyncCommitteeStore {
    fn encode_key(key: &Period) -> Bytes {
        key.get().to_be_bytes().into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<Period> {
        raw.try_into()
            .map_err(|_| {
                StdError::generic_err(format!(
                    "invalid key: expected {N} bytes, found {}: {raw}",
                    raw.len(),
                    N = u64::BITS / 8,
                ))
            })
            .map(u64::from_be_bytes)
            .map(Period::new)
    }
}
impl ValueCodec<InverseSyncCommittee> for SyncCommitteeStore {
    fn encode_value(value: &InverseSyncCommittee) -> Bytes {
        value.encode_as::<Bincode>().into()
    }

    fn decode_value(raw: &Bytes) -> StdResult<InverseSyncCommittee> {
        if raw.len() % 4 != 0 {
            Err(StdError::generic_err(format!(
                "invalid length; expected multiple of 4 bytes but found {}: raw",
                raw.len()
            )))
        } else {
            InverseSyncCommittee::decode_as::<Bincode>(raw).map_err(|e| {
                StdError::generic_err(format!("unable to decode {}: {e}", stringify!($ty)))
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use beacon_api_types::{altair::SyncCommittee, electra};
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        Addr,
    };
    use ethereum_light_client_types::{
        client_state::InitialSyncCommittee, AccountProof, LightClientUpdateData,
        SyncCommitteePeriodChangeUpdate, WithinSyncCommitteePeriodUpdate,
    };
    use ethereum_sync_protocol::utils::compute_timestamp_at_slot;
    use hex_literal::hex;
    use unionlabs::primitives::H160;

    use super::*;

    const SEPOLIA_CHAIN_ID: u64 = 11155111;
    const SEPOLIA_GENESIS_VALIDATORS_ROOT: H256 = H256::new(hex!(
        "d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b8078"
    ));
    const SEPOLIA_GENESIS_TIME: u64 = 1655733600;
    const IBC_CONTRACT_ADDRESS: H160 = H160::new(hex!("bad69711Da45A0FF61e2c50b8c9B1F3314742d2b"));
    const INITIAL_STORAGE_HASH: H256 = H256::new(hex!(
        "b50a9ddbc6bffedde0118e5723996456582a4c18c45914b21231cfa52c5e2ad2"
    ));
    const FINALITY_UPDATE_ACCOUNT_STORAGE_ROOT: H256 = H256::new(hex!(
        "045c10398196b51905129fdbd1cbafdf0328877c575b9da41f15d7718f330d23"
    ));
    const PERIOD_CHANGING_UPDATE_ACCOUNT_STORAGE_ROOT: H256 = H256::new(hex!(
        "bc554ade41b65688ba7edf9e1ad8c4ec28d95e4ec95b9898e11b34d11b24a84e"
    ));

    static INITIAL_HEADER: LazyLock<beacon_api_types::phase0::BeaconBlockHeader> =
        LazyLock::new(|| serde_json::from_str(include_str!("./test/header_7167008.json")).unwrap());
    static CURRENT_SYNC_COMMITTEE: LazyLock<SyncCommittee> = LazyLock::new(|| {
        serde_json::from_str(include_str!("./test/current_sync_committee_7167008.json")).unwrap()
    });
    static NEXT_SYNC_COMMITTEE: LazyLock<SyncCommittee> = LazyLock::new(|| {
        serde_json::from_str(include_str!("./test/next_sync_committee_7167008.json")).unwrap()
    });
    static FINALITY_UPDATE: LazyLock<LightClientUpdateData> = LazyLock::new(|| {
        serde_json::from_str(include_str!("./test/finality_update_7167040.json")).unwrap()
    });
    static FINALITY_UPDATE_ACCOUNT_PROOF: LazyLock<Vec<Bytes>> = LazyLock::new(|| {
        serde_json::from_str(include_str!("./test/account_proof_7167040.json")).unwrap()
    });
    static PERIOD_CHANGING_UPDATE: LazyLock<electra::LightClientUpdate> = LazyLock::new(|| {
        serde_json::from_str(include_str!("./test/period_changing_update_7167040.json")).unwrap()
    });
    static PERIOD_CHANGING_UPDATE_ACCOUNT_PROOF: LazyLock<Vec<Bytes>> = LazyLock::new(|| {
        serde_json::from_str(include_str!("./test/account_proof_7168000.json")).unwrap()
    });

    fn initial_client_state(latest_height: u64) -> ClientState {
        ClientState::V1(ClientStateV1 {
            chain_id: SEPOLIA_CHAIN_ID,
            chain_spec: PresetBaseKind::Mainnet,
            genesis_validators_root: SEPOLIA_GENESIS_VALIDATORS_ROOT,
            genesis_time: SEPOLIA_GENESIS_TIME,
            latest_height,
            frozen_height: Height::default(),
            ibc_contract_address: IBC_CONTRACT_ADDRESS,
            initial_sync_committee: None,
        })
    }

    #[test]
    fn verify_creation_works() {
        let ClientState::V1(mut client_state) = initial_client_state(100);
        client_state.initial_sync_committee = Some(InitialSyncCommittee {
            current_sync_committee: CURRENT_SYNC_COMMITTEE.clone(),
            next_sync_committee: NEXT_SYNC_COMMITTEE.clone(),
        });
        let consensus_state = ConsensusState {
            slot: INITIAL_HEADER.slot,
            state_root: INITIAL_HEADER.state_root,
            storage_root: INITIAL_STORAGE_HASH,
            timestamp: Timestamp::from_secs(compute_timestamp_at_slot::<Mainnet>(
                SEPOLIA_GENESIS_TIME,
                INITIAL_HEADER.slot,
            )),
        };

        let res = EthereumLightClient::verify_creation(
            Addr::unchecked("hello"),
            &ClientState::V1(client_state.clone()),
            &consensus_state,
            Addr::unchecked("hello"),
        )
        .unwrap();

        client_state.initial_sync_committee = None;
        assert_eq!(res.client_state, Some(ClientState::V1(client_state)));

        assert!(res.events.is_empty());

        assert_eq!(res.storage_writes.len(), 2);
        let current_period = compute_sync_committee_period_at_slot::<Mainnet>(consensus_state.slot);
        assert_eq!(
            res.storage_writes
                .get(&depolama::raw_key::<SyncCommitteeStore>(&current_period)),
            Some(
                &InverseSyncCommittee::take_inverse(&CURRENT_SYNC_COMMITTEE)
                    .encode_as::<Bincode>()
                    .into()
            ),
        );

        assert_eq!(
            res.storage_writes
                .get(&depolama::raw_key::<SyncCommitteeStore>(
                    &(current_period + 1)
                )),
            Some(
                &InverseSyncCommittee::take_inverse(&NEXT_SYNC_COMMITTEE)
                    .encode_as::<Bincode>()
                    .into()
            ),
        );
    }

    #[test]
    fn verify_within_period_update() {
        let consensus_state = ConsensusState {
            slot: INITIAL_HEADER.slot,
            state_root: INITIAL_HEADER.state_root,
            storage_root: INITIAL_STORAGE_HASH,
            timestamp: Timestamp::from_secs(compute_timestamp_at_slot::<Mainnet>(
                SEPOLIA_GENESIS_TIME,
                INITIAL_HEADER.slot,
            )),
        };

        let ClientState::V1(mut client_state) =
            initial_client_state(FINALITY_UPDATE.finalized_header.execution.block_number - 1);

        let deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = cosmwasm_std::Timestamp::from_seconds(
            FINALITY_UPDATE.attested_header.execution.timestamp + 24,
        );
        let state_update = verify_header::<Mainnet>(
            &IbcClientCtx {
                client_id: 1.try_into().unwrap(),
                ibc_host: Addr::unchecked("hey bro"),
                deps: deps.as_ref(),
                env,
            },
            client_state.clone(),
            consensus_state,
            InverseSyncCommittee::take_inverse(&CURRENT_SYNC_COMMITTEE),
            Header {
                trusted_height: Height::new(client_state.latest_height),
                consensus_update: LightClientUpdate::WithinSyncCommitteePeriod(Box::new(
                    WithinSyncCommitteePeriodUpdate {
                        update_data: FINALITY_UPDATE.clone(),
                    },
                )),
                ibc_account_proof: AccountProof {
                    storage_root: FINALITY_UPDATE_ACCOUNT_STORAGE_ROOT,
                    proof: FINALITY_UPDATE_ACCOUNT_PROOF.clone(),
                },
            },
        )
        .unwrap();

        let consensus_state = ConsensusState {
            slot: FINALITY_UPDATE.finalized_header.beacon.slot,
            state_root: FINALITY_UPDATE.finalized_header.execution.state_root,
            storage_root: FINALITY_UPDATE_ACCOUNT_STORAGE_ROOT,
            timestamp: Timestamp::from_secs(FINALITY_UPDATE.finalized_header.execution.timestamp),
        };

        client_state.latest_height = FINALITY_UPDATE.finalized_header.execution.block_number;

        assert_eq!(
            state_update.client_state,
            Some(ClientState::V1(client_state))
        );

        assert_eq!(state_update.consensus_state, consensus_state);

        assert_eq!(
            state_update.height,
            FINALITY_UPDATE.finalized_header.execution.block_number
        );

        // No sync committee write because this is within the sync committee period, so no new sync committee to be written
        assert!(state_update.storage_writes.is_empty());
    }

    fn construct_light_client_update_data(
        update: electra::LightClientUpdate,
    ) -> LightClientUpdateData {
        LightClientUpdateData {
            attested_header: update.attested_header.clone().into(),
            finalized_header: update.finalized_header.clone().into(),
            finality_branch: update.finality_branch.to_vec(),
            sync_aggregate: update.sync_aggregate.clone(),
            signature_slot: update.signature_slot,
        }
    }

    // Ensures:
    // - A valid period changing update is verified,
    // - Correct client state, consensus state, and height is returned,
    // - The returned `storage_writes` include the correct sync committee under the correct key.
    #[test]
    fn verify_period_changing_update() {
        let consensus_state = ConsensusState {
            slot: FINALITY_UPDATE.finalized_header.beacon.slot,
            state_root: FINALITY_UPDATE.finalized_header.beacon.state_root,
            storage_root: FINALITY_UPDATE_ACCOUNT_STORAGE_ROOT,
            timestamp: Timestamp::from_secs(compute_timestamp_at_slot::<Mainnet>(
                SEPOLIA_GENESIS_TIME,
                FINALITY_UPDATE.finalized_header.beacon.slot,
            )),
        };

        let ClientState::V1(mut client_state) =
            initial_client_state(FINALITY_UPDATE.finalized_header.execution.block_number);

        let deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = cosmwasm_std::Timestamp::from_seconds(
            PERIOD_CHANGING_UPDATE.attested_header.execution.timestamp + 24,
        );
        let state_update = verify_header::<Mainnet>(
            &IbcClientCtx {
                client_id: 1.try_into().unwrap(),
                ibc_host: Addr::unchecked("hey bro"),
                deps: deps.as_ref(),
                env,
            },
            client_state.clone(),
            consensus_state,
            InverseSyncCommittee::take_inverse(&NEXT_SYNC_COMMITTEE),
            Header {
                trusted_height: Height::new(client_state.latest_height),
                consensus_update: LightClientUpdate::SyncCommitteePeriodChange(Box::new(
                    SyncCommitteePeriodChangeUpdate {
                        update_data: construct_light_client_update_data(
                            PERIOD_CHANGING_UPDATE.clone(),
                        ),
                        next_sync_committee: PERIOD_CHANGING_UPDATE.next_sync_committee.clone(),
                        next_sync_committee_branch: PERIOD_CHANGING_UPDATE
                            .next_sync_committee_branch
                            .into(),
                    },
                )),
                ibc_account_proof: AccountProof {
                    storage_root: PERIOD_CHANGING_UPDATE_ACCOUNT_STORAGE_ROOT,
                    proof: PERIOD_CHANGING_UPDATE_ACCOUNT_PROOF.clone(),
                },
            },
        )
        .unwrap();

        let consensus_state = ConsensusState {
            slot: PERIOD_CHANGING_UPDATE.finalized_header.beacon.slot,
            state_root: PERIOD_CHANGING_UPDATE.finalized_header.execution.state_root,
            storage_root: PERIOD_CHANGING_UPDATE_ACCOUNT_STORAGE_ROOT,
            timestamp: Timestamp::from_secs(
                PERIOD_CHANGING_UPDATE.finalized_header.execution.timestamp,
            ),
        };

        client_state.latest_height = PERIOD_CHANGING_UPDATE
            .finalized_header
            .execution
            .block_number;

        assert_eq!(
            state_update.client_state,
            Some(ClientState::V1(client_state))
        );

        assert_eq!(state_update.consensus_state, consensus_state);

        assert_eq!(
            state_update.height,
            PERIOD_CHANGING_UPDATE
                .finalized_header
                .execution
                .block_number
        );

        let next_sync_period = compute_sync_committee_period_at_slot::<Mainnet>(
            PERIOD_CHANGING_UPDATE.finalized_header.beacon.slot,
        ) + 1;
        assert_eq!(state_update.storage_writes.len(), 1);
        assert_eq!(
            state_update
                .storage_writes
                .get(&depolama::raw_key::<SyncCommitteeStore>(&next_sync_period)),
            Some(
                &InverseSyncCommittee::take_inverse(&PERIOD_CHANGING_UPDATE.next_sync_committee)
                    .encode_as::<Bincode>()
                    .into()
            )
        );
    }

    #[test]
    fn please_work() {
        verify_account_storage_root(
            hex!("69d1e6cec47b2abb7319b11a1f94cff7d532deff9ee5da3cf101dd4c16dd72b1").into(),
            &hex!("ed2af2aD7FE0D92011b26A2e5D1B4dC7D12A47C5").into(),
            vec![
                hex!("f90211a02e8a5ae0114f9e87cfce7f748e78b30c3c8e657afa573873d764390965f9bbeea027cce7ec5bd0d64c3c4c43a34ac8a0ebdf61044c1c8e9b3765804ef98c12b606a06e764ed325954592e828e93edea0d8495f97e7117eebdc3e09efc4cb82dbf1b9a09517e96d8ff66ff58f7831da45addcad21c2bdf58c2ae80ffeffb6c042f90e14a0d7418824f145a59200a234ae2fca19e87bc7b9447bf4f4f1ab641719f0f8e9a5a0167b05d0feb501b9eebf4fcae4407250e703ffc504f8560060b845abcce43444a081aa05ba6e956d19ce38943727a944107a5b96008bb0ec4a9c1cd06f5d3909a9a04e08a6f1dff4965e506b19bd874f4717db1a0b8015dae5124504a0db09706e06a0d0bfec2a0c4210491eeee0de1147ca12e21a8a286356960d77b1d56bf1af778ba080cab207fb3d07baa1bb8aa398e4cb97d38fac06994e85d22aa34d04f6fb4e0ca0843c39734c48fd518e0c782e87b9168b4053768f27a7559d2d30352bc5e64261a0c52469f342b22fb92b7e099d038fe9bda84958a2508bdb093d85ad1073e72c0da04f1967913b504d0abb84ae3e381fe8273536715e4985c17a7b398d533706a704a0323715bdc0a7198f62844a2bc297d8aaa85b4ca0f044f13483dddf2addc2fa78a0213ebe73c28294598d6317486f3cdecd5409542531cf7dfbba30cc1710dda25fa07c055cf086c71e3fb8e02c0a1daec973a238745652110a310041187016c8e05c80").as_slice(),
                hex!("f8b180a0de5d0f8d14997487a666b6a3c137e46422e0c327d2bf20c87467ad7c5f5ea02580a00d460a0db693d5bfda59c16f2e3ee87df226323afa73836ad9ca3dfd6954fe4c8080a0557a3dd917abc049f5cbf3c308799f78332ce7058a75598e6f21fe0498c5e59e8080a00a0e576564c351990f2393de596a429df581ac49557a4ff88f7291c5ccd9d5c280a05b133b1620ecde5f13d72cf1a4afc0b955b0c9eec50236475495bdbd7bea4c008080808080").as_slice(),
                hex!("f869a020fa8211e9294f5039596478f2268d90b881d2de706d04dcf3e2e1a87f451f51b846f8440180a0342bd0953abafd893cb0303edbab9ba83bcca1b32378a516766765a7f69eb290a0f1da0a71fdc1bdb5a344cb66eb8a98a3fa0aef81853ca761b387738e4c1899a4").as_slice(),
              ],
              &hex!("342bd0953abafd893cb0303edbab9ba83bcca1b32378a516766765a7f69eb290").into()
        ).unwrap();
    }
}
