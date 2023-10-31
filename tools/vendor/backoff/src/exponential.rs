use instant::Instant;
use std::marker::PhantomData;
use std::time::Duration;

use crate::backoff::Backoff;
use crate::clock::Clock;
use crate::default;

#[derive(Debug)]
pub struct ExponentialBackoff<C> {
    /// The current retry interval.
    pub current_interval: Duration,
    /// The initial retry interval.
    pub initial_interval: Duration,
    /// The randomization factor to use for creating a range around the retry interval.
    ///
    /// A randomization factor of 0.5 results in a random period ranging between 50% below and 50%
    /// above the retry interval.
    pub randomization_factor: f64,
    /// The value to multiply the current interval with for each retry attempt.
    pub multiplier: f64,
    /// The maximum value of the back off period. Once the retry interval reaches this
    /// value it stops increasing.
    pub max_interval: Duration,
    /// The system time. It is calculated when an [`ExponentialBackoff`](struct.ExponentialBackoff.html) instance is
    /// created and is reset when [`retry`](../trait.Operation.html#method.retry) is called.
    pub start_time: Instant,
    /// The maximum elapsed time after instantiating [`ExponentialBackfff`](struct.ExponentialBackoff.html) or calling
    /// [`reset`](trait.Backoff.html#method.reset) after which [`next_backoff`](../trait.Backoff.html#method.reset) returns `None`.
    pub max_elapsed_time: Option<Duration>,
    /// The clock used to get the current time.
    pub clock: C,
}

impl<C> Default for ExponentialBackoff<C>
where
    C: Clock + Default,
{
    fn default() -> ExponentialBackoff<C> {
        let mut eb = ExponentialBackoff {
            current_interval: Duration::from_millis(default::INITIAL_INTERVAL_MILLIS),
            initial_interval: Duration::from_millis(default::INITIAL_INTERVAL_MILLIS),
            randomization_factor: default::RANDOMIZATION_FACTOR,
            multiplier: default::MULTIPLIER,
            max_interval: Duration::from_millis(default::MAX_INTERVAL_MILLIS),
            max_elapsed_time: Some(Duration::from_millis(default::MAX_ELAPSED_TIME_MILLIS)),
            clock: C::default(),
            start_time: Instant::now(),
        };
        eb.reset();
        eb
    }
}

impl<C: Clock> ExponentialBackoff<C> {
    /// Returns the elapsed time since start_time.
    pub fn get_elapsed_time(&self) -> Duration {
        self.clock.now().duration_since(self.start_time)
    }

    fn get_random_value_from_interval(
        randomization_factor: f64,
        random: f64,
        current_interval: Duration,
    ) -> Duration {
        let current_interval_nanos = duration_to_nanos(current_interval);

        let delta = randomization_factor * current_interval_nanos;
        let min_interval = current_interval_nanos - delta;
        let max_interval = current_interval_nanos + delta;
        // Get a random value from the range [minInterval, maxInterval].
        // The formula used below has a +1 because if the minInterval is 1 and the maxInterval is 3 then
        // we want a 33% chance for selecting either 1, 2 or 3.
        let diff = max_interval - min_interval;
        let nanos = min_interval + (random * (diff + 1.0));
        nanos_to_duration(nanos)
    }

    fn increment_current_interval(&mut self) -> Duration {
        let current_interval_nanos = duration_to_nanos(self.current_interval);
        let max_interval_nanos = duration_to_nanos(self.max_interval);
        // Check for overflow, if overflow is detected set the current interval to the max interval.
        if current_interval_nanos >= max_interval_nanos / self.multiplier {
            self.max_interval
        } else {
            let nanos = current_interval_nanos * self.multiplier;
            nanos_to_duration(nanos)
        }
    }
}

fn duration_to_nanos(d: Duration) -> f64 {
    d.as_secs() as f64 * 1_000_000_000.0 + f64::from(d.subsec_nanos())
}

fn nanos_to_duration(nanos: f64) -> Duration {
    let secs = nanos / 1_000_000_000.0;
    let nanos = nanos as u64 % 1_000_000_000;
    Duration::new(secs as u64, nanos as u32)
}

impl<C> Backoff for ExponentialBackoff<C>
where
    C: Clock,
{
    fn reset(&mut self) {
        self.current_interval = self.initial_interval;
        self.start_time = self.clock.now();
    }

    fn next_backoff(&mut self) -> Option<Duration> {
        let elapsed_time = self.get_elapsed_time();

        match self.max_elapsed_time {
            Some(v) if elapsed_time > v => None,
            _ => {
                let random = rand::random::<f64>();
                let randomized_interval = Self::get_random_value_from_interval(
                    self.randomization_factor,
                    random,
                    self.current_interval,
                );
                self.current_interval = self.increment_current_interval();

                if let Some(max_elapsed_time) = self.max_elapsed_time {
                    if elapsed_time + randomized_interval <= max_elapsed_time {
                        Some(randomized_interval)
                    } else {
                        None
                    }
                } else {
                    Some(randomized_interval)
                }
            }
        }
    }
}

impl<C> Clone for ExponentialBackoff<C>
where
    C: Clone,
{
    fn clone(&self) -> Self {
        let clock = self.clock.clone();
        ExponentialBackoff { clock, ..*self }
    }
}

/// Builder for [`ExponentialBackoff`](type.ExponentialBackoff.html).
///
/// TODO: Example
#[derive(Debug)]
pub struct ExponentialBackoffBuilder<C> {
    initial_interval: Duration,
    randomization_factor: f64,
    multiplier: f64,
    max_interval: Duration,
    max_elapsed_time: Option<Duration>,
    _clock: PhantomData<C>,
}

