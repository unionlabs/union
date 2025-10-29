use alloy_primitives::{U256, ruint::ParseError};
use cosmwasm_std::{Instantiate2AddressError, StdError};
use frissitheto::UpgradeError;
use ibc_union_spec::ChannelId;
use thiserror::Error;
use unionlabs::primitives::Bytes;

pub mod com;
pub mod contract;
pub mod msg;
pub mod state;
#[cfg(test)]
mod tests;
pub mod token_bucket;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Migrate(#[from] UpgradeError),

    #[error(transparent)]
    AccessManaged(#[from] access_managed::error::ContractError),

    #[error(transparent)]
    Upgradable(#[from] upgradable::error::ContractError),

    #[error("invalid ibc version, got {version}")]
    InvalidIbcVersion { version: String },
    #[error("invalid operation, sender must be ibc host")]
    OnlyIbcHost,
    #[error("invalid operation, sender must be self")]
    OnlySelf,
    #[error("invalid operation, sender must be admin")]
    OnlyAdmin,
    #[error(transparent)]
    Alloy(#[from] alloy_sol_types::Error),
    #[error("invalid zkgm instruction version: {version}")]
    UnsupportedVersion { version: u8 },
    #[error("unknown zkgm instruction opcode: {opcode}")]
    UnknownOpcode { opcode: u8 },
    #[error("unknown reply id: {id}")]
    UnknownReply { id: u64 },
    #[error("invalid operation, can only be executed by a market maker")]
    OnlyMaker,
    #[error("market maker failed to fill: {error}")]
    MarketMakerFailed { error: String },
    #[error("packet execution reentrancy not allowed")]
    AlreadyExecuting,
    #[error("order amount must be u128")]
    AmountOverflow,
    #[error("the quote token must be a valid utf8 denom")]
    InvalidQuoteToken,
    #[error("the solver must be a bech32 encoded contract address")]
    InvalidSolverAddress,
    #[error("the base token must be a valid utf8 denom")]
    InvalidBaseToken,
    #[error("invalid channel balance, counterparty has been taken over?")]
    InvalidChannelBalance,
    #[error("amount must be non zero")]
    InvalidAmount,
    #[error("transfer require funds to be submitted along the transaction")]
    MissingFunds,
    #[error("receiver must be a valid address")]
    InvalidReceiver,
    #[error("sender must be a valid address")]
    InvalidSender,
    #[error("the receiver can't be validated, make sure the bech prefix matches the current chain")]
    UnableToValidateReceiver,
    #[error("the receiver can't be validated, make sure the bech prefix matches the current chain")]
    UnableToValidateMarketMaker,
    #[error("the sender can't be validated, make sure the bech prefix matches the current chain")]
    UnableToValidateSender,
    #[error("call contract address must be a valid address")]
    InvalidContractAddress,
    #[error(
        "the call target contract address can't be validated, make sure the bech prefix matches the current chain"
    )]
    UnableToValidateCallTarget,
    #[error("contract creation event not found during handling `reply`")]
    ContractCreationEventNotFound,
    #[error("{0:?}")]
    InvalidPath(ParseError),
    #[error(
        "forward previousDestinationChannelId mismatch, actual: {actual}, expected: {expected}",
        actual = .actual.map(|id| id.to_string()).unwrap_or("None".to_owned())
    )]
    InvalidForwardDestinationChannelId {
        actual: Option<ChannelId>,
        expected: ChannelId,
    },
    #[error("forward packet missing next source channel id")]
    MissingForwardSourceChannelId,
    #[error("forward (sent) packet is missing from the reply")]
    ForwardedPacketMissingInReply,
    #[error("could not deserialize sent packet on reply, data: {sent_packet_data}")]
    CouldNotDeserializeSentPacket {
        error: serde_json_wasm::de::Error,
        sent_packet_data: Bytes,
    },
    #[error("asynchronous calls are not supported")]
    AsyncCallUnsupported,
    #[error("an error happened while calling the destination contract: {error}")]
    CallError { error: String },
    #[error(
        "channel path is full and can't be updated, too many hops? path: {path}, next_hop_index: {next_hop_index}"
    )]
    ChannelPathIsFull { path: U256, next_hop_index: usize },
    #[error("invalid asset origin path: actual={actual}, expected={expected}")]
    InvalidAssetOrigin { actual: U256, expected: U256 },
    #[error("invalid asset name (expected {expected}, found {found})")]
    InvalidAssetName { expected: String, found: String },
    #[error("invalid asset symbol (expected {expected}, found {found})")]
    InvalidAssetSymbol { expected: String, found: String },
    #[error("invalid asset decimals (expected {expected}, found {found})")]
    InvalidAssetDecimals { expected: u8, found: u8 },
    #[error("invalid batch instruction")]
    InvalidBatchInstruction,
    #[error("invalid forward instruction")]
    InvalidForwardInstruction,
    #[error("invalid call sender")]
    InvalidCallSender,
    #[error("async acknowledgements are not allowed in batches as they are atomic")]
    BatchMustBeSync,
    #[error("base amount must be greater or equal than quote amount when unwrapping")]
    BaseAmountLessThanQuoteAmount,
    #[error("invalid token order kind, when initializing, the token must be wrapped")]
    InvalidTokenOrderKind,
    #[error("invalid token order for unescrow, the quote token must unwraps the base token")]
    InvalidTokenOrderUnescrow,
    #[error("invalid metadata image")]
    InvalidMetadataImage,
    #[error("invalid unescrow order, the base token is not the representation of the quote token")]
    InvalidUnescrow,
    #[error("invalid fill type: {fill_type}")]
    InvalidFillType { fill_type: U256 },
    #[error("must be unwrap operation")]
    MustBeUnwrap,
    #[error("must be wrap operation")]
    MustBeWrap,
    #[error("token bucket is absent for {token}")]
    TokenBucketIsAbsent { token: String },
    #[error(transparent)]
    TokenBucket(#[from] token_bucket::Error),
    #[error("the instruction cannot be executed by a market maker")]
    InvalidMarketMakerOperation,
    #[error(transparent)]
    Instantiate2(#[from] Instantiate2AddressError),
    #[error("validator must be a valid address")]
    InvalidValidator,
    #[error(
        "the validator address can't be validated, make sure the bech prefix matches the current chain"
    )]
    UnableToValidateValidator,
    #[error(
        "you tried to transfer a token that was not previously bridged using the image of the metadata"
    )]
    WrappedTokenNotDeployed,
}
