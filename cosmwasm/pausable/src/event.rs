use cosmwasm_std::{Addr, Event};

/// Emitted when the pause is triggered by `account`.
///
/// ```solidity
/// event Paused(address account)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/Pausable.sol#L23>
pub struct Paused<'a> {
    pub account: &'a Addr,
}

impl From<Paused<'_>> for Event {
    fn from(event: Paused<'_>) -> Self {
        Event::new("paused").add_attribute("account", event.account)
    }
}

/// Emitted when the pause is lifted by `account`.
///
/// ```solidity
/// event Unpaused(address account)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/Pausable.sol#L28>
pub struct Unpaused<'a> {
    pub account: &'a Addr,
}

impl From<Unpaused<'_>> for Event {
    fn from(event: Unpaused<'_>) -> Self {
        Event::new("unpaused").add_attribute("account", event.account)
    }
}
