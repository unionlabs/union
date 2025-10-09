use cosmwasm_std::StdError;
use frissitheto::UpgradeError;
use ibc_union_spec::ChannelId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    StdError(#[from] StdError),

    #[error(transparent)]
    Migrate(#[from] UpgradeError),

    #[error(transparent)]
    Cw20(#[from] cw20_base::error::ContractError),

    #[error(transparent)]
    Cw20WrappedTokenfactory(#[from] cw20_wrapped_tokenfactory::error::ContractError),

    #[error("sender is not admin")]
    OnlyAdmin,

    #[error("sender is not zkgm")]
    OnlyZkgm,

    #[error("unsupported")]
    Unsupported,

    #[error("base amount must cover quote amount")]
    BaseAmountMustCoverQuoteAmount,

    #[error("receiver must be a valid bech32 address")]
    InvalidReceiver,

    #[error("intent packet hashes must be whitelisted to be executed")]
    IntentMustBeWhitelisted,

    #[error("the lane has not been configured to be fungible: channel_id={channel_id}")]
    LaneIsNotFungible { channel_id: ChannelId },

    #[error("the quote token must be a valid utf8 denom")]
    InvalidQuoteToken,

    #[error(
        "the order quote token must match the cw20 contract address: quote_token={quote_token} != self"
    )]
    InvalidFill { quote_token: String },
}
