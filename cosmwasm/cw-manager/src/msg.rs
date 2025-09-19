use cosmwasm_std::Addr;
use serde::{Deserialize, Serialize};

use crate::types::{RoleId, Selector};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    AdminRole {},
    PublicRole {},
    CanCall {
        selector: String,
        target: Addr,
        caller: Addr,
    },
    Expiration {},
    MinSetback {},
    IsTargetClosed {
        target: Addr,
    },
    GetTargetFunctionRole {
        target: Addr,
        selector: Selector,
    },
    GetTargetAdminDelay {
        target: Addr,
    },
    GetRoleAdmin {
        role_id: RoleId,
    },
    GetRoleGuardian {
        role_id: RoleId,
    },
    GetRoleGrantDelay {
        role_id: RoleId,
    },
    GetAccess {
        role_id: RoleId,
        account: Addr,
    },
    HasRole {
        role_id: RoleId,
        account: Addr,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize, strum::IntoStaticStr)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    LabelRole {
        role_id: RoleId,
        label: String,
    },
    GrantRole {
        role_id: RoleId,
        account: Addr,
        execution_delay: u32,
    },
    RevokeRole {
        role_id: RoleId,
        account: Addr,
    },
    RenounceRole {
        role_id: RoleId,
        caller_confirmation: Addr,
    },

    SetRoleAdmin {
        role_id: RoleId,
        admin: RoleId,
    },

    SetRoleGuardian {
        role_id: RoleId,
        guardian: RoleId,
    },

    SetGrantDelay {
        role_id: RoleId,
        grant_delay: u32,
    },

    SetTargetAdminDelay {
        target: Addr,
        new_delay: u32,
    },

    SetTargetClosed {
        target: Addr,
        closed: bool,
    },

    SetTargetFunctionRole {
        target: Addr,
        selectors: Vec<Selector>,
        role_id: RoleId,
    },

    UpdateAuthority {
        target: Addr,
        new_authority: Addr,
    },
}

impl ExecuteMsg {
    pub(crate) fn selector(&self) -> Selector {
        Selector::new(<&'static str>::from(self))
    }
}
