use cosmwasm_event::Event;
use cosmwasm_std::Addr;

/// Emitted when the pause is triggered by `account`.
///
/// ```solidity
/// event Paused(address account)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/Pausable.sol#L23>
#[derive(Event)]
#[event("paused")]
pub struct Paused<'a> {
    pub account: &'a Addr,
}

/// Emitted when the pause is lifted by `account`.
///
/// ```solidity
/// event Unpaused(address account)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/Pausable.sol#L28>
#[derive(Event)]
#[event("unpaused")]
pub struct Unpaused<'a> {
    pub account: &'a Addr,
}
