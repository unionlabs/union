// @generated
/// AccessTypeParam
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccessTypeParam {
    #[prost(enumeration = "AccessType", tag = "1")]
    pub value: i32,
}
impl ::prost::Name for AccessTypeParam {
    const NAME: &'static str = "AccessTypeParam";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// AccessConfig access control type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccessConfig {
    #[prost(enumeration = "AccessType", tag = "1")]
    pub permission: i32,
    #[prost(string, repeated, tag = "3")]
    pub addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for AccessConfig {
    const NAME: &'static str = "AccessConfig";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// Params defines the set of wasm parameters.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    #[prost(message, optional, tag = "1")]
    pub code_upload_access: ::core::option::Option<AccessConfig>,
    #[prost(enumeration = "AccessType", tag = "2")]
    pub instantiate_default_permission: i32,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// CodeInfo is data for the uploaded contract WASM code
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CodeInfo {
    /// CodeHash is the unique identifier created by wasmvm
    #[prost(bytes = "vec", tag = "1")]
    pub code_hash: ::prost::alloc::vec::Vec<u8>,
    /// Creator address who initially stored the code
    #[prost(string, tag = "2")]
    pub creator: ::prost::alloc::string::String,
    /// InstantiateConfig access control to apply on contract creation, optional
    #[prost(message, optional, tag = "5")]
    pub instantiate_config: ::core::option::Option<AccessConfig>,
}
impl ::prost::Name for CodeInfo {
    const NAME: &'static str = "CodeInfo";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// ContractInfo stores a WASM contract instance
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ContractInfo {
    /// CodeID is the reference to the stored Wasm code
    #[prost(uint64, tag = "1")]
    pub code_id: u64,
    /// Creator address who initially instantiated the contract
    #[prost(string, tag = "2")]
    pub creator: ::prost::alloc::string::String,
    /// Admin is an optional address that can execute migrations
    #[prost(string, tag = "3")]
    pub admin: ::prost::alloc::string::String,
    /// Label is optional metadata to be stored with a contract instance.
    #[prost(string, tag = "4")]
    pub label: ::prost::alloc::string::String,
    /// Created Tx position when the contract was instantiated.
    #[prost(message, optional, tag = "5")]
    pub created: ::core::option::Option<AbsoluteTxPosition>,
    #[prost(string, tag = "6")]
    pub ibc_port_id: ::prost::alloc::string::String,
    /// Extension is an extension point to store custom metadata within the
    /// persistence model.
    #[prost(message, optional, tag = "7")]
    pub extension: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for ContractInfo {
    const NAME: &'static str = "ContractInfo";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// ContractCodeHistoryEntry metadata to a contract.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ContractCodeHistoryEntry {
    #[prost(enumeration = "ContractCodeHistoryOperationType", tag = "1")]
    pub operation: i32,
    /// CodeID is the reference to the stored WASM code
    #[prost(uint64, tag = "2")]
    pub code_id: u64,
    /// Updated Tx position when the operation was executed.
    #[prost(message, optional, tag = "3")]
    pub updated: ::core::option::Option<AbsoluteTxPosition>,
    #[prost(bytes = "vec", tag = "4")]
    pub msg: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ContractCodeHistoryEntry {
    const NAME: &'static str = "ContractCodeHistoryEntry";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// AbsoluteTxPosition is a unique transaction position that allows for global
/// ordering of transactions.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AbsoluteTxPosition {
    /// BlockHeight is the block the contract was created at
    #[prost(uint64, tag = "1")]
    pub block_height: u64,
    /// TxIndex is a monotonic counter within the block (actual transaction index,
    /// or gas consumed)
    #[prost(uint64, tag = "2")]
    pub tx_index: u64,
}
impl ::prost::Name for AbsoluteTxPosition {
    const NAME: &'static str = "AbsoluteTxPosition";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// Model is a struct that holds a KV pair
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Model {
    /// hex-encode key to read it better (this is often ascii)
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    /// base64-encode raw value
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Model {
    const NAME: &'static str = "Model";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// AccessType permission types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum AccessType {
    /// AccessTypeUnspecified placeholder for empty value
    Unspecified = 0,
    /// AccessTypeNobody forbidden
    Nobody = 1,
    /// AccessTypeEverybody unrestricted
    Everybody = 3,
    /// AccessTypeAnyOfAddresses allow any of the addresses
    AnyOfAddresses = 4,
}
impl AccessType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            AccessType::Unspecified => "ACCESS_TYPE_UNSPECIFIED",
            AccessType::Nobody => "ACCESS_TYPE_NOBODY",
            AccessType::Everybody => "ACCESS_TYPE_EVERYBODY",
            AccessType::AnyOfAddresses => "ACCESS_TYPE_ANY_OF_ADDRESSES",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ACCESS_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "ACCESS_TYPE_NOBODY" => Some(Self::Nobody),
            "ACCESS_TYPE_EVERYBODY" => Some(Self::Everybody),
            "ACCESS_TYPE_ANY_OF_ADDRESSES" => Some(Self::AnyOfAddresses),
            _ => None,
        }
    }
}
/// ContractCodeHistoryOperationType actions that caused a code change
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ContractCodeHistoryOperationType {
    /// ContractCodeHistoryOperationTypeUnspecified placeholder for empty value
    Unspecified = 0,
    /// ContractCodeHistoryOperationTypeInit on chain contract instantiation
    Init = 1,
    /// ContractCodeHistoryOperationTypeMigrate code migration
    Migrate = 2,
    /// ContractCodeHistoryOperationTypeGenesis based on genesis data
    Genesis = 3,
}
impl ContractCodeHistoryOperationType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ContractCodeHistoryOperationType::Unspecified => {
                "CONTRACT_CODE_HISTORY_OPERATION_TYPE_UNSPECIFIED"
            }
            ContractCodeHistoryOperationType::Init => "CONTRACT_CODE_HISTORY_OPERATION_TYPE_INIT",
            ContractCodeHistoryOperationType::Migrate => {
                "CONTRACT_CODE_HISTORY_OPERATION_TYPE_MIGRATE"
            }
            ContractCodeHistoryOperationType::Genesis => {
                "CONTRACT_CODE_HISTORY_OPERATION_TYPE_GENESIS"
            }
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CONTRACT_CODE_HISTORY_OPERATION_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "CONTRACT_CODE_HISTORY_OPERATION_TYPE_INIT" => Some(Self::Init),
            "CONTRACT_CODE_HISTORY_OPERATION_TYPE_MIGRATE" => Some(Self::Migrate),
            "CONTRACT_CODE_HISTORY_OPERATION_TYPE_GENESIS" => Some(Self::Genesis),
            _ => None,
        }
    }
}
/// StoreCodeAuthorization defines authorization for wasm code upload.
/// Since: wasmd 0.42
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StoreCodeAuthorization {
    /// Grants for code upload
    #[prost(message, repeated, tag = "1")]
    pub grants: ::prost::alloc::vec::Vec<CodeGrant>,
}
impl ::prost::Name for StoreCodeAuthorization {
    const NAME: &'static str = "StoreCodeAuthorization";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// ContractExecutionAuthorization defines authorization for wasm execute.
/// Since: wasmd 0.30
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ContractExecutionAuthorization {
    /// Grants for contract executions
    #[prost(message, repeated, tag = "1")]
    pub grants: ::prost::alloc::vec::Vec<ContractGrant>,
}
impl ::prost::Name for ContractExecutionAuthorization {
    const NAME: &'static str = "ContractExecutionAuthorization";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// ContractMigrationAuthorization defines authorization for wasm contract
/// migration. Since: wasmd 0.30
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ContractMigrationAuthorization {
    /// Grants for contract migrations
    #[prost(message, repeated, tag = "1")]
    pub grants: ::prost::alloc::vec::Vec<ContractGrant>,
}
impl ::prost::Name for ContractMigrationAuthorization {
    const NAME: &'static str = "ContractMigrationAuthorization";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// CodeGrant a granted permission for a single code
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CodeGrant {
    /// CodeHash is the unique identifier created by wasmvm
    /// Wildcard "*" is used to specify any kind of grant.
    #[prost(bytes = "vec", tag = "1")]
    pub code_hash: ::prost::alloc::vec::Vec<u8>,
    /// InstantiatePermission is the superset access control to apply
    /// on contract creation.
    /// Optional
    #[prost(message, optional, tag = "2")]
    pub instantiate_permission: ::core::option::Option<AccessConfig>,
}
impl ::prost::Name for CodeGrant {
    const NAME: &'static str = "CodeGrant";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// ContractGrant a granted permission for a single contract
/// Since: wasmd 0.30
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ContractGrant {
    /// Contract is the bech32 address of the smart contract
    #[prost(string, tag = "1")]
    pub contract: ::prost::alloc::string::String,
    /// Limit defines execution limits that are enforced and updated when the grant
    /// is applied. When the limit lapsed the grant is removed.
    #[prost(message, optional, tag = "2")]
    pub limit: ::core::option::Option<::pbjson_types::Any>,
    /// Filter define more fine-grained control on the message payload passed
    /// to the contract in the operation. When no filter applies on execution, the
    /// operation is prohibited.
    #[prost(message, optional, tag = "3")]
    pub filter: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for ContractGrant {
    const NAME: &'static str = "ContractGrant";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MaxCallsLimit limited number of calls to the contract. No funds transferable.
/// Since: wasmd 0.30
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MaxCallsLimit {
    /// Remaining number that is decremented on each execution
    #[prost(uint64, tag = "1")]
    pub remaining: u64,
}
impl ::prost::Name for MaxCallsLimit {
    const NAME: &'static str = "MaxCallsLimit";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MaxFundsLimit defines the maximal amounts that can be sent to the contract.
/// Since: wasmd 0.30
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MaxFundsLimit {
    /// Amounts is the maximal amount of tokens transferable to the contract.
    #[prost(message, repeated, tag = "1")]
    pub amounts: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for MaxFundsLimit {
    const NAME: &'static str = "MaxFundsLimit";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// CombinedLimit defines the maximal amounts that can be sent to a contract and
/// the maximal number of calls executable. Both need to remain >0 to be valid.
/// Since: wasmd 0.30
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CombinedLimit {
    /// Remaining number that is decremented on each execution
    #[prost(uint64, tag = "1")]
    pub calls_remaining: u64,
    /// Amounts is the maximal amount of tokens transferable to the contract.
    #[prost(message, repeated, tag = "2")]
    pub amounts: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for CombinedLimit {
    const NAME: &'static str = "CombinedLimit";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// AllowAllMessagesFilter is a wildcard to allow any type of contract payload
/// message.
/// Since: wasmd 0.30
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllowAllMessagesFilter {}
impl ::prost::Name for AllowAllMessagesFilter {
    const NAME: &'static str = "AllowAllMessagesFilter";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// AcceptedMessageKeysFilter accept only the specific contract message keys in
/// the json object to be executed.
/// Since: wasmd 0.30
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AcceptedMessageKeysFilter {
    /// Messages is the list of unique keys
    #[prost(string, repeated, tag = "1")]
    pub keys: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for AcceptedMessageKeysFilter {
    const NAME: &'static str = "AcceptedMessageKeysFilter";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// AcceptedMessagesFilter accept only the specific raw contract messages to be
/// executed.
/// Since: wasmd 0.30
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AcceptedMessagesFilter {
    /// Messages is the list of raw contract messages
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub messages: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for AcceptedMessagesFilter {
    const NAME: &'static str = "AcceptedMessagesFilter";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// GenesisState - genesis state of x/wasm
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    #[prost(message, repeated, tag = "2")]
    pub codes: ::prost::alloc::vec::Vec<Code>,
    #[prost(message, repeated, tag = "3")]
    pub contracts: ::prost::alloc::vec::Vec<Contract>,
    #[prost(message, repeated, tag = "4")]
    pub sequences: ::prost::alloc::vec::Vec<Sequence>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// Code struct encompasses CodeInfo and CodeBytes
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Code {
    #[prost(uint64, tag = "1")]
    pub code_id: u64,
    #[prost(message, optional, tag = "2")]
    pub code_info: ::core::option::Option<CodeInfo>,
    #[prost(bytes = "vec", tag = "3")]
    pub code_bytes: ::prost::alloc::vec::Vec<u8>,
    /// Pinned to wasmvm cache
    #[prost(bool, tag = "4")]
    pub pinned: bool,
}
impl ::prost::Name for Code {
    const NAME: &'static str = "Code";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// Contract struct encompasses ContractAddress, ContractInfo, and ContractState
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Contract {
    #[prost(string, tag = "1")]
    pub contract_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub contract_info: ::core::option::Option<ContractInfo>,
    #[prost(message, repeated, tag = "3")]
    pub contract_state: ::prost::alloc::vec::Vec<Model>,
    #[prost(message, repeated, tag = "4")]
    pub contract_code_history: ::prost::alloc::vec::Vec<ContractCodeHistoryEntry>,
}
impl ::prost::Name for Contract {
    const NAME: &'static str = "Contract";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// Sequence key and value of an id generation counter
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Sequence {
    #[prost(bytes = "vec", tag = "1")]
    pub id_key: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub value: u64,
}
impl ::prost::Name for Sequence {
    const NAME: &'static str = "Sequence";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgIBCSend
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgIbcSend {
    /// the channel by which the packet will be sent
    #[prost(string, tag = "2")]
    pub channel: ::prost::alloc::string::String,
    /// Timeout height relative to the current block height.
    /// The timeout is disabled when set to 0.
    #[prost(uint64, tag = "4")]
    pub timeout_height: u64,
    /// Timeout timestamp (in nanoseconds) relative to the current block timestamp.
    /// The timeout is disabled when set to 0.
    #[prost(uint64, tag = "5")]
    pub timeout_timestamp: u64,
    /// Data is the payload to transfer. We must not make assumption what format or
    /// content is in here.
    #[prost(bytes = "vec", tag = "6")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgIbcSend {
    const NAME: &'static str = "MsgIBCSend";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgIBCSendResponse
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgIbcSendResponse {
    /// Sequence number of the IBC packet sent
    #[prost(uint64, tag = "1")]
    pub sequence: u64,
}
impl ::prost::Name for MsgIbcSendResponse {
    const NAME: &'static str = "MsgIBCSendResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgIBCWriteAcknowledgementResponse
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgIbcWriteAcknowledgementResponse {}
impl ::prost::Name for MsgIbcWriteAcknowledgementResponse {
    const NAME: &'static str = "MsgIBCWriteAcknowledgementResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgIBCCloseChannel port and channel need to be owned by the contract
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgIbcCloseChannel {
    #[prost(string, tag = "2")]
    pub channel: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgIbcCloseChannel {
    const NAME: &'static str = "MsgIBCCloseChannel";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// Deprecated: Do not use. Since wasmd v0.40, there is no longer a need for
/// an explicit StoreCodeProposal. To submit WASM code to the system,
/// a simple MsgStoreCode can be invoked from the x/gov module via
/// a v1 governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StoreCodeProposal {
    /// Title is a short summary
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// Description is a human readable text
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// RunAs is the address that is passed to the contract's environment as sender
    #[prost(string, tag = "3")]
    pub run_as: ::prost::alloc::string::String,
    /// WASMByteCode can be raw or gzip compressed
    #[prost(bytes = "vec", tag = "4")]
    pub wasm_byte_code: ::prost::alloc::vec::Vec<u8>,
    /// InstantiatePermission to apply on contract creation, optional
    #[prost(message, optional, tag = "7")]
    pub instantiate_permission: ::core::option::Option<AccessConfig>,
    /// UnpinCode code on upload, optional
    #[prost(bool, tag = "8")]
    pub unpin_code: bool,
    /// Source is the URL where the code is hosted
    #[prost(string, tag = "9")]
    pub source: ::prost::alloc::string::String,
    /// Builder is the docker image used to build the code deterministically, used
    /// for smart contract verification
    #[prost(string, tag = "10")]
    pub builder: ::prost::alloc::string::String,
    /// CodeHash is the SHA256 sum of the code outputted by builder, used for smart
    /// contract verification
    #[prost(bytes = "vec", tag = "11")]
    pub code_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for StoreCodeProposal {
    const NAME: &'static str = "StoreCodeProposal";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// Deprecated: Do not use. Since wasmd v0.40, there is no longer a need for
/// an explicit InstantiateContractProposal. To instantiate a contract,
/// a simple MsgInstantiateContract can be invoked from the x/gov module via
/// a v1 governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InstantiateContractProposal {
    /// Title is a short summary
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// Description is a human readable text
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// RunAs is the address that is passed to the contract's environment as sender
    #[prost(string, tag = "3")]
    pub run_as: ::prost::alloc::string::String,
    /// Admin is an optional address that can execute migrations
    #[prost(string, tag = "4")]
    pub admin: ::prost::alloc::string::String,
    /// CodeID is the reference to the stored WASM code
    #[prost(uint64, tag = "5")]
    pub code_id: u64,
    /// Label is optional metadata to be stored with a constract instance.
    #[prost(string, tag = "6")]
    pub label: ::prost::alloc::string::String,
    /// Msg json encoded message to be passed to the contract on instantiation
    #[prost(bytes = "vec", tag = "7")]
    pub msg: ::prost::alloc::vec::Vec<u8>,
    /// Funds coins that are transferred to the contract on instantiation
    #[prost(message, repeated, tag = "8")]
    pub funds: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for InstantiateContractProposal {
    const NAME: &'static str = "InstantiateContractProposal";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// Deprecated: Do not use. Since wasmd v0.40, there is no longer a need for
/// an explicit InstantiateContract2Proposal. To instantiate contract 2,
/// a simple MsgInstantiateContract2 can be invoked from the x/gov module via
/// a v1 governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InstantiateContract2Proposal {
    /// Title is a short summary
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// Description is a human readable text
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// RunAs is the address that is passed to the contract's enviroment as sender
    #[prost(string, tag = "3")]
    pub run_as: ::prost::alloc::string::String,
    /// Admin is an optional address that can execute migrations
    #[prost(string, tag = "4")]
    pub admin: ::prost::alloc::string::String,
    /// CodeID is the reference to the stored WASM code
    #[prost(uint64, tag = "5")]
    pub code_id: u64,
    /// Label is optional metadata to be stored with a constract instance.
    #[prost(string, tag = "6")]
    pub label: ::prost::alloc::string::String,
    /// Msg json encode message to be passed to the contract on instantiation
    #[prost(bytes = "vec", tag = "7")]
    pub msg: ::prost::alloc::vec::Vec<u8>,
    /// Funds coins that are transferred to the contract on instantiation
    #[prost(message, repeated, tag = "8")]
    pub funds: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    /// Salt is an arbitrary value provided by the sender. Size can be 1 to 64.
    #[prost(bytes = "vec", tag = "9")]
    pub salt: ::prost::alloc::vec::Vec<u8>,
    /// FixMsg include the msg value into the hash for the predictable address.
    /// Default is false
    #[prost(bool, tag = "10")]
    pub fix_msg: bool,
}
impl ::prost::Name for InstantiateContract2Proposal {
    const NAME: &'static str = "InstantiateContract2Proposal";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// Deprecated: Do not use. Since wasmd v0.40, there is no longer a need for
/// an explicit MigrateContractProposal. To migrate a contract,
/// a simple MsgMigrateContract can be invoked from the x/gov module via
/// a v1 governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MigrateContractProposal {
    /// Title is a short summary
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// Description is a human readable text
    ///
    /// Note: skipping 3 as this was previously used for unneeded run_as
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// Contract is the address of the smart contract
    #[prost(string, tag = "4")]
    pub contract: ::prost::alloc::string::String,
    /// CodeID references the new WASM code
    #[prost(uint64, tag = "5")]
    pub code_id: u64,
    /// Msg json encoded message to be passed to the contract on migration
    #[prost(bytes = "vec", tag = "6")]
    pub msg: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MigrateContractProposal {
    const NAME: &'static str = "MigrateContractProposal";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// Deprecated: Do not use. Since wasmd v0.40, there is no longer a need for
/// an explicit SudoContractProposal. To call sudo on a contract,
/// a simple MsgSudoContract can be invoked from the x/gov module via
/// a v1 governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SudoContractProposal {
    /// Title is a short summary
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// Description is a human readable text
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// Contract is the address of the smart contract
    #[prost(string, tag = "3")]
    pub contract: ::prost::alloc::string::String,
    /// Msg json encoded message to be passed to the contract as sudo
    #[prost(bytes = "vec", tag = "4")]
    pub msg: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SudoContractProposal {
    const NAME: &'static str = "SudoContractProposal";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// Deprecated: Do not use. Since wasmd v0.40, there is no longer a need for
/// an explicit ExecuteContractProposal. To call execute on a contract,
/// a simple MsgExecuteContract can be invoked from the x/gov module via
/// a v1 governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExecuteContractProposal {
    /// Title is a short summary
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// Description is a human readable text
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// RunAs is the address that is passed to the contract's environment as sender
    #[prost(string, tag = "3")]
    pub run_as: ::prost::alloc::string::String,
    /// Contract is the address of the smart contract
    #[prost(string, tag = "4")]
    pub contract: ::prost::alloc::string::String,
    /// Msg json encoded message to be passed to the contract as execute
    #[prost(bytes = "vec", tag = "5")]
    pub msg: ::prost::alloc::vec::Vec<u8>,
    /// Funds coins that are transferred to the contract on instantiation
    #[prost(message, repeated, tag = "6")]
    pub funds: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for ExecuteContractProposal {
    const NAME: &'static str = "ExecuteContractProposal";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// Deprecated: Do not use. Since wasmd v0.40, there is no longer a need for
/// an explicit UpdateAdminProposal. To set an admin for a contract,
/// a simple MsgUpdateAdmin can be invoked from the x/gov module via
/// a v1 governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateAdminProposal {
    /// Title is a short summary
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// Description is a human readable text
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// NewAdmin address to be set
    #[prost(string, tag = "3")]
    pub new_admin: ::prost::alloc::string::String,
    /// Contract is the address of the smart contract
    #[prost(string, tag = "4")]
    pub contract: ::prost::alloc::string::String,
}
impl ::prost::Name for UpdateAdminProposal {
    const NAME: &'static str = "UpdateAdminProposal";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// Deprecated: Do not use. Since wasmd v0.40, there is no longer a need for
/// an explicit ClearAdminProposal. To clear the admin of a contract,
/// a simple MsgClearAdmin can be invoked from the x/gov module via
/// a v1 governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClearAdminProposal {
    /// Title is a short summary
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// Description is a human readable text
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// Contract is the address of the smart contract
    #[prost(string, tag = "3")]
    pub contract: ::prost::alloc::string::String,
}
impl ::prost::Name for ClearAdminProposal {
    const NAME: &'static str = "ClearAdminProposal";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// Deprecated: Do not use. Since wasmd v0.40, there is no longer a need for
/// an explicit PinCodesProposal. To pin a set of code ids in the wasmvm
/// cache, a simple MsgPinCodes can be invoked from the x/gov module via
/// a v1 governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PinCodesProposal {
    /// Title is a short summary
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// Description is a human readable text
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// CodeIDs references the new WASM codes
    #[prost(uint64, repeated, packed = "false", tag = "3")]
    pub code_ids: ::prost::alloc::vec::Vec<u64>,
}
impl ::prost::Name for PinCodesProposal {
    const NAME: &'static str = "PinCodesProposal";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// Deprecated: Do not use. Since wasmd v0.40, there is no longer a need for
/// an explicit UnpinCodesProposal. To unpin a set of code ids in the wasmvm
/// cache, a simple MsgUnpinCodes can be invoked from the x/gov module via
/// a v1 governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnpinCodesProposal {
    /// Title is a short summary
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// Description is a human readable text
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// CodeIDs references the WASM codes
    #[prost(uint64, repeated, packed = "false", tag = "3")]
    pub code_ids: ::prost::alloc::vec::Vec<u64>,
}
impl ::prost::Name for UnpinCodesProposal {
    const NAME: &'static str = "UnpinCodesProposal";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// AccessConfigUpdate contains the code id and the access config to be
/// applied.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccessConfigUpdate {
    /// CodeID is the reference to the stored WASM code to be updated
    #[prost(uint64, tag = "1")]
    pub code_id: u64,
    /// InstantiatePermission to apply to the set of code ids
    #[prost(message, optional, tag = "2")]
    pub instantiate_permission: ::core::option::Option<AccessConfig>,
}
impl ::prost::Name for AccessConfigUpdate {
    const NAME: &'static str = "AccessConfigUpdate";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// Deprecated: Do not use. Since wasmd v0.40, there is no longer a need for
/// an explicit UpdateInstantiateConfigProposal. To update instantiate config
/// to a set of code ids, a simple MsgUpdateInstantiateConfig can be invoked from
/// the x/gov module via a v1 governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateInstantiateConfigProposal {
    /// Title is a short summary
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// Description is a human readable text
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// AccessConfigUpdate contains the list of code ids and the access config
    /// to be applied.
    #[prost(message, repeated, tag = "3")]
    pub access_config_updates: ::prost::alloc::vec::Vec<AccessConfigUpdate>,
}
impl ::prost::Name for UpdateInstantiateConfigProposal {
    const NAME: &'static str = "UpdateInstantiateConfigProposal";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// Deprecated: Do not use. Since wasmd v0.40, there is no longer a need for
/// an explicit StoreAndInstantiateContractProposal. To store and instantiate
/// the contract, a simple MsgStoreAndInstantiateContract can be invoked from
/// the x/gov module via a v1 governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StoreAndInstantiateContractProposal {
    /// Title is a short summary
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// Description is a human readable text
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// RunAs is the address that is passed to the contract's environment as sender
    #[prost(string, tag = "3")]
    pub run_as: ::prost::alloc::string::String,
    /// WASMByteCode can be raw or gzip compressed
    #[prost(bytes = "vec", tag = "4")]
    pub wasm_byte_code: ::prost::alloc::vec::Vec<u8>,
    /// InstantiatePermission to apply on contract creation, optional
    #[prost(message, optional, tag = "5")]
    pub instantiate_permission: ::core::option::Option<AccessConfig>,
    /// UnpinCode code on upload, optional
    #[prost(bool, tag = "6")]
    pub unpin_code: bool,
    /// Admin is an optional address that can execute migrations
    #[prost(string, tag = "7")]
    pub admin: ::prost::alloc::string::String,
    /// Label is optional metadata to be stored with a constract instance.
    #[prost(string, tag = "8")]
    pub label: ::prost::alloc::string::String,
    /// Msg json encoded message to be passed to the contract on instantiation
    #[prost(bytes = "vec", tag = "9")]
    pub msg: ::prost::alloc::vec::Vec<u8>,
    /// Funds coins that are transferred to the contract on instantiation
    #[prost(message, repeated, tag = "10")]
    pub funds: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    /// Source is the URL where the code is hosted
    #[prost(string, tag = "11")]
    pub source: ::prost::alloc::string::String,
    /// Builder is the docker image used to build the code deterministically, used
    /// for smart contract verification
    #[prost(string, tag = "12")]
    pub builder: ::prost::alloc::string::String,
    /// CodeHash is the SHA256 sum of the code outputted by builder, used for smart
    /// contract verification
    #[prost(bytes = "vec", tag = "13")]
    pub code_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for StoreAndInstantiateContractProposal {
    const NAME: &'static str = "StoreAndInstantiateContractProposal";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryContractInfoRequest is the request type for the Query/ContractInfo RPC
/// method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryContractInfoRequest {
    /// address is the address of the contract to query
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryContractInfoRequest {
    const NAME: &'static str = "QueryContractInfoRequest";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryContractInfoResponse is the response type for the Query/ContractInfo RPC
/// method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryContractInfoResponse {
    /// address is the address of the contract
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub contract_info: ::core::option::Option<ContractInfo>,
}
impl ::prost::Name for QueryContractInfoResponse {
    const NAME: &'static str = "QueryContractInfoResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryContractHistoryRequest is the request type for the Query/ContractHistory
/// RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryContractHistoryRequest {
    /// address is the address of the contract to query
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryContractHistoryRequest {
    const NAME: &'static str = "QueryContractHistoryRequest";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryContractHistoryResponse is the response type for the
/// Query/ContractHistory RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryContractHistoryResponse {
    #[prost(message, repeated, tag = "1")]
    pub entries: ::prost::alloc::vec::Vec<ContractCodeHistoryEntry>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryContractHistoryResponse {
    const NAME: &'static str = "QueryContractHistoryResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryContractsByCodeRequest is the request type for the Query/ContractsByCode
/// RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryContractsByCodeRequest {
    /// grpc-gateway_out does not support Go style CodeID
    #[prost(uint64, tag = "1")]
    pub code_id: u64,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryContractsByCodeRequest {
    const NAME: &'static str = "QueryContractsByCodeRequest";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryContractsByCodeResponse is the response type for the
/// Query/ContractsByCode RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryContractsByCodeResponse {
    /// contracts are a set of contract addresses
    #[prost(string, repeated, tag = "1")]
    pub contracts: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryContractsByCodeResponse {
    const NAME: &'static str = "QueryContractsByCodeResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryAllContractStateRequest is the request type for the
/// Query/AllContractState RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllContractStateRequest {
    /// address is the address of the contract
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryAllContractStateRequest {
    const NAME: &'static str = "QueryAllContractStateRequest";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryAllContractStateResponse is the response type for the
/// Query/AllContractState RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllContractStateResponse {
    #[prost(message, repeated, tag = "1")]
    pub models: ::prost::alloc::vec::Vec<Model>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryAllContractStateResponse {
    const NAME: &'static str = "QueryAllContractStateResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryRawContractStateRequest is the request type for the
/// Query/RawContractState RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRawContractStateRequest {
    /// address is the address of the contract
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub query_data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for QueryRawContractStateRequest {
    const NAME: &'static str = "QueryRawContractStateRequest";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryRawContractStateResponse is the response type for the
/// Query/RawContractState RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRawContractStateResponse {
    /// Data contains the raw store data
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for QueryRawContractStateResponse {
    const NAME: &'static str = "QueryRawContractStateResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QuerySmartContractStateRequest is the request type for the
/// Query/SmartContractState RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySmartContractStateRequest {
    /// address is the address of the contract
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// QueryData contains the query data passed to the contract
    #[prost(bytes = "vec", tag = "2")]
    pub query_data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for QuerySmartContractStateRequest {
    const NAME: &'static str = "QuerySmartContractStateRequest";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QuerySmartContractStateResponse is the response type for the
/// Query/SmartContractState RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySmartContractStateResponse {
    /// Data contains the json data returned from the smart contract
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for QuerySmartContractStateResponse {
    const NAME: &'static str = "QuerySmartContractStateResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryCodeRequest is the request type for the Query/Code RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCodeRequest {
    /// grpc-gateway_out does not support Go style CodeID
    #[prost(uint64, tag = "1")]
    pub code_id: u64,
}
impl ::prost::Name for QueryCodeRequest {
    const NAME: &'static str = "QueryCodeRequest";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryCodeInfoRequest is the request type for the Query/CodeInfo RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCodeInfoRequest {
    /// grpc-gateway_out does not support Go style CodeID
    #[prost(uint64, tag = "1")]
    pub code_id: u64,
}
impl ::prost::Name for QueryCodeInfoRequest {
    const NAME: &'static str = "QueryCodeInfoRequest";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryCodeInfoResponse is the response type for the Query/CodeInfo RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCodeInfoResponse {
    #[prost(uint64, tag = "1")]
    pub code_id: u64,
    #[prost(string, tag = "2")]
    pub creator: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "3")]
    pub checksum: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "4")]
    pub instantiate_permission: ::core::option::Option<AccessConfig>,
}
impl ::prost::Name for QueryCodeInfoResponse {
    const NAME: &'static str = "QueryCodeInfoResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// CodeInfoResponse contains code meta data from CodeInfo
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CodeInfoResponse {
    /// id for legacy support
    #[prost(uint64, tag = "1")]
    pub code_id: u64,
    #[prost(string, tag = "2")]
    pub creator: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "3")]
    pub data_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "6")]
    pub instantiate_permission: ::core::option::Option<AccessConfig>,
}
impl ::prost::Name for CodeInfoResponse {
    const NAME: &'static str = "CodeInfoResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryCodeResponse is the response type for the Query/Code RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCodeResponse {
    #[prost(message, optional, tag = "1")]
    pub code_info: ::core::option::Option<CodeInfoResponse>,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for QueryCodeResponse {
    const NAME: &'static str = "QueryCodeResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryCodesRequest is the request type for the Query/Codes RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCodesRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryCodesRequest {
    const NAME: &'static str = "QueryCodesRequest";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryCodesResponse is the response type for the Query/Codes RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCodesResponse {
    #[prost(message, repeated, tag = "1")]
    pub code_infos: ::prost::alloc::vec::Vec<CodeInfoResponse>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryCodesResponse {
    const NAME: &'static str = "QueryCodesResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryPinnedCodesRequest is the request type for the Query/PinnedCodes
/// RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPinnedCodesRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryPinnedCodesRequest {
    const NAME: &'static str = "QueryPinnedCodesRequest";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryPinnedCodesResponse is the response type for the
/// Query/PinnedCodes RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPinnedCodesResponse {
    #[prost(uint64, repeated, packed = "false", tag = "1")]
    pub code_ids: ::prost::alloc::vec::Vec<u64>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryPinnedCodesResponse {
    const NAME: &'static str = "QueryPinnedCodesResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
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
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryContractsByCreatorRequest is the request type for the
/// Query/ContractsByCreator RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryContractsByCreatorRequest {
    /// CreatorAddress is the address of contract creator
    #[prost(string, tag = "1")]
    pub creator_address: ::prost::alloc::string::String,
    /// Pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryContractsByCreatorRequest {
    const NAME: &'static str = "QueryContractsByCreatorRequest";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryContractsByCreatorResponse is the response type for the
/// Query/ContractsByCreator RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryContractsByCreatorResponse {
    /// ContractAddresses result set
    #[prost(string, repeated, tag = "1")]
    pub contract_addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryContractsByCreatorResponse {
    const NAME: &'static str = "QueryContractsByCreatorResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryBuildAddressRequest is the request type for the Query/BuildAddress RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBuildAddressRequest {
    /// CodeHash is the hash of the code
    #[prost(string, tag = "1")]
    pub code_hash: ::prost::alloc::string::String,
    /// CreatorAddress is the address of the contract instantiator
    #[prost(string, tag = "2")]
    pub creator_address: ::prost::alloc::string::String,
    /// Salt is a hex encoded salt
    #[prost(string, tag = "3")]
    pub salt: ::prost::alloc::string::String,
    /// InitArgs are optional json encoded init args to be used in contract address
    /// building if provided
    #[prost(bytes = "vec", tag = "4")]
    pub init_args: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for QueryBuildAddressRequest {
    const NAME: &'static str = "QueryBuildAddressRequest";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// QueryBuildAddressResponse is the response type for the Query/BuildAddress RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBuildAddressResponse {
    /// Address is the contract address
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryBuildAddressResponse {
    const NAME: &'static str = "QueryBuildAddressResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgStoreCode submit Wasm code to the system
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgStoreCode {
    /// Sender is the actor that signed the messages
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    /// WASMByteCode can be raw or gzip compressed
    #[prost(bytes = "vec", tag = "2")]
    pub wasm_byte_code: ::prost::alloc::vec::Vec<u8>,
    /// InstantiatePermission access control to apply on contract creation,
    /// optional
    #[prost(message, optional, tag = "5")]
    pub instantiate_permission: ::core::option::Option<AccessConfig>,
}
impl ::prost::Name for MsgStoreCode {
    const NAME: &'static str = "MsgStoreCode";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgStoreCodeResponse returns store result data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgStoreCodeResponse {
    /// CodeID is the reference to the stored WASM code
    #[prost(uint64, tag = "1")]
    pub code_id: u64,
    /// Checksum is the sha256 hash of the stored code
    #[prost(bytes = "vec", tag = "2")]
    pub checksum: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgStoreCodeResponse {
    const NAME: &'static str = "MsgStoreCodeResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgInstantiateContract create a new smart contract instance for the given
/// code id.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgInstantiateContract {
    /// Sender is the that actor that signed the messages
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    /// Admin is an optional address that can execute migrations
    #[prost(string, tag = "2")]
    pub admin: ::prost::alloc::string::String,
    /// CodeID is the reference to the stored WASM code
    #[prost(uint64, tag = "3")]
    pub code_id: u64,
    /// Label is optional metadata to be stored with a contract instance.
    #[prost(string, tag = "4")]
    pub label: ::prost::alloc::string::String,
    /// Msg json encoded message to be passed to the contract on instantiation
    #[prost(bytes = "vec", tag = "5")]
    pub msg: ::prost::alloc::vec::Vec<u8>,
    /// Funds coins that are transferred to the contract on instantiation
    #[prost(message, repeated, tag = "6")]
    pub funds: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgInstantiateContract {
    const NAME: &'static str = "MsgInstantiateContract";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgInstantiateContractResponse return instantiation result data
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgInstantiateContractResponse {
    /// Address is the bech32 address of the new contract instance.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// Data contains bytes to returned from the contract
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgInstantiateContractResponse {
    const NAME: &'static str = "MsgInstantiateContractResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgInstantiateContract2 create a new smart contract instance for the given
/// code id with a predictable address.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgInstantiateContract2 {
    /// Sender is the that actor that signed the messages
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    /// Admin is an optional address that can execute migrations
    #[prost(string, tag = "2")]
    pub admin: ::prost::alloc::string::String,
    /// CodeID is the reference to the stored WASM code
    #[prost(uint64, tag = "3")]
    pub code_id: u64,
    /// Label is optional metadata to be stored with a contract instance.
    #[prost(string, tag = "4")]
    pub label: ::prost::alloc::string::String,
    /// Msg json encoded message to be passed to the contract on instantiation
    #[prost(bytes = "vec", tag = "5")]
    pub msg: ::prost::alloc::vec::Vec<u8>,
    /// Funds coins that are transferred to the contract on instantiation
    #[prost(message, repeated, tag = "6")]
    pub funds: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    /// Salt is an arbitrary value provided by the sender. Size can be 1 to 64.
    #[prost(bytes = "vec", tag = "7")]
    pub salt: ::prost::alloc::vec::Vec<u8>,
    /// FixMsg include the msg value into the hash for the predictable address.
    /// Default is false
    #[prost(bool, tag = "8")]
    pub fix_msg: bool,
}
impl ::prost::Name for MsgInstantiateContract2 {
    const NAME: &'static str = "MsgInstantiateContract2";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgInstantiateContract2Response return instantiation result data
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgInstantiateContract2Response {
    /// Address is the bech32 address of the new contract instance.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// Data contains bytes to returned from the contract
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgInstantiateContract2Response {
    const NAME: &'static str = "MsgInstantiateContract2Response";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgExecuteContract submits the given message data to a smart contract
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgExecuteContract {
    /// Sender is the that actor that signed the messages
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    /// Contract is the address of the smart contract
    #[prost(string, tag = "2")]
    pub contract: ::prost::alloc::string::String,
    /// Msg json encoded message to be passed to the contract
    #[prost(bytes = "vec", tag = "3")]
    pub msg: ::prost::alloc::vec::Vec<u8>,
    /// Funds coins that are transferred to the contract on execution
    #[prost(message, repeated, tag = "5")]
    pub funds: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
impl ::prost::Name for MsgExecuteContract {
    const NAME: &'static str = "MsgExecuteContract";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgExecuteContractResponse returns execution result data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgExecuteContractResponse {
    /// Data contains bytes to returned from the contract
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgExecuteContractResponse {
    const NAME: &'static str = "MsgExecuteContractResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgMigrateContract runs a code upgrade/ downgrade for a smart contract
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgMigrateContract {
    /// Sender is the that actor that signed the messages
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    /// Contract is the address of the smart contract
    #[prost(string, tag = "2")]
    pub contract: ::prost::alloc::string::String,
    /// CodeID references the new WASM code
    #[prost(uint64, tag = "3")]
    pub code_id: u64,
    /// Msg json encoded message to be passed to the contract on migration
    #[prost(bytes = "vec", tag = "4")]
    pub msg: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgMigrateContract {
    const NAME: &'static str = "MsgMigrateContract";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgMigrateContractResponse returns contract migration result data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgMigrateContractResponse {
    /// Data contains same raw bytes returned as data from the wasm contract.
    /// (May be empty)
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgMigrateContractResponse {
    const NAME: &'static str = "MsgMigrateContractResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgUpdateAdmin sets a new admin for a smart contract
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateAdmin {
    /// Sender is the that actor that signed the messages
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    /// NewAdmin address to be set
    #[prost(string, tag = "2")]
    pub new_admin: ::prost::alloc::string::String,
    /// Contract is the address of the smart contract
    #[prost(string, tag = "3")]
    pub contract: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgUpdateAdmin {
    const NAME: &'static str = "MsgUpdateAdmin";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgUpdateAdminResponse returns empty data
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateAdminResponse {}
impl ::prost::Name for MsgUpdateAdminResponse {
    const NAME: &'static str = "MsgUpdateAdminResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgClearAdmin removes any admin stored for a smart contract
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgClearAdmin {
    /// Sender is the actor that signed the messages
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    /// Contract is the address of the smart contract
    #[prost(string, tag = "3")]
    pub contract: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgClearAdmin {
    const NAME: &'static str = "MsgClearAdmin";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgClearAdminResponse returns empty data
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgClearAdminResponse {}
impl ::prost::Name for MsgClearAdminResponse {
    const NAME: &'static str = "MsgClearAdminResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgUpdateInstantiateConfig updates instantiate config for a smart contract
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateInstantiateConfig {
    /// Sender is the that actor that signed the messages
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    /// CodeID references the stored WASM code
    #[prost(uint64, tag = "2")]
    pub code_id: u64,
    /// NewInstantiatePermission is the new access control
    #[prost(message, optional, tag = "3")]
    pub new_instantiate_permission: ::core::option::Option<AccessConfig>,
}
impl ::prost::Name for MsgUpdateInstantiateConfig {
    const NAME: &'static str = "MsgUpdateInstantiateConfig";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgUpdateInstantiateConfigResponse returns empty data
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateInstantiateConfigResponse {}
impl ::prost::Name for MsgUpdateInstantiateConfigResponse {
    const NAME: &'static str = "MsgUpdateInstantiateConfigResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParams is the MsgUpdateParams request type.
///
/// Since: 0.40
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// Authority is the address of the governance account.
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the x/wasm parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParamsResponse defines the response structure for executing a
/// MsgUpdateParams message.
///
/// Since: 0.40
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgSudoContract is the MsgSudoContract request type.
///
/// Since: 0.40
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSudoContract {
    /// Authority is the address of the governance account.
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// Contract is the address of the smart contract
    #[prost(string, tag = "2")]
    pub contract: ::prost::alloc::string::String,
    /// Msg json encoded message to be passed to the contract as sudo
    #[prost(bytes = "vec", tag = "3")]
    pub msg: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgSudoContract {
    const NAME: &'static str = "MsgSudoContract";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgSudoContractResponse defines the response structure for executing a
/// MsgSudoContract message.
///
/// Since: 0.40
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSudoContractResponse {
    /// Data contains bytes to returned from the contract
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgSudoContractResponse {
    const NAME: &'static str = "MsgSudoContractResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgPinCodes is the MsgPinCodes request type.
///
/// Since: 0.40
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgPinCodes {
    /// Authority is the address of the governance account.
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// CodeIDs references the new WASM codes
    #[prost(uint64, repeated, packed = "false", tag = "2")]
    pub code_ids: ::prost::alloc::vec::Vec<u64>,
}
impl ::prost::Name for MsgPinCodes {
    const NAME: &'static str = "MsgPinCodes";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgPinCodesResponse defines the response structure for executing a
/// MsgPinCodes message.
///
/// Since: 0.40
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgPinCodesResponse {}
impl ::prost::Name for MsgPinCodesResponse {
    const NAME: &'static str = "MsgPinCodesResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgUnpinCodes is the MsgUnpinCodes request type.
///
/// Since: 0.40
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUnpinCodes {
    /// Authority is the address of the governance account.
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// CodeIDs references the WASM codes
    #[prost(uint64, repeated, packed = "false", tag = "2")]
    pub code_ids: ::prost::alloc::vec::Vec<u64>,
}
impl ::prost::Name for MsgUnpinCodes {
    const NAME: &'static str = "MsgUnpinCodes";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgUnpinCodesResponse defines the response structure for executing a
/// MsgUnpinCodes message.
///
/// Since: 0.40
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUnpinCodesResponse {}
impl ::prost::Name for MsgUnpinCodesResponse {
    const NAME: &'static str = "MsgUnpinCodesResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgStoreAndInstantiateContract is the MsgStoreAndInstantiateContract
/// request type.
///
/// Since: 0.40
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgStoreAndInstantiateContract {
    /// Authority is the address of the governance account.
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// WASMByteCode can be raw or gzip compressed
    #[prost(bytes = "vec", tag = "3")]
    pub wasm_byte_code: ::prost::alloc::vec::Vec<u8>,
    /// InstantiatePermission to apply on contract creation, optional
    #[prost(message, optional, tag = "4")]
    pub instantiate_permission: ::core::option::Option<AccessConfig>,
    /// UnpinCode code on upload, optional. As default the uploaded contract is
    /// pinned to cache.
    #[prost(bool, tag = "5")]
    pub unpin_code: bool,
    /// Admin is an optional address that can execute migrations
    #[prost(string, tag = "6")]
    pub admin: ::prost::alloc::string::String,
    /// Label is optional metadata to be stored with a constract instance.
    #[prost(string, tag = "7")]
    pub label: ::prost::alloc::string::String,
    /// Msg json encoded message to be passed to the contract on instantiation
    #[prost(bytes = "vec", tag = "8")]
    pub msg: ::prost::alloc::vec::Vec<u8>,
    /// Funds coins that are transferred from the authority account to the contract
    /// on instantiation
    #[prost(message, repeated, tag = "9")]
    pub funds: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    /// Source is the URL where the code is hosted
    #[prost(string, tag = "10")]
    pub source: ::prost::alloc::string::String,
    /// Builder is the docker image used to build the code deterministically, used
    /// for smart contract verification
    #[prost(string, tag = "11")]
    pub builder: ::prost::alloc::string::String,
    /// CodeHash is the SHA256 sum of the code outputted by builder, used for smart
    /// contract verification
    #[prost(bytes = "vec", tag = "12")]
    pub code_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgStoreAndInstantiateContract {
    const NAME: &'static str = "MsgStoreAndInstantiateContract";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgStoreAndInstantiateContractResponse defines the response structure
/// for executing a MsgStoreAndInstantiateContract message.
///
/// Since: 0.40
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgStoreAndInstantiateContractResponse {
    /// Address is the bech32 address of the new contract instance.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// Data contains bytes to returned from the contract
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgStoreAndInstantiateContractResponse {
    const NAME: &'static str = "MsgStoreAndInstantiateContractResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgAddCodeUploadParamsAddresses is the
/// MsgAddCodeUploadParamsAddresses request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAddCodeUploadParamsAddresses {
    /// Authority is the address of the governance account.
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "2")]
    pub addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for MsgAddCodeUploadParamsAddresses {
    const NAME: &'static str = "MsgAddCodeUploadParamsAddresses";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgAddCodeUploadParamsAddressesResponse defines the response
/// structure for executing a MsgAddCodeUploadParamsAddresses message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAddCodeUploadParamsAddressesResponse {}
impl ::prost::Name for MsgAddCodeUploadParamsAddressesResponse {
    const NAME: &'static str = "MsgAddCodeUploadParamsAddressesResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgRemoveCodeUploadParamsAddresses is the
/// MsgRemoveCodeUploadParamsAddresses request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRemoveCodeUploadParamsAddresses {
    /// Authority is the address of the governance account.
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "2")]
    pub addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for MsgRemoveCodeUploadParamsAddresses {
    const NAME: &'static str = "MsgRemoveCodeUploadParamsAddresses";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgRemoveCodeUploadParamsAddressesResponse defines the response
/// structure for executing a MsgRemoveCodeUploadParamsAddresses message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRemoveCodeUploadParamsAddressesResponse {}
impl ::prost::Name for MsgRemoveCodeUploadParamsAddressesResponse {
    const NAME: &'static str = "MsgRemoveCodeUploadParamsAddressesResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgStoreAndMigrateContract is the MsgStoreAndMigrateContract
/// request type.
///
/// Since: 0.42
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgStoreAndMigrateContract {
    /// Authority is the address of the governance account.
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// WASMByteCode can be raw or gzip compressed
    #[prost(bytes = "vec", tag = "2")]
    pub wasm_byte_code: ::prost::alloc::vec::Vec<u8>,
    /// InstantiatePermission to apply on contract creation, optional
    #[prost(message, optional, tag = "3")]
    pub instantiate_permission: ::core::option::Option<AccessConfig>,
    /// Contract is the address of the smart contract
    #[prost(string, tag = "4")]
    pub contract: ::prost::alloc::string::String,
    /// Msg json encoded message to be passed to the contract on migration
    #[prost(bytes = "vec", tag = "5")]
    pub msg: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgStoreAndMigrateContract {
    const NAME: &'static str = "MsgStoreAndMigrateContract";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgStoreAndMigrateContractResponse defines the response structure
/// for executing a MsgStoreAndMigrateContract message.
///
/// Since: 0.42
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgStoreAndMigrateContractResponse {
    /// CodeID is the reference to the stored WASM code
    #[prost(uint64, tag = "1")]
    pub code_id: u64,
    /// Checksum is the sha256 hash of the stored code
    #[prost(bytes = "vec", tag = "2")]
    pub checksum: ::prost::alloc::vec::Vec<u8>,
    /// Data contains bytes to returned from the contract
    #[prost(bytes = "vec", tag = "3")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgStoreAndMigrateContractResponse {
    const NAME: &'static str = "MsgStoreAndMigrateContractResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgUpdateContractLabel sets a new label for a smart contract
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateContractLabel {
    /// Sender is the that actor that signed the messages
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    /// NewLabel string to be set
    #[prost(string, tag = "2")]
    pub new_label: ::prost::alloc::string::String,
    /// Contract is the address of the smart contract
    #[prost(string, tag = "3")]
    pub contract: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgUpdateContractLabel {
    const NAME: &'static str = "MsgUpdateContractLabel";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
/// MsgUpdateContractLabelResponse returns empty data
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateContractLabelResponse {}
impl ::prost::Name for MsgUpdateContractLabelResponse {
    const NAME: &'static str = "MsgUpdateContractLabelResponse";
    const PACKAGE: &'static str = "cosmwasm.wasm.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmwasm.wasm.v1.{}", Self::NAME)
    }
}
include!("cosmwasm.wasm.v1.tonic.rs");
// @@protoc_insertion_point(module)
