// @generated
/// BaseAccount defines a base account type. It contains all the necessary fields
/// for basic account functionality. Any custom account type should extend this
/// type for additional functionality (e.g. vesting).
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BaseAccount {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub pub_key: ::core::option::Option<::pbjson_types::Any>,
    #[prost(uint64, tag = "3")]
    pub account_number: u64,
    #[prost(uint64, tag = "4")]
    pub sequence: u64,
}
impl ::prost::Name for BaseAccount {
    const NAME: &'static str = "BaseAccount";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// ModuleAccount defines an account for modules that holds coins on a pool.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModuleAccount {
    #[prost(message, optional, tag = "1")]
    pub base_account: ::core::option::Option<BaseAccount>,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "3")]
    pub permissions: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for ModuleAccount {
    const NAME: &'static str = "ModuleAccount";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// ModuleCredential represents a unclaimable pubkey for base accounts controlled by modules.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModuleCredential {
    /// module_name is the name of the module used for address derivation (passed into address.Module).
    #[prost(string, tag = "1")]
    pub module_name: ::prost::alloc::string::String,
    /// derivation_keys is for deriving a module account address (passed into address.Module)
    /// adding more keys creates sub-account addresses (passed into address.Derive)
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub derivation_keys: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for ModuleCredential {
    const NAME: &'static str = "ModuleCredential";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// Params defines the parameters for the auth module.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    #[prost(uint64, tag = "1")]
    pub max_memo_characters: u64,
    #[prost(uint64, tag = "2")]
    pub tx_sig_limit: u64,
    #[prost(uint64, tag = "3")]
    pub tx_size_cost_per_byte: u64,
    #[prost(uint64, tag = "4")]
    pub sig_verify_cost_ed25519: u64,
    #[prost(uint64, tag = "5")]
    pub sig_verify_cost_secp256k1: u64,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// MsgUpdateParams is the Msg/UpdateParams request type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// authority is the address that controls the module (defaults to x/gov unless overwritten).
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the x/auth parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// MsgUpdateParamsResponse defines the response structure for executing a
/// MsgUpdateParams message.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// MsgNonAtomicExec defines the Msg/NonAtomicExec request type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgNonAtomicExec {
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub msgs: ::prost::alloc::vec::Vec<::pbjson_types::Any>,
}
impl ::prost::Name for MsgNonAtomicExec {
    const NAME: &'static str = "MsgNonAtomicExec";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// NonAtomicExecResult defines the response structure for executing a
/// MsgNonAtomicExec.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NonAtomicExecResult {
    #[prost(string, tag = "1")]
    pub error: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub resp: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for NonAtomicExecResult {
    const NAME: &'static str = "NonAtomicExecResult";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// MsgNonAtomicExecResponse defines the response of MsgNonAtomicExec.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgNonAtomicExecResponse {
    #[prost(message, repeated, tag = "1")]
    pub results: ::prost::alloc::vec::Vec<NonAtomicExecResult>,
}
impl ::prost::Name for MsgNonAtomicExecResponse {
    const NAME: &'static str = "MsgNonAtomicExecResponse";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// MsgMigrateAccount defines a message which allows users to migrate from BaseAccount
/// to other x/accounts types.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgMigrateAccount {
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub account_type: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub account_init_msg: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for MsgMigrateAccount {
    const NAME: &'static str = "MsgMigrateAccount";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// MsgMigrateAccountResponse defines the response given when migrating to
/// an x/accounts account.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgMigrateAccountResponse {
    /// init_response defines the response returned by the x/account account
    /// initialization.
    #[prost(message, optional, tag = "1")]
    pub init_response: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for MsgMigrateAccountResponse {
    const NAME: &'static str = "MsgMigrateAccountResponse";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// QueryAccountsRequest is the request type for the Query/Accounts RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAccountsRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryAccountsRequest {
    const NAME: &'static str = "QueryAccountsRequest";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// QueryAccountsResponse is the response type for the Query/Accounts RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAccountsResponse {
    /// accounts are the existing accounts
    #[prost(message, repeated, tag = "1")]
    pub accounts: ::prost::alloc::vec::Vec<::pbjson_types::Any>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryAccountsResponse {
    const NAME: &'static str = "QueryAccountsResponse";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// QueryAccountRequest is the request type for the Query/Account RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAccountRequest {
    /// address defines the address to query for.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryAccountRequest {
    const NAME: &'static str = "QueryAccountRequest";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// QueryAccountResponse is the response type for the Query/Account RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAccountResponse {
    /// account defines the account of the corresponding address.
    #[prost(message, optional, tag = "1")]
    pub account: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for QueryAccountResponse {
    const NAME: &'static str = "QueryAccountResponse";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params defines the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// QueryModuleAccountsRequest is the request type for the Query/ModuleAccounts RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryModuleAccountsRequest {}
impl ::prost::Name for QueryModuleAccountsRequest {
    const NAME: &'static str = "QueryModuleAccountsRequest";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// QueryModuleAccountsResponse is the response type for the Query/ModuleAccounts RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryModuleAccountsResponse {
    #[prost(message, repeated, tag = "1")]
    pub accounts: ::prost::alloc::vec::Vec<::pbjson_types::Any>,
}
impl ::prost::Name for QueryModuleAccountsResponse {
    const NAME: &'static str = "QueryModuleAccountsResponse";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// QueryModuleAccountByNameRequest is the request type for the Query/ModuleAccountByName RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryModuleAccountByNameRequest {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryModuleAccountByNameRequest {
    const NAME: &'static str = "QueryModuleAccountByNameRequest";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// QueryModuleAccountByNameResponse is the response type for the Query/ModuleAccountByName RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryModuleAccountByNameResponse {
    #[prost(message, optional, tag = "1")]
    pub account: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for QueryModuleAccountByNameResponse {
    const NAME: &'static str = "QueryModuleAccountByNameResponse";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// Bech32PrefixRequest is the request type for Bech32Prefix rpc method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bech32PrefixRequest {}
impl ::prost::Name for Bech32PrefixRequest {
    const NAME: &'static str = "Bech32PrefixRequest";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// Bech32PrefixResponse is the response type for Bech32Prefix rpc method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bech32PrefixResponse {
    #[prost(string, tag = "1")]
    pub bech32_prefix: ::prost::alloc::string::String,
}
impl ::prost::Name for Bech32PrefixResponse {
    const NAME: &'static str = "Bech32PrefixResponse";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// AddressBytesToStringRequest is the request type for AddressString rpc method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddressBytesToStringRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub address_bytes: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for AddressBytesToStringRequest {
    const NAME: &'static str = "AddressBytesToStringRequest";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// AddressBytesToStringResponse is the response type for AddressString rpc method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddressBytesToStringResponse {
    #[prost(string, tag = "1")]
    pub address_string: ::prost::alloc::string::String,
}
impl ::prost::Name for AddressBytesToStringResponse {
    const NAME: &'static str = "AddressBytesToStringResponse";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// AddressStringToBytesRequest is the request type for AccountBytes rpc method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddressStringToBytesRequest {
    #[prost(string, tag = "1")]
    pub address_string: ::prost::alloc::string::String,
}
impl ::prost::Name for AddressStringToBytesRequest {
    const NAME: &'static str = "AddressStringToBytesRequest";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// AddressStringToBytesResponse is the response type for AddressBytes rpc method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddressStringToBytesResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub address_bytes: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for AddressStringToBytesResponse {
    const NAME: &'static str = "AddressStringToBytesResponse";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// QueryAccountAddressByIDRequest is the request type for AccountAddressByID rpc method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAccountAddressByIdRequest {
    /// Deprecated, use account_id instead
    ///
    /// id is the account number of the address to be queried. This field
    /// should have been an uint64 (like all account numbers), and will be
    /// updated to uint64 in a future version of the auth query.
    #[deprecated]
    #[prost(int64, tag = "1")]
    pub id: i64,
    /// account_id is the account number of the address to be queried.
    #[prost(uint64, tag = "2")]
    pub account_id: u64,
}
impl ::prost::Name for QueryAccountAddressByIdRequest {
    const NAME: &'static str = "QueryAccountAddressByIDRequest";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// QueryAccountAddressByIDResponse is the response type for AccountAddressByID rpc method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAccountAddressByIdResponse {
    #[prost(string, tag = "1")]
    pub account_address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryAccountAddressByIdResponse {
    const NAME: &'static str = "QueryAccountAddressByIDResponse";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// QueryAccountInfoRequest is the Query/AccountInfo request type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAccountInfoRequest {
    /// address is the account address string.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryAccountInfoRequest {
    const NAME: &'static str = "QueryAccountInfoRequest";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// QueryAccountInfoResponse is the Query/AccountInfo response type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAccountInfoResponse {
    /// info is the account info which is represented by BaseAccount.
    #[prost(message, optional, tag = "1")]
    pub info: ::core::option::Option<BaseAccount>,
}
impl ::prost::Name for QueryAccountInfoResponse {
    const NAME: &'static str = "QueryAccountInfoResponse";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// QueryLegacyAccount defines a query that can be implemented by an x/account
/// to return an auth understandable representation of an account.
/// This query is only used for accounts retro-compatibility at gRPC
/// level, the state machine must not make any assumptions around this.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryLegacyAccount {}
impl ::prost::Name for QueryLegacyAccount {
    const NAME: &'static str = "QueryLegacyAccount";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// QueryLegacyAccountResponse defines the response type of the
/// `QueryLegacyAccount` query.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryLegacyAccountResponse {
    /// account represents the google.Protobuf.Any wrapped account
    /// the type wrapped by the any does not need to comply with the
    /// sdk.AccountI interface.
    #[prost(message, optional, tag = "1")]
    pub account: ::core::option::Option<::pbjson_types::Any>,
    /// base represents the account as a BaseAccount, this can return
    /// nil if the account cannot be represented as a BaseAccount.
    /// This is used in the gRPC QueryAccountInfo method.
    #[prost(message, optional, tag = "2")]
    pub base: ::core::option::Option<BaseAccount>,
}
impl ::prost::Name for QueryLegacyAccountResponse {
    const NAME: &'static str = "QueryLegacyAccountResponse";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
/// GenesisState defines the auth module's genesis state.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// params defines all the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    /// accounts are the accounts present at genesis.
    #[prost(message, repeated, tag = "2")]
    pub accounts: ::prost::alloc::vec::Vec<::pbjson_types::Any>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "cosmos.auth.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.auth.v1beta1.{}", Self::NAME)
    }
}
include!("cosmos.auth.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
