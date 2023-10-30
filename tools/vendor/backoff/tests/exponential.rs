extern crate backoff;
extern crate instant;

use backoff::backoff::Backoff;
use backoff::exponential::ExponentialBackoff;
use backoff::{Clock, SystemClock};

use instant::Instant;
use std::cell::RefCell;
use std::time::Duration;

struct Inner {
    i: Duration,
    start: Instant,
}

struct TestClock(RefCell<Inner>);

impl TestClock {
    fn new(i: Duration, start: Instant) -> TestClock {
        TestClock(RefCell::new(Inner { i: i, start: start }))
    }
}

impl Clock for TestClock {
    fn now(&self) -> Instant {
        let mut inner = self.0.borrow_mut();
        let t = inner.start + inner.i;
        inner.i += Duration::from_secs(1);
        t
    }
}

impl Default for TestClock {
    fn default() -> Self {
        TestClock::new(Duration::from_secs(1), Instant::now())
    }
}

#[test]
fn get_elapsed_time() {
    let mut exp = ExponentialBackoff::default();
    exp.clock = TestClock::new(Duration::new(0, 0), Instant::now());
    exp.reset();

    let elapsed_time = exp.get_elapsed_time();
    assert_eq!(elapsed_time, Duration::new(1, 0));
}

#[test]
fn max_elapsed_time() {
    let mut exp = ExponentialBackoff::default();
    exp.clock = TestClock::new(Duration::new(0, 0), Instant::now());
    // Change the currentElapsedTime to be 0 ensuring that the elapsed time will be greater
    // than the max elapsed time.
    exp.start_time = Instant::now() - Duration::new(1000, 0);
    assert!(exp.next_backoff().is_none());

    // https://github.com/ihrwein/backoff/issues/42
    // This sets up an ExponentialBackoff that has been retrying for 900ms,
    // will next try for >= 500ms, which would put it after the 1000ms
    // max_elapsed_time.
    let clock = SystemClock::default();
    let mut exp = ExponentialBackoff {
        max_elapsed_time: Some(Duration::from_secs(1)),
        current_interval: Duration::from_millis(500),
        start_time: clock.now() - Duration::from_millis(900),
        clock,
        ..ExponentialBackoff::default()
    };
    assert_eq!(
        None,
        exp.next_backoff(),
        "Should not provide a backoff interval if it would be beyond max_elapsed_time"
    );
}

#[test]
fn backoff() {
    let mut exp = ExponentialBackoff::<SystemClock>::default();
    exp.initial_interval = Duration::from_millis(500);
    exp.randomization_factor = 0.1;
    exp.multiplier = 2.0;
    exp.max_interval = Duration::from_secs(5);
    exp.max_elapsed_time = Some(Duration::new(16 * 60, 0));
    exp.reset();

    let expected_results_millis = [500, 1000, 2000, 4000, 5000, 5000, 5000, 5000, 5000, 5000];
    let expected_results = expected_results_millis
        .iter()
        .map(|&ms| Duration::from_millis(ms))
        .collect::<Vec<_>>();

    for i in expected_results {
        assert_eq!(i, exp.current_interval);
        exp.next_backoff();
    }
}
