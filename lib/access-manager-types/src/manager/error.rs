use cosmwasm_std::Addr;
use unionlabs_primitives::H256;

use crate::{RoleId, Selector};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum AccessManagerError {
    #[error("operation {0} is already scheduled")]
    AccessManagerAlreadyScheduled(H256),

    #[error("operation {0} is not scheduled")]
    AccessManagerNotScheduled(H256),

    #[error("operation {0} is not ready")]
    AccessManagerNotReady(H256),

    #[error("operation {0} is expired")]
    AccessManagerExpired(H256),

    #[error("role {0} is locked")]
    AccessManagerLockedRole(RoleId),

    #[error("confirmation address does not match sender")]
    AccessManagerBadConfirmation,

    #[error("{msg_sender} must have role {required_role_id}")]
    AccessManagerUnauthorizedAccount {
        msg_sender: Addr,
        required_role_id: RoleId,
    },

    #[error("{caller} is not authorized to call {selector} on {target}")]
    AccessManagerUnauthorizedCall {
        caller: Addr,
        target: Addr,
        selector: Box<Selector>,
    },

    #[error("unaothorized op consume on {target}")]
    AccessManagerUnauthorizedConsume { target: Addr },

    #[error(
        "{caller} is not authorized to cancel call with \
        {selector} on {target} (msg_sender: {msg_sender})"
    )]
    AccessManagerUnauthorizedCancel {
        msg_sender: Addr,
        caller: Addr,
        target: Addr,
        selector: Box<Selector>,
    },
}
