/// Tx is the standard type used for broadcasting transactions.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Tx {
    /// body is the processable content of the transaction
    #[prost(message, optional, tag = "1")]
    pub body: ::core::option::Option<TxBody>,
    /// auth_info is the authorization related content of the transaction,
    /// specifically signers, signer modes and fee
    #[prost(message, optional, tag = "2")]
    pub auth_info: ::core::option::Option<AuthInfo>,
    /// signatures is a list of signatures that matches the length and order of
    /// AuthInfo's signer_infos to allow connecting signature meta information like
    /// public key and signing mode by position.
    #[prost(bytes = "vec", repeated, tag = "3")]
    pub signatures: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
/// TxRaw is a variant of Tx that pins the signer's exact binary representation
/// of body and auth_info. This is used for signing, broadcasting and
/// verification. The binary `serialize(tx: TxRaw)` is stored in Tendermint and
/// the hash `sha256(serialize(tx: TxRaw))` becomes the "txhash", commonly used
/// as the transaction ID.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxRaw {
    /// body_bytes is a protobuf serialization of a TxBody that matches the
    /// representation in SignDoc.
    #[prost(bytes = "vec", tag = "1")]
    pub body_bytes: ::prost::alloc::vec::Vec<u8>,
    /// auth_info_bytes is a protobuf serialization of an AuthInfo that matches the
    /// representation in SignDoc.
    #[prost(bytes = "vec", tag = "2")]
    pub auth_info_bytes: ::prost::alloc::vec::Vec<u8>,
    /// signatures is a list of signatures that matches the length and order of
    /// AuthInfo's signer_infos to allow connecting signature meta information like
    /// public key and signing mode by position.
    #[prost(bytes = "vec", repeated, tag = "3")]
    pub signatures: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
