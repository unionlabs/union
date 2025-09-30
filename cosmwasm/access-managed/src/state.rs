use cosmwasm_std::Addr;
use depolama::{value::ValueCodecViaEncoding, Prefix, RawAddrEncoding, Store};
use unionlabs_encoding::Bincode;

/// ```solidity
/// address private _authority;
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManaged.sol#L20>
pub enum Authority {}
impl Store for Authority {
    const PREFIX: Prefix = Prefix::new(b"authority");
    type Key = ();
    type Value = Addr;
}
impl ValueCodecViaEncoding for Authority {
    type Encoding = RawAddrEncoding;
}

/// ```solidity
/// bool private _consumingSchedule;
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManaged.sol#L22>
pub enum ConsumingSchedule {}
impl Store for ConsumingSchedule {
    const PREFIX: Prefix = Prefix::new(b"consuming_schedule");
    type Key = ();
    type Value = bool;
}
impl ValueCodecViaEncoding for ConsumingSchedule {
    type Encoding = Bincode;
}
