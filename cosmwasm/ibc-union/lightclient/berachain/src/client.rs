use beacon_api_types::{chain_spec::Mainnet, deneb};
use berachain_light_client_types::{
    client_state::ClientStateV1, ClientState, ConsensusState, Header,
};
use cometbft_types::types::{
    commit::Commit, signed_header::SignedHeader, validator_set::ValidatorSet,
};
use cosmwasm_std::{Addr, Empty};
use ethereum_light_client_types::StorageProof;
use ibc_union_light_client::{
    spec::Timestamp, ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate,
};
use ibc_union_msg::lightclient::Status;
use tendermint_verifier::types::Verification;
use unionlabs::{
    berachain::LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX,
    bounded::BoundedI64,
    encoding::{Bincode, EncodeAs, Ssz},
    ibc::core::commitment::merkle_root::MerkleRoot,
    primitives::{encoding::HexUnprefixed, H256},
};

use crate::{errors::Error, verify::Bls12381Verifier};

pub struct BerachainLightClient;

impl IbcClient for BerachainLightClient {
    type Error = Error;

    type Header = Header;

    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type Encoding = Bincode;

    type CustomQuery = Empty;

    type StorageProof = StorageProof;

    fn verify_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        ethereum_light_client::client::verify_membership(
            key,
            consensus_state.evm_storage_root,
            storage_proof,
            value,
        )
        .map_err(Into::<Error>::into)?;
        Ok(())
    }

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        ethereum_light_client::client::verify_non_membership(
            key,
            consensus_state.evm_storage_root,
            storage_proof,
        )
        .map_err(Into::<Error>::into)?;
        Ok(())
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> Timestamp {
        consensus_state.evm_timestamp
    }

    fn get_latest_height(ClientState::V1(client_state): &Self::ClientState) -> u64 {
        client_state.latest_height
    }

    fn get_counterparty_chain_id(ClientState::V1(client_state): &Self::ClientState) -> String {
        client_state.chain_id.to_string()
    }

    fn status(ctx: IbcClientCtx<Self>, client_state: &Self::ClientState) -> Status {
        let _ = client_state;
        let _ = ctx;
        // FIXME: expose the ctx to this call to allow threading this call to L1
        // client. generally, we want to thread if a client is an L2 so always
        // provide the ctx?
        Status::Active
    }

    fn verify_creation(
        _caller: Addr,
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
        _relayer: Addr,
    ) -> Result<ClientCreationResult<Self>, IbcClientError<Self>> {
        Ok(ClientCreationResult::new())
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        _caller: Addr,
        header: Self::Header,
        _relayer: Addr,
    ) -> Result<StateUpdate<Self>, IbcClientError<Self>> {
        let ClientState::V1(client_state) = ctx.read_self_client_state()?;
        let consensus_state =
            ctx.read_self_consensus_state(header.tm_header.trusted_height.height())?;

        // 1. verify that the evm execution header is part of the cometbft consensus state
        ics23::ibc_api::verify_membership(
            &header.execution_header_proof,
            &client_state.proof_specs,
            &consensus_state.comet_root.hash.into(),
            &[
                b"beacon".to_vec(),
                [LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX].to_vec(),
            ],
            deneb::ExecutionPayloadHeaderSsz::<Mainnet>::try_from(header.execution_header.clone())
                .map_err(Into::<Error>::into)?
                .encode_as::<Ssz>(),
        )
        .map_err(Into::<Error>::into)?;

        // 2. verify that the contract storage root is part of the evm execution header
        evm_storage_verifier::verify_account_storage_root(
            header.execution_header.state_root,
            &client_state.ibc_contract_address,
            &header.account_proof.proof,
            &header.account_proof.storage_root,
        )
        .map_err(Into::<Error>::into)?;

        // 3. Verify cometbft consensus and signatures
        verify_header(
            client_state,
            consensus_state,
            header.tm_header,
            ctx.env.block.time,
            Timestamp::from_secs(header.execution_header.timestamp),
            header.execution_header.state_root,
            header.account_proof.storage_root,
            Bls12381Verifier::new(ctx.deps),
        )
        .map_err(Into::into)
    }

    fn misbehaviour(
        _ctx: IbcClientCtx<Self>,
        _caller: Addr,
        _misbehaviour: Self::Misbehaviour,
        _relayer: Addr,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("math operation with overflow")]
pub struct MathOverflow;

pub fn set_total_voting_power(validator_set: &mut ValidatorSet) -> Result<(), MathOverflow> {
    validator_set.total_voting_power =
        validator_set
            .validators
            .iter()
            .try_fold(0_i64, |acc, val| {
                acc.checked_add(val.voting_power.inner())
                    .ok_or(MathOverflow)
            })?;
    Ok(())
}

