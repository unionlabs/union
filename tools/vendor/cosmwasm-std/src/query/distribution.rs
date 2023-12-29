use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Addr;

use super::query_response::QueryResponseType;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DistributionQuery {
    /// See <https://github.com/cosmos/cosmos-sdk/blob/c74e2887b0b73e81d48c2f33e6b1020090089ee0/proto/cosmos/distribution/v1beta1/query.proto#L222-L230>
    DelegatorWithdrawAddress { delegator_address: String },
    /// See <https://github.com/cosmos/cosmos-sdk/blob/c74e2887b0b73e81d48c2f33e6b1020090089ee0/proto/cosmos/distribution/v1beta1/query.proto#L157-L167>
    #[cfg(feature = "cosmwasm_1_4")]
    DelegationRewards {
        delegator_address: String,
        validator_address: String,
    },
    /// See <https://github.com/cosmos/cosmos-sdk/blob/c74e2887b0b73e81d48c2f33e6b1020090089ee0/proto/cosmos/distribution/v1beta1/query.proto#L180-L187>
    #[cfg(feature = "cosmwasm_1_4")]
    DelegationTotalRewards { delegator_address: String },
    /// See <https://github.com/cosmos/cosmos-sdk/blob/b0acf60e6c39f7ab023841841fc0b751a12c13ff/proto/cosmos/distribution/v1beta1/query.proto#L202-L210>
    #[cfg(feature = "cosmwasm_1_4")]
    DelegatorValidators { delegator_address: String },
}

/// See <https://github.com/cosmos/cosmos-sdk/blob/c74e2887b0b73e81d48c2f33e6b1020090089ee0/proto/cosmos/distribution/v1beta1/query.proto#L232-L240>
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct DelegatorWithdrawAddressResponse {
    pub withdraw_address: Addr,
}

impl_response_constructor!(DelegatorWithdrawAddressResponse, withdraw_address: Addr);
impl QueryResponseType for DelegatorWithdrawAddressResponse {}

/// See <https://github.com/cosmos/cosmos-sdk/blob/c74e2887b0b73e81d48c2f33e6b1020090089ee0/proto/cosmos/distribution/v1beta1/query.proto#L169-L178>
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DelegationRewardsResponse {
    pub rewards: Vec<DecCoin>,
}

impl_response_constructor!(DelegationRewardsResponse, rewards: Vec<DecCoin>);
impl QueryResponseType for DelegationRewardsResponse {}

/// A coin type with decimal amount.
/// Modeled after the Cosmos SDK's [DecCoin] type.
/// However, in contrast to the Cosmos SDK the `amount` string MUST always have a dot at JSON level,
/// see <https://github.com/cosmos/cosmos-sdk/issues/10863>.
/// Also if Cosmos SDK choses to migrate away from fixed point decimals
/// (as shown [here](https://github.com/cosmos/cosmos-sdk/blob/v0.47.4/x/group/internal/math/dec.go#L13-L21 and discussed [here](https://github.com/cosmos/cosmos-sdk/issues/11783)),
/// wasmd needs to truncate the decimal places to 18.
///
/// [DecCoin]: (https://github.com/cosmos/cosmos-sdk/blob/v0.47.4/proto/cosmos/base/v1beta1/coin.proto#L28-L38)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DecCoin {
    pub denom: String,
    /// An amount in the base denom of the distributed token.
    ///
    /// Some chains have choosen atto (10^-18) for their token's base denomination. If we used `Decimal` here, we could only store
    /// 340282366920938463463.374607431768211455atoken which is 340.28 TOKEN.
    pub amount: crate::Decimal256,
}

impl DecCoin {
    pub fn new(amount: crate::Decimal256, denom: impl Into<String>) -> Self {
        Self {
            denom: denom.into(),
            amount,
        }
    }
}

/// See <https://github.com/cosmos/cosmos-sdk/blob/c74e2887b0b73e81d48c2f33e6b1020090089ee0/proto/cosmos/distribution/v1beta1/query.proto#L189-L200>
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[non_exhaustive]
pub struct DelegationTotalRewardsResponse {
    pub rewards: Vec<DelegatorReward>,
    pub total: Vec<DecCoin>,
}

impl_response_constructor!(
    DelegationTotalRewardsResponse,
    rewards: Vec<DelegatorReward>,
    total: Vec<DecCoin>
);
impl QueryResponseType for DelegationTotalRewardsResponse {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct DelegatorReward {
    pub validator_address: String,
    pub reward: Vec<DecCoin>,
}

/// See <https://github.com/cosmos/cosmos-sdk/blob/b0acf60e6c39f7ab023841841fc0b751a12c13ff/proto/cosmos/distribution/v1beta1/query.proto#L212-L220>
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct DelegatorValidatorsResponse {
    pub validators: Vec<String>,
}

impl_response_constructor!(DelegatorValidatorsResponse, validators: Vec<String>);
impl QueryResponseType for DelegatorValidatorsResponse {}
