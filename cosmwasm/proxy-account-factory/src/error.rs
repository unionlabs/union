use cosmwasm_std::{Checksum, StdError};
use frissitheto::{InitStateVersionError, UpgradeError};

use crate::BYTECODE_BASE_CHECKSUM;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Migrate(#[from] UpgradeError),

    #[error(transparent)]
    InitStateVersion(#[from] InitStateVersionError),

    #[error(
        "invalid bytecode base code id, expected checksum to be \
        {BYTECODE_BASE_CHECKSUM} but found {0}"
    )]
    InvalidBytecodeBaseChecksum(Checksum),
}
