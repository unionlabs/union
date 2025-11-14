use cosmwasm_std::{StdError, StdResult};
use depolama::{Bytes, Prefix, Store, ValueCodec};

const PAUSED: u8 = 0x01;

/// Stores whether or not the contract is paused.
///
/// This is either `0x00` or nonexistent.
///
/// ```solidity
/// bool private _paused;
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/Pausable.sol#L18>
pub enum IsPaused {}
impl Store for IsPaused {
    const PREFIX: Prefix = Prefix::new(b"paused");
    type Key = ();
    type Value = ();
}
impl ValueCodec<()> for IsPaused {
    fn encode_value((): &()) -> Bytes {
        [PAUSED].into()
    }

    fn decode_value(raw: &Bytes) -> StdResult<()> {
        match &**raw {
            [PAUSED] => Ok(()),
            _ => Err(StdError::generic_err(format!(
                "invalid value for pausable `IsPaused` storage: expected 0x01, found {raw}"
            ))),
        }
    }
}
