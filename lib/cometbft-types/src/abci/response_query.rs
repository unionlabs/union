use serde::{Deserialize, Serialize};
use unionlabs::{bounded::BoundedI64, bytes::Bytes, hash::hash_v2::Base64};

use crate::crypto::proof_ops::ProofOps;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryResponse {
    pub code: u32,
    /// nondeterministic
    pub log: String,
    /// nondeterministic
    pub info: String,
    #[serde(with = "::serde_utils::string")]
    pub index: i64,
    pub key: Option<Bytes<Base64>>,
    pub value: Option<Bytes<Base64>>,
    pub proof_ops: Option<ProofOps>,
    #[serde(with = "::serde_utils::string")]
    pub height: BoundedI64<0, { i64::MAX }>,
    pub codespace: String,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::bounded::BoundedIntError;

    use crate::abci::response_query::QueryResponse;

    impl From<QueryResponse> for protos::cometbft::abci::v1::QueryResponse {
        fn from(value: QueryResponse) -> Self {
            Self {
                code: value.code,
                log: value.log,
                info: value.info,
                index: value.index,
                key: value.key.unwrap_or_default().into_vec(),
                value: value.value.unwrap_or_default().into_vec(),
                proof_ops: value.proof_ops.map(Into::into),
                height: value.height.inner(),
                codespace: value.codespace,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid height")]
        Height(#[source] BoundedIntError<i64>),
    }

    impl TryFrom<protos::cometbft::abci::v1::QueryResponse> for QueryResponse {
        type Error = Error;

        fn try_from(value: protos::cometbft::abci::v1::QueryResponse) -> Result<Self, Self::Error> {
            Ok(Self {
                code: value.code,
                log: value.log,
                info: value.info,
                index: value.index,
                key: Some(value.key.into()),
                value: Some(value.value.into()),
                proof_ops: value.proof_ops.map(Into::into),
                height: value.height.try_into().map_err(Error::Height)?,
                codespace: value.codespace,
            })
        }
    }
}
