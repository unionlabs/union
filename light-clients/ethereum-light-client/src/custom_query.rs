use cosmwasm_std::{Binary, Deps, QueryRequest};
use ethereum_verifier::{crypto::BlsPublicKey, BlsVerify};

use crate::errors::Error;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CustomQuery {
    AggregateVerify {
        public_keys: Vec<Binary>,
        message: Binary,
        signature: Binary,
    },

    Aggregate {
        public_keys: Vec<Binary>,
    },
}

impl cosmwasm_std::CustomQuery for CustomQuery {}

pub struct VerificationContext<'a> {
    pub deps: Deps<'a, CustomQuery>,
}

impl<'a> BlsVerify for VerificationContext<'a> {
    fn fast_aggregate_verify(
        &self,
        public_keys: Vec<Vec<u8>>,
        msg: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<(), ethereum_verifier::Error> {
        let is_valid = query_fast_aggregate_verify(
            self.deps,
            public_keys.into_iter().map(Into::into).collect(),
            msg.into(),
            signature.into(),
        )
        .map_err(|e| ethereum_verifier::Error::CustomError(e.to_string()))?;

        if is_valid {
            Ok(())
        } else {
            Err(ethereum_verifier::Error::CustomError(
                "signature cannot be verified".to_string(),
            ))
        }
    }
}

pub fn query_fast_aggregate_verify(
    deps: Deps<CustomQuery>,
    public_keys: Vec<Binary>,
    message: Binary,
    signature: Binary,
) -> Result<bool, Error> {
    let request: QueryRequest<CustomQuery> = QueryRequest::Custom(CustomQuery::AggregateVerify {
        public_keys,
        message,
        signature,
    });

    deps.querier.query(&request).map_err(Error::custom_query)
}

pub fn query_aggregate_public_keys(
    deps: Deps<CustomQuery>,
    public_keys: Vec<Binary>,
) -> Result<BlsPublicKey, Error> {
    let request: QueryRequest<CustomQuery> =
        QueryRequest::Custom(CustomQuery::Aggregate { public_keys });

    let response: Binary = deps.querier.query(&request).map_err(Error::custom_query)?;

    response
        .0
        .as_slice()
        .try_into()
        .map_err(|_| Error::custom_query("Invalid public key type"))
}
