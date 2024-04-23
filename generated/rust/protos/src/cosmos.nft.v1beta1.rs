// @generated
/// EventSend is emitted on Msg/Send
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventSend {
    /// class_id associated with the nft
    #[prost(string, tag = "1")]
    pub class_id: ::prost::alloc::string::String,
    /// id is a unique identifier of the nft
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// sender is the address of the owner of nft
    #[prost(string, tag = "3")]
    pub sender: ::prost::alloc::string::String,
    /// receiver is the receiver address of nft
    #[prost(string, tag = "4")]
    pub receiver: ::prost::alloc::string::String,
}
impl ::prost::Name for EventSend {
    const NAME: &'static str = "EventSend";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// EventMint is emitted on Mint
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventMint {
    /// class_id associated with the nft
    #[prost(string, tag = "1")]
    pub class_id: ::prost::alloc::string::String,
    /// id is a unique identifier of the nft
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// owner is the owner address of the nft
    #[prost(string, tag = "3")]
    pub owner: ::prost::alloc::string::String,
}
impl ::prost::Name for EventMint {
    const NAME: &'static str = "EventMint";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// EventBurn is emitted on Burn
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventBurn {
    /// class_id associated with the nft
    #[prost(string, tag = "1")]
    pub class_id: ::prost::alloc::string::String,
    /// id is a unique identifier of the nft
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// owner is the owner address of the nft
    #[prost(string, tag = "3")]
    pub owner: ::prost::alloc::string::String,
}
impl ::prost::Name for EventBurn {
    const NAME: &'static str = "EventBurn";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// Class defines the class of the nft type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Class {
    /// id defines the unique identifier of the NFT classification, similar to the contract address of ERC721
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    /// name defines the human-readable name of the NFT classification. Optional
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    /// symbol is an abbreviated name for nft classification. Optional
    #[prost(string, tag = "3")]
    pub symbol: ::prost::alloc::string::String,
    /// description is a brief description of nft classification. Optional
    #[prost(string, tag = "4")]
    pub description: ::prost::alloc::string::String,
    /// uri for the class metadata stored off chain. It can define schema for Class and NFT `Data` attributes. Optional
    #[prost(string, tag = "5")]
    pub uri: ::prost::alloc::string::String,
    /// uri_hash is a hash of the document pointed by uri. Optional
    #[prost(string, tag = "6")]
    pub uri_hash: ::prost::alloc::string::String,
    /// data is the app specific metadata of the NFT class. Optional
    #[prost(message, optional, tag = "7")]
    pub data: ::core::option::Option<super::super::super::google::protobuf::Any>,
}
impl ::prost::Name for Class {
    const NAME: &'static str = "Class";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// NFT defines the NFT.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Nft {
    /// class_id associated with the NFT, similar to the contract address of ERC721
    #[prost(string, tag = "1")]
    pub class_id: ::prost::alloc::string::String,
    /// id is a unique identifier of the NFT
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// uri for the NFT metadata stored off chain
    #[prost(string, tag = "3")]
    pub uri: ::prost::alloc::string::String,
    /// uri_hash is a hash of the document pointed by uri
    #[prost(string, tag = "4")]
    pub uri_hash: ::prost::alloc::string::String,
    /// data is an app specific data of the NFT. Optional
    #[prost(message, optional, tag = "10")]
    pub data: ::core::option::Option<super::super::super::google::protobuf::Any>,
}
impl ::prost::Name for Nft {
    const NAME: &'static str = "NFT";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// GenesisState defines the nft module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// class defines the class of the nft type.
    #[prost(message, repeated, tag = "1")]
    pub classes: ::prost::alloc::vec::Vec<Class>,
    /// entry defines all nft owned by a person.
    #[prost(message, repeated, tag = "2")]
    pub entries: ::prost::alloc::vec::Vec<Entry>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// Entry Defines all nft owned by a person
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Entry {
    /// owner is the owner address of the following nft
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    /// nfts is a group of nfts of the same owner
    #[prost(message, repeated, tag = "2")]
    pub nfts: ::prost::alloc::vec::Vec<Nft>,
}
impl ::prost::Name for Entry {
    const NAME: &'static str = "Entry";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// QueryBalanceRequest is the request type for the Query/Balance RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBalanceRequest {
    /// class_id associated with the nft
    #[prost(string, tag = "1")]
    pub class_id: ::prost::alloc::string::String,
    /// owner is the owner address of the nft
    #[prost(string, tag = "2")]
    pub owner: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryBalanceRequest {
    const NAME: &'static str = "QueryBalanceRequest";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// QueryBalanceResponse is the response type for the Query/Balance RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBalanceResponse {
    /// amount is the number of all NFTs of a given class owned by the owner
    #[prost(uint64, tag = "1")]
    pub amount: u64,
}
impl ::prost::Name for QueryBalanceResponse {
    const NAME: &'static str = "QueryBalanceResponse";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// QueryOwnerRequest is the request type for the Query/Owner RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryOwnerRequest {
    /// class_id associated with the nft
    #[prost(string, tag = "1")]
    pub class_id: ::prost::alloc::string::String,
    /// id is a unique identifier of the NFT
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryOwnerRequest {
    const NAME: &'static str = "QueryOwnerRequest";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// QueryOwnerResponse is the response type for the Query/Owner RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryOwnerResponse {
    /// owner is the owner address of the nft
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryOwnerResponse {
    const NAME: &'static str = "QueryOwnerResponse";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// QuerySupplyRequest is the request type for the Query/Supply RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySupplyRequest {
    /// class_id associated with the nft
    #[prost(string, tag = "1")]
    pub class_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QuerySupplyRequest {
    const NAME: &'static str = "QuerySupplyRequest";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// QuerySupplyResponse is the response type for the Query/Supply RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySupplyResponse {
    /// amount is the number of all NFTs from the given class
    #[prost(uint64, tag = "1")]
    pub amount: u64,
}
impl ::prost::Name for QuerySupplyResponse {
    const NAME: &'static str = "QuerySupplyResponse";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// QueryNFTstRequest is the request type for the Query/NFTs RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryNfTsRequest {
    /// class_id associated with the nft
    #[prost(string, tag = "1")]
    pub class_id: ::prost::alloc::string::String,
    /// owner is the owner address of the nft
    #[prost(string, tag = "2")]
    pub owner: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "3")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryNfTsRequest {
    const NAME: &'static str = "QueryNFTsRequest";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// QueryNFTsResponse is the response type for the Query/NFTs RPC methods
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryNfTsResponse {
    /// NFT defines the NFT
    #[prost(message, repeated, tag = "1")]
    pub nfts: ::prost::alloc::vec::Vec<Nft>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryNfTsResponse {
    const NAME: &'static str = "QueryNFTsResponse";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// QueryNFTRequest is the request type for the Query/NFT RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryNftRequest {
    /// class_id associated with the nft
    #[prost(string, tag = "1")]
    pub class_id: ::prost::alloc::string::String,
    /// id is a unique identifier of the NFT
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryNftRequest {
    const NAME: &'static str = "QueryNFTRequest";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// QueryNFTResponse is the response type for the Query/NFT RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryNftResponse {
    /// owner is the owner address of the nft
    #[prost(message, optional, tag = "1")]
    pub nft: ::core::option::Option<Nft>,
}
impl ::prost::Name for QueryNftResponse {
    const NAME: &'static str = "QueryNFTResponse";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// QueryClassRequest is the request type for the Query/Class RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClassRequest {
    /// class_id associated with the nft
    #[prost(string, tag = "1")]
    pub class_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryClassRequest {
    const NAME: &'static str = "QueryClassRequest";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// QueryClassResponse is the response type for the Query/Class RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClassResponse {
    /// class defines the class of the nft type.
    #[prost(message, optional, tag = "1")]
    pub class: ::core::option::Option<Class>,
}
impl ::prost::Name for QueryClassResponse {
    const NAME: &'static str = "QueryClassResponse";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// QueryClassesRequest is the request type for the Query/Classes RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClassesRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryClassesRequest {
    const NAME: &'static str = "QueryClassesRequest";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// QueryClassesResponse is the response type for the Query/Classes RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClassesResponse {
    /// class defines the class of the nft type.
    #[prost(message, repeated, tag = "1")]
    pub classes: ::prost::alloc::vec::Vec<Class>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryClassesResponse {
    const NAME: &'static str = "QueryClassesResponse";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// MsgSend represents a message to send a nft from one account to another account.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSend {
    /// class_id defines the unique identifier of the nft classification, similar to the contract address of ERC721
    #[prost(string, tag = "1")]
    pub class_id: ::prost::alloc::string::String,
    /// id defines the unique identification of nft
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// sender is the address of the owner of nft
    #[prost(string, tag = "3")]
    pub sender: ::prost::alloc::string::String,
    /// receiver is the receiver address of nft
    #[prost(string, tag = "4")]
    pub receiver: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSend {
    const NAME: &'static str = "MsgSend";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
/// MsgSendResponse defines the Msg/Send response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSendResponse {}
impl ::prost::Name for MsgSendResponse {
    const NAME: &'static str = "MsgSendResponse";
    const PACKAGE: &'static str = "cosmos.nft.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.nft.v1beta1.{}", Self::NAME)
    }
}
include!("cosmos.nft.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
