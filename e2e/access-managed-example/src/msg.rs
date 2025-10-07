use cosmwasm_std::Addr;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Increment {
        by: u32,
    },
    IncrementInReply {
        by: u32,
    },
    Decrement {
        by: u32,
        in_sub_msg: bool,
    },
    #[serde(rename = "$$decrement_in_sub_msg")]
    DecrementInSubMsg {
        by: u32,
    },
    Noop {},
    DelegateExecute {
        target: Addr,
        data: String,
    },
    DelegateSchedule {
        target: Addr,
        data: String,
        when: u64,
    },
    #[serde(untagged)]
    AccessManaged(access_manager_types::managed::msg::ExecuteMsg),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    CurrentValue {},
    #[serde(untagged)]
    AccessManaged(access_manager_types::managed::msg::QueryMsg),
}
