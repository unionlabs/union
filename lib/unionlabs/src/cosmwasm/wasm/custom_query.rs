use std::fmt::Debug;

use cosmwasm_std::{Binary, Deps, QueryRequest};

use crate::{
    bls::BlsPublicKey, google::protobuf::any::Any, ibc::core::client::height::Height, TryFromProto,
    TryFromProtoBytesError, TryFromProtoErrorOf,
};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("error while running `fast_aggregate_verify` query ({0})")]
    FastAggregateVerify(String),
    #[error("error while running `aggregate_public_keys` query ({0})")]
    AggregatePublicKeys(String),
    #[error("invalid public key is returned from `aggregate_public_key`")]
    InvalidAggregatePublicKey,
    #[error("error while running `consensus_state` query ({0})")]
    ConsensusState(String),
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum UnionCustomQuery {
    AggregateVerify {
        public_keys: Vec<Binary>,
        message: Binary,
        signature: Binary,
    },
    Aggregate {
        public_keys: Vec<Binary>,
    },
    ConsensusState {
        client_id: String,
        height: Height,
    },
    ClientState {
        client_id: String,
    },
}

impl cosmwasm_std::CustomQuery for UnionCustomQuery {}

pub fn query_fast_aggregate_verify(
    deps: Deps<UnionCustomQuery>,
    public_keys: Vec<Binary>,
    message: Binary,
    signature: Binary,
) -> Result<bool, Error> {
    let request: QueryRequest<UnionCustomQuery> =
        QueryRequest::Custom(UnionCustomQuery::AggregateVerify {
            public_keys,
            message,
            signature,
        });
    deps.querier
        .query(&request)
        .map_err(|e| Error::FastAggregateVerify(e.to_string()).into())
}

pub fn query_aggregate_public_keys(
    deps: Deps<UnionCustomQuery>,
    public_keys: Vec<BlsPublicKey>,
) -> Result<BlsPublicKey, Error> {
    let request: QueryRequest<UnionCustomQuery> =
        QueryRequest::Custom(UnionCustomQuery::Aggregate {
            public_keys: public_keys.into_iter().map(|x| Binary(x.into())).collect(),
        });
    let response: Binary = deps
        .querier
        .query(&request)
        .map_err(|e| Error::AggregatePublicKeys(e.to_string()))?;
    response
        .0
        .as_slice()
        .try_into()
        .map_err(|_| Error::InvalidAggregatePublicKey.into())
}

pub fn query_consensus_state<T>(
    deps: Deps<UnionCustomQuery>,
    client_id: String,
    height: Height,
) -> Result<T, Error>
where
    Any<T>: TryFromProto,
    TryFromProtoBytesError<TryFromProtoErrorOf<Any<T>>>: Debug,
{
    let consensus_state_data = deps
        .querier
        .query::<Binary>(&QueryRequest::Custom(UnionCustomQuery::ConsensusState {
            client_id,
            height,
        }))
        .map_err(|e| Error::ConsensusState(e.to_string()).into())?;
    let Any(consensus_state) = Any::<T>::try_from_proto_bytes(&consensus_state_data)
        .map_err(|e| Error::ConsensusState(format!("{:?}", e)).into())?;
    Ok(consensus_state)
}
