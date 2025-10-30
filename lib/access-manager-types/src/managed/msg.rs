use cosmwasm_std::Addr;
use serde::{Deserialize, Serialize};

use crate::Selector;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub initial_authority: Addr,
}

/// Interface mirroring the executable calls from [`IAccessManaged.sol`](https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManaged.sol).
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Transfers control to a new authority. The caller must be the current authority.
    ///
    /// ```solidity
    /// function setAuthority(address) external
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManaged.sol#L66>
    SetAuthority { new_authority: Addr },
}

/// Interface mirroring the queries from [`IAccessManaged.sol`](https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManaged.sol).
#[derive(Debug, PartialEq, Serialize, Deserialize, strum::IntoStaticStr)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the current authority.
    ///
    /// ```solidity
    /// function authority() external view returns (address)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManaged.sol#L19>
    Authority {},

    /// Returns true only in the context of a delayed restricted call, at the moment that the
    /// scheduled operation is being consumed. Prevents denial of service for delayed restricted
    /// calls in the case that the contract performs attacker controlled calls.
    ///
    /// ```solidity
    /// function isConsumingScheduledOp() external view returns (bytes4)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManaged.sol#L31>
    IsConsumingScheduledOp {},
}

impl QueryMsg {
    pub fn selector(&self) -> &'static Selector {
        Selector::new(<&'static str>::from(self))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}