/// SignDoc is the type used for generating sign bytes for SIGN_MODE_DIRECT.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignDoc {
    /// body_bytes is protobuf serialization of a TxBody that matches the
    /// representation in TxRaw.
    #[prost(bytes = "vec", tag = "1")]
    pub body_bytes: ::prost::alloc::vec::Vec<u8>,
    /// auth_info_bytes is a protobuf serialization of an AuthInfo that matches the
    /// representation in TxRaw.
    #[prost(bytes = "vec", tag = "2")]
    pub auth_info_bytes: ::prost::alloc::vec::Vec<u8>,
    /// chain_id is the unique identifier of the chain this transaction targets.
    /// It prevents signed transactions from being used on another chain by an
    /// attacker
    #[prost(string, tag = "3")]
    pub chain_id: ::prost::alloc::string::String,
    /// account_number is the account number of the account in state
    #[prost(uint64, tag = "4")]
    pub account_number: u64,
}
/// SignDocDirectAux is the type used for generating sign bytes for
/// SIGN_MODE_DIRECT_AUX.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignDocDirectAux {
    /// body_bytes is protobuf serialization of a TxBody that matches the
    /// representation in TxRaw.
    #[prost(bytes = "vec", tag = "1")]
    pub body_bytes: ::prost::alloc::vec::Vec<u8>,
    /// public_key is the public key of the signing account.
    #[prost(message, optional, tag = "2")]
    pub public_key: ::core::option::Option<super::super::super::google::protobuf::Any>,
    /// chain_id is the identifier of the chain this transaction targets.
    /// It prevents signed transactions from being used on another chain by an
    /// attacker.
    #[prost(string, tag = "3")]
    pub chain_id: ::prost::alloc::string::String,
    /// account_number is the account number of the account in state.
    #[prost(uint64, tag = "4")]
    pub account_number: u64,
    /// sequence is the sequence number of the signing account.
    #[prost(uint64, tag = "5")]
    pub sequence: u64,
    /// Tip is the optional tip used for transactions fees paid in another denom.
    /// It should be left empty if the signer is not the tipper for this
    /// transaction.
    ///
    /// This field is ignored if the chain didn't enable tips, i.e. didn't add the
    /// `TipDecorator` in its posthandler.
    #[prost(message, optional, tag = "6")]
    pub tip: ::core::option::Option<Tip>,
}
/// TxBody is the body of a transaction that all signers sign over.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxBody {
    /// messages is a list of messages to be executed. The required signers of
    /// those messages define the number and order of elements in AuthInfo's
    /// signer_infos and Tx's signatures. Each required signer address is added to
    /// the list only the first time it occurs.
    /// By convention, the first required signer (usually from the first message)
    /// is referred to as the primary signer and pays the fee for the whole
    /// transaction.
    #[prost(message, repeated, tag = "1")]
    pub messages: ::prost::alloc::vec::Vec<super::super::super::google::protobuf::Any>,
    /// memo is any arbitrary note/comment to be added to the transaction.
    /// WARNING: in clients, any publicly exposed text should not be called memo,
    /// but should be called `note` instead (see <https://github.com/cosmos/cosmos-sdk/issues/9122>).
    #[prost(string, tag = "2")]
    pub memo: ::prost::alloc::string::String,
    /// timeout is the block height after which this transaction will not
    /// be processed by the chain
    #[prost(uint64, tag = "3")]
    pub timeout_height: u64,
    /// extension_options are arbitrary options that can be added by chains
    /// when the default options are not sufficient. If any of these are present
    /// and can't be handled, the transaction will be rejected
    #[prost(message, repeated, tag = "1023")]
    pub extension_options: ::prost::alloc::vec::Vec<
        super::super::super::google::protobuf::Any,
    >,
    /// extension_options are arbitrary options that can be added by chains
    /// when the default options are not sufficient. If any of these are present
    /// and can't be handled, they will be ignored
    #[prost(message, repeated, tag = "2047")]
    pub non_critical_extension_options: ::prost::alloc::vec::Vec<
        super::super::super::google::protobuf::Any,
    >,
}
/// AuthInfo describes the fee and signer modes that are used to sign a
/// transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthInfo {
    /// signer_infos defines the signing modes for the required signers. The number
    /// and order of elements must match the required signers from TxBody's
    /// messages. The first element is the primary signer and the one which pays
    /// the fee.
    #[prost(message, repeated, tag = "1")]
    pub signer_infos: ::prost::alloc::vec::Vec<SignerInfo>,
    /// Fee is the fee and gas limit for the transaction. The first signer is the
    /// primary signer and the one which pays the fee. The fee can be calculated
    /// based on the cost of evaluating the body and doing signature verification
    /// of the signers. This can be estimated via simulation.
    #[prost(message, optional, tag = "2")]
    pub fee: ::core::option::Option<Fee>,
    /// Tip is the optional tip used for transactions fees paid in another denom.
    ///
    /// This field is ignored if the chain didn't enable tips, i.e. didn't add the
    /// `TipDecorator` in its posthandler.
    ///
    /// Since: cosmos-sdk 0.46
    #[prost(message, optional, tag = "3")]
    pub tip: ::core::option::Option<Tip>,
}
/// SignerInfo describes the public key and signing mode of a single top-level
/// signer.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignerInfo {
    /// public_key is the public key of the signer. It is optional for accounts
    /// that already exist in state. If unset, the verifier can use the required \
    /// signer address for this position and lookup the public key.
    #[prost(message, optional, tag = "1")]
    pub public_key: ::core::option::Option<super::super::super::google::protobuf::Any>,
    /// mode_info describes the signing mode of the signer and is a nested
    /// structure to support nested multisig pubkey's
    #[prost(message, optional, tag = "2")]
    pub mode_info: ::core::option::Option<ModeInfo>,
    /// sequence is the sequence of the account, which describes the
    /// number of committed transactions signed by a given address. It is used to
    /// prevent replay attacks.
    #[prost(uint64, tag = "3")]
    pub sequence: u64,
}
/// ModeInfo describes the signing mode of a single or nested multisig signer.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModeInfo {
    /// sum is the oneof that specifies whether this represents a single or nested
    /// multisig signer
    #[prost(oneof = "mode_info::Sum", tags = "1, 2")]
    pub sum: ::core::option::Option<mode_info::Sum>,
}
/// Nested message and enum types in `ModeInfo`.
pub mod mode_info {
    /// Single is the mode info for a single signer. It is structured as a message
    /// to allow for additional fields such as locale for SIGN_MODE_TEXTUAL in the
    /// future
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Single {
        /// mode is the signing mode of the single signer
        #[prost(enumeration = "super::super::signing::v1beta1::SignMode", tag = "1")]
        pub mode: i32,
    }
    /// Multi is the mode info for a multisig public key
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Multi {
        /// bitarray specifies which keys within the multisig are signing
        #[prost(message, optional, tag = "1")]
        pub bitarray: ::core::option::Option<
            super::super::super::crypto::multisig::v1beta1::CompactBitArray,
        >,
        /// mode_infos is the corresponding modes of the signers of the multisig
        /// which could include nested multisig public keys
        #[prost(message, repeated, tag = "2")]
        pub mode_infos: ::prost::alloc::vec::Vec<super::ModeInfo>,
    }
    /// sum is the oneof that specifies whether this represents a single or nested
    /// multisig signer
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        /// single represents a single signer
        #[prost(message, tag = "1")]
        Single(Single),
        /// multi represents a nested multisig signer
        #[prost(message, tag = "2")]
        Multi(Multi),
    }
}
/// Fee includes the amount of coins paid in fees and the maximum
/// gas to be used by the transaction. The ratio yields an effective "gasprice",
/// which must be above some miminum to be accepted into the mempool.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fee {
    /// amount is the amount of coins to be paid as a fee
    #[prost(message, repeated, tag = "1")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// gas_limit is the maximum gas that can be used in transaction processing
    /// before an out of gas error occurs
    #[prost(uint64, tag = "2")]
    pub gas_limit: u64,
    /// if unset, the first signer is responsible for paying the fees. If set, the specified account must pay the fees.
    /// the payer must be a tx signer (and thus have signed this field in AuthInfo).
    /// setting this field does *not* change the ordering of required signers for the transaction.
    #[prost(string, tag = "3")]
    pub payer: ::prost::alloc::string::String,
    /// if set, the fee payer (either the first signer or the value of the payer field) requests that a fee grant be used
    /// to pay fees instead of the fee payer's own balance. If an appropriate fee grant does not exist or the chain does
    /// not support fee grants, this will fail
    #[prost(string, tag = "4")]
    pub granter: ::prost::alloc::string::String,
}
/// Tip is the tip used for meta-transactions.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Tip {
    /// amount is the amount of the tip
    #[prost(message, repeated, tag = "1")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// tipper is the address of the account paying for the tip
    #[prost(string, tag = "2")]
    pub tipper: ::prost::alloc::string::String,
}
/// AuxSignerData is the intermediary format that an auxiliary signer (e.g. a
/// tipper) builds and sends to the fee payer (who will build and broadcast the
/// actual tx). AuxSignerData is not a valid tx in itself, and will be rejected
/// by the node if sent directly as-is.
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuxSignerData {
    /// address is the bech32-encoded address of the auxiliary signer. If using
    /// AuxSignerData across different chains, the bech32 prefix of the target
    /// chain (where the final transaction is broadcasted) should be used.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// sign_doc is the SIGN_MODE_DIRECT_AUX sign doc that the auxiliary signer
    /// signs. Note: we use the same sign doc even if we're signing with
    /// LEGACY_AMINO_JSON.
    #[prost(message, optional, tag = "2")]
    pub sign_doc: ::core::option::Option<SignDocDirectAux>,
    /// mode is the signing mode of the single signer.
    #[prost(enumeration = "super::signing::v1beta1::SignMode", tag = "3")]
    pub mode: i32,
    /// sig is the signature of the sign doc.
    #[prost(bytes = "vec", tag = "4")]
    pub sig: ::prost::alloc::vec::Vec<u8>,
}
/// GetTxsEventRequest is the request type for the Service.TxsByEvents
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTxsEventRequest {
    /// events is the list of transaction event type.
    #[prost(string, repeated, tag = "1")]
    pub events: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// pagination defines a pagination for the request.
    /// Deprecated post v0.46.x: use page and limit instead.
    #[deprecated]
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::base::query::v1beta1::PageRequest,
    >,
    #[prost(enumeration = "OrderBy", tag = "3")]
    pub order_by: i32,
    /// page is the page number to query, starts at 1. If not provided, will default to first page.
    #[prost(uint64, tag = "4")]
    pub page: u64,
    /// limit is the total number of results to be returned in the result page.
    /// If left empty it will default to a value to be set by each app.
    #[prost(uint64, tag = "5")]
    pub limit: u64,
}
/// GetTxsEventResponse is the response type for the Service.TxsByEvents
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTxsEventResponse {
    /// txs is the list of queried transactions.
    #[prost(message, repeated, tag = "1")]
    pub txs: ::prost::alloc::vec::Vec<Tx>,
    /// tx_responses is the list of queried TxResponses.
    #[prost(message, repeated, tag = "2")]
    pub tx_responses: ::prost::alloc::vec::Vec<
        super::super::base::abci::v1beta1::TxResponse,
    >,
    /// pagination defines a pagination for the response.
    /// Deprecated post v0.46.x: use total instead.
    #[deprecated]
    #[prost(message, optional, tag = "3")]
    pub pagination: ::core::option::Option<
        super::super::base::query::v1beta1::PageResponse,
    >,
    /// total is total number of results available
    #[prost(uint64, tag = "4")]
    pub total: u64,
}
/// BroadcastTxRequest is the request type for the Service.BroadcastTxRequest
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BroadcastTxRequest {
    /// tx_bytes is the raw transaction.
    #[prost(bytes = "vec", tag = "1")]
    pub tx_bytes: ::prost::alloc::vec::Vec<u8>,
    #[prost(enumeration = "BroadcastMode", tag = "2")]
    pub mode: i32,
}
/// BroadcastTxResponse is the response type for the
/// Service.BroadcastTx method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BroadcastTxResponse {
    /// tx_response is the queried TxResponses.
    #[prost(message, optional, tag = "1")]
    pub tx_response: ::core::option::Option<
        super::super::base::abci::v1beta1::TxResponse,
    >,
}
/// SimulateRequest is the request type for the Service.Simulate
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SimulateRequest {
    /// tx is the transaction to simulate.
    /// Deprecated. Send raw tx bytes instead.
    #[deprecated]
    #[prost(message, optional, tag = "1")]
    pub tx: ::core::option::Option<Tx>,
    /// tx_bytes is the raw transaction.
    ///
    /// Since: cosmos-sdk 0.43
    #[prost(bytes = "vec", tag = "2")]
    pub tx_bytes: ::prost::alloc::vec::Vec<u8>,
}
/// SimulateResponse is the response type for the
/// Service.SimulateRPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SimulateResponse {
    /// gas_info is the information about gas used in the simulation.
    #[prost(message, optional, tag = "1")]
    pub gas_info: ::core::option::Option<super::super::base::abci::v1beta1::GasInfo>,
    /// result is the result of the simulation.
    #[prost(message, optional, tag = "2")]
    pub result: ::core::option::Option<super::super::base::abci::v1beta1::Result>,
}
/// GetTxRequest is the request type for the Service.GetTx
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTxRequest {
    /// hash is the tx hash to query, encoded as a hex string.
    #[prost(string, tag = "1")]
    pub hash: ::prost::alloc::string::String,
}
/// GetTxResponse is the response type for the Service.GetTx method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTxResponse {
    /// tx is the queried transaction.
    #[prost(message, optional, tag = "1")]
    pub tx: ::core::option::Option<Tx>,
    /// tx_response is the queried TxResponses.
    #[prost(message, optional, tag = "2")]
    pub tx_response: ::core::option::Option<
        super::super::base::abci::v1beta1::TxResponse,
    >,
}
/// GetBlockWithTxsRequest is the request type for the Service.GetBlockWithTxs
/// RPC method.
///
/// Since: cosmos-sdk 0.45.2
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockWithTxsRequest {
    /// height is the height of the block to query.
    #[prost(int64, tag = "1")]
    pub height: i64,
    /// pagination defines a pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::base::query::v1beta1::PageRequest,
    >,
}
/// GetBlockWithTxsResponse is the response type for the Service.GetBlockWithTxs method.
///
/// Since: cosmos-sdk 0.45.2
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockWithTxsResponse {
    /// txs are the transactions in the block.
    #[prost(message, repeated, tag = "1")]
    pub txs: ::prost::alloc::vec::Vec<Tx>,
    #[prost(message, optional, tag = "2")]
    pub block_id: ::core::option::Option<::tendermint_proto::types::BlockId>,
    #[prost(message, optional, tag = "3")]
    pub block: ::core::option::Option<::tendermint_proto::types::Block>,
    /// pagination defines a pagination for the response.
    #[prost(message, optional, tag = "4")]
    pub pagination: ::core::option::Option<
        super::super::base::query::v1beta1::PageResponse,
    >,
}
/// TxDecodeRequest is the request type for the Service.TxDecode
/// RPC method.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxDecodeRequest {
    /// tx_bytes is the raw transaction.
    #[prost(bytes = "vec", tag = "1")]
    pub tx_bytes: ::prost::alloc::vec::Vec<u8>,
}
/// TxDecodeResponse is the response type for the
/// Service.TxDecode method.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxDecodeResponse {
    /// tx is the decoded transaction.
    #[prost(message, optional, tag = "1")]
    pub tx: ::core::option::Option<Tx>,
}
/// TxEncodeRequest is the request type for the Service.TxEncode
/// RPC method.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxEncodeRequest {
    /// tx is the transaction to encode.
    #[prost(message, optional, tag = "1")]
    pub tx: ::core::option::Option<Tx>,
}
/// TxEncodeResponse is the response type for the
/// Service.TxEncode method.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxEncodeResponse {
    /// tx_bytes is the encoded transaction bytes.
    #[prost(bytes = "vec", tag = "1")]
    pub tx_bytes: ::prost::alloc::vec::Vec<u8>,
}
/// TxEncodeAminoRequest is the request type for the Service.TxEncodeAmino
/// RPC method.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxEncodeAminoRequest {
    #[prost(string, tag = "1")]
    pub amino_json: ::prost::alloc::string::String,
}
/// TxEncodeAminoResponse is the response type for the Service.TxEncodeAmino
/// RPC method.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxEncodeAminoResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub amino_binary: ::prost::alloc::vec::Vec<u8>,
}
/// TxDecodeAminoRequest is the request type for the Service.TxDecodeAmino
/// RPC method.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxDecodeAminoRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub amino_binary: ::prost::alloc::vec::Vec<u8>,
}
/// TxDecodeAminoResponse is the response type for the Service.TxDecodeAmino
/// RPC method.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxDecodeAminoResponse {
    #[prost(string, tag = "1")]
    pub amino_json: ::prost::alloc::string::String,
}
/// OrderBy defines the sorting order
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OrderBy {
    /// ORDER_BY_UNSPECIFIED specifies an unknown sorting order. OrderBy defaults to ASC in this case.
    Unspecified = 0,
    /// ORDER_BY_ASC defines ascending order
    Asc = 1,
    /// ORDER_BY_DESC defines descending order
    Desc = 2,
}
impl OrderBy {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            OrderBy::Unspecified => "ORDER_BY_UNSPECIFIED",
            OrderBy::Asc => "ORDER_BY_ASC",
            OrderBy::Desc => "ORDER_BY_DESC",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ORDER_BY_UNSPECIFIED" => Some(Self::Unspecified),
            "ORDER_BY_ASC" => Some(Self::Asc),
            "ORDER_BY_DESC" => Some(Self::Desc),
            _ => None,
        }
    }
}
/// BroadcastMode specifies the broadcast mode for the TxService.Broadcast RPC method.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum BroadcastMode {
    /// zero-value for mode ordering
    Unspecified = 0,
    /// DEPRECATED: use BROADCAST_MODE_SYNC instead,
    /// BROADCAST_MODE_BLOCK is not supported by the SDK from v0.47.x onwards.
    Block = 1,
    /// BROADCAST_MODE_SYNC defines a tx broadcasting mode where the client waits for
    /// a CheckTx execution response only.
    Sync = 2,
    /// BROADCAST_MODE_ASYNC defines a tx broadcasting mode where the client returns
    /// immediately.
    Async = 3,
}
impl BroadcastMode {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            BroadcastMode::Unspecified => "BROADCAST_MODE_UNSPECIFIED",
            BroadcastMode::Block => "BROADCAST_MODE_BLOCK",
            BroadcastMode::Sync => "BROADCAST_MODE_SYNC",
            BroadcastMode::Async => "BROADCAST_MODE_ASYNC",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "BROADCAST_MODE_UNSPECIFIED" => Some(Self::Unspecified),
            "BROADCAST_MODE_BLOCK" => Some(Self::Block),
            "BROADCAST_MODE_SYNC" => Some(Self::Sync),
            "BROADCAST_MODE_ASYNC" => Some(Self::Async),
            _ => None,
        }
    }
}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Service defines a gRPC service for interacting with transactions.
    #[derive(Debug, Clone)]
    pub struct ServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> ServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            ServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Simulate simulates executing a transaction for estimating gas usage.
        pub async fn simulate(
            &mut self,
            request: impl tonic::IntoRequest<super::SimulateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SimulateResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.tx.v1beta1.Service/Simulate",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.tx.v1beta1.Service", "Simulate"));
            self.inner.unary(req, path, codec).await
        }
        /// GetTx fetches a tx by hash.
        pub async fn get_tx(
            &mut self,
            request: impl tonic::IntoRequest<super::GetTxRequest>,
        ) -> std::result::Result<tonic::Response<super::GetTxResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.tx.v1beta1.Service/GetTx",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.tx.v1beta1.Service", "GetTx"));
            self.inner.unary(req, path, codec).await
        }
        /// BroadcastTx broadcast transaction.
        pub async fn broadcast_tx(
            &mut self,
            request: impl tonic::IntoRequest<super::BroadcastTxRequest>,
        ) -> std::result::Result<
            tonic::Response<super::BroadcastTxResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.tx.v1beta1.Service/BroadcastTx",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.tx.v1beta1.Service", "BroadcastTx"));
            self.inner.unary(req, path, codec).await
        }
        /// GetTxsEvent fetches txs by event.
        pub async fn get_txs_event(
            &mut self,
            request: impl tonic::IntoRequest<super::GetTxsEventRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetTxsEventResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.tx.v1beta1.Service/GetTxsEvent",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.tx.v1beta1.Service", "GetTxsEvent"));
            self.inner.unary(req, path, codec).await
        }
        /// GetBlockWithTxs fetches a block with decoded txs.
        ///
        /// Since: cosmos-sdk 0.45.2
        pub async fn get_block_with_txs(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBlockWithTxsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetBlockWithTxsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.tx.v1beta1.Service/GetBlockWithTxs",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.tx.v1beta1.Service", "GetBlockWithTxs"));
            self.inner.unary(req, path, codec).await
        }
        /// TxDecode decodes the transaction.
        ///
        /// Since: cosmos-sdk 0.47
        pub async fn tx_decode(
            &mut self,
            request: impl tonic::IntoRequest<super::TxDecodeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TxDecodeResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.tx.v1beta1.Service/TxDecode",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.tx.v1beta1.Service", "TxDecode"));
            self.inner.unary(req, path, codec).await
        }
        /// TxEncode encodes the transaction.
        ///
        /// Since: cosmos-sdk 0.47
        pub async fn tx_encode(
            &mut self,
            request: impl tonic::IntoRequest<super::TxEncodeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TxEncodeResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.tx.v1beta1.Service/TxEncode",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.tx.v1beta1.Service", "TxEncode"));
            self.inner.unary(req, path, codec).await
        }
        /// TxEncodeAmino encodes an Amino transaction from JSON to encoded bytes.
        ///
        /// Since: cosmos-sdk 0.47
        pub async fn tx_encode_amino(
            &mut self,
            request: impl tonic::IntoRequest<super::TxEncodeAminoRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TxEncodeAminoResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.tx.v1beta1.Service/TxEncodeAmino",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.tx.v1beta1.Service", "TxEncodeAmino"));
            self.inner.unary(req, path, codec).await
        }
        /// TxDecodeAmino decodes an Amino transaction from encoded bytes to JSON.
        ///
        /// Since: cosmos-sdk 0.47
        pub async fn tx_decode_amino(
            &mut self,
            request: impl tonic::IntoRequest<super::TxDecodeAminoRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TxDecodeAminoResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.tx.v1beta1.Service/TxDecodeAmino",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.tx.v1beta1.Service", "TxDecodeAmino"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ServiceServer.
    #[async_trait]
    pub trait Service: Send + Sync + 'static {
        /// Simulate simulates executing a transaction for estimating gas usage.
        async fn simulate(
            &self,
            request: tonic::Request<super::SimulateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SimulateResponse>,
            tonic::Status,
        >;
        /// GetTx fetches a tx by hash.
        async fn get_tx(
            &self,
            request: tonic::Request<super::GetTxRequest>,
        ) -> std::result::Result<tonic::Response<super::GetTxResponse>, tonic::Status>;
        /// BroadcastTx broadcast transaction.
        async fn broadcast_tx(
            &self,
            request: tonic::Request<super::BroadcastTxRequest>,
        ) -> std::result::Result<
            tonic::Response<super::BroadcastTxResponse>,
            tonic::Status,
        >;
        /// GetTxsEvent fetches txs by event.
        async fn get_txs_event(
            &self,
            request: tonic::Request<super::GetTxsEventRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetTxsEventResponse>,
            tonic::Status,
        >;
        /// GetBlockWithTxs fetches a block with decoded txs.
        ///
        /// Since: cosmos-sdk 0.45.2
        async fn get_block_with_txs(
            &self,
            request: tonic::Request<super::GetBlockWithTxsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetBlockWithTxsResponse>,
            tonic::Status,
        >;
        /// TxDecode decodes the transaction.
        ///
        /// Since: cosmos-sdk 0.47
        async fn tx_decode(
            &self,
            request: tonic::Request<super::TxDecodeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TxDecodeResponse>,
            tonic::Status,
        >;
        /// TxEncode encodes the transaction.
        ///
        /// Since: cosmos-sdk 0.47
        async fn tx_encode(
            &self,
            request: tonic::Request<super::TxEncodeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TxEncodeResponse>,
            tonic::Status,
        >;
        /// TxEncodeAmino encodes an Amino transaction from JSON to encoded bytes.
        ///
        /// Since: cosmos-sdk 0.47
        async fn tx_encode_amino(
            &self,
            request: tonic::Request<super::TxEncodeAminoRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TxEncodeAminoResponse>,
            tonic::Status,
        >;
        /// TxDecodeAmino decodes an Amino transaction from encoded bytes to JSON.
        ///
        /// Since: cosmos-sdk 0.47
        async fn tx_decode_amino(
            &self,
            request: tonic::Request<super::TxDecodeAminoRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TxDecodeAminoResponse>,
            tonic::Status,
        >;
    }
    /// Service defines a gRPC service for interacting with transactions.
    #[derive(Debug)]
    pub struct ServiceServer<T: Service> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Service> ServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ServiceServer<T>
    where
        T: Service,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/cosmos.tx.v1beta1.Service/Simulate" => {
                    #[allow(non_camel_case_types)]
                    struct SimulateSvc<T: Service>(pub Arc<T>);
                    impl<T: Service> tonic::server::UnaryService<super::SimulateRequest>
                    for SimulateSvc<T> {
                        type Response = super::SimulateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SimulateRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).simulate(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SimulateSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.tx.v1beta1.Service/GetTx" => {
                    #[allow(non_camel_case_types)]
                    struct GetTxSvc<T: Service>(pub Arc<T>);
                    impl<T: Service> tonic::server::UnaryService<super::GetTxRequest>
                    for GetTxSvc<T> {
                        type Response = super::GetTxResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetTxRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).get_tx(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetTxSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.tx.v1beta1.Service/BroadcastTx" => {
                    #[allow(non_camel_case_types)]
                    struct BroadcastTxSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::BroadcastTxRequest>
                    for BroadcastTxSvc<T> {
                        type Response = super::BroadcastTxResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BroadcastTxRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).broadcast_tx(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BroadcastTxSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.tx.v1beta1.Service/GetTxsEvent" => {
                    #[allow(non_camel_case_types)]
                    struct GetTxsEventSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::GetTxsEventRequest>
                    for GetTxsEventSvc<T> {
                        type Response = super::GetTxsEventResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetTxsEventRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).get_txs_event(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetTxsEventSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.tx.v1beta1.Service/GetBlockWithTxs" => {
                    #[allow(non_camel_case_types)]
                    struct GetBlockWithTxsSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::GetBlockWithTxsRequest>
                    for GetBlockWithTxsSvc<T> {
                        type Response = super::GetBlockWithTxsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetBlockWithTxsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).get_block_with_txs(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetBlockWithTxsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.tx.v1beta1.Service/TxDecode" => {
                    #[allow(non_camel_case_types)]
                    struct TxDecodeSvc<T: Service>(pub Arc<T>);
                    impl<T: Service> tonic::server::UnaryService<super::TxDecodeRequest>
                    for TxDecodeSvc<T> {
                        type Response = super::TxDecodeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TxDecodeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).tx_decode(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TxDecodeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.tx.v1beta1.Service/TxEncode" => {
                    #[allow(non_camel_case_types)]
                    struct TxEncodeSvc<T: Service>(pub Arc<T>);
                    impl<T: Service> tonic::server::UnaryService<super::TxEncodeRequest>
                    for TxEncodeSvc<T> {
                        type Response = super::TxEncodeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TxEncodeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).tx_encode(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TxEncodeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.tx.v1beta1.Service/TxEncodeAmino" => {
                    #[allow(non_camel_case_types)]
                    struct TxEncodeAminoSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::TxEncodeAminoRequest>
                    for TxEncodeAminoSvc<T> {
                        type Response = super::TxEncodeAminoResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TxEncodeAminoRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).tx_encode_amino(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TxEncodeAminoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.tx.v1beta1.Service/TxDecodeAmino" => {
                    #[allow(non_camel_case_types)]
                    struct TxDecodeAminoSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::TxDecodeAminoRequest>
                    for TxDecodeAminoSvc<T> {
                        type Response = super::TxDecodeAminoResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TxDecodeAminoRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).tx_decode_amino(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TxDecodeAminoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Service> Clone for ServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: Service> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Service> tonic::server::NamedService for ServiceServer<T> {
        const NAME: &'static str = "cosmos.tx.v1beta1.Service";
    }
}
