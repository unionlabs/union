use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "stargate")]
use crate::Binary;
use crate::Empty;

/// Implements a hidden constructor for query responses.
macro_rules! impl_response_constructor {
    ( $response:ty, $( $field: ident : $t: ty),* ) => {
        impl $response {
            /// Constructor for testing frameworks such as cw-multi-test.
            /// This is required because query response types should be #[non_exhaustive].
            /// As a contract developer you should not need this constructor since
            /// query responses are constructed for you via deserialization.
            ///
            /// Warning: This can change in breaking ways in minor versions.
            #[doc(hidden)]
            #[allow(dead_code)]
            pub fn new($( $field: $t),*) -> Self {
                Self { $( $field ),* }
            }
        }
    };
}

mod bank;
mod distribution;
mod ibc;
mod query_response;
mod staking;
mod wasm;

pub use bank::*;
pub use distribution::*;
pub use ibc::*;
pub use staking::*;
pub use wasm::*;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryRequest<C> {
    Bank(BankQuery),
    Custom(C),
    #[cfg(feature = "staking")]
    Staking(StakingQuery),
    #[cfg(feature = "cosmwasm_1_3")]
    Distribution(DistributionQuery),
    /// A Stargate query is encoded the same way as abci_query, with path and protobuf encoded request data.
    /// The format is defined in [ADR-21](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-021-protobuf-query-encoding.md).
    /// The response is protobuf encoded data directly without a JSON response wrapper.
    /// The caller is responsible for compiling the proper protobuf definitions for both requests and responses.
    #[cfg(feature = "stargate")]
    Stargate {
        /// this is the fully qualified service path used for routing,
        /// eg. custom/cosmos_sdk.x.bank.v1.Query/QueryBalance
        path: String,
        /// this is the expected protobuf message type (not any), binary encoded
        data: Binary,
    },
    #[cfg(feature = "stargate")]
    Ibc(IbcQuery),
    Wasm(WasmQuery),
}

/// A trait that is required to avoid conflicts with other query types like BankQuery and WasmQuery
/// in generic implementations.
/// You need to implement it in your custom query type.
///
/// # Examples
///
/// ```
/// # use cosmwasm_std::CustomQuery;
/// # use schemars::JsonSchema;
/// # use serde::{Deserialize, Serialize};
/// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// #[serde(rename_all = "snake_case")]
/// pub enum MyCustomQuery {
///     Ping {},
///     Capitalized { text: String },
/// }
///
/// impl CustomQuery for MyCustomQuery {}
/// ```
pub trait CustomQuery: Serialize + Clone {}
// We require `Clone` because `Clone` in `QueryRequest<C>` is only derived for
// `C: Clone` and we want consistent behaviour for all `QueryRequest<C>`

impl CustomQuery for Empty {}

impl<C: CustomQuery> From<BankQuery> for QueryRequest<C> {
    fn from(msg: BankQuery) -> Self {
        QueryRequest::Bank(msg)
    }
}

impl<C: CustomQuery> From<C> for QueryRequest<C> {
    fn from(msg: C) -> Self {
        QueryRequest::Custom(msg)
    }
}

#[cfg(feature = "staking")]
impl<C: CustomQuery> From<StakingQuery> for QueryRequest<C> {
    fn from(msg: StakingQuery) -> Self {
        QueryRequest::Staking(msg)
    }
}

impl<C: CustomQuery> From<WasmQuery> for QueryRequest<C> {
    fn from(msg: WasmQuery) -> Self {
        QueryRequest::Wasm(msg)
    }
}

#[cfg(feature = "stargate")]
impl<C: CustomQuery> From<IbcQuery> for QueryRequest<C> {
    fn from(msg: IbcQuery) -> Self {
        QueryRequest::Ibc(msg)
    }
}

#[cfg(feature = "cosmwasm_1_3")]
impl<C: CustomQuery> From<DistributionQuery> for QueryRequest<C> {
    fn from(msg: DistributionQuery) -> Self {
        QueryRequest::Distribution(msg)
    }
}
