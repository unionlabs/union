use core::fmt::{Debug, Display};

use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, StdError};
use protos::ibc::core::client::v1::GenesisMetadata;
use unionlabs::{
    ibc::{
        core::client::height::Height,
        lightclients::wasm::{client_state::ClientState, consensus_state::ConsensusState},
    },
    Proto, TryFromProto, TryFromProtoBytesError, TryFromProtoErrorOf,
};

use crate::{
    msg::{ClientMessage, MerklePath, QueryMsg},
    CheckForMisbehaviourResult, EmptyResult, Error, ExportMetadataResult, Status, StatusResult,
    SudoMsg, TimestampAtHeightResult, UpdateStateResult,
};

pub enum StorageState {
    Occupied(Vec<u8>),
    Empty,
}

pub trait IbcClient {
    type Error: From<TryFromProtoBytesError<TryFromProtoErrorOf<Self::Header>>>
        + From<Error>
        + From<StdError>
        + Display;
    type CustomQuery: cosmwasm_std::CustomQuery;
    // TODO(aeryz): see #583
    type Header: TryFromProto;
    // TODO(aeryz): see #583, #588
    type Misbehaviour;
    type ClientState: TryFromProto;
    type ConsensusState: TryFromProto;

    fn sudo(deps: DepsMut<Self::CustomQuery>, env: Env, msg: SudoMsg) -> Result<Binary, Self::Error>
    where
        // NOTE(aeryz): unfortunately bounding to `Debug` in associated type creates a
        // recursion in the compiler, see this issue: https://github.com/rust-lang/rust/issues/87755
        <Self::ClientState as Proto>::Proto: prost::Message + Default,
        TryFromProtoErrorOf<Self::ClientState>: Debug,
        <Self::ConsensusState as Proto>::Proto: prost::Message + Default,
        TryFromProtoErrorOf<Self::ConsensusState>: Debug,
    {
        match msg {
            SudoMsg::VerifyMembership {
                height,
                delay_time_period,
                delay_block_period,
                proof,
                path,
                value,
            } => to_binary(&Self::verify_membership(
                deps.as_ref(),
                height,
                delay_time_period,
                delay_block_period,
                proof,
                path,
                StorageState::Occupied(value.0),
            )?),
            SudoMsg::VerifyNonMembership {
                height,
                delay_time_period,
                delay_block_period,
                proof,
                path,
            } => to_binary(&Self::verify_membership(
                deps.as_ref(),
                height,
                delay_time_period,
                delay_block_period,
                proof,
                path,
                StorageState::Empty,
            )?),
            SudoMsg::UpdateState { client_message } => {
                if let Ok(header) =
                    <Self::Header as TryFromProto>::try_from_proto_bytes(&client_message.data)
                {
                    to_binary(&Self::update_state(deps, env, header)?)
                } else {
                    Err(Error::UnexpectedCallDataFromHostModule(
                        "`UpdateState` cannot be called with `Misbehaviour`".to_string(),
                    ))?
                }
            }
            SudoMsg::UpdateStateOnMisbehaviour { client_message } => {
                Self::update_state_on_misbehaviour(deps, env, client_message)?;
                to_binary(&EmptyResult {})
            }
            SudoMsg::VerifyUpgradeAndUpdateState {
                upgrade_client_state,
                upgrade_consensus_state,
                proof_upgrade_client,
                proof_upgrade_consensus_state,
            } => {
                Self::verify_upgrade_and_update_state(
                    deps,
                    <_>::try_from_proto(upgrade_client_state)
                        .map_err(|err| Error::Decode(format!("{err:?}")))?,
                    <_>::try_from_proto(upgrade_consensus_state)
                        .map_err(|err| Error::Decode(format!("{err:?}")))?,
                    proof_upgrade_client,
                    proof_upgrade_consensus_state,
                )?;

                to_binary(&EmptyResult {})
            }
            SudoMsg::MigrateClientStore {} => {
                Self::migrate_client_store(deps.as_ref())?;
                to_binary(&EmptyResult {})
            }
        }
        .map_err(Into::into)
    }

    fn query(
        deps: Deps<Self::CustomQuery>,
        env: Env,
        msg: QueryMsg,
    ) -> Result<Binary, Self::Error> {
        match msg {
            QueryMsg::Status {} => {
                to_binary(&Into::<StatusResult>::into(Self::status(deps, &env)?))
            }
            QueryMsg::ExportMetadata {} => to_binary(&ExportMetadataResult {
                genesis_metadata: Self::export_metadata(deps, &env)?,
            }),
            QueryMsg::VerifyClientMessage { client_message } => {
                if let Ok(header) =
                    <Self::Header as TryFromProto>::try_from_proto_bytes(&client_message.data)
                {
                    to_binary(&Self::verify_header(deps, env, header)?)
                } else {
                    Err(Error::UnexpectedCallDataFromHostModule(
                        "`UpdateState` cannot be called with `Misbehaviour`".to_string(),
                    ))?
                }
            }
            QueryMsg::CheckForMisbehaviour { client_message } => {
                if let Ok(header) =
                    <Self::Header as TryFromProto>::try_from_proto_bytes(&client_message.data)
                {
                    to_binary(&Self::verify_header(deps, env, header)?)
                } else {
                    Err(Error::UnexpectedCallDataFromHostModule(
                        "`UpdateState` cannot be called with `Misbehaviour`".to_string(),
                    ))?
                }
            }
            QueryMsg::TimestampAtHeight { height } => to_binary(&TimestampAtHeightResult {
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
        proof: Binary,
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
    ) -> Result<UpdateStateResult, Self::Error>;

    // TODO(aeryz): make this client message generic over the underlying types
    fn update_state_on_misbehaviour(
        deps: DepsMut<Self::CustomQuery>,
        env: Env,
        client_message: ClientMessage,
    ) -> Result<(), Self::Error>;

    fn check_for_misbehaviour_on_header(
        deps: Deps<Self::CustomQuery>,
        header: Self::Header,
    ) -> Result<CheckForMisbehaviourResult, Self::Error>;

    fn check_for_misbehaviour_on_misbehaviour(
        deps: Deps<Self::CustomQuery>,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<CheckForMisbehaviourResult, Self::Error>;

    fn verify_upgrade_and_update_state(
        deps: DepsMut<Self::CustomQuery>,
        upgrade_client_state: ClientState<Self::ClientState>,
        upgrade_consensus_state: ConsensusState<Self::ConsensusState>,
        proof_upgrade_client: Binary,
        proof_upgrade_consensus_state: Binary,
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
