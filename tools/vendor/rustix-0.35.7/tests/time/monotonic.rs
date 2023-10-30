#[cfg(feature = "thread")]
use rustix::thread::nanosleep;
use rustix::time::{clock_gettime, ClockId, Timespec};

/// Attempt to test that the monotonic clock is monotonic. Time may or may not
/// advance, but it shouldn't regress.
#[test]
fn test_monotonic_clock() {
    let a = clock_gettime(ClockId::Monotonic);
    let b = clock_gettime(ClockId::Monotonic);
    if b.tv_sec == a.tv_sec {
        assert!(b.tv_nsec >= a.tv_nsec);
    } else {
        assert!(b.tv_sec > a.tv_sec);
    }
}

/// With the "thread" feature, we can sleep so that we're guaranteed that time
/// has advanced.
#[cfg(feature = "thread")]
#[test]
fn test_monotonic_clock_with_sleep_1s() {
    let a = clock_gettime(ClockId::Monotonic);
    let _rem = nanosleep(&Timespec {
        tv_sec: 1,
        tv_nsec: 0,
    });
    let b = clock_gettime(ClockId::Monotonic);
    assert!(b.tv_sec > a.tv_sec);
}

/// With the "thread" feature, we can sleep so that we're guaranteed that time
/// has advanced.
#[cfg(feature = "thread")]
#[test]
fn test_monotonic_clock_with_sleep_1ms() {
    let a = clock_gettime(ClockId::Monotonic);
    let _rem = nanosleep(&Timespec {
        tv_sec: 0,
        tv_nsec: 1_000_000,
    });
    let b = clock_gettime(ClockId::Monotonic);
    assert!(b.tv_sec >= a.tv_sec);
    assert!(b.tv_sec != a.tv_sec || b.tv_nsec > a.tv_nsec);
}
