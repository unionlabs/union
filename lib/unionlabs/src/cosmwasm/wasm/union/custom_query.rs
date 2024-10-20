use core::fmt::Debug;

use cosmwasm_std::{Binary, Deps, QueryRequest};

#[cfg(feature = "stargate")]
use crate::ibc::core::client::height::Height;
use crate::{bls::BlsPublicKey, ics24::Path};

#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum Error {
    #[error("error while running `fast_aggregate_verify` query ({0})")]
    // REVIEW: Why not put the StdError in this directly?
    FastAggregateVerify(String),
    #[error("error while running `aggregate_public_keys` query ({0})")]
    AggregatePublicKeys(String),
    #[error("invalid public key is returned from `aggregate_public_key`")]
    InvalidAggregatePublicKey,
    #[error("abci query for {path} failed: {err}")]
    ABCI { path: Path, err: String },
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
        .map_err(|e| Error::FastAggregateVerify(e.to_string()))
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
        .map_err(|_| Error::InvalidAggregatePublicKey)
}

#[cfg(feature = "stargate")]
use {
    crate::{
        encoding::{Decode, DecodeAs, Proto},
        google::protobuf::any::Any,
        ics24::{ClientConsensusStatePath, ClientStatePath},
    },
    cosmwasm_std::{to_json_vec, ContractResult, Env, SystemResult},
    prost::Message,
    protos::cosmos::base::tendermint::v1beta1::AbciQueryResponse,
};

#[allow(clippy::missing_panics_doc)]
#[cfg(feature = "stargate")]
pub fn query_ibc_abci<T>(deps: Deps<UnionCustomQuery>, env: &Env, path: Path) -> Result<T, Error>
where
    Any<T>: Decode<Proto>,
{
    let query = protos::cosmos::base::tendermint::v1beta1::AbciQueryRequest {
        data: path.clone().to_string().into_bytes(),
        path: "store/ibc/key".to_string(),
        height: env
            .block
            .height
            .wrapping_sub(1)
            .try_into()
            .expect("impossible"),
        prove: false,
    };
    let raw = to_json_vec(&QueryRequest::<UnionCustomQuery>::Stargate {
        path: "/cosmos.base.tendermint.v1beta1.Service/ABCIQuery".into(),
        data: query.encode_to_vec().into(),
    })
    .map_err(|e| Error::ABCI {
        path: path.clone(),
        err: format!("{e:?}"),
    })?;
    let abci_response_data = match deps.querier.raw_query(&raw) {
        SystemResult::Err(system_err) => Err(Error::ABCI {
            path: path.clone(),
            err: format!("Querier system error: {system_err}"),
        }),
        SystemResult::Ok(ContractResult::Err(contract_err)) => Err(Error::ABCI {
            path: path.clone(),
            err: format!("Querier contract error: {contract_err}"),
        }),
        SystemResult::Ok(ContractResult::Ok(value)) => Ok(value),
    }?;
    let abci_response =
        AbciQueryResponse::decode(abci_response_data.as_ref()).map_err(|e| Error::ABCI {
            path: path.clone(),
            err: format!("AbciQueryResponse decoding: {e:?}"),
        })?;
    let Any(value) =
        Any::<T>::decode_as::<Proto>(&abci_response.value).map_err(|e| Error::ABCI {
            path,
            err: format!("AnyProto decoding: {e:?}"),
        })?;
    Ok(value)
}

#[allow(clippy::missing_panics_doc)]
#[cfg(feature = "stargate")]
pub fn query_consensus_state<T>(
    deps: Deps<UnionCustomQuery>,
    env: &Env,
    // TODO: Use ClientId here
    client_id: String,
    height: Height,
) -> Result<T, Error>
where
    Any<T>: Decode<Proto>,
{
    use crate::validated::ValidateT;

    query_ibc_abci::<T>(
        deps,
        env,
        Path::ClientConsensusState(ClientConsensusStatePath {
            client_id: client_id.validate().expect("invalid client id"),
            height,
        }),
    )
}

#[allow(clippy::missing_panics_doc)]
#[cfg(feature = "stargate")]
pub fn query_client_state<T>(
    deps: Deps<UnionCustomQuery>,
    env: &Env,
    // TODO: Use ClientId here
    client_id: String,
) -> Result<T, Error>
where
    Any<T>: Decode<Proto>,
{
    use crate::validated::ValidateT;

    query_ibc_abci::<T>(
        deps,
        env,
        Path::ClientState(ClientStatePath {
            client_id: client_id.validate().expect("invalid client id"),
        }),
    )
}
