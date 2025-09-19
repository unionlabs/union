use cosmwasm_std::Addr;
use serde::{Deserialize, Serialize};

use crate::{execute::Method, types::RoleId};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    AdminRole {},
    PublicRole {},
    CanCall {
        method: String,
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
        method: Method,
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

    SetRoleGrantDelay {
        role_id: RoleId,
        grant_delay: RoleId,
    },
}
