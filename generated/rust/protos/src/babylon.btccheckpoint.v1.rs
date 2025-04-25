// @generated
/// Consider we have a Merkle tree with following structure:
///             ROOT
///            /    \
///       H1234      H5555
///      /     \       \
///    H12     H34      H55
///   /  \    /  \     /
/// H1  H2  H3  H4  H5
/// L1  L2  L3  L4  L5
/// To prove L3 was part of ROOT we need:
/// - btc_transaction_index = 2 which in binary is 010
/// (where 0 means going left, 1 means going right in the tree)
/// - merkle_nodes we'd have H4 || H12 || H5555
/// By looking at 010 we would know that H4 is a right sibling,
/// H12 is left, H5555 is right again.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcSpvProof {
    /// Valid bitcoin transaction containing OP_RETURN opcode.
    #[prost(bytes = "vec", tag = "1")]
    pub btc_transaction: ::prost::alloc::vec::Vec<u8>,
    /// Index of transaction within the block. Index is needed to determine if
    /// currently hashed node is left or right.
    #[prost(uint32, tag = "2")]
    pub btc_transaction_index: u32,
    /// List of concatenated intermediate merkle tree nodes, without root node and
    /// leaf node against which we calculate the proof. Each node has 32 byte
    /// length. Example proof can look like: 32_bytes_of_node1 || 32_bytes_of_node2
    /// ||  32_bytes_of_node3 so the length of the proof will always be divisible
    /// by 32.
    #[prost(bytes = "vec", tag = "3")]
    pub merkle_nodes: ::prost::alloc::vec::Vec<u8>,
    /// Valid btc header which confirms btc_transaction.
    /// Should have exactly 80 bytes
    #[prost(bytes = "vec", tag = "4")]
    pub confirming_btc_header: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for BtcSpvProof {
    const NAME: &'static str = "BTCSpvProof";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// Each provided OP_RETURN transaction can be identified by hash of block in
/// which transaction was included and transaction index in the block
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionKey {
    #[prost(uint32, tag = "1")]
    pub index: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for TransactionKey {
    const NAME: &'static str = "TransactionKey";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// Checkpoint can be composed from multiple transactions, so to identify whole
/// submission we need list of transaction keys.
/// Each submission can generally be identified by this list of (txIdx,
/// blockHash) tuples. Note: this could possibly be optimized as if transactions
/// were in one block they would have the same block hash and different indexes,
/// but each blockhash is only 33 (1  byte for prefix encoding and 32 byte hash),
/// so there should be other strong arguments for this optimization
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubmissionKey {
    #[prost(message, repeated, tag = "1")]
    pub key: ::prost::alloc::vec::Vec<TransactionKey>,
}
impl ::prost::Name for SubmissionKey {
    const NAME: &'static str = "SubmissionKey";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// TransactionInfo is the info of a tx on Bitcoin,
/// including
/// - the position of the tx on BTC blockchain
/// - the full tx content
/// - the Merkle proof that this tx is on the above position
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionInfo {
    /// key is the position (txIdx, blockHash) of this tx on BTC blockchain
    /// Although it is already a part of SubmissionKey, we store it here again
    /// to make TransactionInfo self-contained.
    /// For example, storing the key allows TransactionInfo to not relay on
    /// the fact that TransactionInfo will be ordered in the same order as
    /// TransactionKeys in SubmissionKey.
    #[prost(message, optional, tag = "1")]
    pub key: ::core::option::Option<TransactionKey>,
    /// transaction is the full transaction in bytes
    #[prost(bytes = "vec", tag = "2")]
    pub transaction: ::prost::alloc::vec::Vec<u8>,
    /// proof is the Merkle proof that this tx is included in the position in `key`
    /// TODO: maybe it could use here better format as we already processed and
    /// validated the proof?
    #[prost(bytes = "vec", tag = "3")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for TransactionInfo {
    const NAME: &'static str = "TransactionInfo";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// TODO: Determine if we should keep any block number or depth info.
/// On one hand it may be useful to determine if block is stable or not, on
/// other depth/block number info, without context (i.e info about chain) is
/// pretty useless and blockhash in enough to retrieve is from lightclient
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubmissionData {
    /// address of the submitter and reporter
    #[prost(message, optional, tag = "1")]
    pub vigilante_addresses: ::core::option::Option<CheckpointAddresses>,
    /// txs_info is the two `TransactionInfo`s corresponding to the submission
    /// It is used for
    /// - recovering address of sender of btc transaction to payup the reward.
    /// - allowing the ZoneConcierge module to prove the checkpoint is submitted to
    /// BTC
    #[prost(message, repeated, tag = "2")]
    pub txs_info: ::prost::alloc::vec::Vec<TransactionInfo>,
    #[prost(uint64, tag = "3")]
    pub epoch: u64,
}
impl ::prost::Name for SubmissionData {
    const NAME: &'static str = "SubmissionData";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// Data stored in db and indexed by epoch number
/// TODO: Add btc blockheight at epoch end, when adding handling of epoching
/// callbacks
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EpochData {
    /// keys is the list of all received checkpoints during this epoch, sorted by
    /// order of submission.
    #[prost(message, repeated, tag = "1")]
    pub keys: ::prost::alloc::vec::Vec<SubmissionKey>,
    /// status is the current btc status of the epoch
    #[prost(enumeration = "BtcStatus", tag = "2")]
    pub status: i32,
}
impl ::prost::Name for EpochData {
    const NAME: &'static str = "EpochData";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// CheckpointAddresses contains the addresses of the submitter and reporter of a
/// given checkpoint
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CheckpointAddresses {
    /// TODO: this could probably be better typed
    /// submitter is the address of the checkpoint submitter to BTC, extracted from
    /// the checkpoint itself.
    #[prost(bytes = "vec", tag = "1")]
    pub submitter: ::prost::alloc::vec::Vec<u8>,
    /// reporter is the address of the reporter who reported the submissions,
    /// calculated from submission message MsgInsertBTCSpvProof itself
    #[prost(bytes = "vec", tag = "2")]
    pub reporter: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for CheckpointAddresses {
    const NAME: &'static str = "CheckpointAddresses";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// BTCCheckpointInfo contains all data about best submission of checkpoint for
/// given epoch. Best submission is the submission which is deeper in btc ledger
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcCheckpointInfo {
    /// epoch number of this checkpoint
    #[prost(uint64, tag = "1")]
    pub epoch_number: u64,
    /// btc height of the best submission of the epoch
    #[prost(uint32, tag = "2")]
    pub best_submission_btc_block_height: u32,
    /// hash of the btc block which determines checkpoint btc block height i.e.
    /// youngest block of best submission
    #[prost(bytes = "vec", tag = "3")]
    pub best_submission_btc_block_hash: ::prost::alloc::vec::Vec<u8>,
    /// the BTC checkpoint transactions of the best submission
    #[prost(message, repeated, tag = "4")]
    pub best_submission_transactions: ::prost::alloc::vec::Vec<TransactionInfo>,
    /// list of vigilantes' addresses of the best submission
    #[prost(message, repeated, tag = "5")]
    pub best_submission_vigilante_address_list: ::prost::alloc::vec::Vec<CheckpointAddresses>,
}
impl ::prost::Name for BtcCheckpointInfo {
    const NAME: &'static str = "BTCCheckpointInfo";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// BtcStatus is an enum describing the current btc status of the checkpoint
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum BtcStatus {
    /// SUBMITTED Epoch has Submitted btc status if there ever was at least one
    /// known submission on btc main chain
    EpochStatusSubmitted = 0,
    /// CONFIRMED Epoch has Confirmed btc status if there ever was at least one
    /// known submission on btc main chain which was k-deep
    EpochStatusConfirmed = 1,
    /// CONFIRMED Epoch has Finalized btc status if there is was at exactly one
    /// knon submission on btc main chain which is w-deep
    EpochStatusFinalized = 2,
}
impl BtcStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            BtcStatus::EpochStatusSubmitted => "EPOCH_STATUS_SUBMITTED",
            BtcStatus::EpochStatusConfirmed => "EPOCH_STATUS_CONFIRMED",
            BtcStatus::EpochStatusFinalized => "EPOCH_STATUS_FINALIZED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "EPOCH_STATUS_SUBMITTED" => Some(Self::EpochStatusSubmitted),
            "EPOCH_STATUS_CONFIRMED" => Some(Self::EpochStatusConfirmed),
            "EPOCH_STATUS_FINALIZED" => Some(Self::EpochStatusFinalized),
            _ => None,
        }
    }
}
/// Params defines the parameters for the module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// btc_confirmation_depth is the confirmation depth in BTC.
    /// A block is considered irreversible only when it is at least k-deep in BTC
    /// (k in research paper)
    #[prost(uint32, tag = "1")]
    pub btc_confirmation_depth: u32,
    /// checkpoint_finalization_timeout is the maximum time window (measured in BTC
    /// blocks) between a checkpoint
    /// - being submitted to BTC, and
    /// - being reported back to BBN
    /// If a checkpoint has not been reported back within w BTC blocks, then BBN
    /// has dishonest majority and is stalling checkpoints (w in research paper)
    #[prost(uint32, tag = "2")]
    pub checkpoint_finalization_timeout: u32,
    /// 4byte tag in hex format, required to be present in the OP_RETURN transaction
    /// related to babylon
    #[prost(string, tag = "3")]
    pub checkpoint_tag: ::prost::alloc::string::String,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// GenesisState defines the btccheckpoint module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// QueryParamsRequest is request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// QueryParamsResponse is response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params holds all the parameters of this module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// QueryBtcCheckpointInfoRequest defines the query to get the best checkpoint
/// for a given epoch
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBtcCheckpointInfoRequest {
    /// Number of epoch for which the earliest checkpointing btc height is
    /// requested
    #[prost(uint64, tag = "1")]
    pub epoch_num: u64,
}
impl ::prost::Name for QueryBtcCheckpointInfoRequest {
    const NAME: &'static str = "QueryBtcCheckpointInfoRequest";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// QueryBtcCheckpointInfoResponse is response type for the
/// Query/BtcCheckpointInfo RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBtcCheckpointInfoResponse {
    #[prost(message, optional, tag = "1")]
    pub info: ::core::option::Option<BtcCheckpointInfoResponse>,
}
impl ::prost::Name for QueryBtcCheckpointInfoResponse {
    const NAME: &'static str = "QueryBtcCheckpointInfoResponse";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// QueryBtcCheckpointsInfoRequest is request type for the
/// Query/BtcCheckpointsInfo RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBtcCheckpointsInfoRequest {
    /// pagination defines whether to have the pagination in the request
    #[prost(message, optional, tag = "1")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryBtcCheckpointsInfoRequest {
    const NAME: &'static str = "QueryBtcCheckpointsInfoRequest";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// QueryBtcCheckpointsInfoResponse is response type for the
/// Query/BtcCheckpointsInfo RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBtcCheckpointsInfoResponse {
    #[prost(message, repeated, tag = "1")]
    pub info_list: ::prost::alloc::vec::Vec<BtcCheckpointInfoResponse>,
    /// pagination defines the pagination in the response
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryBtcCheckpointsInfoResponse {
    const NAME: &'static str = "QueryBtcCheckpointsInfoResponse";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// QueryEpochSubmissionsRequest defines a request to get all submissions in
/// given epoch
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEpochSubmissionsRequest {
    /// Number of epoch for which submissions are requested
    #[prost(uint64, tag = "1")]
    pub epoch_num: u64,
}
impl ::prost::Name for QueryEpochSubmissionsRequest {
    const NAME: &'static str = "QueryEpochSubmissionsRequest";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// QueryEpochSubmissionsResponse defines a response to get all submissions in
/// given epoch (QueryEpochSubmissionsRequest)
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEpochSubmissionsResponse {
    /// Keys All submissions transactions key saved during an epoch.
    #[prost(message, repeated, tag = "1")]
    pub keys: ::prost::alloc::vec::Vec<SubmissionKeyResponse>,
}
impl ::prost::Name for QueryEpochSubmissionsResponse {
    const NAME: &'static str = "QueryEpochSubmissionsResponse";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// BTCCheckpointInfoResponse contains all data about best submission of checkpoint for
/// given epoch. Best submission is the submission which is deeper in btc ledger.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcCheckpointInfoResponse {
    /// EpochNumber of this checkpoint.
    #[prost(uint64, tag = "1")]
    pub epoch_number: u64,
    /// btc height of the best submission of the epoch
    #[prost(uint32, tag = "2")]
    pub best_submission_btc_block_height: u32,
    /// hash of the btc block which determines checkpoint btc block height i.e.
    /// youngest block of best submission Hexadecimal
    #[prost(string, tag = "3")]
    pub best_submission_btc_block_hash: ::prost::alloc::string::String,
    /// the BTC checkpoint transactions of the best submission
    #[prost(message, repeated, tag = "4")]
    pub best_submission_transactions: ::prost::alloc::vec::Vec<TransactionInfoResponse>,
    /// list of vigilantes' addresses of the best submission
    #[prost(message, repeated, tag = "5")]
    pub best_submission_vigilante_address_list:
        ::prost::alloc::vec::Vec<CheckpointAddressesResponse>,
}
impl ::prost::Name for BtcCheckpointInfoResponse {
    const NAME: &'static str = "BTCCheckpointInfoResponse";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// TransactionInfoResponse is the info of a tx on Bitcoin,
/// including
/// - the position of the tx on BTC blockchain
/// - the full tx content
/// - the Merkle proof that this tx is on the above position
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionInfoResponse {
    /// Index Bitcoin Transaction index in block.
    #[prost(uint32, tag = "1")]
    pub index: u32,
    /// Hash BTC Header hash as hex.
    #[prost(string, tag = "2")]
    pub hash: ::prost::alloc::string::String,
    /// transaction is the full transaction data as str hex.
    #[prost(string, tag = "3")]
    pub transaction: ::prost::alloc::string::String,
    /// proof is the Merkle proof that this tx is included in the position in `key`
    #[prost(string, tag = "4")]
    pub proof: ::prost::alloc::string::String,
}
impl ::prost::Name for TransactionInfoResponse {
    const NAME: &'static str = "TransactionInfoResponse";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// CheckpointAddressesResponse contains the addresses of the submitter and reporter of a
/// given checkpoint
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CheckpointAddressesResponse {
    /// submitter is the address of the checkpoint submitter to BTC, extracted from
    /// the checkpoint itself.
    #[prost(string, tag = "1")]
    pub submitter: ::prost::alloc::string::String,
    /// reporter is the address of the reporter who reported the submissions,
    /// calculated from submission message MsgInsertBTCSpvProof itself
    #[prost(string, tag = "2")]
    pub reporter: ::prost::alloc::string::String,
}
impl ::prost::Name for CheckpointAddressesResponse {
    const NAME: &'static str = "CheckpointAddressesResponse";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// SubmissionKeyResponse Checkpoint can be composed from multiple transactions,
/// so to identify whole submission we need list of transaction keys.
/// Each submission can generally be identified by this list of (txIdx,
/// blockHash) tuples. Note: this could possibly be optimized as if transactions
/// were in one block they would have the same block hash and different indexes,
/// but each blockhash is only 33 (1  byte for prefix encoding and 32 byte hash),
/// so there should be other strong arguments for this optimization
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubmissionKeyResponse {
    /// FirstTxBlockHash is the BTCHeaderHashBytes in hex.
    #[prost(string, tag = "1")]
    pub first_tx_block_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag = "2")]
    pub first_tx_index: u32,
    /// SecondBlockHash is the BTCHeaderHashBytes in hex.
    #[prost(string, tag = "3")]
    pub second_tx_block_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag = "4")]
    pub second_tx_index: u32,
}
impl ::prost::Name for SubmissionKeyResponse {
    const NAME: &'static str = "SubmissionKeyResponse";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// MsgInsertBTCSpvProof defines request to insert a new checkpoint into the
/// store
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgInsertBtcSpvProof {
    #[prost(string, tag = "1")]
    pub submitter: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub proofs: ::prost::alloc::vec::Vec<BtcSpvProof>,
}
impl ::prost::Name for MsgInsertBtcSpvProof {
    const NAME: &'static str = "MsgInsertBTCSpvProof";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// MsgInsertBTCSpvProofResponse defines the response for the
/// MsgInsertBTCSpvProof message
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgInsertBtcSpvProofResponse {}
impl ::prost::Name for MsgInsertBtcSpvProofResponse {
    const NAME: &'static str = "MsgInsertBTCSpvProofResponse";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParams defines a message to update the btccheckpoint module params.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// authority is the address of the governance account.
    /// just FYI: cosmos.AddressString marks that this field should use type alias
    /// for AddressString instead of string, but the functionality is not yet implemented
    /// in cosmos-proto
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the btccheckpoint parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParamsResponse defines the response to the MsgUpdateParams message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "babylon.btccheckpoint.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btccheckpoint.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
