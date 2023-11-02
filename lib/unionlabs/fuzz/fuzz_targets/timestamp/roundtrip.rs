#![no_main]

use libfuzzer_sys::fuzz_target;
use unionlabs::{google::protobuf::timestamp::Timestamp, test_utils::*};

fuzz_target!(|data: Timestamp| {
    assert_proto_roundtrip(&data);
    assert_json_roundtrip(&data);
    assert_string_roundtrip(&data);
});
