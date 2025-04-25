// @generated
/// VotingPowerDistCache is the cache for voting power distribution of finality providers
/// and their BTC delegations at a height
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VotingPowerDistCache {
    /// total_voting_power is the total voting power of all (active) finality providers
    /// in the cache
    #[prost(uint64, tag = "1")]
    pub total_voting_power: u64,
    /// finality_providers is a list of finality providers' voting power information
    #[prost(message, repeated, tag = "2")]
    pub finality_providers: ::prost::alloc::vec::Vec<FinalityProviderDistInfo>,
    /// num_active_fps is the number of finality providers that have active BTC
    /// delegations as well as timestamped public randomness
    #[prost(uint32, tag = "3")]
    pub num_active_fps: u32,
}
impl ::prost::Name for VotingPowerDistCache {
    const NAME: &'static str = "VotingPowerDistCache";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// FinalityProviderDistInfo is the reward distribution of a finality provider and its BTC delegations
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FinalityProviderDistInfo {
    /// btc_pk is the Bitcoin secp256k1 PK of this finality provider
    /// the PK follows encoding in BIP-340 spec
    #[prost(bytes = "vec", tag = "1")]
    pub btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// addr is the bytes of the address to receive commission from delegations.
    #[prost(bytes = "vec", tag = "2")]
    pub addr: ::prost::alloc::vec::Vec<u8>,
    /// commission defines the commission rate of finality provider
    #[prost(string, tag = "3")]
    pub commission: ::prost::alloc::string::String,
    /// total_bonded_sat is the total amount of bonded BTC stake (in Satoshi) of the finality provider
    #[prost(uint64, tag = "4")]
    pub total_bonded_sat: u64,
    /// is_timestamped indicates whether the finality provider
    /// has timestamped public randomness committed
    /// if no, it should not be assigned voting power
    #[prost(bool, tag = "5")]
    pub is_timestamped: bool,
    /// is_jailed indicates whether the finality provider
    /// is jailed, if so, it should not be assigned voting power
    #[prost(bool, tag = "6")]
    pub is_jailed: bool,
    /// is_slashed indicates whether the finality provider
    /// is slashed, if so, it should not be assigned voting power
    #[prost(bool, tag = "7")]
    pub is_slashed: bool,
}
impl ::prost::Name for FinalityProviderDistInfo {
    const NAME: &'static str = "FinalityProviderDistInfo";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// IndexedBlock is the necessary metadata and finalization status of a block
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IndexedBlock {
    /// height is the height of the block
    #[prost(uint64, tag = "1")]
    pub height: u64,
    /// app_hash is the AppHash of the block
    #[prost(bytes = "vec", tag = "2")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
    /// finalized indicates whether the IndexedBlock is finalised by 2/3
    /// finality providers or not
    #[prost(bool, tag = "3")]
    pub finalized: bool,
}
impl ::prost::Name for IndexedBlock {
    const NAME: &'static str = "IndexedBlock";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// PubRandCommit is a commitment to a series of public randomness
/// currently, the commitment is a root of a Merkle tree that includes
/// a series of public randomness
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PubRandCommit {
    /// start_height is the height of the first commitment
    #[prost(uint64, tag = "1")]
    pub start_height: u64,
    /// num_pub_rand is the number of committed public randomness
    #[prost(uint64, tag = "2")]
    pub num_pub_rand: u64,
    /// commitment is the value of the commitment
    /// currently, it is the root of the merkle tree constructed by the public randomness
    #[prost(bytes = "vec", tag = "3")]
    pub commitment: ::prost::alloc::vec::Vec<u8>,
    /// epoch_num defines the epoch number that the commit falls into
    #[prost(uint64, tag = "4")]
    pub epoch_num: u64,
}
impl ::prost::Name for PubRandCommit {
    const NAME: &'static str = "PubRandCommit";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// Evidence is the evidence that a finality provider has signed finality
/// signatures with correct public randomness on two conflicting Babylon headers
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Evidence {
    /// fp_btc_pk is the BTC PK of the finality provider that casts this vote
    #[prost(bytes = "vec", tag = "1")]
    pub fp_btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// block_height is the height of the conflicting blocks
    #[prost(uint64, tag = "2")]
    pub block_height: u64,
    /// pub_rand is the public randomness the finality provider has committed to
    #[prost(bytes = "vec", tag = "3")]
    pub pub_rand: ::prost::alloc::vec::Vec<u8>,
    /// canonical_app_hash is the AppHash of the canonical block
    #[prost(bytes = "vec", tag = "4")]
    pub canonical_app_hash: ::prost::alloc::vec::Vec<u8>,
    /// fork_app_hash is the AppHash of the fork block
    #[prost(bytes = "vec", tag = "5")]
    pub fork_app_hash: ::prost::alloc::vec::Vec<u8>,
    /// canonical_finality_sig is the finality signature to the canonical block
    /// where finality signature is an EOTS signature, i.e.,
    /// the `s` in a Schnorr signature `(r, s)`
    /// `r` is the public randomness that is already committed by the finality provider
    #[prost(bytes = "vec", tag = "6")]
    pub canonical_finality_sig: ::prost::alloc::vec::Vec<u8>,
    /// fork_finality_sig is the finality signature to the fork block
    /// where finality signature is an EOTS signature
    #[prost(bytes = "vec", tag = "7")]
    pub fork_finality_sig: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Evidence {
    const NAME: &'static str = "Evidence";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// FinalityProviderSigningInfo defines a finality provider's signing info for monitoring their
/// liveness activity.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FinalityProviderSigningInfo {
    /// fp_btc_pk is the BTC PK of the finality provider that casts this vote
    #[prost(bytes = "vec", tag = "1")]
    pub fp_btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// start_height is the block height at which finality provider become active
    #[prost(int64, tag = "2")]
    pub start_height: i64,
    /// missed_blocks_counter defines a counter to avoid unnecessary array reads.
    /// Note that `Sum(MissedBlocksBitArray)` always equals `MissedBlocksCounter`.
    #[prost(int64, tag = "3")]
    pub missed_blocks_counter: i64,
    /// Timestamp until which the validator is jailed due to liveness downtime.
    #[prost(message, optional, tag = "4")]
    pub jailed_until: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
}
impl ::prost::Name for FinalityProviderSigningInfo {
    const NAME: &'static str = "FinalityProviderSigningInfo";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// EventSlashedFinalityProvider is the event emitted when a finality provider is slashed
/// due to signing two conflicting blocks
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventSlashedFinalityProvider {
    /// evidence is the evidence that the finality provider double signs
    #[prost(message, optional, tag = "1")]
    pub evidence: ::core::option::Option<Evidence>,
}
impl ::prost::Name for EventSlashedFinalityProvider {
    const NAME: &'static str = "EventSlashedFinalityProvider";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// EventJailedFinalityProvider is the event emitted when a finality provider is
/// jailed due to inactivity
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventJailedFinalityProvider {
    /// public_key is the BTC public key of the finality provider
    #[prost(string, tag = "1")]
    pub public_key: ::prost::alloc::string::String,
}
impl ::prost::Name for EventJailedFinalityProvider {
    const NAME: &'static str = "EventJailedFinalityProvider";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// Params defines the parameters for the module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// max_active_finality_providers is the maximum number of active finality providers in the BTC staking protocol
    #[prost(uint32, tag = "1")]
    pub max_active_finality_providers: u32,
    /// signed_blocks_window defines the size of the sliding window for tracking finality provider liveness
    #[prost(int64, tag = "2")]
    pub signed_blocks_window: i64,
    /// finality_sig_timeout defines how much time (in terms of blocks) finality providers have to cast a finality
    /// vote before being judged as missing their voting turn on the given block
    #[prost(int64, tag = "3")]
    pub finality_sig_timeout: i64,
    /// min_signed_per_window defines the minimum number of blocks that a finality provider is required to sign
    /// within the sliding window to avoid being jailed
    #[prost(bytes = "vec", tag = "4")]
    pub min_signed_per_window: ::prost::alloc::vec::Vec<u8>,
    /// min_pub_rand is the minimum number of public randomness each
    /// message should commit
    #[prost(uint64, tag = "5")]
    pub min_pub_rand: u64,
    /// jail_duration is the minimum period of time that a finality provider remains jailed
    #[prost(message, optional, tag = "6")]
    pub jail_duration: ::core::option::Option<super::super::super::google::protobuf::Duration>,
    /// finality_activation_height is the babylon block height which the finality module will
    /// start to accept finality voting and the minimum allowed value for the public randomness
    /// commit start height.
    #[prost(uint64, tag = "7")]
    pub finality_activation_height: u64,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// GenesisState defines the finality module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// params the current params of the state.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    /// indexed_blocks all the btc blocks and if their status are finalized.
    #[prost(message, repeated, tag = "2")]
    pub indexed_blocks: ::prost::alloc::vec::Vec<IndexedBlock>,
    /// evidences all the evidences ever registered.
    #[prost(message, repeated, tag = "3")]
    pub evidences: ::prost::alloc::vec::Vec<Evidence>,
    /// votes_sigs contains all the votes of finality providers ever registered.
    #[prost(message, repeated, tag = "4")]
    pub vote_sigs: ::prost::alloc::vec::Vec<VoteSig>,
    /// public_randomness contains all the public randomness ever committed from the finality providers.
    #[prost(message, repeated, tag = "5")]
    pub public_randomness: ::prost::alloc::vec::Vec<PublicRandomness>,
    /// pub_rand_commit contains all the public randomness commitment ever committed from the finality providers.
    #[prost(message, repeated, tag = "6")]
    pub pub_rand_commit: ::prost::alloc::vec::Vec<PubRandCommitWithPk>,
    /// signing_infos represents a map between finality provider public key and their
    /// signing infos.
    #[prost(message, repeated, tag = "7")]
    pub signing_infos: ::prost::alloc::vec::Vec<SigningInfo>,
    /// missed_blocks represents a map between finality provider public key and their
    /// missed blocks.
    #[prost(message, repeated, tag = "8")]
    pub missed_blocks: ::prost::alloc::vec::Vec<FinalityProviderMissedBlocks>,
    /// voting_powers the voting power of every finality provider at every block height.
    #[prost(message, repeated, tag = "9")]
    pub voting_powers: ::prost::alloc::vec::Vec<VotingPowerFp>,
    /// vp_dst_cache is the table of all providers voting power with the total at one specific block.
    #[prost(message, repeated, tag = "10")]
    pub vp_dst_cache: ::prost::alloc::vec::Vec<VotingPowerDistCacheBlkHeight>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// VoteSig the vote of an finality provider
/// with the block of the vote, the finality provider btc public key and the vote signature.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoteSig {
    /// block_height is the height of the voted block.
    #[prost(uint64, tag = "1")]
    pub block_height: u64,
    /// fp_btc_pk is the BTC PK of the finality provider that casts this vote
    #[prost(bytes = "vec", tag = "2")]
    pub fp_btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// finality_sig is the finality signature to this block
    /// where finality signature is an EOTS signature, i.e.
    #[prost(bytes = "vec", tag = "3")]
    pub finality_sig: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for VoteSig {
    const NAME: &'static str = "VoteSig";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// PublicRandomness the block height and public randomness that the finality provider has submitted.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PublicRandomness {
    /// block_height is the height of block which the finality provider submitted public randomness.
    #[prost(uint64, tag = "1")]
    pub block_height: u64,
    /// fp_btc_pk is the BTC PK of the finality provider that casts this vote.
    #[prost(bytes = "vec", tag = "2")]
    pub fp_btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// pub_rand is the public randomness the finality provider has committed to.
    #[prost(bytes = "vec", tag = "3")]
    pub pub_rand: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for PublicRandomness {
    const NAME: &'static str = "PublicRandomness";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// PubRandCommitWithPK is the public randomness commitment with the finality provider's BTC public key
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PubRandCommitWithPk {
    /// fp_btc_pk is the BTC PK of the finality provider that commits the public randomness
    #[prost(bytes = "vec", tag = "1")]
    pub fp_btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// pub_rand_commit is the public randomness commitment
    #[prost(message, optional, tag = "2")]
    pub pub_rand_commit: ::core::option::Option<PubRandCommit>,
}
impl ::prost::Name for PubRandCommitWithPk {
    const NAME: &'static str = "PubRandCommitWithPK";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// SigningInfo stores finality provider signing info of corresponding BTC public key.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SigningInfo {
    /// fp_btc_pk is the BTC PK of the finality provider
    #[prost(bytes = "vec", tag = "1")]
    pub fp_btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// fp_signing_info represents the signing info of this finality provider.
    #[prost(message, optional, tag = "2")]
    pub fp_signing_info: ::core::option::Option<FinalityProviderSigningInfo>,
}
impl ::prost::Name for SigningInfo {
    const NAME: &'static str = "SigningInfo";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// FinalityProviderMissedBlocks contains array of missed blocks of corresponding
/// BTC public key.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FinalityProviderMissedBlocks {
    /// fp_btc_pk is the BTC PK of the finality provider
    #[prost(bytes = "vec", tag = "1")]
    pub fp_btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// missed_blocks is an array of missed blocks by the finality provider.
    #[prost(message, repeated, tag = "2")]
    pub missed_blocks: ::prost::alloc::vec::Vec<MissedBlock>,
}
impl ::prost::Name for FinalityProviderMissedBlocks {
    const NAME: &'static str = "FinalityProviderMissedBlocks";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// MissedBlock contains height and missed status as boolean.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MissedBlock {
    /// index is the height at which the block was missed.
    #[prost(int64, tag = "1")]
    pub index: i64,
    /// missed is the missed status.
    #[prost(bool, tag = "2")]
    pub missed: bool,
}
impl ::prost::Name for MissedBlock {
    const NAME: &'static str = "MissedBlock";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// VotingPowerFP contains the information about the voting power
/// of an finality provider in a specific block height.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VotingPowerFp {
    /// block_height is the height of the block the voting power was stored.
    #[prost(uint64, tag = "1")]
    pub block_height: u64,
    /// fp_btc_pk the finality provider btc public key.
    #[prost(bytes = "vec", tag = "2")]
    pub fp_btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// voting_power is the power of the finality provider at this specific block height.
    #[prost(uint64, tag = "3")]
    pub voting_power: u64,
}
impl ::prost::Name for VotingPowerFp {
    const NAME: &'static str = "VotingPowerFP";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// VotingPowerDistCacheBlkHeight the total voting power of the finality providers at one specific block height
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VotingPowerDistCacheBlkHeight {
    /// block_height is the height of the block the voting power distribution cached was stored.
    #[prost(uint64, tag = "1")]
    pub block_height: u64,
    /// vp_distribution the finality providers distribution cache at that height.
    #[prost(message, optional, tag = "2")]
    pub vp_distribution: ::core::option::Option<VotingPowerDistCache>,
}
impl ::prost::Name for VotingPowerDistCacheBlkHeight {
    const NAME: &'static str = "VotingPowerDistCacheBlkHeight";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryParamsRequest is request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
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
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryFinalityProviderPowerAtHeightRequest is the request type for the
/// Query/FinalityProviderPowerAtHeight RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryFinalityProviderPowerAtHeightRequest {
    /// fp_btc_pk_hex is the hex str of Bitcoin secp256k1 PK of the finality provider that
    /// this BTC delegation delegates to
    /// the PK follows encoding in BIP-340 spec
    #[prost(string, tag = "1")]
    pub fp_btc_pk_hex: ::prost::alloc::string::String,
    /// height is used for querying the given finality provider's voting power at this height
    #[prost(uint64, tag = "2")]
    pub height: u64,
}
impl ::prost::Name for QueryFinalityProviderPowerAtHeightRequest {
    const NAME: &'static str = "QueryFinalityProviderPowerAtHeightRequest";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryFinalityProviderPowerAtHeightResponse is the response type for the
/// Query/FinalityProviderPowerAtHeight RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryFinalityProviderPowerAtHeightResponse {
    /// voting_power is the voting power of the finality provider
    #[prost(uint64, tag = "1")]
    pub voting_power: u64,
}
impl ::prost::Name for QueryFinalityProviderPowerAtHeightResponse {
    const NAME: &'static str = "QueryFinalityProviderPowerAtHeightResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryFinalityProviderCurrentPowerRequest is the request type for the
/// Query/FinalityProviderCurrentPower RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryFinalityProviderCurrentPowerRequest {
    /// fp_btc_pk_hex is the hex str of Bitcoin secp256k1 PK of the finality provider that
    /// this BTC delegation delegates to
    /// the PK follows encoding in BIP-340 spec
    #[prost(string, tag = "1")]
    pub fp_btc_pk_hex: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryFinalityProviderCurrentPowerRequest {
    const NAME: &'static str = "QueryFinalityProviderCurrentPowerRequest";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryFinalityProviderCurrentPowerResponse is the response type for the
/// Query/FinalityProviderCurrentPower RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryFinalityProviderCurrentPowerResponse {
    /// height is the current height
    #[prost(uint64, tag = "1")]
    pub height: u64,
    /// voting_power is the voting power of the finality provider
    #[prost(uint64, tag = "2")]
    pub voting_power: u64,
}
impl ::prost::Name for QueryFinalityProviderCurrentPowerResponse {
    const NAME: &'static str = "QueryFinalityProviderCurrentPowerResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryActiveFinalityProvidersAtHeightRequest is the request type for the
/// Query/ActiveFinalityProvidersAtHeight RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryActiveFinalityProvidersAtHeightRequest {
    /// height defines at which Babylon height to query the finality providers info.
    #[prost(uint64, tag = "1")]
    pub height: u64,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryActiveFinalityProvidersAtHeightRequest {
    const NAME: &'static str = "QueryActiveFinalityProvidersAtHeightRequest";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// ActiveFinalityProvidersAtHeightResponse wraps the FinalityProvider with metadata.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActiveFinalityProvidersAtHeightResponse {
    /// btc_pk is the Bitcoin secp256k1 PK of thisfinality provider
    /// the PK follows encoding in BIP-340 spec
    #[prost(string, tag = "1")]
    pub btc_pk_hex: ::prost::alloc::string::String,
    /// height is the queried Babylon height
    #[prost(uint64, tag = "2")]
    pub height: u64,
    /// voting_power is the voting power of this finality provider at the given height
    #[prost(uint64, tag = "3")]
    pub voting_power: u64,
    /// slashed_babylon_height indicates the Babylon height when
    /// the finality provider is slashed.
    /// if it's 0 then the finality provider is not slashed
    #[prost(uint64, tag = "4")]
    pub slashed_babylon_height: u64,
    /// slashed_btc_height indicates the BTC height when
    /// the finality provider is slashed.
    /// if it's 0 then the finality provider is not slashed
    #[prost(uint32, tag = "5")]
    pub slashed_btc_height: u32,
    /// jailed defines whether the finality provider is detected jailed
    #[prost(bool, tag = "6")]
    pub jailed: bool,
    /// highest_voted_height is the highest height for which the
    /// finality provider has voted
    #[prost(uint32, tag = "7")]
    pub highest_voted_height: u32,
}
impl ::prost::Name for ActiveFinalityProvidersAtHeightResponse {
    const NAME: &'static str = "ActiveFinalityProvidersAtHeightResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryActiveFinalityProvidersAtHeightResponse is the response type for the
/// Query/ActiveFinalityProvidersAtHeight RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryActiveFinalityProvidersAtHeightResponse {
    /// finality_providers contains all the queried finality providersn.
    #[prost(message, repeated, tag = "1")]
    pub finality_providers: ::prost::alloc::vec::Vec<ActiveFinalityProvidersAtHeightResponse>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryActiveFinalityProvidersAtHeightResponse {
    const NAME: &'static str = "QueryActiveFinalityProvidersAtHeightResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryActivatedHeightRequest is the request type for the Query/ActivatedHeight RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryActivatedHeightRequest {}
impl ::prost::Name for QueryActivatedHeightRequest {
    const NAME: &'static str = "QueryActivatedHeightRequest";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryActivatedHeightResponse is the response type for the Query/ActivatedHeight RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryActivatedHeightResponse {
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
impl ::prost::Name for QueryActivatedHeightResponse {
    const NAME: &'static str = "QueryActivatedHeightResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryListPublicRandomnessRequest is the request type for the
/// Query/ListPublicRandomness RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryListPublicRandomnessRequest {
    /// fp_btc_pk_hex is the hex str of Bitcoin secp256k1 PK of the finality provider
    #[prost(string, tag = "1")]
    pub fp_btc_pk_hex: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryListPublicRandomnessRequest {
    const NAME: &'static str = "QueryListPublicRandomnessRequest";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryListPublicRandomnessResponse is the response type for the
/// Query/ListPublicRandomness RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryListPublicRandomnessResponse {
    /// pub_rand_map is the map where the key is the height and the value
    /// is the public randomness at this height for the given finality provider
    #[prost(map = "uint64, bytes", tag = "1")]
    pub pub_rand_map: ::std::collections::HashMap<u64, ::prost::alloc::vec::Vec<u8>>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryListPublicRandomnessResponse {
    const NAME: &'static str = "QueryListPublicRandomnessResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// PubRandCommitResponse is the response type for a public randomness commitment
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PubRandCommitResponse {
    /// num_pub_rand is the number of committed public randomness
    #[prost(uint64, tag = "1")]
    pub num_pub_rand: u64,
    /// commitment is the value of the commitment
    #[prost(bytes = "vec", tag = "2")]
    pub commitment: ::prost::alloc::vec::Vec<u8>,
    /// epoch_num defines the epoch number that the commit falls into
    #[prost(uint64, tag = "3")]
    pub epoch_num: u64,
}
impl ::prost::Name for PubRandCommitResponse {
    const NAME: &'static str = "PubRandCommitResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryListPubRandCommitRequest is the request type for the
/// Query/ListPubRandCommit RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryListPubRandCommitRequest {
    /// fp_btc_pk_hex is the hex str of Bitcoin secp256k1 PK of the finality provider
    #[prost(string, tag = "1")]
    pub fp_btc_pk_hex: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryListPubRandCommitRequest {
    const NAME: &'static str = "QueryListPubRandCommitRequest";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryListPubRandCommitResponse is the response type for the
/// Query/ListPubRandCommit RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryListPubRandCommitResponse {
    /// pub_rand_commit_map is the map where the key is the start height and the value
    /// is the public randomness commitment at this height for the given finality provider
    #[prost(map = "uint64, message", tag = "1")]
    pub pub_rand_commit_map: ::std::collections::HashMap<u64, PubRandCommitResponse>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryListPubRandCommitResponse {
    const NAME: &'static str = "QueryListPubRandCommitResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryBlockRequest is the request type for the
/// Query/Block RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBlockRequest {
    /// height is the height of the Babylon block
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
impl ::prost::Name for QueryBlockRequest {
    const NAME: &'static str = "QueryBlockRequest";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryBlockResponse is the response type for the
/// Query/Block RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBlockResponse {
    /// block is the Babylon at the given height
    #[prost(message, optional, tag = "1")]
    pub block: ::core::option::Option<IndexedBlock>,
}
impl ::prost::Name for QueryBlockResponse {
    const NAME: &'static str = "QueryBlockResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryListBlocksRequest is the request type for the
/// Query/ListBlocks RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryListBlocksRequest {
    /// status indicates the status of blocks that the querier wants to query
    #[prost(enumeration = "QueriedBlockStatus", tag = "1")]
    pub status: i32,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryListBlocksRequest {
    const NAME: &'static str = "QueryListBlocksRequest";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryListBlocksResponse is the response type for the
/// Query/ListBlocks RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryListBlocksResponse {
    /// blocks is the list of blocks at the given status
    #[prost(message, repeated, tag = "1")]
    pub blocks: ::prost::alloc::vec::Vec<IndexedBlock>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryListBlocksResponse {
    const NAME: &'static str = "QueryListBlocksResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryVotesAtHeightRequest is the request type for the
/// Query/VotesAtHeight RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryVotesAtHeightRequest {
    /// height defines at which height to query the finality providers.
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
impl ::prost::Name for QueryVotesAtHeightRequest {
    const NAME: &'static str = "QueryVotesAtHeightRequest";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryVotesAtHeightResponse is the response type for the
/// Query/VotesAtHeight RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryVotesAtHeightResponse {
    /// btc_pk is the Bitcoin secp256k1 PK of finality providers who have signed the block at given height.
    /// the PK follows encoding in BIP-340 spec
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub btc_pks: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for QueryVotesAtHeightResponse {
    const NAME: &'static str = "QueryVotesAtHeightResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryEvidenceRequest is the request type for the
/// Query/Evidence RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEvidenceRequest {
    /// fp_btc_pk_hex is the hex str of Bitcoin secp256k1 PK
    /// (in BIP340 format) of the finality provider
    #[prost(string, tag = "1")]
    pub fp_btc_pk_hex: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryEvidenceRequest {
    const NAME: &'static str = "QueryEvidenceRequest";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// Evidence is the evidence that a finality provider has signed finality
/// signatures with correct public randomness on two conflicting Babylon headers
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EvidenceResponse {
    /// fp_btc_pk_hex is the BTC PK of the finality provider that casts this vote
    #[prost(string, tag = "1")]
    pub fp_btc_pk_hex: ::prost::alloc::string::String,
    /// block_height is the height of the conflicting blocks
    #[prost(uint64, tag = "2")]
    pub block_height: u64,
    /// pub_rand is the public randomness the finality provider has committed to
    #[prost(bytes = "vec", tag = "3")]
    pub pub_rand: ::prost::alloc::vec::Vec<u8>,
    /// canonical_app_hash is the AppHash of the canonical block
    #[prost(bytes = "vec", tag = "4")]
    pub canonical_app_hash: ::prost::alloc::vec::Vec<u8>,
    /// fork_app_hash is the AppHash of the fork block
    #[prost(bytes = "vec", tag = "5")]
    pub fork_app_hash: ::prost::alloc::vec::Vec<u8>,
    /// canonical_finality_sig is the finality signature to the canonical block
    /// where finality signature is an EOTS signature, i.e.,
    /// the `s` in a Schnorr signature `(r, s)`
    /// `r` is the public randomness that is already committed by the finality provider
    #[prost(bytes = "vec", tag = "6")]
    pub canonical_finality_sig: ::prost::alloc::vec::Vec<u8>,
    /// fork_finality_sig is the finality signature to the fork block
    /// where finality signature is an EOTS signature
    #[prost(bytes = "vec", tag = "7")]
    pub fork_finality_sig: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for EvidenceResponse {
    const NAME: &'static str = "EvidenceResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryEvidenceResponse is the response type for the
/// Query/Evidence RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEvidenceResponse {
    #[prost(message, optional, tag = "1")]
    pub evidence: ::core::option::Option<EvidenceResponse>,
}
impl ::prost::Name for QueryEvidenceResponse {
    const NAME: &'static str = "QueryEvidenceResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryListEvidencesRequest is the request type for the
/// Query/ListEvidences RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryListEvidencesRequest {
    /// start_height is the starting height that the querier specifies
    /// such that the RPC will only return evidences since this height
    #[prost(uint64, tag = "1")]
    pub start_height: u64,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryListEvidencesRequest {
    const NAME: &'static str = "QueryListEvidencesRequest";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueryListEvidencesResponse is the response type for the
/// Query/ListEvidences RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryListEvidencesResponse {
    /// blocks is the list of evidences
    #[prost(message, repeated, tag = "1")]
    pub evidences: ::prost::alloc::vec::Vec<EvidenceResponse>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryListEvidencesResponse {
    const NAME: &'static str = "QueryListEvidencesResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QuerySigningInfoRequest is the request type for the Query/SigningInfo RPC
/// method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySigningInfoRequest {
    /// fp_btc_pk_hex is the hex str of Bitcoin secp256k1 PK
    /// (in BIP340 format) of the finality provider
    #[prost(string, tag = "1")]
    pub fp_btc_pk_hex: ::prost::alloc::string::String,
}
impl ::prost::Name for QuerySigningInfoRequest {
    const NAME: &'static str = "QuerySigningInfoRequest";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// SigningInfoResponse defines the API response containing a finality provider's signing info
/// for monitoring their liveness activity.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SigningInfoResponse {
    /// fp_btc_pk is the BTC PK of the finality provider that casts this vote
    #[prost(string, tag = "1")]
    pub fp_btc_pk_hex: ::prost::alloc::string::String,
    /// start_height is the block height at which finality provider become active
    #[prost(int64, tag = "2")]
    pub start_height: i64,
    /// missed_blocks_counter defines a counter to avoid unnecessary array reads.
    /// Note that `Sum(MissedBlocksBitArray)` always equals `MissedBlocksCounter`.
    #[prost(int64, tag = "3")]
    pub missed_blocks_counter: i64,
    /// Timestamp until which the validator is jailed due to liveness downtime.
    #[prost(message, optional, tag = "4")]
    pub jailed_until: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
}
impl ::prost::Name for SigningInfoResponse {
    const NAME: &'static str = "SigningInfoResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QuerySigningInfoResponse is the response type for the Query/SigningInfo RPC
/// method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySigningInfoResponse {
    #[prost(message, optional, tag = "1")]
    pub signing_info: ::core::option::Option<SigningInfoResponse>,
}
impl ::prost::Name for QuerySigningInfoResponse {
    const NAME: &'static str = "QuerySigningInfoResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QuerySigningInfosRequest is the request type for the Query/SigningInfos RPC
/// method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySigningInfosRequest {
    #[prost(message, optional, tag = "1")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QuerySigningInfosRequest {
    const NAME: &'static str = "QuerySigningInfosRequest";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QuerySigningInfosResponse is the response type for the Query/SigningInfos RPC
/// method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuerySigningInfosResponse {
    /// info is the signing info of all finality providers with signing info
    #[prost(message, repeated, tag = "1")]
    pub signing_infos: ::prost::alloc::vec::Vec<SigningInfoResponse>,
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QuerySigningInfosResponse {
    const NAME: &'static str = "QuerySigningInfosResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// QueriedBlockStatus is the status of blocks that the querier wants to query.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum QueriedBlockStatus {
    /// NON_FINALIZED means the block is not finalised
    NonFinalized = 0,
    /// FINALIZED means the block is finalized
    Finalized = 1,
    /// ANY means the block can be in any status
    Any = 2,
}
impl QueriedBlockStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            QueriedBlockStatus::NonFinalized => "NON_FINALIZED",
            QueriedBlockStatus::Finalized => "FINALIZED",
            QueriedBlockStatus::Any => "ANY",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "NON_FINALIZED" => Some(Self::NonFinalized),
            "FINALIZED" => Some(Self::Finalized),
            "ANY" => Some(Self::Any),
            _ => None,
        }
    }
}
/// MsgCommitPubRandList defines a message for committing a list of public randomness for EOTS
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCommitPubRandList {
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    /// fp_btc_pk is the BTC PK of the finality provider that commits the public randomness
    #[prost(bytes = "vec", tag = "2")]
    pub fp_btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// start_height is the start block height of the list of public randomness
    #[prost(uint64, tag = "3")]
    pub start_height: u64,
    /// num_pub_rand is the number of public randomness committed
    #[prost(uint64, tag = "4")]
    pub num_pub_rand: u64,
    /// commitment is the commitment of these public randomness
    /// currently it's the root of the Merkle tree that includes these public randomness
    #[prost(bytes = "vec", tag = "5")]
    pub commitment: ::prost::alloc::vec::Vec<u8>,
    /// sig is the signature on (start_height || num_pub_rand || commitment) signed by
    /// SK corresponding to fp_btc_pk. This prevents others to commit public
    /// randomness on behalf of fp_btc_pk
    /// TODO: another option is to restrict signer to correspond to fp_btc_pk. This restricts
    /// the tx submitter to be the holder of fp_btc_pk. Decide this later
    #[prost(bytes = "vec", tag = "6")]
    pub sig: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgCommitPubRandList {
    const NAME: &'static str = "MsgCommitPubRandList";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// MsgCommitPubRandListResponse is the response to the MsgCommitPubRandList message
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCommitPubRandListResponse {}
impl ::prost::Name for MsgCommitPubRandListResponse {
    const NAME: &'static str = "MsgCommitPubRandListResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// MsgAddFinalitySig defines a message for adding a finality vote
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAddFinalitySig {
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    /// fp_btc_pk is the BTC PK of the finality provider that casts this vote
    #[prost(bytes = "vec", tag = "2")]
    pub fp_btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// block_height is the height of the voted block
    #[prost(uint64, tag = "3")]
    pub block_height: u64,
    /// pub_rand is the public randomness committed at this height
    #[prost(bytes = "vec", tag = "4")]
    pub pub_rand: ::prost::alloc::vec::Vec<u8>,
    /// proof is the proof that the given public randomness is committed under the commitment
    #[prost(message, optional, tag = "5")]
    pub proof: ::core::option::Option<super::super::super::tendermint::crypto::Proof>,
    /// block_app_hash is the AppHash of the voted block
    #[prost(bytes = "vec", tag = "6")]
    pub block_app_hash: ::prost::alloc::vec::Vec<u8>,
    /// finality_sig is the finality signature to this block
    /// where finality signature is an EOTS signature, i.e.,
    /// the `s` in a Schnorr signature `(r, s)`
    /// `r` is the public randomness that is already committed by the finality provider
    #[prost(bytes = "vec", tag = "7")]
    pub finality_sig: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgAddFinalitySig {
    const NAME: &'static str = "MsgAddFinalitySig";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// MsgAddFinalitySigResponse is the response to the MsgAddFinalitySig message
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAddFinalitySigResponse {}
impl ::prost::Name for MsgAddFinalitySigResponse {
    const NAME: &'static str = "MsgAddFinalitySigResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParams defines a message for updating finality module parameters.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// authority is the address of the governance account.
    /// just FYI: cosmos.AddressString marks that this field should use type alias
    /// for AddressString instead of string, but the functionality is not yet implemented
    /// in cosmos-proto
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the finality parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParamsResponse is the response to the MsgUpdateParams message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// MsgUnjailFinalityProvider defines the Msg/UnjailFinalityProvider request type
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUnjailFinalityProvider {
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    /// fp_btc_pk is the BTC PK of the finality provider that commits the public randomness
    #[prost(bytes = "vec", tag = "2")]
    pub fp_btc_pk: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgUnjailFinalityProvider {
    const NAME: &'static str = "MsgUnjailFinalityProvider";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// MsgUnjailFinalityProviderResponse defines the Msg/UnjailFinalityProvider response type
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUnjailFinalityProviderResponse {}
impl ::prost::Name for MsgUnjailFinalityProviderResponse {
    const NAME: &'static str = "MsgUnjailFinalityProviderResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// MsgResumeFinalityProposal is a governance proposal to resume finality from halting
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgResumeFinalityProposal {
    /// authority is the address of the governance account.
    /// just FYI: cosmos.AddressString marks that this field should use type alias
    /// for AddressString instead of string, but the functionality is not yet implemented
    /// in cosmos-proto
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// fp_pks_hex is a list of finality provider public keys to jail
    /// the public key follows encoding in BIP-340 spec
    #[prost(string, repeated, tag = "2")]
    pub fp_pks_hex: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// halting_height is the height where the finality halting begins
    #[prost(uint32, tag = "3")]
    pub halting_height: u32,
}
impl ::prost::Name for MsgResumeFinalityProposal {
    const NAME: &'static str = "MsgResumeFinalityProposal";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
/// MsgResumeFinalityProposalResponse is the response to the MsgResumeFinalityProposal message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgResumeFinalityProposalResponse {}
impl ::prost::Name for MsgResumeFinalityProposalResponse {
    const NAME: &'static str = "MsgResumeFinalityProposalResponse";
    const PACKAGE: &'static str = "babylon.finality.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.finality.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
