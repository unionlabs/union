use depolama::{Prefix, Store, value::ValueCodecViaEncoding};
use unionlabs_encoding::Bincode;

pub enum CwAccountCodeId {}
impl Store for CwAccountCodeId {
    const PREFIX: Prefix = Prefix::new(b"cw_account_code_id");
    type Key = ();
    type Value = u64;
}
impl ValueCodecViaEncoding for CwAccountCodeId {
    type Encoding = Bincode;
}

pub enum BytecodeBaseCodeId {}
impl Store for BytecodeBaseCodeId {
    const PREFIX: Prefix = Prefix::new(b"bytecode_base_code_id");
    type Key = ();
    type Value = u64;
}
impl ValueCodecViaEncoding for BytecodeBaseCodeId {
    type Encoding = Bincode;
}
