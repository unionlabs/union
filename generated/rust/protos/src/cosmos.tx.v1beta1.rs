// @generated
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
impl ::prost::Name for Tx {
    const NAME: &'static str = "Tx";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
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
impl ::prost::Name for TxRaw {
    const NAME: &'static str = "TxRaw";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
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
impl ::prost::Name for SignDoc {
    const NAME: &'static str = "SignDoc";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
}
/// SignDocDirectAux is the type used for generating sign bytes for
/// SIGN_MODE_DIRECT_AUX.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignDocDirectAux {
    /// body_bytes is protobuf serialization of a TxBody that matches the
    /// representation in TxRaw.
    #[prost(bytes = "vec", tag = "1")]
    pub body_bytes: ::prost::alloc::vec::Vec<u8>,
    /// public_key is the public key of the signing account.
    #[prost(message, optional, tag = "2")]
    pub public_key: ::core::option::Option<::pbjson_types::Any>,
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
    /// tips have been deprecated and should not be used
    #[deprecated]
    #[prost(message, optional, tag = "6")]
    pub tip: ::core::option::Option<Tip>,
}
impl ::prost::Name for SignDocDirectAux {
    const NAME: &'static str = "SignDocDirectAux";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
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
    pub messages: ::prost::alloc::vec::Vec<::pbjson_types::Any>,
    /// memo is any arbitrary note/comment to be added to the transaction.
    /// WARNING: in clients, any publicly exposed text should not be called memo,
    /// but should be called `note` instead (see
    /// <https://github.com/cosmos/cosmos-sdk/issues/9122>).
    #[prost(string, tag = "2")]
    pub memo: ::prost::alloc::string::String,
    /// timeout_height is the block height after which this transaction will not
    /// be processed by the chain.
    #[prost(uint64, tag = "3")]
    pub timeout_height: u64,
    /// unordered, when set to true, indicates that the transaction signer(s)
    /// intend for the transaction to be evaluated and executed in an un-ordered
    /// fashion. Specifically, the account's nonce will NOT be checked or
    /// incremented, which allows for fire-and-forget as well as concurrent
    /// transaction execution.
    ///
    /// Note, when set to true, the existing 'timeout_height' value must
    /// be set and will be used to correspond to a time_stamp in which the transaction is deemed
    /// valid.
    #[prost(bool, tag = "4")]
    pub unordered: bool,
    /// timeout_timestamp is the block time after which this transaction will not
    /// be processed by the chain.
    ///
    /// Note, if unordered=true this value MUST be set
    /// and will act as a short-lived TTL in which the transaction is deemed valid
    /// and kept in memory to prevent duplicates.
    #[prost(message, optional, tag = "5")]
    pub timeout_timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
    /// extension_options are arbitrary options that can be added by chains
    /// when the default options are not sufficient. If any of these are present
    /// and can't be handled, the transaction will be rejected
    #[prost(message, repeated, tag = "1023")]
    pub extension_options: ::prost::alloc::vec::Vec<::pbjson_types::Any>,
    /// extension_options are arbitrary options that can be added by chains
    /// when the default options are not sufficient. If any of these are present
    /// and can't be handled, they will be ignored
    #[prost(message, repeated, tag = "2047")]
    pub non_critical_extension_options: ::prost::alloc::vec::Vec<::pbjson_types::Any>,
}
impl ::prost::Name for TxBody {
    const NAME: &'static str = "TxBody";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
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
    #[deprecated]
    #[prost(message, optional, tag = "3")]
    pub tip: ::core::option::Option<Tip>,
}
impl ::prost::Name for AuthInfo {
    const NAME: &'static str = "AuthInfo";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
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
    pub public_key: ::core::option::Option<::pbjson_types::Any>,
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
impl ::prost::Name for SignerInfo {
    const NAME: &'static str = "SignerInfo";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
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
    impl ::prost::Name for Single {
        const NAME: &'static str = "Single";
        const PACKAGE: &'static str = "cosmos.tx.v1beta1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!("cosmos.tx.v1beta1.ModeInfo.{}", Self::NAME)
        }
    }
    /// Multi is the mode info for a multisig public key
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Multi {
        /// bitarray specifies which keys within the multisig are signing
        #[prost(message, optional, tag = "1")]
        pub bitarray:
            ::core::option::Option<super::super::super::crypto::multisig::v1beta1::CompactBitArray>,
        /// mode_infos is the corresponding modes of the signers of the multisig
        /// which could include nested multisig public keys
        #[prost(message, repeated, tag = "2")]
        pub mode_infos: ::prost::alloc::vec::Vec<super::ModeInfo>,
    }
    impl ::prost::Name for Multi {
        const NAME: &'static str = "Multi";
        const PACKAGE: &'static str = "cosmos.tx.v1beta1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!("cosmos.tx.v1beta1.ModeInfo.{}", Self::NAME)
        }
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
impl ::prost::Name for ModeInfo {
    const NAME: &'static str = "ModeInfo";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
}
/// Fee includes the amount of coins paid in fees and the maximum
/// gas to be used by the transaction. The ratio yields an effective "gasprice",
/// which must be above some minimum to be accepted into the mempool.
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
    /// if unset, the first signer is responsible for paying the fees. If set, the
    /// specified account must pay the fees. the payer must be a tx signer (and
    /// thus have signed this field in AuthInfo). setting this field does *not*
    /// change the ordering of required signers for the transaction.
    #[prost(string, tag = "3")]
    pub payer: ::prost::alloc::string::String,
    /// if set, the fee payer (either the first signer or the value of the payer
    /// field) requests that a fee grant be used to pay fees instead of the fee
    /// payer's own balance. If an appropriate fee grant does not exist or the
    /// chain does not support fee grants, this will fail
    #[prost(string, tag = "4")]
    pub granter: ::prost::alloc::string::String,
}
impl ::prost::Name for Fee {
    const NAME: &'static str = "Fee";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
}
/// Tip is the tip used for meta-transactions.
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
impl ::prost::Name for Tip {
    const NAME: &'static str = "Tip";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
}
/// AuxSignerData is the intermediary format that an auxiliary signer (e.g. a
/// tipper) builds and sends to the fee payer (who will build and broadcast the
/// actual tx). AuxSignerData is not a valid tx in itself, and will be rejected
/// by the node if sent directly as-is.
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
impl ::prost::Name for AuxSignerData {
    const NAME: &'static str = "AuxSignerData";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
}
/// GetTxsEventRequest is the request type for the Service.TxsByEvents
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTxsEventRequest {
    /// events is the list of transaction event type.
    /// Deprecated: post v0.47.x use query instead, which should contain a valid
    /// events query.
    #[deprecated]
    #[prost(string, repeated, tag = "1")]
    pub events: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// pagination defines a pagination for the request.
    /// Deprecated: post v0.46.x use page and limit instead.
    #[deprecated]
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
    #[prost(enumeration = "OrderBy", tag = "3")]
    pub order_by: i32,
    /// page is the page number to query, starts at 1. If not provided, will
    /// default to first page.
    #[prost(uint64, tag = "4")]
    pub page: u64,
    /// limit is the total number of results to be returned in the result page.
    /// If left empty it will default to a value to be set by each app.
    #[prost(uint64, tag = "5")]
    pub limit: u64,
    /// query defines the transaction event query that is proxied to Tendermint's
    /// TxSearch RPC method. The query must be valid.
    #[prost(string, tag = "6")]
    pub query: ::prost::alloc::string::String,
}
impl ::prost::Name for GetTxsEventRequest {
    const NAME: &'static str = "GetTxsEventRequest";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
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
    pub tx_responses: ::prost::alloc::vec::Vec<super::super::base::abci::v1beta1::TxResponse>,
    /// pagination defines a pagination for the response.
    /// Deprecated: post v0.46.x use total instead.
    #[deprecated]
    #[prost(message, optional, tag = "3")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
    /// total is total number of results available
    #[prost(uint64, tag = "4")]
    pub total: u64,
}
impl ::prost::Name for GetTxsEventResponse {
    const NAME: &'static str = "GetTxsEventResponse";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
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
impl ::prost::Name for BroadcastTxRequest {
    const NAME: &'static str = "BroadcastTxRequest";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
}
/// BroadcastTxResponse is the response type for the
/// Service.BroadcastTx method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BroadcastTxResponse {
    /// tx_response is the queried TxResponses.
    #[prost(message, optional, tag = "1")]
    pub tx_response: ::core::option::Option<super::super::base::abci::v1beta1::TxResponse>,
}
impl ::prost::Name for BroadcastTxResponse {
    const NAME: &'static str = "BroadcastTxResponse";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
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
    #[prost(bytes = "vec", tag = "2")]
    pub tx_bytes: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SimulateRequest {
    const NAME: &'static str = "SimulateRequest";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
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
impl ::prost::Name for SimulateResponse {
    const NAME: &'static str = "SimulateResponse";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
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
impl ::prost::Name for GetTxRequest {
    const NAME: &'static str = "GetTxRequest";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
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
    pub tx_response: ::core::option::Option<super::super::base::abci::v1beta1::TxResponse>,
}
impl ::prost::Name for GetTxResponse {
    const NAME: &'static str = "GetTxResponse";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
}
/// GetBlockWithTxsRequest is the request type for the Service.GetBlockWithTxs
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockWithTxsRequest {
    /// height is the height of the block to query.
    #[prost(int64, tag = "1")]
    pub height: i64,
    /// pagination defines a pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for GetBlockWithTxsRequest {
    const NAME: &'static str = "GetBlockWithTxsRequest";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
}
/// GetBlockWithTxsResponse is the response type for the Service.GetBlockWithTxs
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockWithTxsResponse {
    /// txs are the transactions in the block.
    #[prost(message, repeated, tag = "1")]
    pub txs: ::prost::alloc::vec::Vec<Tx>,
    #[prost(message, optional, tag = "2")]
    pub block_id: ::core::option::Option<super::super::super::cometbft::types::v1::BlockId>,
    #[prost(message, optional, tag = "3")]
    pub block: ::core::option::Option<super::super::super::cometbft::types::v1::Block>,
    /// pagination defines a pagination for the response.
    #[prost(message, optional, tag = "4")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for GetBlockWithTxsResponse {
    const NAME: &'static str = "GetBlockWithTxsResponse";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
}
/// TxDecodeRequest is the request type for the Service.TxDecode
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxDecodeRequest {
    /// tx_bytes is the raw transaction.
    #[prost(bytes = "vec", tag = "1")]
    pub tx_bytes: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for TxDecodeRequest {
    const NAME: &'static str = "TxDecodeRequest";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
}
/// TxDecodeResponse is the response type for the
/// Service.TxDecode method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxDecodeResponse {
    /// tx is the decoded transaction.
    #[prost(message, optional, tag = "1")]
    pub tx: ::core::option::Option<Tx>,
}
impl ::prost::Name for TxDecodeResponse {
    const NAME: &'static str = "TxDecodeResponse";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
}
/// TxEncodeRequest is the request type for the Service.TxEncode
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxEncodeRequest {
    /// tx is the transaction to encode.
    #[prost(message, optional, tag = "1")]
    pub tx: ::core::option::Option<Tx>,
}
impl ::prost::Name for TxEncodeRequest {
    const NAME: &'static str = "TxEncodeRequest";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
}
/// TxEncodeResponse is the response type for the
/// Service.TxEncode method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxEncodeResponse {
    /// tx_bytes is the encoded transaction bytes.
    #[prost(bytes = "vec", tag = "1")]
    pub tx_bytes: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for TxEncodeResponse {
    const NAME: &'static str = "TxEncodeResponse";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
}
/// TxEncodeAminoRequest is the request type for the Service.TxEncodeAmino
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxEncodeAminoRequest {
    #[prost(string, tag = "1")]
    pub amino_json: ::prost::alloc::string::String,
}
impl ::prost::Name for TxEncodeAminoRequest {
    const NAME: &'static str = "TxEncodeAminoRequest";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
}
/// TxEncodeAminoResponse is the response type for the Service.TxEncodeAmino
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxEncodeAminoResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub amino_binary: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for TxEncodeAminoResponse {
    const NAME: &'static str = "TxEncodeAminoResponse";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
}
/// TxDecodeAminoRequest is the request type for the Service.TxDecodeAmino
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxDecodeAminoRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub amino_binary: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for TxDecodeAminoRequest {
    const NAME: &'static str = "TxDecodeAminoRequest";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
}
/// TxDecodeAminoResponse is the response type for the Service.TxDecodeAmino
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxDecodeAminoResponse {
    #[prost(string, tag = "1")]
    pub amino_json: ::prost::alloc::string::String,
}
impl ::prost::Name for TxDecodeAminoResponse {
    const NAME: &'static str = "TxDecodeAminoResponse";
    const PACKAGE: &'static str = "cosmos.tx.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.tx.v1beta1.{}", Self::NAME)
    }
}
/// OrderBy defines the sorting order
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OrderBy {
    /// ORDER_BY_UNSPECIFIED specifies an unknown sorting order. OrderBy defaults
    /// to ASC in this case.
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
/// BroadcastMode specifies the broadcast mode for the TxService.Broadcast RPC
/// method.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum BroadcastMode {
    /// zero-value for mode ordering
    Unspecified = 0,
    /// Deprecated: use BROADCAST_MODE_SYNC instead,
    /// BROADCAST_MODE_BLOCK is not supported by the SDK from v0.47.x onwards.
    Block = 1,
    /// BROADCAST_MODE_SYNC defines a tx broadcasting mode where the client waits
    /// for a CheckTx execution response only.
    Sync = 2,
    /// BROADCAST_MODE_ASYNC defines a tx broadcasting mode where the client
    /// returns immediately.
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
include!("cosmos.tx.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