pub fn construct_partial_header(
    chain_id: String,
    height: BoundedI64<0, { i64::MAX }>,
    time: unionlabs::google::protobuf::timestamp::Timestamp,
    next_validators_hash: H256<HexUnprefixed>,
) -> SignedHeader {
    SignedHeader {
        header: cometbft_types::types::header::Header {
            chain_id,
            time,
            next_validators_hash,
            height,
            version: Default::default(),
            last_block_id: Default::default(),
            last_commit_hash: Default::default(),
            data_hash: Default::default(),
            validators_hash: Default::default(),
            consensus_hash: Default::default(),
            app_hash: Default::default(),
            last_results_hash: Default::default(),
            evidence_hash: Default::default(),
            proposer_address: Default::default(),
        },
        commit: Commit {
            height,
            round: 0.try_into().expect("impossible"),
            block_id: Default::default(),
            signatures: Default::default(),
        },
    }
}

pub fn check_trusted_header(
    header: &tendermint_light_client_types::header::Header,
    next_validators_hash: &H256,
) -> Result<(), Error> {
    let val_hash = tendermint_verifier::utils::validators_hash(&header.trusted_validators);

    if &val_hash != next_validators_hash {
        Err(Error::TrustedValidatorsMismatch {
            calculated: val_hash,
            given: *next_validators_hash,
        })
    } else {
        Ok(())
    }
}

pub fn parse_revision_number(chain_id: &str) -> Option<u64> {
    chain_id
        .rsplit('-')
        .next()
        .map(|height_str| height_str.parse().ok())?
}

#[allow(clippy::too_many_arguments)]
pub fn verify_header<V: Verification>(
    mut client_state: ClientStateV1,
    consensus_state: ConsensusState,
    mut header: tendermint_light_client_types::header::Header,
    block_timestamp: cosmwasm_std::Timestamp,
    evm_timestamp: Timestamp,
    evm_state_root: H256,
    evm_storage_root: H256,
    mut signature_verifier: V,
) -> Result<StateUpdate<BerachainLightClient>, Error> {
    set_total_voting_power(&mut header.validator_set).unwrap();
    set_total_voting_power(&mut header.trusted_validators).unwrap();

    check_trusted_header(
        &header,
        consensus_state.comet_next_validators_hash.as_encoding(),
    )?;

    let revision_number = parse_revision_number(&header.signed_header.header.chain_id).ok_or(
        Error::InvalidChainId(header.signed_header.header.chain_id.clone()),
    )?;

    if revision_number != header.trusted_height.revision() {
        return Err(Error::RevisionNumberMismatch {
            trusted_revision_number: revision_number,
            header_revision_number: header.trusted_height.revision(),
        });
    }

    let signed_height = header
        .signed_header
        .header
        .height
        .inner()
        .try_into()
        .expect("value is bounded >= 0; qed;");

    if signed_height <= header.trusted_height.height() {
        return Err(Error::SignedHeaderHeightMustBeMoreRecent {
            signed_height,
            trusted_height: header.trusted_height.height(),
        });
    }

    // FIXME: unionlabs is tied to cosmwasm <2, the TryFrom impl can't be used
    let block_timestamp_proto = unionlabs::google::protobuf::timestamp::Timestamp {
        seconds: i64::try_from(block_timestamp.seconds())
            .expect("impossible")
            .try_into()
            .expect("impossible"),
        nanos: i32::try_from(block_timestamp.subsec_nanos())
            .expect("impossible")
            .try_into()
            .expect("impossible"),
    };

    tendermint_verifier::verify::verify(
        &construct_partial_header(
            client_state.chain_id.clone(),
            i64::try_from(header.trusted_height.height())
                .map_err(|_| Error::HeightTooLarge(header.trusted_height.height()))?
                .try_into()
                .expect(
                    "value is converted from u64, which is positive, \
                        and the expected bounded type is >= 0; qed;",
                ),
            unionlabs::google::protobuf::timestamp::Timestamp::from_unix_nanos(
                (consensus_state.comet_timestamp.as_nanos() as i128)
                    .try_into()
                    .expect("safe conversion"),
            ),
            consensus_state.comet_next_validators_hash,
        ),
        &header.trusted_validators,
        &header.signed_header,
        &header.validator_set,
        client_state.trusting_period,
        block_timestamp_proto,
        client_state.max_clock_drift,
        &client_state.trust_level,
        &mut signature_verifier,
    )
    .map_err(Error::TendermintVerify)?;

    let update_height = header
        .signed_header
        .header
        .height
        .inner()
        .try_into()
        .expect("impossible");

    let state_update = StateUpdate::new(
        update_height,
        ConsensusState {
            evm_timestamp,
            evm_state_root,
            evm_storage_root,
            comet_timestamp: Timestamp::from_nanos(
                header.signed_header.header.time.as_unix_nanos(),
            ),
            comet_root: MerkleRoot {
                hash: (*header.signed_header.header.app_hash.get()).into(),
            },
            comet_next_validators_hash: header.signed_header.header.next_validators_hash,
        },
    );

    if client_state.latest_height < update_height {
        client_state.latest_height = update_height;
        Ok(state_update.overwrite_client_state(ClientState::V1(client_state)))
    } else {
        Ok(state_update)
    }
}
