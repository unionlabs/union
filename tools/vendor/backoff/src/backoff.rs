use std::time::Duration;

/// `Backoff` is a backoff policy for retrying an operation.
pub trait Backoff {
    /// Resets the internal state to the initial value.
    fn reset(&mut self) {}
    /// next_backoff() time is elapsed before it is called again.
    /// If it returns None, it means the operation timed out and no
    /// further retries are done.
    fn next_backoff(&mut self) -> Option<Duration>;
}

impl<B: Backoff + ?Sized> Backoff for Box<B> {
    fn next_backoff(&mut self) -> Option<Duration> {
        let this: &mut B = self;
        this.next_backoff()
    }

    fn reset(&mut self) {
        let this: &mut B = self;
        this.reset()
    }
}

/// Immediately retry the operation.
#[derive(Debug)]
pub struct Zero {}

impl Backoff for Zero {
    fn next_backoff(&mut self) -> Option<Duration> {
        Some(Duration::default())
    }
}

/// The operation should never be retried.
#[derive(Debug)]
pub struct Stop {}

impl Backoff for Stop {
    fn next_backoff(&mut self) -> Option<Duration> {
        None
    }
}

/// Contant is a backoff policy which always returns
/// a constant duration.
#[derive(Debug)]
pub struct Constant {
    interval: Duration,
}

impl Constant {
    /// Creates a new Constant backoff with `interval` contant
    /// backoff.
    pub fn new(interval: Duration) -> Constant {
        Constant { interval }
    }
}

impl Backoff for Constant {
    fn next_backoff(&mut self) -> Option<Duration> {
        Some(self.interval)
    }
}
