use core::fmt::Debug;

use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, StdError};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use unionlabs::{
    encoding::{Decode, DecodeAs, DecodeErrorOf, Encode, Encoding, Proto},
    google::protobuf::any::Any,
    ibc::{
        core::{
            client::{genesis_metadata::GenesisMetadata, height::Height},
            commitment::merkle_path::MerklePath,
        },
        lightclients::wasm,
    },
};

use crate::{
    msg::QueryMsg, CheckForMisbehaviourResult, EmptyResult, ExportMetadataResult, Status,
    StatusResult, SudoMsg, TimestampAtHeightResult, UpdateStateResult,
};

/// Signals that the client is not frozen
pub const ZERO_HEIGHT: Height = Height {
    revision_number: 0,
    revision_height: 0,
};

/// Signals that a client is frozen. This should be used as the `frozen_height`
/// when the client is needed to be frozen.
pub const FROZEN_HEIGHT: Height = Height {
    revision_number: 0,
    revision_height: 1,
};

pub enum StorageState {
    Occupied(Vec<u8>),
    Empty,
}

// TODO: Add #[source] to all variants
#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, thiserror::Error)]
pub enum DecodeError<T: IbcClient> {
    #[error("unable to decode header")]
    Header(DecodeErrorOf<T::Encoding, T::Header>),
    #[error("unable to decode misbehaviour")]
    Misbehaviour(DecodeErrorOf<T::Encoding, T::Misbehaviour>),
    #[error("unable to decode client state")]
    ClientState(DecodeErrorOf<T::Encoding, T::ClientState>),
    #[error("unable to decode consensus state")]
    ConsensusState(DecodeErrorOf<T::Encoding, T::ConsensusState>),

    #[error("unable to decode protobuf encoded `Any<Wasm<_>>` client state")]
    AnyWasmClientState(DecodeErrorOf<Proto, Any<wasm::client_state::ClientState<T::ClientState>>>),
    #[error("unable to decode protobuf encoded `Any<Wasm<_>>` consensus state")]
    AnyWasmConsensusState(
        DecodeErrorOf<Proto, Any<wasm::consensus_state::ConsensusState<T::ConsensusState>>>,
    ),
}

