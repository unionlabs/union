use cosmwasm_std::StdError;
use token_factory_api::TokenFactoryMsg;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    StdError(#[from] StdError),

    #[error("only admin can execute")]
    OnlyAdmin,

    #[error("this contract cant mint the token with denom {0} because it's not the owner")]
    CantMint(String),

    #[error("unexpected reply ({0})")]
    UnexpectedReply(u64),

    #[error("denom to store does not exist during handling `reply`")]
    DenomToStoreDoesNotExist,

    #[error("submessage error: {0}")]
    SubMsgError(String),

    #[error("contract creation event not found during handling `reply`")]
    ContractCreationEventNotFound,

    #[error("token with denom `{0}` does not exist")]
    TokenDoesNotExist(String),

    #[error("unexpected execute msg: {0:?}")]
    UnexpectedExecuteMsg(TokenFactoryMsg),
}
