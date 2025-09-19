use cosmwasm_std::Addr;
use depolama::{
    key::KeyCodecViaEncoding, value::ValueCodecViaEncoding, Prefix, RawAddrEncoding, Store,
};
use ibc_union_spec::ChannelId;
use unionlabs_encoding::Bincode;
use unionlabs_primitives::{Bytes, H256, U256};

#[derive(bincode::Encode, bincode::Decode)]
pub enum Cw20ImplType {
    Base,
    Tokenfactory,
}

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

pub enum Cw20Type {}
impl Store for Cw20Type {
    const PREFIX: Prefix = Prefix::new(b"cw20_type");

    type Key = ();
    type Value = Cw20ImplType;
}
impl ValueCodecViaEncoding for Cw20Type {
    type Encoding = Bincode;
}

pub enum Minters {}

impl Store for Minters {
    const PREFIX: Prefix = Prefix::new(b"minters");
    type Key = ();
    type Value = Vec<String>;
}

impl ValueCodecViaEncoding for Minters {
    type Encoding = Bincode;
}
