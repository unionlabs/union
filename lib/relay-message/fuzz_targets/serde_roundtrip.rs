#![no_main]

use libfuzzer_sys::fuzz_target;
use queue_msg::QueueMsg;
use unionlabs::test_utils::*;
use voyager_message::RelayerMsgTypes;

fuzz_target!(|data: QueueMsg<RelayerMsgTypes>| {
    assert_json_roundtrip(&data);
});
