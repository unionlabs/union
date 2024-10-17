// @generated
/// Block is tendermint type Block, with the Header proposer address
/// field converted to bech32 string.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Block {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<Header>,
    #[prost(message, optional, tag = "2")]
    pub data: ::core::option::Option<super::super::super::super::cometbft::types::v1::Data>,
    #[prost(message, optional, tag = "3")]
    pub evidence:
        ::core::option::Option<super::super::super::super::cometbft::types::v1::EvidenceList>,
    #[prost(message, optional, tag = "4")]
    pub last_commit:
        ::core::option::Option<super::super::super::super::cometbft::types::v1::Commit>,
}
impl ::prost::Name for Block {
    const NAME: &'static str = "Block";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// Header defines the structure of a Tendermint block header.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    /// basic block info
    #[prost(message, optional, tag = "1")]
    pub version:
        ::core::option::Option<super::super::super::super::cometbft::version::v1::Consensus>,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(int64, tag = "3")]
    pub height: i64,
    #[prost(message, optional, tag = "4")]
    pub time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// prev block info
    #[prost(message, optional, tag = "5")]
    pub last_block_id:
        ::core::option::Option<super::super::super::super::cometbft::types::v1::BlockId>,
    /// hashes of block data
    ///
    /// commit from validators from the last block
    #[prost(bytes = "vec", tag = "6")]
    pub last_commit_hash: ::prost::alloc::vec::Vec<u8>,
    /// transactions
    #[prost(bytes = "vec", tag = "7")]
    pub data_hash: ::prost::alloc::vec::Vec<u8>,
    /// hashes from the app output from the prev block
    ///
    /// validators for the current block
    #[prost(bytes = "vec", tag = "8")]
    pub validators_hash: ::prost::alloc::vec::Vec<u8>,
    /// validators for the next block
    #[prost(bytes = "vec", tag = "9")]
    pub next_validators_hash: ::prost::alloc::vec::Vec<u8>,
    /// consensus params for current block
    #[prost(bytes = "vec", tag = "10")]
    pub consensus_hash: ::prost::alloc::vec::Vec<u8>,
    /// state after txs from the previous block
    #[prost(bytes = "vec", tag = "11")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
    /// root hash of all results from the txs from the previous block
    #[prost(bytes = "vec", tag = "12")]
    pub last_results_hash: ::prost::alloc::vec::Vec<u8>,
    /// consensus info
    ///
    /// evidence included in the block
    #[prost(bytes = "vec", tag = "13")]
    pub evidence_hash: ::prost::alloc::vec::Vec<u8>,
    /// proposer_address is the original block proposer address, formatted as a Bech32 string.
    /// In Tendermint, this type is `bytes`, but in the SDK, we convert it to a Bech32 string
    /// for better UX.
    ///
    /// original proposer of the block
    #[prost(string, tag = "14")]
    pub proposer_address: ::prost::alloc::string::String,
}
impl ::prost::Name for Header {
    const NAME: &'static str = "Header";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// GetValidatorSetByHeightRequest is the request type for the Query/GetValidatorSetByHeight RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetValidatorSetByHeightRequest {
    #[prost(int64, tag = "1")]
    pub height: i64,
    /// pagination defines an pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::query::v1beta1::PageRequest>,
}
impl ::prost::Name for GetValidatorSetByHeightRequest {
    const NAME: &'static str = "GetValidatorSetByHeightRequest";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// GetValidatorSetByHeightResponse is the response type for the Query/GetValidatorSetByHeight RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetValidatorSetByHeightResponse {
    #[prost(int64, tag = "1")]
    pub block_height: i64,
    #[prost(message, repeated, tag = "2")]
    pub validators: ::prost::alloc::vec::Vec<Validator>,
    /// pagination defines an pagination for the response.
    #[prost(message, optional, tag = "3")]
    pub pagination: ::core::option::Option<super::super::query::v1beta1::PageResponse>,
}
impl ::prost::Name for GetValidatorSetByHeightResponse {
    const NAME: &'static str = "GetValidatorSetByHeightResponse";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// GetLatestValidatorSetRequest is the request type for the Query/GetValidatorSetByHeight RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLatestValidatorSetRequest {
    /// pagination defines an pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<super::super::query::v1beta1::PageRequest>,
}
impl ::prost::Name for GetLatestValidatorSetRequest {
    const NAME: &'static str = "GetLatestValidatorSetRequest";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// GetLatestValidatorSetResponse is the response type for the Query/GetValidatorSetByHeight RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLatestValidatorSetResponse {
    #[prost(int64, tag = "1")]
    pub block_height: i64,
    #[prost(message, repeated, tag = "2")]
    pub validators: ::prost::alloc::vec::Vec<Validator>,
    /// pagination defines an pagination for the response.
    #[prost(message, optional, tag = "3")]
    pub pagination: ::core::option::Option<super::super::query::v1beta1::PageResponse>,
}
impl ::prost::Name for GetLatestValidatorSetResponse {
    const NAME: &'static str = "GetLatestValidatorSetResponse";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// Validator is the type for the validator-set.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Validator {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub pub_key: ::core::option::Option<::pbjson_types::Any>,
    #[prost(int64, tag = "3")]
    pub voting_power: i64,
    #[prost(int64, tag = "4")]
    pub proposer_priority: i64,
}
impl ::prost::Name for Validator {
    const NAME: &'static str = "Validator";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// GetBlockByHeightRequest is the request type for the Query/GetBlockByHeight RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockByHeightRequest {
    #[prost(int64, tag = "1")]
    pub height: i64,
}
impl ::prost::Name for GetBlockByHeightRequest {
    const NAME: &'static str = "GetBlockByHeightRequest";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// GetBlockByHeightResponse is the response type for the Query/GetBlockByHeight RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockByHeightResponse {
    #[prost(message, optional, tag = "1")]
    pub block_id: ::core::option::Option<super::super::super::super::cometbft::types::v1::BlockId>,
    /// Deprecated: please use `sdk_block` instead
    #[prost(message, optional, tag = "2")]
    pub block: ::core::option::Option<super::super::super::super::cometbft::types::v1::Block>,
    #[prost(message, optional, tag = "3")]
    pub sdk_block: ::core::option::Option<Block>,
}
impl ::prost::Name for GetBlockByHeightResponse {
    const NAME: &'static str = "GetBlockByHeightResponse";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// GetLatestBlockRequest is the request type for the Query/GetLatestBlock RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLatestBlockRequest {}
impl ::prost::Name for GetLatestBlockRequest {
    const NAME: &'static str = "GetLatestBlockRequest";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// GetLatestBlockResponse is the response type for the Query/GetLatestBlock RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLatestBlockResponse {
    #[prost(message, optional, tag = "1")]
    pub block_id: ::core::option::Option<super::super::super::super::cometbft::types::v1::BlockId>,
    /// Deprecated: please use `sdk_block` instead
    #[prost(message, optional, tag = "2")]
    pub block: ::core::option::Option<super::super::super::super::cometbft::types::v1::Block>,
    #[prost(message, optional, tag = "3")]
    pub sdk_block: ::core::option::Option<Block>,
}
impl ::prost::Name for GetLatestBlockResponse {
    const NAME: &'static str = "GetLatestBlockResponse";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// GetSyncingRequest is the request type for the Query/GetSyncing RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetSyncingRequest {}
impl ::prost::Name for GetSyncingRequest {
    const NAME: &'static str = "GetSyncingRequest";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// GetSyncingResponse is the response type for the Query/GetSyncing RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetSyncingResponse {
    #[prost(bool, tag = "1")]
    pub syncing: bool,
}
impl ::prost::Name for GetSyncingResponse {
    const NAME: &'static str = "GetSyncingResponse";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// GetNodeInfoRequest is the request type for the Query/GetNodeInfo RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetNodeInfoRequest {}
impl ::prost::Name for GetNodeInfoRequest {
    const NAME: &'static str = "GetNodeInfoRequest";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// GetNodeInfoResponse is the response type for the Query/GetNodeInfo RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetNodeInfoResponse {
    #[prost(message, optional, tag = "1")]
    pub default_node_info:
        ::core::option::Option<super::super::super::super::cometbft::p2p::v1::DefaultNodeInfo>,
    #[prost(message, optional, tag = "2")]
    pub application_version: ::core::option::Option<VersionInfo>,
}
impl ::prost::Name for GetNodeInfoResponse {
    const NAME: &'static str = "GetNodeInfoResponse";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// VersionInfo is the type for the GetNodeInfoResponse message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VersionInfo {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub app_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub version: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub git_commit: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub build_tags: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub go_version: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "7")]
    pub build_deps: ::prost::alloc::vec::Vec<Module>,
    #[prost(string, tag = "8")]
    pub cosmos_sdk_version: ::prost::alloc::string::String,
}
impl ::prost::Name for VersionInfo {
    const NAME: &'static str = "VersionInfo";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// Module is the type for VersionInfo
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Module {
    /// module path
    #[prost(string, tag = "1")]
    pub path: ::prost::alloc::string::String,
    /// module version
    #[prost(string, tag = "2")]
    pub version: ::prost::alloc::string::String,
    /// checksum
    #[prost(string, tag = "3")]
    pub sum: ::prost::alloc::string::String,
}
impl ::prost::Name for Module {
    const NAME: &'static str = "Module";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// ABCIQueryRequest defines the request structure for the ABCIQuery gRPC query.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AbciQueryRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "2")]
    pub path: ::prost::alloc::string::String,
    #[prost(int64, tag = "3")]
    pub height: i64,
    #[prost(bool, tag = "4")]
    pub prove: bool,
}
impl ::prost::Name for AbciQueryRequest {
    const NAME: &'static str = "ABCIQueryRequest";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// ABCIQueryResponse defines the response structure for the ABCIQuery gRPC query.
///
/// Note: This type is a duplicate of the ResponseQuery proto type defined in
/// Tendermint.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AbciQueryResponse {
    #[prost(uint32, tag = "1")]
    pub code: u32,
    /// nondeterministic
    #[prost(string, tag = "3")]
    pub log: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(string, tag = "4")]
    pub info: ::prost::alloc::string::String,
    #[prost(int64, tag = "5")]
    pub index: i64,
    #[prost(bytes = "vec", tag = "6")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "7")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag = "9")]
    pub height: i64,
    #[prost(string, tag = "10")]
    pub codespace: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "11")]
    pub proof_ops:
        ::core::option::Option<super::super::super::super::cometbft::crypto::v1::ProofOps>,
}
impl ::prost::Name for AbciQueryResponse {
    const NAME: &'static str = "ABCIQueryResponse";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// ProofOp defines an operation used for calculating Merkle root. The data could
/// be arbitrary format, providing necessary data for example neighbouring node
/// hash.
///
/// Note: This type is a duplicate of the ProofOp proto type defined in Tendermint.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProofOp {
    #[prost(string, tag = "1")]
    pub r#type: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ProofOp {
    const NAME: &'static str = "ProofOp";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
/// ProofOps is Merkle proof defined by the list of ProofOps.
///
/// Note: This type is a duplicate of the ProofOps proto type defined in Tendermint.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProofOps {
    #[prost(message, repeated, tag = "1")]
    pub ops: ::prost::alloc::vec::Vec<ProofOp>,
}
impl ::prost::Name for ProofOps {
    const NAME: &'static str = "ProofOps";
    const PACKAGE: &'static str = "cosmos.base.tendermint.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.tendermint.v1beta1.{}", Self::NAME)
    }
}
include!("cosmos.base.tendermint.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
