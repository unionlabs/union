use std::{fmt::Display, num::NonZeroU64, str::FromStr};

use gno_types::{
    Block, BlockMeta, Event, InfoResponse, NodeInfo, PublicKey, QueryResponse, ResponseBase,
    SignedHeader, Validator,
};
use serde::{Deserialize, Serialize};
use unionlabs::{
    bounded::BoundedI64,
    google::protobuf::{any::RawAny, timestamp::Timestamp},
    ibc::core::commitment::merkle_proof::MerkleProof,
    primitives::{Bech32, Bech32DecodeError, Bytes, FixedBytesError, H160, H256, encoding::Base64},
    prost::{self, Message},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Order {
    Asc,
    Desc,
}

impl Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Order::Asc => "asc",
            Order::Desc => "desc",
        })
    }
}

impl FromStr for Order {
    type Err = InvalidOrder;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asc" => Ok(Order::Asc),
            "desc" => Ok(Order::Desc),
            _ => Err(InvalidOrder {}),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid order, must be either `asc` or `desc`")]
pub struct InvalidOrder {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockResponse {
    pub block_meta: BlockMeta,
    pub block: Block,
}

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(deny_unknown_fields)]
// pub struct BlockchainResponse {
//     #[serde(with = "::serde_utils::string")]
//     pub last_height: u64,
//     pub block_metas: Vec<BlockMeta>,
// }

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(deny_unknown_fields)]
// pub struct BlockMeta {
//     pub block_id: BlockId,
//     #[serde(with = "::serde_utils::string")]
//     pub block_size: u64,
//     pub header: Header,
//     #[serde(with = "::serde_utils::string")]
//     pub num_txs: u64,
// }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusResponse {
    pub node_info: NodeInfo,
    pub sync_info: SyncInfo,
    pub validator_info: ValidatorInfo,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyncInfo {
    pub latest_block_hash: Option<H256<Base64>>,
    pub latest_app_hash: Option<H256<Base64>>,
    #[serde(with = "::serde_utils::string")]
    pub latest_block_height: u64,
    pub latest_block_time: Timestamp,
    pub catching_up: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidatorInfo {
    pub address: Bech32<H160>,
    pub pub_key: PublicKey,
    // REVIEW: is this bounded the same way as Validator?
    #[serde(with = "::serde_utils::string")]
    pub voting_power: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidatorsResponse {
    #[serde(with = "::serde_utils::string")]
    pub block_height: NonZeroU64,
    pub validators: Vec<Validator>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbciQueryResponse {
    pub response: QueryResponse,
}

impl AbciQueryResponse {
    pub fn decode_merkle_proof(&self) -> Result<MerkleProof, DecodeMerkleProofError> {
        Ok(MerkleProof::try_from(protos::ibc::core::commitment::v1::MerkleProof {
            proofs: self
                .response
                .proof
                .as_ref().ok_or(DecodeMerkleProofError::NoProof)?
                .ops
                .iter()
                .map(|op| {
                    <protos::cosmos::ics23::v1::CommitmentProof as unionlabs::prost::Message>::decode(
                        &*op.data,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        })?)
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum DecodeMerkleProofError {
    #[error("abci query response does not contain a proof")]
    NoProof,
    #[error(transparent)]
    CommitmentProofDecode(#[from] unionlabs::prost::DecodeError),
    #[error(transparent)]
    InvalidProof(#[from] unionlabs::ibc::core::commitment::merkle_proof::TryFromMerkleProofError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbciInfoResponse {
    pub response: InfoResponse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitResponse {
    pub signed_header: SignedHeader,
    pub canonical: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockResultsResponse {
    #[serde(with = "::serde_utils::string")]
    pub height: u64,
    pub results: AbciResponses,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbciResponses {
    pub deliver_tx: Option<Vec<DeliverTxResponse>>,
    pub end_block: EndBlockResponse,
    pub begin_block: BeginBlockResponse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BeginBlockResponse {
    #[serde(rename = "ResponseBase")]
    pub response_base: ResponseBase,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseCheckTx {
    #[serde(rename = "ResponseBase")]
    pub response_base: ResponseBase,
    /// nondeterministic
    #[serde(rename = "GasWanted", with = "::serde_utils::string")]
    pub gas_wanted: BoundedI64<0>,
    #[serde(rename = "GasUsed", with = "::serde_utils::string")]
    pub gas_used: BoundedI64<0>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeliverTxResponse {
    #[serde(rename = "ResponseBase")]
    pub response_base: ResponseBase,
    #[serde(rename = "GasWanted", with = "::serde_utils::string")]
    pub gas_wanted: BoundedI64<0>,
    #[serde(rename = "GasUsed", with = "::serde_utils::string")]
    pub gas_used: BoundedI64<0>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TxResponse {
    pub hash: H256<Base64>,
    #[serde(with = "::serde_utils::string")]
    pub height: BoundedI64<0>,
    pub index: usize, // TODO: What should this actually be?
    pub tx_result: DeliverTxResponse,
    pub tx: Bytes<Base64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndBlockResponse {
    #[serde(rename = "ResponseBase")]
    pub response_base: ResponseBase,
    #[serde(rename = "ValidatorUpdates")]
    pub validator_updates: Option<Vec<ValidatorUpdate>>,
    #[serde(rename = "ConsensusParams")]
    pub consensus_params: Option<ConsensusParams>,
    #[serde(rename = "Events")]
    pub events: Option<Vec<Event>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidatorUpdate {
    #[serde(rename = "Address")]
    pub address: Bech32<H160>,
    #[serde(rename = "PubKey")]
    pub pub_key: Bytes<Base64>,
    #[serde(rename = "Power", with = "::serde_utils::string")]
    pub power: i64,
}

// #[derive(macros::Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(deny_unknown_fields)]
// pub struct BroadcastTxSyncResponse {
//     pub codespace: String,
//     pub code: Code,
//     pub data: Bytes<Base64>,
//     pub log: String,
//     pub hash: H256<HexUnprefixed>,
// }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsensusParams {
    #[serde(rename = "Block")]
    pub block: Option<BlockParams>,
    #[serde(rename = "Validator")]
    pub validator: Option<ValidatorParams>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockParams {
    #[serde(rename = "MaxTxBytes", with = "::serde_utils::string")]
    pub max_tx_bytes: BoundedI64<1>,
    #[serde(rename = "MaxDataBytes", with = "::serde_utils::string")]
    pub max_data_bytes: BoundedI64<1>,
    #[serde(rename = "MaxBlockBytes", with = "::serde_utils::string")]
    pub max_block_bytes: BoundedI64<1>,
    #[serde(rename = "MaxGas", with = "::serde_utils::string")]
    pub max_gas: BoundedI64<{ -1 }>,
    #[serde(rename = "TimeIotaMS", with = "::serde_utils::string")]
    pub time_iota_ms: BoundedI64<1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidatorParams {
    #[serde(rename = "PubKeyTypeURLs")]
    pub pub_key_type_urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BroadcastTxCommitResult {
    pub check_tx: DeliverTxResponse,
    pub deliver_tx: DeliverTxResponse,
    pub hash: H256<Base64>,
    pub height: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AbciAccount {
    #[serde(rename = "BaseAccount")]
    pub base_account: BaseAccount,
    #[serde(with = "::serde_utils::string")]
    pub attributes: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BaseAccount {
    pub address: String,
    pub coins: String,
    pub public_key: Option<PublicKey>,
    #[serde(with = "::serde_utils::string")]
    pub account_number: u64,
    #[serde(with = "::serde_utils::string")]
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxSignature {
    pub pub_key: PublicKey,
    pub signature: Bytes<Base64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tx {
    pub messages: Vec<Msg>,
    pub fee: TxFee,
    pub signatures: Vec<TxSignature>,
    pub memo: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TxFee {
    #[serde(with = "::serde_utils::string")]
    pub gas_wanted: i64,
    pub gas_fee: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgRun {
    pub caller: String,
    pub send: String,
    pub max_deposit: String,
    pub package: MemPackage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemPackage {
    pub name: String,
    pub path: String,
    pub files: Vec<MemFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<RawAny>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub info: Option<RawAny>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemFile {
    pub name: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgCall {
    /// the bech32 address of the caller
    pub caller: Bech32<H160>,
    /// the amount of funds to be deposited to the package, if any `("<amount><denomination>")`
    // TODO: Coin
    pub send: String,
    /// the amount of funds to lock for the storage, if any `("<amount><denomination>")`
    // TODO: Coin
    pub max_deposit: String,
    /// the gno package path
    pub pkg_path: String,
    /// the function name being invoked
    pub func: String,
    /// the function arguments
    ///
    /// null | string\[\]
    // TODO: null is empty vec
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgSend {
    /// the bech32 address of the fund sender
    pub from_address: Bech32<H160>,
    /// the bech32 address of the fund receiver
    pub to_address: Bech32<H160>,
    /// the denomination and amount of fund sent `("<amount><denomination>")`
    // TODO: Coin
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum Msg {
    #[serde(rename = "/vm.m_run")]
    Run(MsgRun),
    #[serde(rename = "/vm.m_call")]
    Call(MsgCall),
    // #[serde(rename = "/vm.m_addpkg")]
    // AddPkg(MsgAddPkg),
    #[serde(rename = "/bank.MsgSend")]
    Send(MsgSend),
}

impl Msg {
    pub fn into_any(self) -> protos::google::protobuf::Any {
        use unionlabs::prost::Message;

        match self {
            Msg::Run(msg) => protos::google::protobuf::Any {
                type_url: "/vm.m_run".to_owned(),
                value: protos::gno::vm::MsgRun {
                    caller: msg.caller,
                    send: msg.send,
                    max_deposit: msg.max_deposit,
                    package: Some(protos::gno::vm::MemPackage {
                        name: msg.package.name,
                        path: msg.package.path,
                        files: msg
                            .package
                            .files
                            .into_iter()
                            .map(|file| protos::gno::vm::MemFile {
                                name: file.name,
                                body: file.body,
                            })
                            .collect(),
                        r#type: msg.package.r#type.map(Into::into),
                        info: msg.package.info.map(Into::into),
                    }),
                }
                .encode_to_vec(),
            },
            Msg::Call(msg) => protos::google::protobuf::Any {
                type_url: "/vm.m_run".to_owned(),
                value: protos::gno::vm::MsgCall {
                    caller: msg.caller.to_string(),
                    send: msg.send,
                    max_deposit: msg.max_deposit,
                    pkg_path: msg.pkg_path,
                    func: msg.func,
                    args: msg.args.unwrap_or_default(),
                }
                .encode_to_vec(),
            },
            Msg::Send(msg) => protos::google::protobuf::Any {
                type_url: "/bank.MsgSend".to_owned(),
                value: protos::gno::bank::MsgSend {
                    from_address: msg.from_address.to_string(),
                    to_address: msg.to_address.to_string(),
                    amount: msg.amount,
                }
                .encode_to_vec(),
            },
        }
    }

    pub fn try_from_any(any: &protos::google::protobuf::Any) -> Result<Self, MsgTryFromAnyError> {
        match &*any.type_url {
            "/vm.m_run" => {
                let msg = protos::gno::vm::MsgRun::decode(&*any.value)?;

                let package = msg.package.ok_or(MsgTryFromAnyError::NoPackage)?;
                Ok(Msg::Run(MsgRun {
                    caller: msg.caller,
                    send: msg.send,
                    max_deposit: msg.max_deposit,
                    package: MemPackage {
                        name: package.name,
                        path: package.path,
                        files: package
                            .files
                            .into_iter()
                            .map(|file| MemFile {
                                name: file.name,
                                body: file.body,
                            })
                            .collect(),
                        r#type: package.r#type.map(Into::into),
                        info: package.info.map(Into::into),
                    },
                }))
            }
            "/vm.m_call" => {
                let msg = protos::gno::vm::MsgCall::decode(&*any.value)?;

                Ok(Msg::Call(MsgCall {
                    caller: msg.caller.parse()?,
                    send: msg.send,
                    max_deposit: msg.max_deposit,
                    pkg_path: msg.pkg_path,
                    func: msg.func,
                    args: Some(msg.args),
                }))
            }
            "/bank.MsgSend" => {
                let msg = protos::gno::bank::MsgSend::decode(&*any.value)?;

                Ok(Msg::Send(MsgSend {
                    from_address: msg.from_address.parse()?,
                    to_address: msg.to_address.parse()?,
                    amount: msg.amount,
                }))
            }
            ty => Err(MsgTryFromAnyError::UnknownMsgType(ty.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum MsgTryFromAnyError {
    #[error(transparent)]
    Prost(#[from] prost::DecodeError),
    #[error("no package")]
    NoPackage,
    #[error("unknown msg type: {0}")]
    UnknownMsgType(String),
    #[error("invalid address")]
    InvalidAddress(#[from] Bech32DecodeError<FixedBytesError>),
}
