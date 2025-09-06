use cosmwasm_std::Addr;
use depolama::{
    key::KeyCodecViaEncoding,
    value::{ValueCodecViaEncoding, ValueUnitEncoding},
    Prefix, RawAddrEncoding, Store,
};
use unionlabs_encoding::Bincode;

use crate::types::Admin;

/// The address of the [`ucs03-zkgm`] contract on this chain.
///
/// [`ucs03-zkgm`]: https://docs.union.build/ucs/03
pub enum Zkgm {}

impl Store for Zkgm {
    const PREFIX: Prefix = Prefix::new(b"zkgm");
    type Key = ();
    type Value = Addr;
}

impl ValueCodecViaEncoding for Zkgm {
    type Encoding = RawAddrEncoding;
}

/// All configured admins for this proxy account.
pub enum Admins {}

impl Store for Admins {
    const PREFIX: Prefix = Prefix::new(b"admins");
    type Key = Admin;
    type Value = ();
}

impl KeyCodecViaEncoding for Admins {
    type Encoding = Bincode;
}

impl ValueCodecViaEncoding for Admins {
    type Encoding = ValueUnitEncoding;
}
