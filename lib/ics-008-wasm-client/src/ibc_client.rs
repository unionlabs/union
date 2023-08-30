use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo};
use unionlabs::{
    ibc::core::client::height::Height, TryFromProto, TryFromProtoBytesError, TryFromProtoErrorOf,
};

use crate::{
    msg::{ClientMessage, ContractResult, ExecuteMsg, MerklePath, QueryMsg, QueryResponse},
    Error,
};

pub enum StorageState {
    Occupied(Vec<u8>),
    Empty,
}

pub trait IBCClient {
    type Error: From<TryFromProtoBytesError<TryFromProtoErrorOf<Self::Header>>>
        + From<TryFromProtoBytesError<TryFromProtoErrorOf<Self::Misbehaviour>>>
        + From<Error>;
    type CustomQuery: cosmwasm_std::CustomQuery;
    type Header: TryFromProto;
    type Misbehaviour: TryFromProto;
    type ClientState;
    type ConsensusState;

    fn execute(
        deps: DepsMut<Self::CustomQuery>,
        _env: Env,
        _info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<ContractResult, Self::Error> {
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
            ExecuteMsg::VerifyClientMessage {
                client_message:
                    ClientMessage {
                        header,
                        misbehaviour,
                    },
            } => {
                if let Some(header) = header {
                    let header = Self::Header::try_from_proto_bytes(&header.data)?;
                    Self::verify_header(deps.as_ref(), header)
                } else if let Some(misbehaviour) = misbehaviour {
                    let misbehaviour =
                        Self::Misbehaviour::try_from_proto_bytes(&misbehaviour.data)?;
                    Self::verify_misbehaviour(deps.as_ref(), misbehaviour)
                } else {
                    // Note(aeryz): There is nothing in spec that makes either of the fields to present
                    Ok(ContractResult::valid(None))
                }
            }
            ExecuteMsg::UpdateState {
                client_message: ClientMessage { header, .. },
            } => {
                if let Some(header) = header {
                    let header = Self::Header::try_from_proto_bytes(&header.data)?;
                    Self::update_state(deps, header)
                } else {
                    Err(Error::NotSpecCompilant(
                        "`UpdateState` is not valid for misbehaviour".to_string(),
                    )
                    .into())
                }
            }
            _ => Ok(ContractResult::valid(None)),
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
        header: Self::Header,
    ) -> Result<ContractResult, Self::Error>;

    fn verify_misbehaviour(
        deps: Deps<Self::CustomQuery>,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<ContractResult, Self::Error>;

    fn update_state(
        deps: DepsMut<Self::CustomQuery>,
        header: Self::Header,
    ) -> Result<ContractResult, Self::Error>;

    // TODO(aeryz): make this client message generic over the underlying types
    fn update_state_on_misbeviour(
        deps: Deps<Self::CustomQuery>,
        client_message: ClientMessage,
    ) -> Result<ContractResult, Self::Error>;

    fn check_for_misbehaviour(
        deps: Deps<Self::CustomQuery>,
        client_message: ClientMessage,
    ) -> Result<ContractResult, Self::Error>;

    fn verify_upgrade_and_update_state(
        deps: Deps<Self::CustomQuery>,
        upgrade_client_state: Self::ClientState,
        upgrade_consensus_state: Self::ConsensusState,
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
