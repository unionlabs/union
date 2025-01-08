// @generated
/// DenomAuthorityMetadata specifies metadata for addresses that have specific
/// capabilities over a token factory denom. Right now there is only one Admin
/// permission, but is planned to be extended to the future.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DenomAuthorityMetadata {
    /// Can be empty for no admin, or a valid address
    #[prost(string, tag = "1")]
    pub admin: ::prost::alloc::string::String,
}
impl ::prost::Name for DenomAuthorityMetadata {
    const NAME: &'static str = "DenomAuthorityMetadata";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
/// Params defines the parameters for the tokenfactory module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    #[prost(message, repeated, tag = "1")]
    pub denom_creation_fee: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
    /// if denom_creation_fee is an empty array, then this field is used to add more gas consumption
    /// to the base cost.
    /// <https://github.com/CosmWasm/token-factory/issues/11>
    #[prost(uint64, tag = "2")]
    pub denom_creation_gas_consume: u64,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
/// GenesisState defines the tokenfactory module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// params defines the paramaters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    #[prost(message, repeated, tag = "2")]
    pub factory_denoms: ::prost::alloc::vec::Vec<GenesisDenom>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
/// GenesisDenom defines a tokenfactory denom that is defined within genesis
/// state. The structure contains DenomAuthorityMetadata which defines the
/// denom's admin.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisDenom {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub authority_metadata: ::core::option::Option<DenomAuthorityMetadata>,
}
impl ::prost::Name for GenesisDenom {
    const NAME: &'static str = "GenesisDenom";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
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
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
/// QueryDenomAuthorityMetadataRequest defines the request structure for the
/// DenomAuthorityMetadata gRPC query.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomAuthorityMetadataRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDenomAuthorityMetadataRequest {
    const NAME: &'static str = "QueryDenomAuthorityMetadataRequest";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
/// QueryDenomAuthorityMetadataResponse defines the response structure for the
/// DenomAuthorityMetadata gRPC query.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomAuthorityMetadataResponse {
    #[prost(message, optional, tag = "1")]
    pub authority_metadata: ::core::option::Option<DenomAuthorityMetadata>,
}
impl ::prost::Name for QueryDenomAuthorityMetadataResponse {
    const NAME: &'static str = "QueryDenomAuthorityMetadataResponse";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
/// QueryDenomsFromCreatorRequest defines the request structure for the
/// DenomsFromCreator gRPC query.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomsFromCreatorRequest {
    #[prost(string, tag = "1")]
    pub creator: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryDenomsFromCreatorRequest {
    const NAME: &'static str = "QueryDenomsFromCreatorRequest";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
/// QueryDenomsFromCreatorRequest defines the response structure for the
/// DenomsFromCreator gRPC query.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDenomsFromCreatorResponse {
    #[prost(string, repeated, tag = "1")]
    pub denoms: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for QueryDenomsFromCreatorResponse {
    const NAME: &'static str = "QueryDenomsFromCreatorResponse";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
/// MsgCreateDenom defines the message structure for the CreateDenom gRPC service
/// method. It allows an account to create a new denom. It requires a sender
/// address and a sub denomination. The (sender_address, sub_denomination) tuple
/// must be unique and cannot be re-used.
///
/// The resulting denom created is defined as
/// <factory/{creatorAddress}/{subdenom}>. The resulting denom's admin is
/// originally set to be the creator, but this can be changed later. The token
/// denom does not indicate the current admin.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateDenom {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    /// subdenom can be up to 44 "alphanumeric" characters long.
    #[prost(string, tag = "2")]
    pub subdenom: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgCreateDenom {
    const NAME: &'static str = "MsgCreateDenom";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
/// MsgCreateDenomResponse is the return value of MsgCreateDenom
/// It returns the full string of the newly created denom
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateDenomResponse {
    #[prost(string, tag = "1")]
    pub new_token_denom: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgCreateDenomResponse {
    const NAME: &'static str = "MsgCreateDenomResponse";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
/// MsgMint is the sdk.Msg type for allowing an admin account to mint
/// more of a token.  For now, we only support minting to the sender account
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgMint {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub amount: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(string, tag = "3")]
    pub mint_to_address: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgMint {
    const NAME: &'static str = "MsgMint";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgMintResponse {}
impl ::prost::Name for MsgMintResponse {
    const NAME: &'static str = "MsgMintResponse";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
/// MsgBurn is the sdk.Msg type for allowing an admin account to burn
/// a token.  For now, we only support burning from the sender account.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBurn {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub amount: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(string, tag = "3")]
    pub burn_from_address: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgBurn {
    const NAME: &'static str = "MsgBurn";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBurnResponse {}
impl ::prost::Name for MsgBurnResponse {
    const NAME: &'static str = "MsgBurnResponse";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
/// MsgChangeAdmin is the sdk.Msg type for allowing an admin account to reassign
/// adminship of a denom to a new account
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChangeAdmin {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub denom: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub new_admin: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgChangeAdmin {
    const NAME: &'static str = "MsgChangeAdmin";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
/// MsgChangeAdminResponse defines the response structure for an executed
/// MsgChangeAdmin message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChangeAdminResponse {}
impl ::prost::Name for MsgChangeAdminResponse {
    const NAME: &'static str = "MsgChangeAdminResponse";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetDenomMetadata is the sdk.Msg type for allowing an admin account to set
/// the denom's bank metadata
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetDenomMetadata {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub metadata: ::core::option::Option<super::super::cosmos::bank::v1beta1::Metadata>,
}
impl ::prost::Name for MsgSetDenomMetadata {
    const NAME: &'static str = "MsgSetDenomMetadata";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
/// MsgSetDenomMetadataResponse defines the response structure for an executed
/// MsgSetDenomMetadata message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetDenomMetadataResponse {}
impl ::prost::Name for MsgSetDenomMetadataResponse {
    const NAME: &'static str = "MsgSetDenomMetadataResponse";
    const PACKAGE: &'static str = "tokenfactory.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tokenfactory.v1beta1.{}", Self::NAME)
    }
}
include!("tokenfactory.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
