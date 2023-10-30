mod test_sleep;

use crate::time::{self, Instant};
use std::time::Duration;

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

#[test]
fn registration_is_send_and_sync() {
    use crate::time::Sleep;

    assert_send::<Sleep>();
    assert_sync::<Sleep>();
}

#[test]
#[should_panic]
fn sleep_is_eager() {
    let when = Instant::now() + Duration::from_millis(100);
    let _ = time::sleep_until(when);
}
