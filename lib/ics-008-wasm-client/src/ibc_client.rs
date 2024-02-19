use core::fmt::Debug;

use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, StdError};
use frame_support_procedural::DebugNoBound;
use unionlabs::{
    encoding::{Decode, Encoding},
    ibc::core::{
        client::{genesis_metadata::GenesisMetadata, height::Height},
        commitment::merkle_path::MerklePath,
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

#[derive(DebugNoBound)]
pub enum DecodeError<T: IbcClient> {
    Header(<T::Header as Decode<T::Encoding>>::Error),
    Misbehaviour(<T::Misbehaviour as Decode<T::Encoding>>::Error),
    ClientState(<T::ClientState as Decode<T::Encoding>>::Error),
    ConsensusState(<T::ConsensusState as Decode<T::Encoding>>::Error),
}

#[derive(thiserror::Error, DebugNoBound)]
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
}

pub trait IbcClient: Sized {
    type Error: std::error::Error;
    type CustomQuery: cosmwasm_std::CustomQuery;
    type Header: Decode<Self::Encoding> + Debug;
    type Misbehaviour: Decode<Self::Encoding> + Debug;
    type ClientState: Decode<Self::Encoding> + Debug;
    type ConsensusState: Decode<Self::Encoding> + Debug;
    type Encoding: Encoding;

    fn sudo(
        deps: DepsMut<Self::CustomQuery>,
        env: Env,
        msg: SudoMsg,
    ) -> Result<Binary, IbcClientError<Self>>
where {
        match msg {
            SudoMsg::VerifyMembership {
                height,
                delay_time_period,
                delay_block_period,
                proof,
                path,
                value,
            } => to_json_binary(
                &Self::verify_membership(
                    deps.as_ref(),
                    height,
                    delay_time_period,
                    delay_block_period,
                    proof.into(),
                    path,
                    StorageState::Occupied(value.0),
                )
                .map_err(IbcClientError::ClientSpecific)?,
            ),
            SudoMsg::VerifyNonMembership {
                height,
                delay_time_period,
                delay_block_period,
                proof,
                path,
            } => to_json_binary(
                &Self::verify_membership(
                    deps.as_ref(),
                    height,
                    delay_time_period,
                    delay_block_period,
                    proof.into(),
                    path,
                    StorageState::Empty,
                )
                .map_err(IbcClientError::ClientSpecific)?,
            ),
            SudoMsg::UpdateState { client_message } => {
                if let Ok(header) =
                    <Self::Header as Decode<Self::Encoding>>::decode(&client_message.0)
                {
                    to_json_binary(&UpdateStateResult {
                        heights: Self::update_state(deps, env, header)
                            .map_err(IbcClientError::ClientSpecific)?,
                    })
                } else {
                    return Err(IbcClientError::UnexpectedCallDataFromHostModule(
                        "`UpdateState` cannot be called with `Misbehaviour`".to_string(),
                    ));
                }
            }
            SudoMsg::UpdateStateOnMisbehaviour { client_message } => {
                Self::update_state_on_misbehaviour(deps, env, client_message.0)
                    .map_err(IbcClientError::ClientSpecific)?;
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
                    <Self::ClientState as Decode<Self::Encoding>>::decode(
                        upgrade_client_state.as_slice(),
                    )
                    .map_err(DecodeError::ClientState)?,
                    <Self::ConsensusState as Decode<Self::Encoding>>::decode(
                        upgrade_consensus_state.as_slice(),
                    )
                    .map_err(DecodeError::ConsensusState)?,
                    proof_upgrade_client.into(),
                    proof_upgrade_consensus_state.into(),
                )
                .map_err(IbcClientError::ClientSpecific)?;

                to_json_binary(&EmptyResult {})
            }
            SudoMsg::MigrateClientStore {} => {
                Self::migrate_client_store(deps.as_ref())
                    .map_err(IbcClientError::ClientSpecific)?;
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
            QueryMsg::Status {} => to_json_binary(&Into::<StatusResult>::into(
                Self::status(deps, &env).map_err(IbcClientError::ClientSpecific)?,
            )),
            QueryMsg::ExportMetadata {} => to_json_binary(&ExportMetadataResult {
                genesis_metadata: Self::export_metadata(deps, &env)
                    .map_err(IbcClientError::ClientSpecific)?,
            }),
            QueryMsg::VerifyClientMessage { client_message } => {
                if let Ok(header) =
                    <Self::Header as Decode<Self::Encoding>>::decode(&client_message.0)
                {
                    to_json_binary(
                        &Self::verify_header(deps, env, header)
                            .map_err(IbcClientError::ClientSpecific)?,
                    )
                } else if let Ok(misbehaviour) =
                    <Self::Misbehaviour as Decode<Self::Encoding>>::decode(&client_message.0)
                {
                    to_json_binary(
                        &Self::verify_misbehaviour(deps, misbehaviour)
                            .map_err(IbcClientError::ClientSpecific)?,
                    )
                } else {
                    return Err(IbcClientError::InvalidClientMessage(client_message.0));
                }
            }
            QueryMsg::CheckForMisbehaviour { client_message } => {
                if let Ok(header) =
                    <Self::Header as Decode<Self::Encoding>>::decode(&client_message.0)
                {
                    to_json_binary(&CheckForMisbehaviourResult {
                        found_misbehaviour: Self::check_for_misbehaviour_on_header(deps, header)
                            .map_err(IbcClientError::ClientSpecific)?,
                    })
                } else if let Ok(misbehaviour) =
                    <Self::Misbehaviour as Decode<Self::Encoding>>::decode(&client_message.0)
                {
                    to_json_binary(&CheckForMisbehaviourResult {
                        found_misbehaviour: Self::check_for_misbehaviour_on_misbehaviour(
                            deps,
                            misbehaviour,
                        )
                        .map_err(IbcClientError::ClientSpecific)?,
                    })
                } else {
                    return Err(IbcClientError::InvalidClientMessage(client_message.0));
                }
            }
            QueryMsg::TimestampAtHeight { height } => to_json_binary(&TimestampAtHeightResult {
                timestamp: Self::timestamp_at_height(deps, height)
                    .map_err(IbcClientError::ClientSpecific)?,
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
    ) -> Result<(), Self::Error>;

    fn verify_header(
        deps: Deps<Self::CustomQuery>,
        env: Env,
        header: Self::Header,
    ) -> Result<(), Self::Error>;

    fn verify_misbehaviour(
        deps: Deps<Self::CustomQuery>,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<(), Self::Error>;

    fn update_state(
        deps: DepsMut<Self::CustomQuery>,
        env: Env,
        header: Self::Header,
    ) -> Result<Vec<Height>, Self::Error>;

    /// `client_message` is being left without decoding because it could be either `Header`
    /// or `Misbehaviour` and it is generally not being used.
    fn update_state_on_misbehaviour(
        deps: DepsMut<Self::CustomQuery>,
        env: Env,
        client_message: Vec<u8>,
    ) -> Result<(), Self::Error>;

    fn check_for_misbehaviour_on_header(
        deps: Deps<Self::CustomQuery>,
        header: Self::Header,
    ) -> Result<bool, Self::Error>;

    fn check_for_misbehaviour_on_misbehaviour(
        deps: Deps<Self::CustomQuery>,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<bool, Self::Error>;

    fn verify_upgrade_and_update_state(
        deps: DepsMut<Self::CustomQuery>,
        upgrade_client_state: Self::ClientState,
        upgrade_consensus_state: Self::ConsensusState,
        proof_upgrade_client: Vec<u8>,
        proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<(), Self::Error>;

    fn migrate_client_store(deps: Deps<Self::CustomQuery>) -> Result<(), Self::Error>;

    fn status(deps: Deps<Self::CustomQuery>, env: &Env) -> Result<Status, Self::Error>;

    fn export_metadata(
        deps: Deps<Self::CustomQuery>,
        env: &Env,
    ) -> Result<Vec<GenesisMetadata>, Self::Error>;

    fn timestamp_at_height(
        deps: Deps<Self::CustomQuery>,
        height: Height,
    ) -> Result<u64, Self::Error>;
}
