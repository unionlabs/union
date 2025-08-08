use cosmwasm_std::{Addr, StdError, StdResult};
use depolama::{value::ValueCodecViaEncoding, KeyCodec, Prefix, Store, ValueCodec};
use ibc_union_spec::ChannelId;
use unionlabs::{
    encoding::Bincode,
    primitives::{Bytes, H256},
};

pub enum IntentWhitelist {}
impl Store for IntentWhitelist {
    const PREFIX: Prefix = Prefix::new(b"intent_whitelist");

    type Key = H256;
    type Value = bool;
}
impl KeyCodec<H256> for IntentWhitelist {
    fn encode_key(key: &H256) -> Bytes {
        key.into_bytes()
    }

    fn decode_key(raw: &Bytes) -> StdResult<H256> {
        Ok(H256::new(raw.as_ref().try_into().expect("impossible")))
    }
}
impl ValueCodecViaEncoding for IntentWhitelist {
    type Encoding = Bincode;
}

pub type BaseToken = Bytes;

#[derive(bincode::Encode, bincode::Decode)]
pub struct FungibleLane {
    pub counterparty_beneficiary: Bytes,
    pub escrowed_denom: String,
    pub is_cw20: bool,
}

pub enum FungibleCounterparty {}
impl Store for FungibleCounterparty {
    const PREFIX: Prefix = Prefix::new(b"fungible_counterparty");

    type Key = (ChannelId, BaseToken);
    type Value = FungibleLane;
}

impl KeyCodec<(ChannelId, BaseToken)> for FungibleCounterparty {
    fn encode_key((channel_id, base_token): &(ChannelId, BaseToken)) -> Bytes {
        channel_id
            .raw()
            .to_be_bytes()
            .as_ref()
            .iter()
            .chain(base_token.as_ref())
            .collect()
    }

    fn decode_key(raw: &Bytes) -> StdResult<(ChannelId, BaseToken)> {
        let (raw_channel_id, base_token) = raw.split_at(4);
        let channel_id = ChannelId::new(
            u32::from_be_bytes(raw_channel_id.try_into().expect("impossible"))
                .try_into()
                .expect("impossible"),
        );
        Ok((channel_id, base_token.into()))
    }
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

impl ValueCodec<Addr> for Admin {
    fn encode_value(value: &Addr) -> Bytes {
        value.as_bytes().into()
    }

    fn decode_value(raw: &Bytes) -> StdResult<Addr> {
        String::from_utf8(raw.to_vec())
            .map(Addr::unchecked)
            .map_err(|e| StdError::generic_err(format!("invalid value: {e}")))
    }
}

pub enum Zkgm {}
impl Store for Zkgm {
    const PREFIX: Prefix = Prefix::new(b"zkgm");

    type Key = ();
    type Value = Addr;
}

impl ValueCodec<Addr> for Zkgm {
    fn encode_value(value: &Addr) -> Bytes {
        value.as_bytes().into()
    }

    fn decode_value(raw: &Bytes) -> StdResult<Addr> {
        String::from_utf8(raw.to_vec())
            .map(Addr::unchecked)
            .map_err(|e| StdError::generic_err(format!("invalid value: {e}")))
    }
}
