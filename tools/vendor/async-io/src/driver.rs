use std::cell::Cell;
use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread;
use std::time::{Duration, Instant};

use async_lock::OnceCell;
use futures_lite::pin;
use waker_fn::waker_fn;

use crate::reactor::Reactor;

/// Number of currently active `block_on()` invocations.
static BLOCK_ON_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Unparker for the "async-io" thread.
fn unparker() -> &'static parking::Unparker {
    static UNPARKER: OnceCell<parking::Unparker> = OnceCell::new();

    UNPARKER.get_or_init_blocking(|| {
        let (parker, unparker) = parking::pair();

        // Spawn a helper thread driving the reactor.
        //
        // Note that this thread is not exactly necessary, it's only here to help push things
        // forward if there are no `Parker`s around or if `Parker`s are just idling and never
        // parking.
        thread::Builder::new()
            .name("async-io".to_string())
            .spawn(move || main_loop(parker))
            .expect("cannot spawn async-io thread");

        unparker
    })
}

/// Initializes the "async-io" thread.
pub(crate) fn init() {
    let _ = unparker();
}

/// The main loop for the "async-io" thread.
fn main_loop(parker: parking::Parker) {
    // The last observed reactor tick.
    let mut last_tick = 0;
    // Number of sleeps since this thread has called `react()`.
    let mut sleeps = 0u64;

    loop {
        let tick = Reactor::get().ticker();

        if last_tick == tick {
            let reactor_lock = if sleeps >= 10 {
                // If no new ticks have occurred for a while, stop sleeping and spinning in
                // this loop and just block on the reactor lock.
                Some(Reactor::get().lock())
            } else {
                Reactor::get().try_lock()
            };

            if let Some(mut reactor_lock) = reactor_lock {
                log::trace!("main_loop: waiting on I/O");
                reactor_lock.react(None).ok();
                last_tick = Reactor::get().ticker();
                sleeps = 0;
            }
        } else {
            last_tick = tick;
        }

        if BLOCK_ON_COUNT.load(Ordering::SeqCst) > 0 {
            // Exponential backoff from 50us to 10ms.
            let delay_us = [50, 75, 100, 250, 500, 750, 1000, 2500, 5000]
                .get(sleeps as usize)
                .unwrap_or(&10_000);

            log::trace!("main_loop: sleeping for {} us", delay_us);
            if parker.park_timeout(Duration::from_micros(*delay_us)) {
                log::trace!("main_loop: notified");

                // If notified before timeout, reset the last tick and the sleep counter.
                last_tick = Reactor::get().ticker();
                sleeps = 0;
            } else {
                sleeps += 1;
            }
        }
    }
}

/// Blocks the current thread on a future, processing I/O events when idle.
///
/// # Examples
///
/// ```
/// use async_io::Timer;
/// use std::time::Duration;
///
/// async_io::block_on(async {
///     // This timer will likely be processed by the current
///     // thread rather than the fallback "async-io" thread.
///     Timer::after(Duration::from_millis(1)).await;
/// });
/// ```
pub fn block_on<T>(future: impl Future<Output = T>) -> T {
    log::trace!("block_on()");

    // Increment `BLOCK_ON_COUNT` so that the "async-io" thread becomes less aggressive.
    BLOCK_ON_COUNT.fetch_add(1, Ordering::SeqCst);

    // Make sure to decrement `BLOCK_ON_COUNT` at the end and wake the "async-io" thread.
    let _guard = CallOnDrop(|| {
        BLOCK_ON_COUNT.fetch_sub(1, Ordering::SeqCst);
        unparker().unpark();
    });

    // Parker and unparker for notifying the current thread.
    let (p, u) = parking::pair();
    // This boolean is set to `true` when the current thread is blocked on I/O.
    let io_blocked = Arc::new(AtomicBool::new(false));

    thread_local! {
        // Indicates that the current thread is polling I/O, but not necessarily blocked on it.
        static IO_POLLING: Cell<bool> = Cell::new(false);
    }

    // Prepare the waker.
    let waker = waker_fn({
        let io_blocked = io_blocked.clone();
        move || {
            if u.unpark() {
                // Check if waking from another thread and if currently blocked on I/O.
                if !IO_POLLING.with(Cell::get) && io_blocked.load(Ordering::SeqCst) {
                    Reactor::get().notify();
                }
            }
        }
    });
    let cx = &mut Context::from_waker(&waker);
    pin!(future);

    loop {
        // Poll the future.
        if let Poll::Ready(t) = future.as_mut().poll(cx) {
            log::trace!("block_on: completed");
            return t;
        }

        // Check if a notification was received.
        if p.park_timeout(Duration::from_secs(0)) {
            log::trace!("block_on: notified");

            // Try grabbing a lock on the reactor to process I/O events.
            if let Some(mut reactor_lock) = Reactor::get().try_lock() {
                // First let wakers know this parker is processing I/O events.
                IO_POLLING.with(|io| io.set(true));
                let _guard = CallOnDrop(|| {
                    IO_POLLING.with(|io| io.set(false));
                });

                // Process available I/O events.
                reactor_lock.react(Some(Duration::from_secs(0))).ok();
            }
            continue;
        }

        // Try grabbing a lock on the reactor to wait on I/O.
        if let Some(mut reactor_lock) = Reactor::get().try_lock() {
            // Record the instant at which the lock was grabbed.
            let start = Instant::now();

            loop {
                // First let wakers know this parker is blocked on I/O.
                IO_POLLING.with(|io| io.set(true));
                io_blocked.store(true, Ordering::SeqCst);
                let _guard = CallOnDrop(|| {
                    IO_POLLING.with(|io| io.set(false));
                    io_blocked.store(false, Ordering::SeqCst);
                });

                // Check if a notification has been received before `io_blocked` was updated
                // because in that case the reactor won't receive a wakeup.
                if p.park_timeout(Duration::from_secs(0)) {
                    log::trace!("block_on: notified");
                    break;
                }

                // Wait for I/O events.
                log::trace!("block_on: waiting on I/O");
                reactor_lock.react(None).ok();

                // Check if a notification has been received.
                if p.park_timeout(Duration::from_secs(0)) {
                    log::trace!("block_on: notified");
                    break;
                }

                // Check if this thread been handling I/O events for a long time.
                if start.elapsed() > Duration::from_micros(500) {
                    log::trace!("block_on: stops hogging the reactor");

                    // This thread is clearly processing I/O events for some other threads
                    // because it didn't get a notification yet. It's best to stop hogging the
                    // reactor and give other threads a chance to process I/O events for
                    // themselves.
                    drop(reactor_lock);

                    // Unpark the "async-io" thread in case no other thread is ready to start
                    // processing I/O events. This way we prevent a potential latency spike.
                    unparker().unpark();

                    // Wait for a notification.
                    p.park();
                    break;
                }
            }
        } else {
            // Wait for an actual notification.
            log::trace!("block_on: sleep until notification");
            p.park();
        }
    }
}

/// Runs a closure when dropped.
struct CallOnDrop<F: Fn()>(F);

impl<F: Fn()> Drop for CallOnDrop<F> {
    fn drop(&mut self) {
        (self.0)();
    }
}
