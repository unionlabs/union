use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Addr, Coin, Decimal};

use super::query_response::QueryResponseType;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StakingQuery {
    /// Returns the denomination that can be bonded (if there are multiple native tokens on the chain)
    BondedDenom {},
    /// AllDelegations will return all delegations by the delegator
    AllDelegations { delegator: String },
    /// Delegation will return more detailed info on a particular
    /// delegation, defined by delegator/validator pair
    Delegation {
        delegator: String,
        validator: String,
    },
    /// Returns all validators in the currently active validator set.
    ///
    /// The query response type is `AllValidatorsResponse`.
    AllValidators {},
    /// Returns the validator at the given address. Returns None if the validator is
    /// not part of the currently active validator set.
    ///
    /// The query response type is `ValidatorResponse`.
    Validator {
        /// The validator's address (e.g. (e.g. cosmosvaloper1...))
        address: String,
    },
}

/// BondedDenomResponse is data format returned from StakingRequest::BondedDenom query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BondedDenomResponse {
    pub denom: String,
}

impl QueryResponseType for BondedDenomResponse {}

impl_response_constructor!(BondedDenomResponse, denom: String);

/// DelegationsResponse is data format returned from StakingRequest::AllDelegations query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllDelegationsResponse {
    pub delegations: Vec<Delegation>,
}

impl QueryResponseType for AllDelegationsResponse {}

impl_response_constructor!(AllDelegationsResponse, delegations: Vec<Delegation>);

/// Delegation is basic (cheap to query) data about a delegation.
///
/// Instances are created in the querier.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Delegation {
    pub delegator: Addr,
    /// A validator address (e.g. cosmosvaloper1...)
    pub validator: String,
    /// How much we have locked in the delegation
    pub amount: Coin,
}

impl From<FullDelegation> for Delegation {
    fn from(full: FullDelegation) -> Self {
        Delegation {
            delegator: full.delegator,
            validator: full.validator,
            amount: full.amount,
        }
    }
}

/// DelegationResponse is data format returned from StakingRequest::Delegation query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DelegationResponse {
    pub delegation: Option<FullDelegation>,
}

impl QueryResponseType for DelegationResponse {}

impl_response_constructor!(DelegationResponse, delegation: Option<FullDelegation>);

/// FullDelegation is all the info on the delegation, some (like accumulated_reward and can_redelegate)
/// is expensive to query.
///
/// Instances are created in the querier.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct FullDelegation {
    pub delegator: Addr,
    /// A validator address (e.g. cosmosvaloper1...)
    pub validator: String,
    /// How much we have locked in the delegation
    pub amount: Coin,
    /// can_redelegate captures how much can be immediately redelegated.
    /// 0 is no redelegation and can_redelegate == amount is redelegate all
    /// but there are many places between the two
    pub can_redelegate: Coin,
    /// How much we can currently withdraw
    pub accumulated_rewards: Vec<Coin>,
}

/// The data format returned from StakingRequest::AllValidators query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AllValidatorsResponse {
    pub validators: Vec<Validator>,
}

impl QueryResponseType for AllValidatorsResponse {}

impl_response_constructor!(AllValidatorsResponse, validators: Vec<Validator>);

/// The data format returned from StakingRequest::Validator query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ValidatorResponse {
    pub validator: Option<Validator>,
}

impl QueryResponseType for ValidatorResponse {}

impl_response_constructor!(ValidatorResponse, validator: Option<Validator>);

/// Instances are created in the querier.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Validator {
    /// The operator address of the validator (e.g. cosmosvaloper1...).
    /// See https://github.com/cosmos/cosmos-sdk/blob/v0.47.4/proto/cosmos/staking/v1beta1/staking.proto#L95-L96
    /// for more information.
    ///
    /// This uses `String` instead of `Addr` since the bech32 address prefix is different from
    /// the ones that regular user accounts use.
    pub address: String,
    pub commission: Decimal,
    pub max_commission: Decimal,
    /// The maximum daily increase of the commission
    pub max_change_rate: Decimal,
}
