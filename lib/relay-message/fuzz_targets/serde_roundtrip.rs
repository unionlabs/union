#![no_main]

use libfuzzer_sys::fuzz_target;
use queue_msg::Op;
use unionlabs::test_utils::*;
use voyager_message::RelayMessageTypes;

fuzz_target!(|data: Op<RelayMessageTypes>| {
    assert_json_roundtrip(&data);
});
