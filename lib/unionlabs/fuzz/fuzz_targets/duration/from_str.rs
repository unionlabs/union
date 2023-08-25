#![no_main]

use libfuzzer_sys::fuzz_target;
use unionlabs::ibc::google::protobuf::duration::Duration;

fuzz_target!(|data: &str| {
    let _ = data.parse::<Duration>();
});
