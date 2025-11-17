#![no_std]

extern crate alloc;

use cosmwasm_std::Addr;
use ibc_union_spec::Packet;
use serde::{Deserialize, Serialize};
use unionlabs_primitives::{Bytes, U256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Solvable {
    DoSolve {
        packet: Packet,
        order: CwTokenOrderV2,
        path: U256,
        caller: Addr,
        relayer: Addr,
        relayer_msg: Bytes,
        intent: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum SolverQuery {
    /// Returns unit if the contract is a solver.
    IsSolver,
    /// Whether the solver allows the relayer to fulfill the order on our behalf.
    AllowMarketMakers,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct CwTokenOrderV2 {
    pub sender: Bytes,
    pub receiver: Bytes,
    pub base_token: Bytes,
    pub base_amount: U256,
    pub quote_token: Bytes,
    pub quote_amount: U256,
    pub kind: u8,
    pub metadata: Bytes,
}
