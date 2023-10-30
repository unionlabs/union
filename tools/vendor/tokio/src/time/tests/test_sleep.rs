//use crate::time::driver::{Driver, Entry, Handle};

/*
macro_rules! poll {
    ($e:expr) => {
        $e.enter(|cx, e| e.poll_elapsed(cx))
    };
}

#[test]
fn frozen_utility_returns_correct_advanced_duration() {
    let clock = Clock::new();
    clock.pause();
    let start = clock.now();

    clock.advance(ms(10));
    assert_eq!(clock.now() - start, ms(10));
}

#[test]
fn immediate_sleep() {
    let (mut driver, clock, handle) = setup();
    let start = clock.now();

    let when = clock.now();
    let mut e = task::spawn(sleep_until(&handle, when));

    assert_ready_ok!(poll!(e));

    assert_ok!(driver.park_timeout(Duration::from_millis(1000)));

    // The time has not advanced. The `turn` completed immediately.
    assert_eq!(clock.now() - start, ms(1000));
}

#[test]
fn delayed_sleep_level_0() {
    let (mut driver, clock, handle) = setup();
    let start = clock.now();

    for &i in &[1, 10, 60] {
        // Create a `Sleep` that elapses in the future
        let mut e = task::spawn(sleep_until(&handle, start + ms(i)));

        // The sleep instance has not elapsed.
        assert_pending!(poll!(e));

        assert_ok!(driver.park());
        assert_eq!(clock.now() - start, ms(i));

        assert_ready_ok!(poll!(e));
    }
}

#[test]
fn sub_ms_delayed_sleep() {
    let (mut driver, clock, handle) = setup();

    for _ in 0..5 {
        let deadline = clock.now() + ms(1) + Duration::new(0, 1);

        let mut e = task::spawn(sleep_until(&handle, deadline));

        assert_pending!(poll!(e));

        assert_ok!(driver.park());
        assert_ready_ok!(poll!(e));

        assert!(clock.now() >= deadline);

        clock.advance(Duration::new(0, 1));
    }
}

#[test]
fn delayed_sleep_wrapping_level_0() {
    let (mut driver, clock, handle) = setup();
    let start = clock.now();

    assert_ok!(driver.park_timeout(ms(5)));
    assert_eq!(clock.now() - start, ms(5));

    let mut e = task::spawn(sleep_until(&handle, clock.now() + ms(60)));

    assert_pending!(poll!(e));

    assert_ok!(driver.park());
    assert_eq!(clock.now() - start, ms(64));
    assert_pending!(poll!(e));

    assert_ok!(driver.park());
    assert_eq!(clock.now() - start, ms(65));

    assert_ready_ok!(poll!(e));
}

#[test]
fn timer_wrapping_with_higher_levels() {
    let (mut driver, clock, handle) = setup();
    let start = clock.now();

    // Set sleep to hit level 1
    let mut e1 = task::spawn(sleep_until(&handle, clock.now() + ms(64)));
    assert_pending!(poll!(e1));

    // Turn a bit
    assert_ok!(driver.park_timeout(ms(5)));

    // Set timeout such that it will hit level 0, but wrap
    let mut e2 = task::spawn(sleep_until(&handle, clock.now() + ms(60)));
    assert_pending!(poll!(e2));

    // This should result in s1 firing
    assert_ok!(driver.park());
    assert_eq!(clock.now() - start, ms(64));

    assert_ready_ok!(poll!(e1));
    assert_pending!(poll!(e2));

    assert_ok!(driver.park());
    assert_eq!(clock.now() - start, ms(65));

    assert_ready_ok!(poll!(e1));
}

#[test]
fn sleep_with_deadline_in_past() {
    let (mut driver, clock, handle) = setup();
    let start = clock.now();

    // Create `Sleep` that elapsed immediately.
    let mut e = task::spawn(sleep_until(&handle, clock.now() - ms(100)));

    // Even though the `Sleep` expires in the past, it is not ready yet
    // because the timer must observe it.
    assert_ready_ok!(poll!(e));

    // Turn the timer, it runs for the elapsed time
    assert_ok!(driver.park_timeout(ms(1000)));

    // The time has not advanced. The `turn` completed immediately.
    assert_eq!(clock.now() - start, ms(1000));
}

#[test]
fn delayed_sleep_level_1() {
    let (mut driver, clock, handle) = setup();
    let start = clock.now();

    // Create a `Sleep` that elapses in the future
    let mut e = task::spawn(sleep_until(&handle, clock.now() + ms(234)));

    // The sleep has not elapsed.
    assert_pending!(poll!(e));

    // Turn the timer, this will wake up to cascade the timer down.
    assert_ok!(driver.park_timeout(ms(1000)));
    assert_eq!(clock.now() - start, ms(192));

    // The sleep has not elapsed.
    assert_pending!(poll!(e));

    // Turn the timer again
    assert_ok!(driver.park_timeout(ms(1000)));
    assert_eq!(clock.now() - start, ms(234));

    // The sleep has elapsed.
    assert_ready_ok!(poll!(e));

    let (mut driver, clock, handle) = setup();
    let start = clock.now();

    // Create a `Sleep` that elapses in the future
    let mut e = task::spawn(sleep_until(&handle, clock.now() + ms(234)));

    // The sleep has not elapsed.
    assert_pending!(poll!(e));

    // Turn the timer with a smaller timeout than the cascade.
    assert_ok!(driver.park_timeout(ms(100)));
    assert_eq!(clock.now() - start, ms(100));

    assert_pending!(poll!(e));

    // Turn the timer, this will wake up to cascade the timer down.
    assert_ok!(driver.park_timeout(ms(1000)));
    assert_eq!(clock.now() - start, ms(192));

    // The sleep has not elapsed.
    assert_pending!(poll!(e));

    // Turn the timer again
    assert_ok!(driver.park_timeout(ms(1000)));
    assert_eq!(clock.now() - start, ms(234));

    // The sleep has elapsed.
    assert_ready_ok!(poll!(e));
}

#[test]
fn concurrently_set_two_timers_second_one_shorter() {
    let (mut driver, clock, handle) = setup();
    let start = clock.now();

    let mut e1 = task::spawn(sleep_until(&handle, clock.now() + ms(500)));
    let mut e2 = task::spawn(sleep_until(&handle, clock.now() + ms(200)));

    // The sleep has not elapsed
    assert_pending!(poll!(e1));
    assert_pending!(poll!(e2));

    // Sleep until a cascade
    assert_ok!(driver.park());
    assert_eq!(clock.now() - start, ms(192));

    // Sleep until the second timer.
    assert_ok!(driver.park());
    assert_eq!(clock.now() - start, ms(200));

    // The shorter sleep fires
    assert_ready_ok!(poll!(e2));
    assert_pending!(poll!(e1));

    assert_ok!(driver.park());
    assert_eq!(clock.now() - start, ms(448));

    assert_pending!(poll!(e1));

    // Turn again, this time the time will advance to the second sleep
    assert_ok!(driver.park());
    assert_eq!(clock.now() - start, ms(500));

    assert_ready_ok!(poll!(e1));
}

#[test]
fn short_sleep() {
    let (mut driver, clock, handle) = setup();
    let start = clock.now();

    // Create a `Sleep` that elapses in the future
    let mut e = task::spawn(sleep_until(&handle, clock.now() + ms(1)));

    // The sleep has not elapsed.
    assert_pending!(poll!(e));

    // Turn the timer, but not enough time will go by.
    assert_ok!(driver.park());

    // The sleep has elapsed.
    assert_ready_ok!(poll!(e));

    // The time has advanced to the point of the sleep elapsing.
    assert_eq!(clock.now() - start, ms(1));
}

#[test]
fn sorta_long_sleep_until() {
    const MIN_5: u64 = 5 * 60 * 1000;

    let (mut driver, clock, handle) = setup();
    let start = clock.now();

    // Create a `Sleep` that elapses in the future
    let mut e = task::spawn(sleep_until(&handle, clock.now() + ms(MIN_5)));

    // The sleep has not elapsed.
    assert_pending!(poll!(e));

    let cascades = &[262_144, 262_144 + 9 * 4096, 262_144 + 9 * 4096 + 15 * 64];

    for &elapsed in cascades {
        assert_ok!(driver.park());
        assert_eq!(clock.now() - start, ms(elapsed));

        assert_pending!(poll!(e));
    }

    assert_ok!(driver.park());
    assert_eq!(clock.now() - start, ms(MIN_5));

    // The sleep has elapsed.
    assert_ready_ok!(poll!(e));
}

#[test]
fn very_long_sleep() {
    const MO_5: u64 = 5 * 30 * 24 * 60 * 60 * 1000;

    let (mut driver, clock, handle) = setup();
    let start = clock.now();

    // Create a `Sleep` that elapses in the future
    let mut e = task::spawn(sleep_until(&handle, clock.now() + ms(MO_5)));

    // The sleep has not elapsed.
    assert_pending!(poll!(e));

    let cascades = &[
        12_884_901_888,
        12_952_010_752,
        12_959_875_072,
        12_959_997_952,
    ];

    for &elapsed in cascades {
        assert_ok!(driver.park());
        assert_eq!(clock.now() - start, ms(elapsed));

        assert_pending!(poll!(e));
    }

    // Turn the timer, but not enough time will go by.
    assert_ok!(driver.park());

    // The time has advanced to the point of the sleep elapsing.
    assert_eq!(clock.now() - start, ms(MO_5));

    // The sleep has elapsed.
    assert_ready_ok!(poll!(e));
}

#[test]
fn unpark_is_delayed() {
    // A special park that will take much longer than the requested duration
    struct MockPark(Clock);

    struct MockUnpark;

    impl Park for MockPark {
        type Unpark = MockUnpark;
        type Error = ();

        fn unpark(&self) -> Self::Unpark {
            MockUnpark
        }

        fn park(&mut self) -> Result<(), Self::Error> {
            panic!("parking forever");
        }

        fn park_timeout(&mut self, duration: Duration) -> Result<(), Self::Error> {
            assert_eq!(duration, ms(0));
            self.0.advance(ms(436));
            Ok(())
        }

        fn shutdown(&mut self) {}
    }

    impl Unpark for MockUnpark {
        fn unpark(&self) {}
    }

    let clock = Clock::new();
    clock.pause();
    let start = clock.now();
    let mut driver = Driver::new(MockPark(clock.clone()), clock.clone());
    let handle = driver.handle();

    let mut e1 = task::spawn(sleep_until(&handle, clock.now() + ms(100)));
    let mut e2 = task::spawn(sleep_until(&handle, clock.now() + ms(101)));
    let mut e3 = task::spawn(sleep_until(&handle, clock.now() + ms(200)));

    assert_pending!(poll!(e1));
    assert_pending!(poll!(e2));
    assert_pending!(poll!(e3));

    assert_ok!(driver.park());

    assert_eq!(clock.now() - start, ms(500));

    assert_ready_ok!(poll!(e1));
    assert_ready_ok!(poll!(e2));
    assert_ready_ok!(poll!(e3));
}

#[test]
fn set_timeout_at_deadline_greater_than_max_timer() {
    const YR_1: u64 = 365 * 24 * 60 * 60 * 1000;
    const YR_5: u64 = 5 * YR_1;

    let (mut driver, clock, handle) = setup();
    let start = clock.now();

    for _ in 0..5 {
        assert_ok!(driver.park_timeout(ms(YR_1)));
    }

    let mut e = task::spawn(sleep_until(&handle, clock.now() + ms(1)));
    assert_pending!(poll!(e));

    assert_ok!(driver.park_timeout(ms(1000)));
    assert_eq!(clock.now() - start, ms(YR_5) + ms(1));

    assert_ready_ok!(poll!(e));
}

fn setup() -> (Driver<MockPark>, Clock, Handle) {
    let clock = Clock::new();
    clock.pause();
    let driver = Driver::new(MockPark(clock.clone()), clock.clone());
    let handle = driver.handle();

    (driver, clock, handle)
}

fn sleep_until(handle: &Handle, when: Instant) -> Arc<Entry> {
    Entry::new(&handle, when, ms(0))
}

struct MockPark(Clock);

struct MockUnpark;

impl Park for MockPark {
    type Unpark = MockUnpark;
    type Error = ();

    fn unpark(&self) -> Self::Unpark {
        MockUnpark
    }

    fn park(&mut self) -> Result<(), Self::Error> {
        panic!("parking forever");
    }

    fn park_timeout(&mut self, duration: Duration) -> Result<(), Self::Error> {
        self.0.advance(duration);
        Ok(())
    }

    fn shutdown(&mut self) {}
}

impl Unpark for MockUnpark {
    fn unpark(&self) {}
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}
*/
