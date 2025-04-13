// @generated
/// ClientState defines a solo machine client that tracks the current consensus
/// state and if the client is frozen.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    /// latest sequence of the client state
    #[prost(uint64, tag = "1")]
    pub sequence: u64,
    /// frozen sequence of the solo machine
    #[prost(bool, tag = "2")]
    pub is_frozen: bool,
    #[prost(message, optional, tag = "3")]
    pub consensus_state: ::core::option::Option<ConsensusState>,
    /// when set to true, will allow governance to update a solo machine client.
    /// The client will be unfrozen if it is frozen.
    #[prost(bool, tag = "4")]
    pub allow_update_after_proposal: bool,
}
impl ::prost::Name for ClientState {
    const NAME: &'static str = "ClientState";
    const PACKAGE: &'static str = "ibc.lightclients.solomachine.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.solomachine.v2.{}", Self::NAME)
    }
}
/// ConsensusState defines a solo machine consensus state. The sequence of a
/// consensus state is contained in the "height" key used in storing the
/// consensus state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    /// public key of the solo machine
    #[prost(message, optional, tag = "1")]
    pub public_key: ::core::option::Option<super::super::super::super::google::protobuf::Any>,
    /// diversifier allows the same public key to be re-used across different solo
    /// machine clients (potentially on different chains) without being considered
    /// misbehaviour.
    #[prost(string, tag = "2")]
    pub diversifier: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub timestamp: u64,
}
impl ::prost::Name for ConsensusState {
    const NAME: &'static str = "ConsensusState";
    const PACKAGE: &'static str = "ibc.lightclients.solomachine.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.solomachine.v2.{}", Self::NAME)
    }
}
/// Header defines a solo machine consensus header
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    /// sequence to update solo machine public key at
    #[prost(uint64, tag = "1")]
    pub sequence: u64,
    #[prost(uint64, tag = "2")]
    pub timestamp: u64,
    #[prost(bytes = "vec", tag = "3")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "4")]
    pub new_public_key: ::core::option::Option<super::super::super::super::google::protobuf::Any>,
    #[prost(string, tag = "5")]
    pub new_diversifier: ::prost::alloc::string::String,
}
impl ::prost::Name for Header {
    const NAME: &'static str = "Header";
    const PACKAGE: &'static str = "ibc.lightclients.solomachine.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.solomachine.v2.{}", Self::NAME)
    }
}
/// Misbehaviour defines misbehaviour for a solo machine which consists
/// of a sequence and two signatures over different messages at that sequence.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Misbehaviour {
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub sequence: u64,
    #[prost(message, optional, tag = "3")]
    pub signature_one: ::core::option::Option<SignatureAndData>,
    #[prost(message, optional, tag = "4")]
    pub signature_two: ::core::option::Option<SignatureAndData>,
}
impl ::prost::Name for Misbehaviour {
    const NAME: &'static str = "Misbehaviour";
    const PACKAGE: &'static str = "ibc.lightclients.solomachine.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.solomachine.v2.{}", Self::NAME)
    }
}
/// SignatureAndData contains a signature and the data signed over to create that
/// signature.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignatureAndData {
    #[prost(bytes = "vec", tag = "1")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
    #[prost(enumeration = "DataType", tag = "2")]
    pub data_type: i32,
    #[prost(bytes = "vec", tag = "3")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "4")]
    pub timestamp: u64,
}
impl ::prost::Name for SignatureAndData {
    const NAME: &'static str = "SignatureAndData";
    const PACKAGE: &'static str = "ibc.lightclients.solomachine.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.solomachine.v2.{}", Self::NAME)
    }
}
/// TimestampedSignatureData contains the signature data and the timestamp of the
/// signature.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimestampedSignatureData {
    #[prost(bytes = "vec", tag = "1")]
    pub signature_data: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub timestamp: u64,
}
impl ::prost::Name for TimestampedSignatureData {
    const NAME: &'static str = "TimestampedSignatureData";
    const PACKAGE: &'static str = "ibc.lightclients.solomachine.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.solomachine.v2.{}", Self::NAME)
    }
}
/// SignBytes defines the signed bytes used for signature verification.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignBytes {
    #[prost(uint64, tag = "1")]
    pub sequence: u64,
    #[prost(uint64, tag = "2")]
    pub timestamp: u64,
    #[prost(string, tag = "3")]
    pub diversifier: ::prost::alloc::string::String,
    /// type of the data used
    #[prost(enumeration = "DataType", tag = "4")]
    pub data_type: i32,
    /// marshaled data
    #[prost(bytes = "vec", tag = "5")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SignBytes {
    const NAME: &'static str = "SignBytes";
    const PACKAGE: &'static str = "ibc.lightclients.solomachine.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.solomachine.v2.{}", Self::NAME)
    }
}
/// HeaderData returns the SignBytes data for update verification.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeaderData {
    /// header public key
    #[prost(message, optional, tag = "1")]
    pub new_pub_key: ::core::option::Option<super::super::super::super::google::protobuf::Any>,
    /// header diversifier
    #[prost(string, tag = "2")]
    pub new_diversifier: ::prost::alloc::string::String,
}
impl ::prost::Name for HeaderData {
    const NAME: &'static str = "HeaderData";
    const PACKAGE: &'static str = "ibc.lightclients.solomachine.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.solomachine.v2.{}", Self::NAME)
    }
}
/// ClientStateData returns the SignBytes data for client state verification.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientStateData {
    #[prost(bytes = "vec", tag = "1")]
    pub path: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub client_state: ::core::option::Option<super::super::super::super::google::protobuf::Any>,
}
impl ::prost::Name for ClientStateData {
    const NAME: &'static str = "ClientStateData";
    const PACKAGE: &'static str = "ibc.lightclients.solomachine.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.solomachine.v2.{}", Self::NAME)
    }
}
/// ConsensusStateData returns the SignBytes data for consensus state
/// verification.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusStateData {
    #[prost(bytes = "vec", tag = "1")]
    pub path: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub consensus_state: ::core::option::Option<super::super::super::super::google::protobuf::Any>,
}
impl ::prost::Name for ConsensusStateData {
    const NAME: &'static str = "ConsensusStateData";
    const PACKAGE: &'static str = "ibc.lightclients.solomachine.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.solomachine.v2.{}", Self::NAME)
    }
}
/// ConnectionStateData returns the SignBytes data for connection state
/// verification.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectionStateData {
    #[prost(bytes = "vec", tag = "1")]
    pub path: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub connection:
        ::core::option::Option<super::super::super::core::connection::v1::ConnectionEnd>,
}
impl ::prost::Name for ConnectionStateData {
    const NAME: &'static str = "ConnectionStateData";
    const PACKAGE: &'static str = "ibc.lightclients.solomachine.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.solomachine.v2.{}", Self::NAME)
    }
}
/// ChannelStateData returns the SignBytes data for channel state
/// verification.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChannelStateData {
    #[prost(bytes = "vec", tag = "1")]
    pub path: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub channel: ::core::option::Option<super::super::super::core::channel::v1::Channel>,
}
impl ::prost::Name for ChannelStateData {
    const NAME: &'static str = "ChannelStateData";
    const PACKAGE: &'static str = "ibc.lightclients.solomachine.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.solomachine.v2.{}", Self::NAME)
    }
}
/// PacketCommitmentData returns the SignBytes data for packet commitment
/// verification.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketCommitmentData {
    #[prost(bytes = "vec", tag = "1")]
    pub path: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub commitment: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for PacketCommitmentData {
    const NAME: &'static str = "PacketCommitmentData";
    const PACKAGE: &'static str = "ibc.lightclients.solomachine.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.solomachine.v2.{}", Self::NAME)
    }
}
/// PacketAcknowledgementData returns the SignBytes data for acknowledgement
/// verification.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketAcknowledgementData {
    #[prost(bytes = "vec", tag = "1")]
    pub path: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub acknowledgement: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for PacketAcknowledgementData {
    const NAME: &'static str = "PacketAcknowledgementData";
    const PACKAGE: &'static str = "ibc.lightclients.solomachine.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.solomachine.v2.{}", Self::NAME)
    }
}
/// PacketReceiptAbsenceData returns the SignBytes data for
/// packet receipt absence verification.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketReceiptAbsenceData {
    #[prost(bytes = "vec", tag = "1")]
    pub path: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for PacketReceiptAbsenceData {
    const NAME: &'static str = "PacketReceiptAbsenceData";
    const PACKAGE: &'static str = "ibc.lightclients.solomachine.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.solomachine.v2.{}", Self::NAME)
    }
}
/// NextSequenceRecvData returns the SignBytes data for verification of the next
/// sequence to be received.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NextSequenceRecvData {
    #[prost(bytes = "vec", tag = "1")]
    pub path: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub next_seq_recv: u64,
}
impl ::prost::Name for NextSequenceRecvData {
    const NAME: &'static str = "NextSequenceRecvData";
    const PACKAGE: &'static str = "ibc.lightclients.solomachine.v2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.solomachine.v2.{}", Self::NAME)
    }
}
/// DataType defines the type of solo machine proof being created. This is done
/// to preserve uniqueness of different data sign byte encodings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DataType {
    /// Default State
    UninitializedUnspecified = 0,
    /// Data type for client state verification
    ClientState = 1,
    /// Data type for consensus state verification
    ConsensusState = 2,
    /// Data type for connection state verification
    ConnectionState = 3,
    /// Data type for channel state verification
    ChannelState = 4,
    /// Data type for packet commitment verification
    PacketCommitment = 5,
    /// Data type for packet acknowledgement verification
    PacketAcknowledgement = 6,
    /// Data type for packet receipt absence verification
    PacketReceiptAbsence = 7,
    /// Data type for next sequence recv verification
    NextSequenceRecv = 8,
    /// Data type for header verification
    Header = 9,
}
impl DataType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            DataType::UninitializedUnspecified => "DATA_TYPE_UNINITIALIZED_UNSPECIFIED",
            DataType::ClientState => "DATA_TYPE_CLIENT_STATE",
            DataType::ConsensusState => "DATA_TYPE_CONSENSUS_STATE",
            DataType::ConnectionState => "DATA_TYPE_CONNECTION_STATE",
            DataType::ChannelState => "DATA_TYPE_CHANNEL_STATE",
            DataType::PacketCommitment => "DATA_TYPE_PACKET_COMMITMENT",
            DataType::PacketAcknowledgement => "DATA_TYPE_PACKET_ACKNOWLEDGEMENT",
            DataType::PacketReceiptAbsence => "DATA_TYPE_PACKET_RECEIPT_ABSENCE",
            DataType::NextSequenceRecv => "DATA_TYPE_NEXT_SEQUENCE_RECV",
            DataType::Header => "DATA_TYPE_HEADER",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "DATA_TYPE_UNINITIALIZED_UNSPECIFIED" => Some(Self::UninitializedUnspecified),
            "DATA_TYPE_CLIENT_STATE" => Some(Self::ClientState),
            "DATA_TYPE_CONSENSUS_STATE" => Some(Self::ConsensusState),
            "DATA_TYPE_CONNECTION_STATE" => Some(Self::ConnectionState),
            "DATA_TYPE_CHANNEL_STATE" => Some(Self::ChannelState),
            "DATA_TYPE_PACKET_COMMITMENT" => Some(Self::PacketCommitment),
            "DATA_TYPE_PACKET_ACKNOWLEDGEMENT" => Some(Self::PacketAcknowledgement),
            "DATA_TYPE_PACKET_RECEIPT_ABSENCE" => Some(Self::PacketReceiptAbsence),
            "DATA_TYPE_NEXT_SEQUENCE_RECV" => Some(Self::NextSequenceRecv),
            "DATA_TYPE_HEADER" => Some(Self::Header),
            _ => None,
        }
    }
}
// @@protoc_insertion_point(module)
