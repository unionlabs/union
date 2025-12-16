use cosmwasm_event::Event;
use cosmwasm_std::Addr;

/// Authority that manages this contract was updated.
///
/// ```solidity
/// event AuthorityUpdated(address authority);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManaged.sol#L10>
#[derive(Event)]
#[event("authority_updated")]
pub struct AuthorityUpdated<'a> {
    pub authority: &'a Addr,
}
