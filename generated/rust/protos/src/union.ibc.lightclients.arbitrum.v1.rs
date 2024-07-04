// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(string, tag = "1")]
    pub l1_client_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub l1_latest_slot: u64,
    /// The address of the [`RollupCore`](<https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/RollupCore.sol>) contract on the L1.
    #[prost(bytes = "vec", tag = "4")]
    pub l1_contract_address: ::prost::alloc::vec::Vec<u8>,
    /// The slot containing the next node num. If tracking full finalization, this will be the slot containing `_latestConfirmed`, otherwise if tracking head this will be the slot containing `_latestNodeCreated`.
    ///
    /// Since these are [stored in the same slot](<https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/RollupCore.sol#L60-L63>) in the arbitrum rollup contract, `l1_next_node_num_slot_offset` is provided as a way to subscript into the slot.
    #[prost(bytes = "vec", tag = "5")]
    pub l1_next_node_num_slot: ::prost::alloc::vec::Vec<u8>,
    /// The offset of the u64 value of the node num within the `l1_lext_node_num_slot`.
    #[prost(uint32, tag = "6")]
    pub l1_next_node_num_slot_offset_bytes: u32,
    /// _nodes\[l1_next_node_num_slot\].confirmData
    #[prost(bytes = "vec", tag = "7")]
    pub l1_nodes_slot: ::prost::alloc::vec::Vec<u8>,
    /// offset of Node.confirmData
    #[prost(bytes = "vec", tag = "8")]
    pub l1_nodes_confirm_data_offset: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "9")]
    pub frozen_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    #[prost(bytes = "vec", tag = "10")]
    pub l2_ibc_contract_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "11")]
    pub l2_ibc_commitment_slot: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ClientState {
    const NAME: &'static str = "ClientState";
    const PACKAGE: &'static str = "union.ibc.lightclients.arbitrum.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.arbitrum.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    #[prost(bytes = "vec", tag = "1")]
    pub ibc_storage_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub timestamp: u64,
}
impl ::prost::Name for ConsensusState {
    const NAME: &'static str = "ConsensusState";
    const PACKAGE: &'static str = "union.ibc.lightclients.arbitrum.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.arbitrum.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    /// The height of the L1 to update to. The L1 client must have a consensus state at this height.
    #[prost(message, optional, tag = "1")]
    pub l1_height:
        ::core::option::Option<super::super::super::super::super::ibc::core::client::v1::Height>,
    /// Proof of the L1 rollup account in the L1 state root.
    #[prost(message, optional, tag = "2")]
    pub l1_account_proof: ::core::option::Option<super::super::ethereum::v1::AccountProof>,
    /// Proof of the L2 ibc contract address in the L2 state root.
    #[prost(message, optional, tag = "3")]
    pub l2_ibc_account_proof: ::core::option::Option<super::super::ethereum::v1::AccountProof>,
    /// The latest confirmed node number, as stored in `ClientState.l1_next_node_num_slot` at `ClientState.l1_next_node_num_slot_offset_bytes`.
    #[prost(message, optional, tag = "7")]
    pub l1_next_node_num_slot_proof:
        ::core::option::Option<super::super::ethereum::v1::StorageProof>,
    /// The proof of the \[`_nodes`\] mapping at `latest_confirmed`, offset to \[`Node.confirmData`\].
    ///
    /// \[`_nodes`\]: <https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/RollupCore.sol#L64>
    /// \[`Node.confirmData`\]: <https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/Node.sol#L27>
    #[prost(message, optional, tag = "8")]
    pub l1_nodes_slot_proof: ::core::option::Option<super::super::ethereum::v1::StorageProof>,
    /// Arbitrum block header, used to recompute the block hash and verify the timestamp.
    #[prost(message, optional, tag = "9")]
    pub l2_header: ::core::option::Option<L2Header>,
}
impl ::prost::Name for Header {
    const NAME: &'static str = "Header";
    const PACKAGE: &'static str = "union.ibc.lightclients.arbitrum.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.arbitrum.v1.{}", Self::NAME)
    }
}
/// The Arbitrum header as returned from `eth_getBlockByNumber`, with all non-standard fields removed.
///
/// Note that certain fields are different than a typical eth_getBlockByNumber response; see [here](<https://docs.arbitrum.io/build-decentralized-apps/arbitrum-vs-ethereum/rpc-methods#existing-fields-with-different-behavior-1>) for more information.
///
/// <https://github.com/OffchainLabs/go-ethereum/blob/f94174378de6ea7cf02963d99489e69b6671d1aa/core/types/block.go#L66-L80>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct L2Header {
    /// H256
    #[prost(bytes = "vec", tag = "1")]
    pub parent_hash: ::prost::alloc::vec::Vec<u8>,
    /// H256
    #[prost(bytes = "vec", tag = "2")]
    pub sha3_uncles: ::prost::alloc::vec::Vec<u8>,
    /// H160
    #[prost(bytes = "vec", tag = "3")]
    pub miner: ::prost::alloc::vec::Vec<u8>,
    /// H256
    #[prost(bytes = "vec", tag = "4")]
    pub state_root: ::prost::alloc::vec::Vec<u8>,
    /// H256
    #[prost(bytes = "vec", tag = "5")]
    pub transactions_root: ::prost::alloc::vec::Vec<u8>,
    /// H256
    #[prost(bytes = "vec", tag = "6")]
    pub receipts_root: ::prost::alloc::vec::Vec<u8>,
    /// H2048
    #[prost(bytes = "vec", tag = "7")]
    pub logs_bloom: ::prost::alloc::vec::Vec<u8>,
    /// U256
    #[prost(bytes = "vec", tag = "8")]
    pub difficulty: ::prost::alloc::vec::Vec<u8>,
    /// U256
    #[prost(bytes = "vec", tag = "9")]
    pub number: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "10")]
    pub gas_limit: u64,
    #[prost(uint64, tag = "11")]
    pub gas_used: u64,
    #[prost(uint64, tag = "12")]
    pub timestamp: u64,
    /// This field is equivalent to sendRoot.
    ///
    /// H256
    #[prost(bytes = "vec", tag = "13")]
    pub extra_data: ::prost::alloc::vec::Vec<u8>,
    /// H256
    #[prost(bytes = "vec", tag = "14")]
    pub mix_hash: ::prost::alloc::vec::Vec<u8>,
    /// H64
    #[prost(bytes = "vec", tag = "15")]
    pub nonce: ::prost::alloc::vec::Vec<u8>,
    /// U256
    #[prost(bytes = "vec", tag = "16")]
    pub base_fee_per_gas: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for L2Header {
    const NAME: &'static str = "L2Header";
    const PACKAGE: &'static str = "union.ibc.lightclients.arbitrum.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.arbitrum.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
