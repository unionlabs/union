use depolama::{
    Prefix, Store,
    value::{ValueCodecViaEncoding, ValueUnitEncoding},
};
use unionlabs_encoding::Bincode;

pub enum Counter {}
impl Store for Counter {
    const PREFIX: Prefix = Prefix::new(b"counter");
    type Key = ();
    type Value = u32;
}
impl ValueCodecViaEncoding for Counter {
    type Encoding = Bincode;
}

pub enum IncrementInReplyValue {}
impl Store for IncrementInReplyValue {
    const PREFIX: Prefix = Prefix::new(b"increment_in_reply_value");
    type Key = ();
    type Value = u32;
}
impl ValueCodecViaEncoding for IncrementInReplyValue {
    type Encoding = Bincode;
}

pub enum Executing {}
impl Store for Executing {
    const PREFIX: Prefix = Prefix::new(b"executing");
    type Key = ();
    type Value = ();
}
impl ValueCodecViaEncoding for Executing {
    type Encoding = ValueUnitEncoding;
}