impl<C> Default for ExponentialBackoffBuilder<C> {
    fn default() -> Self {
        Self {
            initial_interval: Duration::from_millis(default::INITIAL_INTERVAL_MILLIS),
            randomization_factor: default::RANDOMIZATION_FACTOR,
            multiplier: default::MULTIPLIER,
            max_interval: Duration::from_millis(default::MAX_INTERVAL_MILLIS),
            max_elapsed_time: Some(Duration::from_millis(default::MAX_ELAPSED_TIME_MILLIS)),
            _clock: PhantomData,
        }
    }
}

impl<C> ExponentialBackoffBuilder<C>
where
    C: Clock + Default,
{
    pub fn new() -> Self {
        Default::default()
    }

    /// The initial retry interval.
    pub fn with_initial_interval(&mut self, initial_interval: Duration) -> &mut Self {
        self.initial_interval = initial_interval;
        self
    }

    /// The randomization factor to use for creating a range around the retry interval.
    ///
    /// A randomization factor of 0.5 results in a random period ranging between 50% below and 50%
    /// above the retry interval.
    pub fn with_randomization_factor(&mut self, randomization_factor: f64) -> &mut Self {
        self.randomization_factor = randomization_factor;
        self
    }

    /// The value to multiply the current interval with for each retry attempt.
    pub fn with_multiplier(&mut self, multiplier: f64) -> &mut Self {
        self.multiplier = multiplier;
        self
    }

    /// The maximum value of the back off period. Once the retry interval reaches this
    /// value it stops increasing.
    pub fn with_max_interval(&mut self, max_interval: Duration) -> &mut Self {
        self.max_interval = max_interval;
        self
    }

    /// The maximum elapsed time after instantiating [`ExponentialBackfff`](struct.ExponentialBackoff.html) or calling
    /// [`reset`](trait.Backoff.html#method.reset) after which [`next_backoff`](../trait.Backoff.html#method.reset) returns `None`.
    pub fn with_max_elapsed_time(&mut self, max_elapsed_time: Option<Duration>) -> &mut Self {
        self.max_elapsed_time = max_elapsed_time;
        self
    }

    pub fn build(&self) -> ExponentialBackoff<C> {
        ExponentialBackoff {
            current_interval: self.initial_interval,
            initial_interval: self.initial_interval,
            randomization_factor: self.randomization_factor,
            multiplier: self.multiplier,
            max_interval: self.max_interval,
            max_elapsed_time: self.max_elapsed_time,
            clock: C::default(),
            start_time: Instant::now(),
        }
    }
}

#[cfg(test)]
use crate::clock::SystemClock;

#[test]
fn get_randomized_interval() {
    // 33% chance of being 1.
    let f = ExponentialBackoff::<SystemClock>::get_random_value_from_interval;
    assert_eq!(Duration::new(0, 1), f(0.5, 0.0, Duration::new(0, 2)));
    assert_eq!(Duration::new(0, 1), f(0.5, 0.33, Duration::new(0, 2)));
    // 33% chance of being 2.
    assert_eq!(Duration::new(0, 2), f(0.5, 0.34, Duration::new(0, 2)));
    assert_eq!(Duration::new(0, 2), f(0.5, 0.66, Duration::new(0, 2)));
    // 33% chance of being 3.
    assert_eq!(Duration::new(0, 3), f(0.5, 0.67, Duration::new(0, 2)));
    assert_eq!(Duration::new(0, 3), f(0.5, 0.99, Duration::new(0, 2)));
}

#[test]
fn exponential_backoff_builder() {
    let initial_interval = Duration::from_secs(1);
    let max_interval = Duration::from_secs(2);
    let multiplier = 3.0;
    let randomization_factor = 4.0;
    let backoff: ExponentialBackoff<SystemClock> = ExponentialBackoffBuilder::new()
        .with_initial_interval(initial_interval)
        .with_multiplier(multiplier)
        .with_randomization_factor(randomization_factor)
        .with_max_interval(max_interval)
        .with_max_elapsed_time(None)
        .build();
    assert_eq!(backoff.initial_interval, initial_interval);
    assert_eq!(backoff.current_interval, initial_interval);
    assert_eq!(backoff.multiplier, multiplier);
    assert_eq!(backoff.randomization_factor, randomization_factor);
    assert_eq!(backoff.max_interval, max_interval);
    assert_eq!(backoff.max_elapsed_time, None);
}

#[test]
fn exponential_backoff_default_builder() {
    let backoff: ExponentialBackoff<SystemClock> = ExponentialBackoffBuilder::new().build();
    assert_eq!(
        backoff.initial_interval,
        Duration::from_millis(default::INITIAL_INTERVAL_MILLIS)
    );
    assert_eq!(
        backoff.current_interval,
        Duration::from_millis(default::INITIAL_INTERVAL_MILLIS)
    );
    assert_eq!(backoff.multiplier, default::MULTIPLIER);
    assert_eq!(backoff.randomization_factor, default::RANDOMIZATION_FACTOR);
    assert_eq!(
        backoff.max_interval,
        Duration::from_millis(default::MAX_INTERVAL_MILLIS)
    );
    assert_eq!(
        backoff.max_elapsed_time,
        Some(Duration::from_millis(default::MAX_ELAPSED_TIME_MILLIS))
    );
}
