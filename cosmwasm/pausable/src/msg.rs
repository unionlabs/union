use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Pausable {
    /// Triggers stopped state.
    ///
    /// Requirements:
    ///
    /// - The contract must not be paused.
    ///
    /// ```solidity
    /// function _pause() internal virtual whenNotPaused;
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/Pausable.sol#L96>
    Pause {},

    /// Returns to normal state.
    ///
    /// Requirements:
    ///
    /// - The contract must be paused.
    ///
    /// ```solidity
    /// function _unpause() internal virtual whenPaused;
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/Pausable.sol#L108>
    Unpause {},
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum PausableQuery {
    /// Returns true if the contract is paused, and false otherwise.
    ///
    /// ```solidity
    /// function paused() public view virtual returns (bool);
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/Pausable.sol#L67>
    Paused {},
}
