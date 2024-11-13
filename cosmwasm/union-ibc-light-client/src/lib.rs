use core::fmt::Debug;

use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, StdError};
use frame_support_procedural::{CloneNoBound, PartialEqNoBound};
use union_ibc::lightclient::query::QueryMsg;
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

// TODO: Add #[source] to all variants
#[derive(macros::Debug, CloneNoBound, PartialEqNoBound, thiserror::Error)]
#[debug(bound())]
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

#[derive(macros::Debug, PartialEqNoBound, thiserror::Error)]
#[debug(bound())]
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

pub trait IbcClient: Sized {
    type Error: core::error::Error + PartialEq + Clone + Into<IbcClientError<Self>>;
    type CustomQuery: cosmwasm_std::CustomQuery;
    type Header: Decode<Self::Encoding, Error: Debug + PartialEq + Clone> + Debug + 'static;
    type Misbehaviour: Decode<Self::Encoding, Error: PartialEq + Clone> + Debug + 'static;
    type ClientState: Decode<Self::Encoding, Error: PartialEq + Clone>
        + Decode<Proto, Error: PartialEq + Clone + core::error::Error>
        + Encode<Proto>
        + Debug
        + 'static;
    type ConsensusState: Decode<Self::Encoding, Error: PartialEq + Clone>
        + Decode<Proto, Error: PartialEq + Clone + core::error::Error>
        + Encode<Proto>
        + Debug
        + 'static;
    type Encoding: Encoding;

    fn query(
        deps: Deps<Self::CustomQuery>,
        env: Env,
        msg: QueryMsg,
    ) -> Result<Binary, IbcClientError<Self>> {
        todo!()
    }
}
