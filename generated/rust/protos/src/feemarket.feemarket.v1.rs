// @generated
/// Params contains the required set of parameters for the EIP1559 fee market
/// plugin implementation.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// Alpha is the amount we additively increase the learning rate
    /// when it is above or below the target +/- threshold.
    ///
    /// Must be > 0.
    #[prost(string, tag = "1")]
    pub alpha: ::prost::alloc::string::String,
    /// Beta is the amount we multiplicatively decrease the learning rate
    /// when it is within the target +/- threshold.
    ///
    /// Must be \[0, 1\].
    #[prost(string, tag = "2")]
    pub beta: ::prost::alloc::string::String,
    /// Gamma is the threshold for the learning rate. If the learning rate is
    /// above or below the target +/- threshold, we additively increase the
    /// learning rate by Alpha. Otherwise, we multiplicatively decrease the
    /// learning rate by Beta.
    ///
    /// Must be \[0, 0.5\].
    #[prost(string, tag = "3")]
    pub gamma: ::prost::alloc::string::String,
    /// Delta is the amount we additively increase/decrease the gas price when the
    /// net block utilization difference in the window is above/below the target
    /// utilization.
    #[prost(string, tag = "4")]
    pub delta: ::prost::alloc::string::String,
    /// MinBaseGasPrice determines the initial gas price of the module and the
    /// global minimum for the network.
    #[prost(string, tag = "5")]
    pub min_base_gas_price: ::prost::alloc::string::String,
    /// MinLearningRate is the lower bound for the learning rate.
    #[prost(string, tag = "6")]
    pub min_learning_rate: ::prost::alloc::string::String,
    /// MaxLearningRate is the upper bound for the learning rate.
    #[prost(string, tag = "7")]
    pub max_learning_rate: ::prost::alloc::string::String,
    /// MaxBlockUtilization is the maximum block utilization.
    #[prost(uint64, tag = "8")]
    pub max_block_utilization: u64,
    /// Window defines the window size for calculating an adaptive learning rate
    /// over a moving window of blocks.
    #[prost(uint64, tag = "9")]
    pub window: u64,
    /// FeeDenom is the denom that will be used for all fee payments.
    #[prost(string, tag = "10")]
    pub fee_denom: ::prost::alloc::string::String,
    /// Enabled is a boolean that determines whether the EIP1559 fee market is
    /// enabled.
    #[prost(bool, tag = "11")]
    pub enabled: bool,
    /// DistributeFees is a boolean that determines whether the fees are burned or
    /// distributed to all stakers.
    #[prost(bool, tag = "12")]
    pub distribute_fees: bool,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "feemarket.feemarket.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("feemarket.feemarket.v1.{}", Self::NAME)
    }
}
/// GenesisState defines the feemarket module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// Params are the parameters for the feemarket module. These parameters
    /// can be utilized to implement both the base EIP-1559 fee market and
    /// and the AIMD EIP-1559 fee market.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    /// State contains the current state of the AIMD fee market.
    #[prost(message, optional, tag = "2")]
    pub state: ::core::option::Option<State>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "feemarket.feemarket.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("feemarket.feemarket.v1.{}", Self::NAME)
    }
}
/// State is utilized to track the current state of the fee market. This includes
/// the current base fee, learning rate, and block utilization within the
/// specified AIMD window.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct State {
    /// BaseGasPrice is the current base fee. This is denominated in the fee per
    /// gas unit.
    #[prost(string, tag = "1")]
    pub base_gas_price: ::prost::alloc::string::String,
    /// LearningRate is the current learning rate.
    #[prost(string, tag = "2")]
    pub learning_rate: ::prost::alloc::string::String,
    /// Window contains a list of the last blocks' utilization values. This is used
    /// to calculate the next base fee. This stores the number of units of gas
    /// consumed per block.
    #[prost(uint64, repeated, tag = "3")]
    pub window: ::prost::alloc::vec::Vec<u64>,
    /// Index is the index of the current block in the block utilization window.
    #[prost(uint64, tag = "4")]
    pub index: u64,
}
impl ::prost::Name for State {
    const NAME: &'static str = "State";
    const PACKAGE: &'static str = "feemarket.feemarket.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("feemarket.feemarket.v1.{}", Self::NAME)
    }
}
/// ParamsRequest is the request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ParamsRequest {}
impl ::prost::Name for ParamsRequest {
    const NAME: &'static str = "ParamsRequest";
    const PACKAGE: &'static str = "feemarket.feemarket.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("feemarket.feemarket.v1.{}", Self::NAME)
    }
}
/// ParamsResponse is the response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ParamsResponse {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for ParamsResponse {
    const NAME: &'static str = "ParamsResponse";
    const PACKAGE: &'static str = "feemarket.feemarket.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("feemarket.feemarket.v1.{}", Self::NAME)
    }
}
/// StateRequest is the request type for the Query/State RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StateRequest {}
impl ::prost::Name for StateRequest {
    const NAME: &'static str = "StateRequest";
    const PACKAGE: &'static str = "feemarket.feemarket.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("feemarket.feemarket.v1.{}", Self::NAME)
    }
}
/// StateResponse is the response type for the Query/State RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StateResponse {
    #[prost(message, optional, tag = "1")]
    pub state: ::core::option::Option<State>,
}
impl ::prost::Name for StateResponse {
    const NAME: &'static str = "StateResponse";
    const PACKAGE: &'static str = "feemarket.feemarket.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("feemarket.feemarket.v1.{}", Self::NAME)
    }
}
/// GasPriceRequest is the request type for the Query/GasPrice RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GasPriceRequest {
    /// denom we are querying gas price in
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for GasPriceRequest {
    const NAME: &'static str = "GasPriceRequest";
    const PACKAGE: &'static str = "feemarket.feemarket.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("feemarket.feemarket.v1.{}", Self::NAME)
    }
}
/// GasPriceResponse is the response type for the Query/GasPrice RPC method.
/// Returns a gas price in specified denom.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GasPriceResponse {
    #[prost(message, optional, tag = "1")]
    pub price: ::core::option::Option<super::super::super::cosmos::base::v1beta1::DecCoin>,
}
impl ::prost::Name for GasPriceResponse {
    const NAME: &'static str = "GasPriceResponse";
    const PACKAGE: &'static str = "feemarket.feemarket.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("feemarket.feemarket.v1.{}", Self::NAME)
    }
}
/// GasPriceRequest is the request type for the Query/GasPrices RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GasPricesRequest {}
impl ::prost::Name for GasPricesRequest {
    const NAME: &'static str = "GasPricesRequest";
    const PACKAGE: &'static str = "feemarket.feemarket.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("feemarket.feemarket.v1.{}", Self::NAME)
    }
}
/// GasPricesResponse is the response type for the Query/GasPrices RPC method.
/// Returns a gas price in all available denoms.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GasPricesResponse {
    #[prost(message, repeated, tag = "1")]
    pub prices: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::DecCoin>,
}
impl ::prost::Name for GasPricesResponse {
    const NAME: &'static str = "GasPricesResponse";
    const PACKAGE: &'static str = "feemarket.feemarket.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("feemarket.feemarket.v1.{}", Self::NAME)
    }
}
/// MsgParams defines the Msg/Params request type. It contains the
/// new parameters for the feemarket module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgParams {
    /// Params defines the new parameters for the feemarket module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    /// Authority defines the authority that is updating the feemarket module
    /// parameters.
    #[prost(string, tag = "2")]
    pub authority: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgParams {
    const NAME: &'static str = "MsgParams";
    const PACKAGE: &'static str = "feemarket.feemarket.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("feemarket.feemarket.v1.{}", Self::NAME)
    }
}
/// MsgParamsResponse defines the Msg/Params response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgParamsResponse {}
impl ::prost::Name for MsgParamsResponse {
    const NAME: &'static str = "MsgParamsResponse";
    const PACKAGE: &'static str = "feemarket.feemarket.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("feemarket.feemarket.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
