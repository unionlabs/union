// @generated
/// Minter represents the minting state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Minter {
    /// epoch_provisions represent rewards for the current epoch.
    #[prost(string, tag = "1")]
    pub epoch_provisions: ::prost::alloc::string::String,
}
impl ::prost::Name for Minter {
    const NAME: &'static str = "Minter";
    const PACKAGE: &'static str = "osmosis.mint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.mint.v1beta1.{}", Self::NAME)
    }
}
/// WeightedAddress represents an address with a weight assigned to it.
/// The weight is used to determine the proportion of the total minted
/// tokens to be minted to the address.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WeightedAddress {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub weight: ::prost::alloc::string::String,
}
impl ::prost::Name for WeightedAddress {
    const NAME: &'static str = "WeightedAddress";
    const PACKAGE: &'static str = "osmosis.mint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.mint.v1beta1.{}", Self::NAME)
    }
}
/// DistributionProportions defines the distribution proportions of the minted
/// denom. In other words, defines which stakeholders will receive the minted
/// denoms and how much.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DistributionProportions {
    /// staking defines the proportion of the minted mint_denom that is to be
    /// allocated as staking rewards.
    #[prost(string, tag = "1")]
    pub staking: ::prost::alloc::string::String,
    /// pool_incentives defines the proportion of the minted mint_denom that is
    /// to be allocated as pool incentives.
    #[prost(string, tag = "2")]
    pub pool_incentives: ::prost::alloc::string::String,
    /// developer_rewards defines the proportion of the minted mint_denom that is
    /// to be allocated to developer rewards address.
    #[prost(string, tag = "3")]
    pub developer_rewards: ::prost::alloc::string::String,
    /// community_pool defines the proportion of the minted mint_denom that is
    /// to be allocated to the community pool.
    #[prost(string, tag = "4")]
    pub community_pool: ::prost::alloc::string::String,
}
impl ::prost::Name for DistributionProportions {
    const NAME: &'static str = "DistributionProportions";
    const PACKAGE: &'static str = "osmosis.mint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.mint.v1beta1.{}", Self::NAME)
    }
}
/// Params holds parameters for the x/mint module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// mint_denom is the denom of the coin to mint.
    #[prost(string, tag = "1")]
    pub mint_denom: ::prost::alloc::string::String,
    /// genesis_epoch_provisions epoch provisions from the first epoch.
    #[prost(string, tag = "2")]
    pub genesis_epoch_provisions: ::prost::alloc::string::String,
    /// epoch_identifier mint epoch identifier e.g. (day, week).
    #[prost(string, tag = "3")]
    pub epoch_identifier: ::prost::alloc::string::String,
    /// reduction_period_in_epochs the number of epochs it takes
    /// to reduce the rewards.
    #[prost(int64, tag = "4")]
    pub reduction_period_in_epochs: i64,
    /// reduction_factor is the reduction multiplier to execute
    /// at the end of each period set by reduction_period_in_epochs.
    #[prost(string, tag = "5")]
    pub reduction_factor: ::prost::alloc::string::String,
    /// distribution_proportions defines the distribution proportions of the minted
    /// denom. In other words, defines which stakeholders will receive the minted
    /// denoms and how much.
    #[prost(message, optional, tag = "6")]
    pub distribution_proportions: ::core::option::Option<DistributionProportions>,
    /// weighted_developer_rewards_receivers is the address to receive developer
    /// rewards with weights assignedt to each address. The final amount that each
    /// address receives is: epoch_provisions *
    /// distribution_proportions.developer_rewards * Address's Weight.
    #[prost(message, repeated, tag = "7")]
    pub weighted_developer_rewards_receivers: ::prost::alloc::vec::Vec<WeightedAddress>,
    /// minting_rewards_distribution_start_epoch start epoch to distribute minting
    /// rewards
    #[prost(int64, tag = "8")]
    pub minting_rewards_distribution_start_epoch: i64,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "osmosis.mint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.mint.v1beta1.{}", Self::NAME)
    }
}
/// GenesisState defines the mint module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// minter is an abstraction for holding current rewards information.
    #[prost(message, optional, tag = "1")]
    pub minter: ::core::option::Option<Minter>,
    /// params defines all the parameters of the mint module.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
    /// reduction_started_epoch is the first epoch in which the reduction of mint
    /// begins.
    #[prost(int64, tag = "3")]
    pub reduction_started_epoch: i64,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "osmosis.mint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.mint.v1beta1.{}", Self::NAME)
    }
}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "osmosis.mint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.mint.v1beta1.{}", Self::NAME)
    }
}
/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params defines the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "osmosis.mint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.mint.v1beta1.{}", Self::NAME)
    }
}
/// QueryEpochProvisionsRequest is the request type for the
/// Query/EpochProvisions RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEpochProvisionsRequest {}
impl ::prost::Name for QueryEpochProvisionsRequest {
    const NAME: &'static str = "QueryEpochProvisionsRequest";
    const PACKAGE: &'static str = "osmosis.mint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.mint.v1beta1.{}", Self::NAME)
    }
}
/// QueryEpochProvisionsResponse is the response type for the
/// Query/EpochProvisions RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEpochProvisionsResponse {
    /// epoch_provisions is the current minting per epoch provisions value.
    #[prost(bytes = "vec", tag = "1")]
    pub epoch_provisions: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for QueryEpochProvisionsResponse {
    const NAME: &'static str = "QueryEpochProvisionsResponse";
    const PACKAGE: &'static str = "osmosis.mint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.mint.v1beta1.{}", Self::NAME)
    }
}
include!("osmosis.mint.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
