use cosmwasm_std::{Addr, StdError, StdResult};
use depolama::{Prefix, Store, ValueCodec};
use unionlabs::primitives::Bytes;

pub enum IbcHost {}
impl Store for IbcHost {
    const PREFIX: Prefix = Prefix::new(b"ibc_host");

    type Key = ();
    type Value = Addr;
}

impl ValueCodec<Addr> for IbcHost {
    fn encode_value(value: &Addr) -> Bytes {
        value.as_bytes().into()
    }

    fn decode_value(raw: &Bytes) -> StdResult<Addr> {
        String::from_utf8(raw.to_vec())
            .map(Addr::unchecked)
            .map_err(|e| StdError::generic_err(format!("invalid value: {e}")))
    }
}
