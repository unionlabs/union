use alloy::primitives::ruint::ParseError;
use cosmwasm_std::{Addr, StdError, Uint128};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    StdError(#[from] StdError),

    #[error("only the admin can execute")]
    OnlyAdmin,

    #[error("the protocol needs {needed} amount but given {given}")]
    InvalidFunds { needed: Uint128, given: Uint128 },

    #[error("invalid path: {0}")]
    InvalidPath(ParseError),

    #[error("invalid denom: {0}")]
    InvalidDenom(String),

    #[error("minter config is expected to be osmosis-tokenfactory")]
    InvalidMinterConfig,

    #[error(
        "the token ownership can only be changed by this contract or the minter operator since this token is owned by this contract"
    )]
    UnauthorizedWhenSelfOwned,

    #[error("the token is owned by {owner} and, {sender} cannot change the ownership")]
    UnauthorizedThirdParty { owner: Addr, sender: Addr },

    #[error("empty name or symbol in metadata")]
    EmptyNameOrSymbol,

    #[error("alloy solidity parsing error: {0}")]
    Alloy(#[from] alloy::sol_types::Error),

    #[error(
        "wrapped token metadata is invalid, it must be a valid tokenfactory metadata json string"
    )]
    CouldNotDecodeMetadata,

    #[error(
        "tokenfactory minter expects the implementation field of foa v2 to be a constant 'tokenfactory' string"
    )]
    UnexpectedImplementation,

    #[error("cannot deploy a new token with v1 TokenOrder")]
    TokenOrderV1DeploymentIsDeprecated,
}
