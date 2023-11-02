#![no_main]

use libfuzzer_sys::fuzz_target;
use unionlabs::{google::protobuf::duration::Duration, test_utils::*};

fuzz_target!(|data: Duration| {
    assert_proto_roundtrip(&data);
    assert_json_roundtrip(&data);
    assert_string_roundtrip(&data);
});
