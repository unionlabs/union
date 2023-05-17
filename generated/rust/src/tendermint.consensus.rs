// @generated
/// NewRoundStep is sent for every step taken in the ConsensusState.
/// For every height/round/step transition
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewRoundStep {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(int32, tag = "2")]
    pub round: i32,
    #[prost(uint32, tag = "3")]
    pub step: u32,
    #[prost(int64, tag = "4")]
    pub seconds_since_start_time: i64,
    #[prost(int32, tag = "5")]
    pub last_commit_round: i32,
}
/// NewValidBlock is sent when a validator observes a valid block B in some round r,
/// i.e., there is a Proposal for block B and 2/3+ prevotes for the block B in the round r.
/// In case the block is also committed, then IsCommit flag is set to true.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewValidBlock {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(int32, tag = "2")]
    pub round: i32,
    #[prost(message, optional, tag = "3")]
    pub block_part_set_header: ::core::option::Option<super::types::PartSetHeader>,
    #[prost(message, optional, tag = "4")]
    pub block_parts: ::core::option::Option<super::libs::bits::BitArray>,
    #[prost(bool, tag = "5")]
    pub is_commit: bool,
}
/// Proposal is sent when a new block is proposed.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Proposal {
    #[prost(message, optional, tag = "1")]
    pub proposal: ::core::option::Option<super::types::Proposal>,
}
/// ProposalPOL is sent when a previous proposal is re-proposed.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalPol {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(int32, tag = "2")]
    pub proposal_pol_round: i32,
    #[prost(message, optional, tag = "3")]
    pub proposal_pol: ::core::option::Option<super::libs::bits::BitArray>,
}
/// BlockPart is sent when gossipping a piece of the proposed block.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockPart {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(int32, tag = "2")]
    pub round: i32,
    #[prost(message, optional, tag = "3")]
    pub part: ::core::option::Option<super::types::Part>,
}
/// Vote is sent when voting for a proposal (or lack thereof).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Vote {
    #[prost(message, optional, tag = "1")]
    pub vote: ::core::option::Option<super::types::Vote>,
}
/// HasVote is sent to indicate that a particular vote has been received.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HasVote {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(int32, tag = "2")]
    pub round: i32,
    #[prost(enumeration = "super::types::SignedMsgType", tag = "3")]
    pub r#type: i32,
    #[prost(int32, tag = "4")]
    pub index: i32,
}
/// VoteSetMaj23 is sent to indicate that a given BlockID has seen +2/3 votes.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoteSetMaj23 {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(int32, tag = "2")]
    pub round: i32,
    #[prost(enumeration = "super::types::SignedMsgType", tag = "3")]
    pub r#type: i32,
    #[prost(message, optional, tag = "4")]
    pub block_id: ::core::option::Option<super::types::BlockId>,
}
/// VoteSetBits is sent to communicate the bit-array of votes seen for the BlockID.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoteSetBits {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(int32, tag = "2")]
    pub round: i32,
    #[prost(enumeration = "super::types::SignedMsgType", tag = "3")]
    pub r#type: i32,
    #[prost(message, optional, tag = "4")]
    pub block_id: ::core::option::Option<super::types::BlockId>,
    #[prost(message, optional, tag = "5")]
    pub votes: ::core::option::Option<super::libs::bits::BitArray>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    #[prost(oneof = "message::Sum", tags = "1, 2, 3, 4, 5, 6, 7, 8, 9")]
    pub sum: ::core::option::Option<message::Sum>,
}
/// Nested message and enum types in `Message`.
pub mod message {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag = "1")]
        NewRoundStep(super::NewRoundStep),
        #[prost(message, tag = "2")]
        NewValidBlock(super::NewValidBlock),
        #[prost(message, tag = "3")]
        Proposal(super::Proposal),
        #[prost(message, tag = "4")]
        ProposalPol(super::ProposalPol),
        #[prost(message, tag = "5")]
        BlockPart(super::BlockPart),
        #[prost(message, tag = "6")]
        Vote(super::Vote),
        #[prost(message, tag = "7")]
        HasVote(super::HasVote),
        #[prost(message, tag = "8")]
        VoteSetMaj23(super::VoteSetMaj23),
        #[prost(message, tag = "9")]
        VoteSetBits(super::VoteSetBits),
    }
}
/// MsgInfo are msgs from the reactor which may update the state
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgInfo {
    #[prost(message, optional, tag = "1")]
    pub msg: ::core::option::Option<Message>,
    #[prost(string, tag = "2")]
    pub peer_id: ::prost::alloc::string::String,
}
/// TimeoutInfo internally generated messages which may update the state
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimeoutInfo {
    #[prost(message, optional, tag = "1")]
    pub duration: ::core::option::Option<super::super::google::protobuf::Duration>,
    #[prost(int64, tag = "2")]
    pub height: i64,
    #[prost(int32, tag = "3")]
    pub round: i32,
    #[prost(uint32, tag = "4")]
    pub step: u32,
}
/// EndHeight marks the end of the given height inside WAL.
/// @internal used by scripts/wal2json util.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EndHeight {
    #[prost(int64, tag = "1")]
    pub height: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WalMessage {
    #[prost(oneof = "wal_message::Sum", tags = "1, 2, 3, 4")]
    pub sum: ::core::option::Option<wal_message::Sum>,
}
/// Nested message and enum types in `WALMessage`.
pub mod wal_message {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag = "1")]
        EventDataRoundState(super::super::types::EventDataRoundState),
        #[prost(message, tag = "2")]
        MsgInfo(super::MsgInfo),
        #[prost(message, tag = "3")]
        TimeoutInfo(super::TimeoutInfo),
        #[prost(message, tag = "4")]
        EndHeight(super::EndHeight),
    }
}
/// TimedWALMessage wraps WALMessage and adds Time for debugging purposes.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimedWalMessage {
    #[prost(message, optional, tag = "1")]
    pub time: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    #[prost(message, optional, tag = "2")]
    pub msg: ::core::option::Option<WalMessage>,
}
// @@protoc_insertion_point(module)