#[derive(DebugNoBound, PartialEqNoBound, thiserror::Error)]
pub enum IbcClientError<T: IbcClient> {
    #[error("decode error ({0:?})")]
    Decode(#[from] DecodeError<T>),
    #[error("std error ({0:?})")]
    Std(#[from] StdError),
    #[error("unexpected call from the host module ({0})")]
    UnexpectedCallDataFromHostModule(String),
    #[error("client state not found")]
    ClientStateNotFound,
    #[error(transparent)]
    ClientSpecific(T::Error),
    #[error("`ClientMessage` cannot be decoded ({data})", data = serde_utils::to_hex(.0))]
    InvalidClientMessage(Vec<u8>),
    #[error("consensus state not found at height `{0}`")]
    ConsensusStateNotFound(Height),
}

pub type WasmClientStateOf<T> = wasm::client_state::ClientState<<T as IbcClient>::ClientState>;
pub type WasmConsensusStateOf<T> =
    wasm::consensus_state::ConsensusState<<T as IbcClient>::ConsensusState>;

pub trait IbcClient: Sized {
    type Error: std::error::Error + PartialEq + Clone + Into<IbcClientError<Self>>;
    type CustomQuery: cosmwasm_std::CustomQuery;
    type Header: Decode<Self::Encoding, Error: PartialEq + Clone> + Debug + 'static;
    type Misbehaviour: Decode<Self::Encoding, Error: PartialEq + Clone> + Debug + 'static;
    type ClientState: Decode<Self::Encoding, Error: PartialEq + Clone>
        + Decode<Proto, Error: PartialEq + Clone + std::error::Error>
        + Encode<Proto>
        + Debug
        + 'static;
    type ConsensusState: Decode<Self::Encoding, Error: PartialEq + Clone>
        + Decode<Proto, Error: PartialEq + Clone + std::error::Error>
        + Encode<Proto>
        + Debug
        + 'static;
    type Encoding: Encoding;

    fn sudo(
        deps: DepsMut<Self::CustomQuery>,
        env: Env,
        msg: SudoMsg,
    ) -> Result<Binary, IbcClientError<Self>> {
        match msg {
            SudoMsg::VerifyMembership {
                height,
                delay_time_period,
                delay_block_period,
                proof,
                path,
                value,
            } => to_json_binary(&Self::verify_membership(
                deps.as_ref(),
                height,
                delay_time_period,
                delay_block_period,
                proof.into(),
                path,
                StorageState::Occupied(value.into()),
            )?),
            SudoMsg::VerifyNonMembership {
                height,
                delay_time_period,
                delay_block_period,
                proof,
                path,
            } => to_json_binary(&Self::verify_membership(
                deps.as_ref(),
                height,
                delay_time_period,
                delay_block_period,
                proof.into(),
                path,
                StorageState::Empty,
            )?),
            SudoMsg::UpdateState { client_message } => {
                if let Ok(header) = Self::Header::decode_as::<Self::Encoding>(&client_message) {
                    to_json_binary(&UpdateStateResult {
                        heights: Self::update_state(deps, env, header)?,
                    })
                } else {
                    return Err(IbcClientError::UnexpectedCallDataFromHostModule(
                        "`UpdateState` cannot be called with `Misbehaviour`".to_string(),
                    ));
                }
            }
            SudoMsg::UpdateStateOnMisbehaviour { client_message } => {
                Self::update_state_on_misbehaviour(deps, env, client_message.into())?;
                to_json_binary(&EmptyResult {})
            }
            SudoMsg::VerifyUpgradeAndUpdateState {
                upgrade_client_state,
                upgrade_consensus_state,
                proof_upgrade_client,
                proof_upgrade_consensus_state,
            } => {
                Self::verify_upgrade_and_update_state(
                    deps,
                    Self::ClientState::decode_as::<Self::Encoding>(upgrade_client_state.as_slice())
                        .map_err(DecodeError::ClientState)?,
                    Self::ConsensusState::decode_as::<Self::Encoding>(
                        upgrade_consensus_state.as_slice(),
                    )
                    .map_err(DecodeError::ConsensusState)?,
                    proof_upgrade_client.into(),
                    proof_upgrade_consensus_state.into(),
                )?;

                to_json_binary(&EmptyResult {})
            }
            SudoMsg::MigrateClientStore {} => {
                Self::migrate_client_store(deps)?;
                to_json_binary(&EmptyResult {})
            }
        }
        .map_err(Into::into)
    }

    fn query(
        deps: Deps<Self::CustomQuery>,
        env: Env,
        msg: QueryMsg,
    ) -> Result<Binary, IbcClientError<Self>> {
        match msg {
            QueryMsg::Status {} => {
                to_json_binary(&Into::<StatusResult>::into(Self::status(deps, &env)?))
            }
            QueryMsg::ExportMetadata {} => to_json_binary(&ExportMetadataResult {
                genesis_metadata: Self::export_metadata(deps, &env)?,
            }),
            QueryMsg::VerifyClientMessage { client_message } => {
                if let Ok(header) = Self::Header::decode_as::<Self::Encoding>(&client_message) {
                    to_json_binary(&Self::verify_header(deps, env, header)?)
                } else if let Ok(misbehaviour) =
                    Self::Misbehaviour::decode_as::<Self::Encoding>(&client_message)
                {
                    to_json_binary(&Self::verify_misbehaviour(deps, env, misbehaviour)?)
                } else {
                    return Err(IbcClientError::InvalidClientMessage(client_message.into()));
                }
            }
            QueryMsg::CheckForMisbehaviour { client_message } => {
                if let Ok(header) = Self::Header::decode_as::<Self::Encoding>(&client_message) {
                    to_json_binary(&CheckForMisbehaviourResult {
                        found_misbehaviour: Self::check_for_misbehaviour_on_header(deps, header)?,
                    })
                } else if let Ok(misbehaviour) =
                    Self::Misbehaviour::decode_as::<Self::Encoding>(&client_message)
                {
                    to_json_binary(&CheckForMisbehaviourResult {
                        found_misbehaviour: Self::check_for_misbehaviour_on_misbehaviour(
                            deps,
                            misbehaviour,
                        )?,
                    })
                } else {
                    return Err(IbcClientError::InvalidClientMessage(client_message.into()));
                }
            }
            QueryMsg::TimestampAtHeight { height } => to_json_binary(&TimestampAtHeightResult {
                timestamp: Self::timestamp_at_height(deps, height)?,
            }),
        }
        .map_err(Into::into)
    }

    #[allow(clippy::too_many_arguments)]
    fn verify_membership(
        deps: Deps<Self::CustomQuery>,
        height: Height,
        delay_time_period: u64,
        delay_block_period: u64,
        proof: Vec<u8>,
        path: MerklePath,
        value: StorageState,
    ) -> Result<(), IbcClientError<Self>>;

    fn verify_header(
        deps: Deps<Self::CustomQuery>,
        env: Env,
        header: Self::Header,
    ) -> Result<(), IbcClientError<Self>>;

    fn verify_misbehaviour(
        deps: Deps<Self::CustomQuery>,
        env: Env,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<(), IbcClientError<Self>>;

    fn update_state(
        deps: DepsMut<Self::CustomQuery>,
        env: Env,
        header: Self::Header,
    ) -> Result<Vec<Height>, IbcClientError<Self>>;

    /// `client_message` is being left without decoding because it could be either `Header`
    /// or `Misbehaviour` and it is generally not being used.
    // TODO: Use an enum generic over Header/ Misbehaviour
    fn update_state_on_misbehaviour(
        deps: DepsMut<Self::CustomQuery>,
        env: Env,
        client_message: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>>;

    fn check_for_misbehaviour_on_header(
        deps: Deps<Self::CustomQuery>,
        header: Self::Header,
    ) -> Result<bool, IbcClientError<Self>>;

    fn check_for_misbehaviour_on_misbehaviour(
        deps: Deps<Self::CustomQuery>,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<bool, IbcClientError<Self>>;

    fn verify_upgrade_and_update_state(
        deps: DepsMut<Self::CustomQuery>,
        upgrade_client_state: Self::ClientState,
        upgrade_consensus_state: Self::ConsensusState,
        proof_upgrade_client: Vec<u8>,
        proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>>;

    fn migrate_client_store(deps: DepsMut<Self::CustomQuery>) -> Result<(), IbcClientError<Self>>;

    fn status(deps: Deps<Self::CustomQuery>, env: &Env) -> Result<Status, IbcClientError<Self>>;

    fn export_metadata(
        deps: Deps<Self::CustomQuery>,
        env: &Env,
    ) -> Result<Vec<GenesisMetadata>, IbcClientError<Self>>;

    fn timestamp_at_height(
        deps: Deps<Self::CustomQuery>,
        height: Height,
    ) -> Result<u64, IbcClientError<Self>>;
}

pub type CustomQueryOf<T> = <T as IbcClient>::CustomQuery;
