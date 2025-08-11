use cosmwasm_std::StdError;
use frissitheto::UpgradeError;
use ibc_union_spec::ChannelId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    StdError(#[from] StdError),

    #[error("migration error: {0}")]
    Migrate(#[from] UpgradeError),

    #[error("sender is not admin")]
    OnlyAdmin,

    #[error("sender is not zkgm")]
    OnlyZkgm,

    #[error("base amount must cover quote amount")]
    BaseAmountMustCoverQuoteAmount,

    #[error("receiver must be a valid bech32 address")]
    InvalidReceiver,

    #[error("intent packet hashes must be whitelisted to be executed")]
    IntentMustBeWhitelisted,

    #[error("the lane has not been configured to be fungible")]
    LaneIsNotFungible { channel_id: ChannelId },
}
