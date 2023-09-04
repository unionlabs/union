use core::fmt::Debug;

use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo};
use unionlabs::{
    ibc::{
        core::client::height::Height,
        lightclients::wasm::{client_state::ClientState, consensus_state::ConsensusState},
    },
    Proto, TryFromProto, TryFromProtoBytesError, TryFromProtoErrorOf,
};

use crate::{
    msg::{ClientMessage, ContractResult, ExecuteMsg, MerklePath, QueryMsg, QueryResponse},
    Error,
};

pub enum StorageState {
    Occupied(Vec<u8>),
    Empty,
}

pub trait IbcClient {
    type Error: From<TryFromProtoBytesError<TryFromProtoErrorOf<Self::Header>>> + From<Error>;
    type CustomQuery: cosmwasm_std::CustomQuery;
    // TODO(aeryz): see #583
    type Header: TryFromProto;
    // TODO(aeryz): see #583, #588
    type Misbehaviour;
    type ClientState: TryFromProto;
    type ConsensusState: TryFromProto;

    fn execute(
        deps: DepsMut<Self::CustomQuery>,
        env: Env,
        _info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<ContractResult, Self::Error>
    where
        // NOTE(aeryz): unfortunately bounding to `Debug` in associated type creates a
        // recursion in the compiler, see this issue: https://github.com/rust-lang/rust/issues/87755
        <Self::ClientState as Proto>::Proto: prost::Message + Default,
        TryFromProtoErrorOf<Self::ClientState>: Debug,
        <Self::ConsensusState as Proto>::Proto: prost::Message + Default,
        TryFromProtoErrorOf<Self::ConsensusState>: Debug,
    {
        match msg {
            ExecuteMsg::VerifyMembership {
                height,
                delay_time_period,
                delay_block_period,
                proof,
                path,
                value,
            } => Self::verify_membership(
                deps.as_ref(),
                height,
                delay_time_period,
                delay_block_period,
                proof,
                path,
                StorageState::Occupied(value.0),
            ),
            ExecuteMsg::VerifyNonMembership {
                height,
                delay_time_period,
                delay_block_period,
                proof,
                path,
            } => Self::verify_membership(
                deps.as_ref(),
                height,
                delay_time_period,
                delay_block_period,
                proof,
                path,
                StorageState::Empty,
            ),
            ExecuteMsg::VerifyClientMessage { client_message } => match client_message {
                ClientMessage::Header(header) => {
                    let header = Self::Header::try_from_proto_bytes(&header.data)?;
                    Self::verify_header(deps.as_ref(), env, header)
                }
                ClientMessage::Misbehaviour(_misbehaviour) => {
                    Ok(ContractResult::invalid("Not implemented".to_string()))
                }
            },
            ExecuteMsg::UpdateState { client_message } => match client_message {
                ClientMessage::Header(header) => {
                    let header = Self::Header::try_from_proto_bytes(&header.data)?;
                    Self::update_state(deps, env, header)
                }
                ClientMessage::Misbehaviour(_) => Err(Error::UnexpectedCallDataFromHostModule(
                    "`UpdateState` cannot be called with `Misbehaviour`".to_string(),
                )
                .into()),
            },
            ExecuteMsg::UpdateStateOnMisbehaviour { client_message } => {
                Self::update_state_on_misbehaviour(deps, client_message)
            }
            ExecuteMsg::CheckForMisbehaviour { client_message } => match client_message {
                ClientMessage::Header(header) => {
                    let header = Self::Header::try_from_proto_bytes(&header.data)?;
                    Self::verify_header(deps.as_ref(), env, header)
                }
                ClientMessage::Misbehaviour(_) => {
                    Ok(ContractResult::invalid("Not implemented".to_string()))
                }
            },
            ExecuteMsg::VerifyUpgradeAndUpdateState {
                upgrade_client_state,
                upgrade_consensus_state,
                proof_upgrade_client,
                proof_upgrade_consensus_state,
            } => Self::verify_upgrade_and_update_state(
                deps,
                <_>::try_from_proto(upgrade_client_state)
                    .map_err(|err| Error::Decode(format!("{err:?}")))?,
                <_>::try_from_proto(upgrade_consensus_state)
                    .map_err(|err| Error::Decode(format!("{err:?}")))?,
                proof_upgrade_client,
                proof_upgrade_consensus_state,
            ),
            ExecuteMsg::CheckSubstituteAndUpdateState {} => {
                Self::check_substitute_and_update_state(deps.as_ref())
            }
        }
    }

    fn query(
        deps: Deps<Self::CustomQuery>,
        env: Env,
        msg: QueryMsg,
    ) -> Result<QueryResponse, Self::Error> {
        match msg {
            QueryMsg::Status {} => Self::status(deps, &env),
            QueryMsg::ExportMetadata {} => Self::export_metadata(deps, &env),
        }
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
    ) -> Result<ContractResult, Self::Error>;

    fn verify_header(
        deps: Deps<Self::CustomQuery>,
        env: Env,
        header: Self::Header,
    ) -> Result<ContractResult, Self::Error>;

    fn verify_misbehaviour(
        deps: Deps<Self::CustomQuery>,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<ContractResult, Self::Error>;

    fn update_state(
        deps: DepsMut<Self::CustomQuery>,
        env: Env,
        header: Self::Header,
    ) -> Result<ContractResult, Self::Error>;

    // TODO(aeryz): make this client message generic over the underlying types
    fn update_state_on_misbehaviour(
        deps: DepsMut<Self::CustomQuery>,
        client_message: ClientMessage,
    ) -> Result<ContractResult, Self::Error>;

    fn check_for_misbehaviour_on_header(
        deps: Deps<Self::CustomQuery>,
        header: Self::Header,
    ) -> Result<ContractResult, Self::Error>;

    fn check_for_misbehaviour_on_misbehaviour(
        deps: Deps<Self::CustomQuery>,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<ContractResult, Self::Error>;

    fn verify_upgrade_and_update_state(
        deps: DepsMut<Self::CustomQuery>,
        upgrade_client_state: ClientState<Self::ClientState>,
        upgrade_consensus_state: ConsensusState<Self::ConsensusState>,
        proof_upgrade_client: Binary,
        proof_upgrade_consensus_state: Binary,
    ) -> Result<ContractResult, Self::Error>;

    fn check_substitute_and_update_state(
        deps: Deps<Self::CustomQuery>,
    ) -> Result<ContractResult, Self::Error>;

    fn status(deps: Deps<Self::CustomQuery>, env: &Env) -> Result<QueryResponse, Self::Error>;

    fn export_metadata(
        deps: Deps<Self::CustomQuery>,
        env: &Env,
    ) -> Result<QueryResponse, Self::Error>;
}
