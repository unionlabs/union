use cosmwasm_std::{Addr, StdError};
use frissitheto::UpgradeError;
use unionlabs_primitives::H256;

use crate::types::{Method, RoleId};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("migration error")]
    Migrate(#[from] UpgradeError),

    // AccessManagerAlreadyScheduled(H256),
    // AccessManagerNotScheduled(H256),
    // AccessManagerNotReady(H256),
    // AccessManagerExpired(H256),
    #[error("role {0} is locked")]
    AccessManagerLockedRole(RoleId),
    // AccessManagerBadConfirmation(),
    // AccessManagerUnauthorizedAccount {
    //     msg_sender: Addr,
    //     role_id: RoleId,
    // },
    // AccessManagerUnauthorizedCall {
    //     caller: Addr,
    //     target: Addr,
    //     method: Method,
    // },
    // AccessManagerUnauthorizedConsume {
    //     target: Addr,
    // },
    // AccessManagerUnauthorizedCancel {
    //     msgsender: Addr,
    //     caller: Addr,
    //     target: Addr,
    //     method: Method,
    // },
    // AccessManagerInvalidInitialAdmin {
    //     initial_admin: Addr,
    // },
}
