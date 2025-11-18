use cosmwasm_std::{Addr, Event};

/// Authority that manages this contract was updated.
///
/// ```solidity
/// event AuthorityUpdated(address authority);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManaged.sol#L10>
pub struct AuthorityUpdated<'a> {
    pub authority: &'a Addr,
}

impl From<AuthorityUpdated<'_>> for Event {
    fn from(event: AuthorityUpdated<'_>) -> Self {
        Event::new("authority_updated").add_attribute("authority", event.authority)
    }
}
