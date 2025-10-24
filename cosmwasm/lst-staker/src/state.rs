use std::collections::BTreeMap;

use cosmwasm_std::Addr;
use depolama::{Prefix, RawAddrEncoding, Store, value::ValueCodecViaEncoding};
use unionlabs_encoding::Bincode;

/// The address of the [`lst`] contract this lst-staker stakes for.
pub enum LstHub {}
impl Store for LstHub {
    const PREFIX: Prefix = Prefix::new(b"hub");
    type Key = ();
    type Value = Addr;
}
impl ValueCodecViaEncoding for LstHub {
    type Encoding = RawAddrEncoding;
}

/// The config for all of the validators to stake against for new stake operations.
///
/// This is a map of all validator addresses to shares. Shares are used to calculate the percentage
/// of stake that will be allocated to the respective validator:
///
/// | validator | shares (400 total) | percentage |
/// |-----------|--------------------|------------|
/// | val-1     | 100                | 25%        |
/// | val-2     | 50                 | 12.5%      |
/// | val-3     | 250                | 62.5%      |
pub enum Validators {}
impl Store for Validators {
    const PREFIX: Prefix = Prefix::new(b"validators");
    type Key = ();
    type Value = BTreeMap<String, u128>;
}
impl ValueCodecViaEncoding for Validators {
    type Encoding = Bincode;
}
