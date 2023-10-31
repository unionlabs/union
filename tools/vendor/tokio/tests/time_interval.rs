#![warn(rust_2018_idioms)]
#![cfg(feature = "full")]

use tokio::time::{self, Duration, Instant, MissedTickBehavior};
use tokio_test::{assert_pending, assert_ready_eq, task};

use std::task::Poll;

// Takes the `Interval` task, `start` variable, and optional time deltas
// For each time delta, it polls the `Interval` and asserts that the result is
// equal to `start` + the specific time delta. Then it asserts that the
// `Interval` is pending.
macro_rules! check_interval_poll {
    ($i:ident, $start:ident, $($delta:expr),*$(,)?) => {
        $(
            assert_ready_eq!(poll_next(&mut $i), $start + ms($delta));
        )*
        assert_pending!(poll_next(&mut $i));
    };
    ($i:ident, $start:ident) => {
        check_interval_poll!($i, $start,);
    };
}

#[tokio::test]
#[should_panic]
async fn interval_zero_duration() {
    let _ = time::interval_at(Instant::now(), ms(0));
}

// Expected ticks: |     1     |     2     |     3     |     4     |     5     |     6     |
// Actual ticks:   | work -----|          delay          | work | work | work -| work -----|
// Poll behavior:  |   |       |                         |      |      |       |           |
//                 |   |       |                         |      |      |       |           |
//          Ready(s)   |       |             Ready(s + 2p)      |      |       |           |
//               Pending       |                    Ready(s + 3p)      |       |           |
//                  Ready(s + p)                           Ready(s + 4p)       |           |
//                                                                 Ready(s + 5p)           |
//                                                                             Ready(s + 6p)
#[tokio::test(start_paused = true)]
async fn burst() {
    let start = Instant::now();

    // This is necessary because the timer is only so granular, and in order for
    // all our ticks to resolve, the time needs to be 1ms ahead of what we
    // expect, so that the runtime will see that it is time to resolve the timer
    time::advance(ms(1)).await;

    let mut i = task::spawn(time::interval_at(start, ms(300)));

    check_interval_poll!(i, start, 0);

    time::advance(ms(100)).await;
    check_interval_poll!(i, start);

    time::advance(ms(200)).await;
    check_interval_poll!(i, start, 300);

    time::advance(ms(650)).await;
    check_interval_poll!(i, start, 600, 900);

    time::advance(ms(200)).await;
    check_interval_poll!(i, start);

    time::advance(ms(100)).await;
    check_interval_poll!(i, start, 1200);

    time::advance(ms(250)).await;
    check_interval_poll!(i, start, 1500);

    time::advance(ms(300)).await;
    check_interval_poll!(i, start, 1800);
}

// Expected ticks: |     1     |     2     |     3     |     4     |     5     |     6     |
// Actual ticks:   | work -----|          delay          | work -----| work -----| work -----|
// Poll behavior:  |   |       |                         |   |       |           |           |
//                 |   |       |                         |   |       |           |           |
//          Ready(s)   |       |             Ready(s + 2p)   |       |           |           |
//               Pending       |                       Pending       |           |           |
//                  Ready(s + p)                     Ready(s + 2p + d)           |           |
//                                                               Ready(s + 3p + d)           |
//                                                                           Ready(s + 4p + d)
#[tokio::test(start_paused = true)]
async fn delay() {
    let start = Instant::now();

    // This is necessary because the timer is only so granular, and in order for
    // all our ticks to resolve, the time needs to be 1ms ahead of what we
    // expect, so that the runtime will see that it is time to resolve the timer
    time::advance(ms(1)).await;

    let mut i = task::spawn(time::interval_at(start, ms(300)));
    i.set_missed_tick_behavior(MissedTickBehavior::Delay);

    check_interval_poll!(i, start, 0);

    time::advance(ms(100)).await;
    check_interval_poll!(i, start);

    time::advance(ms(200)).await;
    check_interval_poll!(i, start, 300);

    time::advance(ms(650)).await;
    check_interval_poll!(i, start, 600);

    time::advance(ms(100)).await;
    check_interval_poll!(i, start);

    // We have to add one here for the same reason as is above.
    // Because `Interval` has reset its timer according to `Instant::now()`,
    // we have to go forward 1 more millisecond than is expected so that the
    // runtime realizes that it's time to resolve the timer.
    time::advance(ms(201)).await;
    // We add one because when using the `Delay` behavior, `Interval`
    // adds the `period` from `Instant::now()`, which will always be off by one
    // because we have to advance time by 1 (see above).
    check_interval_poll!(i, start, 1251);

    time::advance(ms(300)).await;
    // Again, we add one.
    check_interval_poll!(i, start, 1551);

    time::advance(ms(300)).await;
    check_interval_poll!(i, start, 1851);
}

// Expected ticks: |     1     |     2     |     3     |     4     |     5     |     6     |
// Actual ticks:   | work -----|          delay          | work ---| work -----| work -----|
// Poll behavior:  |   |       |                         |         |           |           |
//                 |   |       |                         |         |           |           |
//          Ready(s)   |       |             Ready(s + 2p)         |           |           |
//               Pending       |                       Ready(s + 4p)           |           |
//                  Ready(s + p)                                   Ready(s + 5p)           |
//                                                                             Ready(s + 6p)
#[tokio::test(start_paused = true)]
async fn skip() {
    let start = Instant::now();

    // This is necessary because the timer is only so granular, and in order for
    // all our ticks to resolve, the time needs to be 1ms ahead of what we
    // expect, so that the runtime will see that it is time to resolve the timer
    time::advance(ms(1)).await;

    let mut i = task::spawn(time::interval_at(start, ms(300)));
    i.set_missed_tick_behavior(MissedTickBehavior::Skip);

    check_interval_poll!(i, start, 0);

    time::advance(ms(100)).await;
    check_interval_poll!(i, start);

    time::advance(ms(200)).await;
    check_interval_poll!(i, start, 300);

    time::advance(ms(650)).await;
    check_interval_poll!(i, start, 600);

    time::advance(ms(250)).await;
    check_interval_poll!(i, start, 1200);

    time::advance(ms(300)).await;
    check_interval_poll!(i, start, 1500);

    time::advance(ms(300)).await;
    check_interval_poll!(i, start, 1800);
}

#[tokio::test(start_paused = true)]
async fn reset() {
    let start = Instant::now();

    // This is necessary because the timer is only so granular, and in order for
    // all our ticks to resolve, the time needs to be 1ms ahead of what we
    // expect, so that the runtime will see that it is time to resolve the timer
    time::advance(ms(1)).await;

    let mut i = task::spawn(time::interval_at(start, ms(300)));

    check_interval_poll!(i, start, 0);

    time::advance(ms(100)).await;
    check_interval_poll!(i, start);

    time::advance(ms(200)).await;
    check_interval_poll!(i, start, 300);

    time::advance(ms(100)).await;
    check_interval_poll!(i, start);

    i.reset();

    time::advance(ms(250)).await;
    check_interval_poll!(i, start);

    time::advance(ms(50)).await;
    // We add one because when using `reset` method, `Interval` adds the
    // `period` from `Instant::now()`, which will always be off by one
    check_interval_poll!(i, start, 701);

    time::advance(ms(300)).await;
    check_interval_poll!(i, start, 1001);
}

fn poll_next(interval: &mut task::Spawn<time::Interval>) -> Poll<Instant> {
    interval.enter(|cx, mut interval| interval.poll_tick(cx))
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}
