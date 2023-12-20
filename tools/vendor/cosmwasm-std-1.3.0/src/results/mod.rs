//! This module contains the messages that are sent from the contract to the VM as an execution result

mod contract_result;
mod cosmos_msg;
mod empty;
mod events;
mod query;
mod response;
mod submessages;
mod system_result;

pub use contract_result::ContractResult;
#[cfg(all(feature = "stargate", feature = "cosmwasm_1_2"))]
pub use cosmos_msg::WeightedVoteOption;
pub use cosmos_msg::{wasm_execute, wasm_instantiate, BankMsg, CosmosMsg, CustomMsg, WasmMsg};
#[cfg(feature = "staking")]
pub use cosmos_msg::{DistributionMsg, StakingMsg};
#[cfg(feature = "stargate")]
pub use cosmos_msg::{GovMsg, VoteOption};
pub use empty::Empty;
pub use events::{attr, Attribute, Event};
pub use query::QueryResponse;
pub use response::Response;
#[allow(deprecated)]
pub use submessages::SubMsgExecutionResponse;
pub use submessages::{Reply, ReplyOn, SubMsg, SubMsgResponse, SubMsgResult};
pub use system_result::SystemResult;
