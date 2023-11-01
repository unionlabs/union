#![no_main]

use libfuzzer_sys::fuzz_target;
use unionlabs::google::protobuf::duration::Duration;

fuzz_target!(|data: (Duration, Duration)| {
    let (a, b) = data;

    // ensure commutativity
    assert_eq!(a.checked_add(b), b.checked_add(a));

    assert_eq!(a.checked_add(b), (-b).checked_add(-a).map(|d| -d));

    assert_eq!(
        (-a).checked_add(-b).map(|d| -d),
        (-b).checked_add(-a).map(|d| -d)
    );

    assert_eq!((-a).checked_add(-b).map(|d| -d), b.checked_add(a));
});
