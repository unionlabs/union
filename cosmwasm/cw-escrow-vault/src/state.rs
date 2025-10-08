use cosmwasm_std::Addr;
use depolama::{
    Prefix, RawAddrEncoding, Store, key::KeyCodecViaEncoding, value::ValueCodecViaEncoding,
};
use ibc_union_spec::ChannelId;
use unionlabs_encoding::Bincode;
use unionlabs_primitives::{Bytes, H256, U256};

pub enum IntentWhitelist {}
impl Store for IntentWhitelist {
    const PREFIX: Prefix = Prefix::new(b"intent_whitelist");

    type Key = H256;
    type Value = bool;
}
impl KeyCodecViaEncoding for IntentWhitelist {
    type Encoding = Bincode;
}
impl ValueCodecViaEncoding for IntentWhitelist {
    type Encoding = Bincode;
}

#[derive(serde::Serialize, bincode::Encode, bincode::Decode)]
pub struct FungibleLane {
    pub counterparty_beneficiary: Bytes,
    pub escrowed_denom: String,
    pub is_cw20: bool,
}

pub enum FungibleCounterparty {}
impl Store for FungibleCounterparty {
    const PREFIX: Prefix = Prefix::new(b"fungible_counterparty");

    type Key = (U256, ChannelId, Bytes);
    type Value = FungibleLane;
}
impl KeyCodecViaEncoding for FungibleCounterparty {
    type Encoding = Bincode;
}
impl ValueCodecViaEncoding for FungibleCounterparty {
    type Encoding = Bincode;
}

pub enum Admin {}
impl Store for Admin {
    const PREFIX: Prefix = Prefix::new(b"admin");

    type Key = ();
    type Value = Addr;
}
impl ValueCodecViaEncoding for Admin {
    type Encoding = RawAddrEncoding;
}

pub enum Zkgm {}
impl Store for Zkgm {
    const PREFIX: Prefix = Prefix::new(b"zkgm");

    type Key = ();
    type Value = Addr;
}
impl ValueCodecViaEncoding for Zkgm {
    type Encoding = RawAddrEncoding;
}
