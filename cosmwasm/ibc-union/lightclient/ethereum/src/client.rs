use beacon_api_types::{
    chain_spec::{ChainSpec, Mainnet, Minimal, PresetBaseKind},
    slot::Slot,
};
use cosmwasm_std::{Addr, Empty, StdError, StdResult};
use depolama::{KeyCodec, Prefix, Store, ValueCodec};
use ethereum_light_client_types::{
    ClientState, ClientStateV1, ConsensusState, Header, LightClientUpdate, Misbehaviour,
    StorageProof,
};
use ethereum_sync_protocol::{
    utils::{
        compute_epoch_at_slot, compute_slot_at_timestamp, compute_sync_committee_period_at_slot,
        validate_signature_supermajority,
    },
    validate_light_client_update,
};
use evm_storage_verifier::{
    verify_account_storage_root, verify_storage_absence, verify_storage_proof,
};
use ibc_union_light_client::{
    ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate,
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

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64 {
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
        _consensus_state: &Self::ConsensusState,
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
            compute_sync_committee_period_at_slot::<Minimal>(Slot::new(client_state.latest_height))
        } else {
            compute_sync_committee_period_at_slot::<Mainnet>(Slot::new(client_state.latest_height))
        };

        // Set the client state so that it overwrites the one that is passed.
        // Also save the current and next sync committees with the corresponding epoch numbers.
        Ok(ClientCreationResult::new()
            .overwrite_client_state(ClientState::V1(client_state))
            .add_storage_write(
                current_sync_period.to_le_bytes().into(),
                InverseSyncCommittee::take_inverse(&initial_sync_committee.current_sync_committee),
            )
            .add_storage_write(
                {
                    let sync_period = current_sync_period + 1;
                    sync_period.to_le_bytes().into()
                },
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
        .map_err(Into::into)
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
        LightClientUpdate::EpochChange(_) => (None, Some(&sync_committee)),
        LightClientUpdate::WithinEpoch(_) => (Some(&sync_committee), None),
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
            consensus_update.finalized_header.execution.timestamp * 1_000_000_000;

        if client_state.latest_height < consensus_update.finalized_header.execution.block_number {
            client_state.latest_height = consensus_update.finalized_header.execution.block_number;
        }
    }

    let mut state_update = StateUpdate::new(updated_height, consensus_state);
    if client_state.latest_height == consensus_update.finalized_header.execution.block_number {
        state_update = state_update.overwrite_client_state(ClientState::V1(client_state));
    }

    if let LightClientUpdate::EpochChange(update) = &mut header.consensus_update {
        let current_epoch =
            compute_epoch_at_slot::<C>(update.update_data.finalized_header.beacon.slot);
        state_update = state_update.add_storage_write::<SyncCommitteeStore>(
            current_epoch + 1,
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

    let epoch = compute_epoch_at_slot::<C>(slot_1);

    let sync_committee = ctx
        .read_self_storage::<SyncCommitteeStore>(epoch)?
        .as_sync_committee();
    let (current_sync_committee, next_sync_committee) = match misbehaviour.update_1 {
        LightClientUpdate::EpochChange(_) => (None, Some(&sync_committee)),
        LightClientUpdate::WithinEpoch(_) => (Some(&sync_committee), None),
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

    type Key = u64;
    type Value = InverseSyncCommittee;
}
impl KeyCodec<u64> for SyncCommitteeStore {
    fn encode_key(key: &u64) -> Bytes {
        key.to_be_bytes().into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<u64> {
        raw.try_into()
            .map_err(|_| {
                StdError::generic_err(format!(
                    "invalid key: expected {N} bytes, found {}: {raw}",
                    raw.len(),
                    N = u64::BITS / 8,
                ))
            })
            .map(u64::from_be_bytes)
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

    use beacon_api_types::altair::SyncCommittee;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        Addr, Timestamp,
    };
    use ethereum_light_client_types::{AccountProof, LightClientUpdateData, WithinEpochUpdate};
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

    static INITIAL_HEADER: LazyLock<beacon_api_types::phase0::BeaconBlockHeader> =
        LazyLock::new(|| serde_json::from_str(include_str!("./test/header_7167008.json")).unwrap());
    static CURRENT_SYNC_COMMITTEE: LazyLock<SyncCommittee> = LazyLock::new(|| {
        serde_json::from_str(include_str!("./test/current_sync_committee_7167008.json")).unwrap()
    });
    static _NEXT_SYNC_COMMITTEE: LazyLock<SyncCommittee> = LazyLock::new(|| {
        serde_json::from_str(include_str!("./test/next_sync_committee_7167008.json")).unwrap()
    });
    static FINALITY_UPDATE: LazyLock<LightClientUpdateData> = LazyLock::new(|| {
        serde_json::from_str(include_str!("./test/finality_update_7167040.json")).unwrap()
    });
    static FINALITY_UPDATE_ACCOUNT_PROOF: LazyLock<Vec<Bytes>> = LazyLock::new(|| {
        serde_json::from_str(include_str!("./test/account_proof_7167040.json")).unwrap()
    });

    fn initial_state() -> (ClientState, ConsensusState) {
        let client_state = ClientState::V1(ClientStateV1 {
            chain_id: SEPOLIA_CHAIN_ID,
            chain_spec: PresetBaseKind::Mainnet,
            genesis_validators_root: SEPOLIA_GENESIS_VALIDATORS_ROOT,
            genesis_time: SEPOLIA_GENESIS_TIME,
            latest_height: INITIAL_HEADER.slot.get(),
            frozen_height: Height::default(),
            ibc_contract_address: IBC_CONTRACT_ADDRESS,
            initial_sync_committee: None,
        });

        let consensus_state = ConsensusState {
            slot: INITIAL_HEADER.slot,
            state_root: INITIAL_HEADER.state_root,
            storage_root: INITIAL_STORAGE_HASH,
            timestamp: compute_timestamp_at_slot::<Mainnet>(
                SEPOLIA_GENESIS_TIME,
                INITIAL_HEADER.slot,
            ),
        };

        (client_state, consensus_state)
    }

    #[test]
    fn verify_header_works() {
        let (ClientState::V1(client_state), consensus_state) = initial_state();
        let deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time =
            Timestamp::from_seconds(FINALITY_UPDATE.attested_header.execution.timestamp + 24);
        verify_header::<Mainnet>(
            &IbcClientCtx {
                client_id: 1,
                ibc_host: Addr::unchecked("hey bro"),
                deps: deps.as_ref(),
                env,
            },
            client_state,
            consensus_state,
            InverseSyncCommittee::take_inverse(&CURRENT_SYNC_COMMITTEE),
            Header {
                trusted_height: Height::new(INITIAL_HEADER.slot.get()),
                consensus_update: LightClientUpdate::WithinEpoch(Box::new(WithinEpochUpdate {
                    update_data: FINALITY_UPDATE.clone(),
                })),
                ibc_account_proof: AccountProof {
                    storage_root: FINALITY_UPDATE_ACCOUNT_STORAGE_ROOT,
                    proof: FINALITY_UPDATE_ACCOUNT_PROOF.clone(),
                },
            },
        )
        .unwrap();
    }
}
