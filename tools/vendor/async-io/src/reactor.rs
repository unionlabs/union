use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::fmt;
use std::future::Future;
use std::io;
use std::marker::PhantomData;
use std::mem;
#[cfg(unix)]
use std::os::unix::io::RawFd;
#[cfg(windows)]
use std::os::windows::io::RawSocket;
use std::panic;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

use async_lock::OnceCell;
use concurrent_queue::ConcurrentQueue;
use futures_lite::ready;
use polling::{Event, Poller};
use slab::Slab;

const READ: usize = 0;
const WRITE: usize = 1;

/// The reactor.
///
/// There is only one global instance of this type, accessible by [`Reactor::get()`].
pub(crate) struct Reactor {
    /// Portable bindings to epoll/kqueue/event ports/wepoll.
    ///
    /// This is where I/O is polled, producing I/O events.
    poller: Poller,

    /// Ticker bumped before polling.
    ///
    /// This is useful for checking what is the current "round" of `ReactorLock::react()` when
    /// synchronizing things in `Source::readable()` and `Source::writable()`. Both of those
    /// methods must make sure they don't receive stale I/O events - they only accept events from a
    /// fresh "round" of `ReactorLock::react()`.
    ticker: AtomicUsize,

    /// Registered sources.
    sources: Mutex<Slab<Arc<Source>>>,

    /// Temporary storage for I/O events when polling the reactor.
    ///
    /// Holding a lock on this event list implies the exclusive right to poll I/O.
    events: Mutex<Vec<Event>>,

    /// An ordered map of registered timers.
    ///
    /// Timers are in the order in which they fire. The `usize` in this type is a timer ID used to
    /// distinguish timers that fire at the same time. The `Waker` represents the task awaiting the
    /// timer.
    timers: Mutex<BTreeMap<(Instant, usize), Waker>>,

    /// A queue of timer operations (insert and remove).
    ///
    /// When inserting or removing a timer, we don't process it immediately - we just push it into
    /// this queue. Timers actually get processed when the queue fills up or the reactor is polled.
    timer_ops: ConcurrentQueue<TimerOp>,
}

impl Reactor {
    /// Returns a reference to the reactor.
    pub(crate) fn get() -> &'static Reactor {
        static REACTOR: OnceCell<Reactor> = OnceCell::new();

        REACTOR.get_or_init_blocking(|| {
            crate::driver::init();
            Reactor {
                poller: Poller::new().expect("cannot initialize I/O event notification"),
                ticker: AtomicUsize::new(0),
                sources: Mutex::new(Slab::new()),
                events: Mutex::new(Vec::new()),
                timers: Mutex::new(BTreeMap::new()),
                timer_ops: ConcurrentQueue::bounded(1000),
            }
        })
    }

    /// Returns the current ticker.
    pub(crate) fn ticker(&self) -> usize {
        self.ticker.load(Ordering::SeqCst)
    }

    /// Registers an I/O source in the reactor.
    pub(crate) fn insert_io(
        &self,
        #[cfg(unix)] raw: RawFd,
        #[cfg(windows)] raw: RawSocket,
    ) -> io::Result<Arc<Source>> {
        // Create an I/O source for this file descriptor.
        let source = {
            let mut sources = self.sources.lock().unwrap();
            let key = sources.vacant_entry().key();
            let source = Arc::new(Source {
                raw,
                key,
                state: Default::default(),
            });
            sources.insert(source.clone());
            source
        };

        // Register the file descriptor.
        if let Err(err) = self.poller.add(raw, Event::none(source.key)) {
            let mut sources = self.sources.lock().unwrap();
            sources.remove(source.key);
            return Err(err);
        }

        Ok(source)
    }

    /// Deregisters an I/O source from the reactor.
    pub(crate) fn remove_io(&self, source: &Source) -> io::Result<()> {
        let mut sources = self.sources.lock().unwrap();
        sources.remove(source.key);
        self.poller.delete(source.raw)
    }

    /// Registers a timer in the reactor.
    ///
    /// Returns the inserted timer's ID.
    pub(crate) fn insert_timer(&self, when: Instant, waker: &Waker) -> usize {
        // Generate a new timer ID.
        static ID_GENERATOR: AtomicUsize = AtomicUsize::new(1);
        let id = ID_GENERATOR.fetch_add(1, Ordering::Relaxed);

        // Push an insert operation.
        while self
            .timer_ops
            .push(TimerOp::Insert(when, id, waker.clone()))
            .is_err()
        {
            // If the queue is full, drain it and try again.
            let mut timers = self.timers.lock().unwrap();
            self.process_timer_ops(&mut timers);
        }

        // Notify that a timer has been inserted.
        self.notify();

        id
    }

    /// Deregisters a timer from the reactor.
    pub(crate) fn remove_timer(&self, when: Instant, id: usize) {
        // Push a remove operation.
        while self.timer_ops.push(TimerOp::Remove(when, id)).is_err() {
            // If the queue is full, drain it and try again.
            let mut timers = self.timers.lock().unwrap();
            self.process_timer_ops(&mut timers);
        }
    }

    /// Notifies the thread blocked on the reactor.
    pub(crate) fn notify(&self) {
        self.poller.notify().expect("failed to notify reactor");
    }

    /// Locks the reactor, potentially blocking if the lock is held by another thread.
    pub(crate) fn lock(&self) -> ReactorLock<'_> {
        let reactor = self;
        let events = self.events.lock().unwrap();
        ReactorLock { reactor, events }
    }

    /// Attempts to lock the reactor.
    pub(crate) fn try_lock(&self) -> Option<ReactorLock<'_>> {
        self.events.try_lock().ok().map(|events| {
            let reactor = self;
            ReactorLock { reactor, events }
        })
    }

    /// Processes ready timers and extends the list of wakers to wake.
    ///
    /// Returns the duration until the next timer before this method was called.
    fn process_timers(&self, wakers: &mut Vec<Waker>) -> Option<Duration> {
        let mut timers = self.timers.lock().unwrap();
        self.process_timer_ops(&mut timers);

        let now = Instant::now();

        // Split timers into ready and pending timers.
        //
        // Careful to split just *after* `now`, so that a timer set for exactly `now` is considered
        // ready.
        let pending = timers.split_off(&(now + Duration::from_nanos(1), 0));
        let ready = mem::replace(&mut *timers, pending);

        // Calculate the duration until the next event.
        let dur = if ready.is_empty() {
            // Duration until the next timer.
            timers
                .keys()
                .next()
                .map(|(when, _)| when.saturating_duration_since(now))
        } else {
            // Timers are about to fire right now.
            Some(Duration::from_secs(0))
        };

        // Drop the lock before waking.
        drop(timers);

        // Add wakers to the list.
        log::trace!("process_timers: {} ready wakers", ready.len());
        for (_, waker) in ready {
            wakers.push(waker);
        }

        dur
    }

    /// Processes queued timer operations.
    fn process_timer_ops(&self, timers: &mut MutexGuard<'_, BTreeMap<(Instant, usize), Waker>>) {
        // Process only as much as fits into the queue, or else this loop could in theory run
        // forever.
        for _ in 0..self.timer_ops.capacity().unwrap() {
            match self.timer_ops.pop() {
                Ok(TimerOp::Insert(when, id, waker)) => {
                    timers.insert((when, id), waker);
                }
                Ok(TimerOp::Remove(when, id)) => {
                    timers.remove(&(when, id));
                }
                Err(_) => break,
            }
        }
    }
}

/// A lock on the reactor.
pub(crate) struct ReactorLock<'a> {
    reactor: &'a Reactor,
    events: MutexGuard<'a, Vec<Event>>,
}

impl ReactorLock<'_> {
    /// Processes new events, blocking until the first event or the timeout.
    pub(crate) fn react(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        let mut wakers = Vec::new();

        // Process ready timers.
        let next_timer = self.reactor.process_timers(&mut wakers);

        // compute the timeout for blocking on I/O events.
        let timeout = match (next_timer, timeout) {
            (None, None) => None,
            (Some(t), None) | (None, Some(t)) => Some(t),
            (Some(a), Some(b)) => Some(a.min(b)),
        };

        // Bump the ticker before polling I/O.
        let tick = self
            .reactor
            .ticker
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1);

        self.events.clear();

        // Block on I/O events.
        let res = match self.reactor.poller.wait(&mut self.events, timeout) {
            // No I/O events occurred.
            Ok(0) => {
                if timeout != Some(Duration::from_secs(0)) {
                    // The non-zero timeout was hit so fire ready timers.
                    self.reactor.process_timers(&mut wakers);
                }
                Ok(())
            }

            // At least one I/O event occurred.
            Ok(_) => {
                // Iterate over sources in the event list.
                let sources = self.reactor.sources.lock().unwrap();

                for ev in self.events.iter() {
                    // Check if there is a source in the table with this key.
                    if let Some(source) = sources.get(ev.key) {
                        let mut state = source.state.lock().unwrap();

                        // Collect wakers if a writability event was emitted.
                        for &(dir, emitted) in &[(WRITE, ev.writable), (READ, ev.readable)] {
                            if emitted {
                                state[dir].tick = tick;
                                state[dir].drain_into(&mut wakers);
                            }
                        }

                        // Re-register if there are still writers or readers. The can happen if
                        // e.g. we were previously interested in both readability and writability,
                        // but only one of them was emitted.
                        if !state[READ].is_empty() || !state[WRITE].is_empty() {
                            self.reactor.poller.modify(
                                source.raw,
                                Event {
                                    key: source.key,
                                    readable: !state[READ].is_empty(),
                                    writable: !state[WRITE].is_empty(),
                                },
                            )?;
                        }
                    }
                }

                Ok(())
            }

            // The syscall was interrupted.
            Err(err) if err.kind() == io::ErrorKind::Interrupted => Ok(()),

            // An actual error occureed.
            Err(err) => Err(err),
        };

        // Wake up ready tasks.
        log::trace!("react: {} ready wakers", wakers.len());
        for waker in wakers {
            // Don't let a panicking waker blow everything up.
            panic::catch_unwind(|| waker.wake()).ok();
        }

        res
    }
}

/// A single timer operation.
enum TimerOp {
    Insert(Instant, usize, Waker),
    Remove(Instant, usize),
}

/// A registered source of I/O events.
#[derive(Debug)]
pub(crate) struct Source {
    /// Raw file descriptor on Unix platforms.
    #[cfg(unix)]
    pub(crate) raw: RawFd,

    /// Raw socket handle on Windows.
    #[cfg(windows)]
    pub(crate) raw: RawSocket,

    /// The key of this source obtained during registration.
    key: usize,

    /// Inner state with registered wakers.
    state: Mutex<[Direction; 2]>,
}

/// A read or write direction.
#[derive(Debug, Default)]
struct Direction {
    /// Last reactor tick that delivered an event.
    tick: usize,

    /// Ticks remembered by `Async::poll_readable()` or `Async::poll_writable()`.
    ticks: Option<(usize, usize)>,

    /// Waker stored by `Async::poll_readable()` or `Async::poll_writable()`.
    waker: Option<Waker>,

    /// Wakers of tasks waiting for the next event.
    ///
    /// Registered by `Async::readable()` and `Async::writable()`.
    wakers: Slab<Option<Waker>>,
}

impl Direction {
    /// Returns `true` if there are no wakers interested in this direction.
    fn is_empty(&self) -> bool {
        self.waker.is_none() && self.wakers.iter().all(|(_, opt)| opt.is_none())
    }

    /// Moves all wakers into a `Vec`.
    fn drain_into(&mut self, dst: &mut Vec<Waker>) {
        if let Some(w) = self.waker.take() {
            dst.push(w);
        }
        for (_, opt) in self.wakers.iter_mut() {
            if let Some(w) = opt.take() {
                dst.push(w);
            }
        }
    }
}

impl Source {
    /// Polls the I/O source for readability.
    pub(crate) fn poll_readable(&self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.poll_ready(READ, cx)
    }

    /// Polls the I/O source for writability.
    pub(crate) fn poll_writable(&self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.poll_ready(WRITE, cx)
    }

    /// Registers a waker from `poll_readable()` or `poll_writable()`.
    ///
    /// If a different waker is already registered, it gets replaced and woken.
    fn poll_ready(&self, dir: usize, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let mut state = self.state.lock().unwrap();

        // Check if the reactor has delivered an event.
        if let Some((a, b)) = state[dir].ticks {
            // If `state[dir].tick` has changed to a value other than the old reactor tick,
            // that means a newer reactor tick has delivered an event.
            if state[dir].tick != a && state[dir].tick != b {
                state[dir].ticks = None;
                return Poll::Ready(Ok(()));
            }
        }

        let was_empty = state[dir].is_empty();

        // Register the current task's waker.
        if let Some(w) = state[dir].waker.take() {
            if w.will_wake(cx.waker()) {
                state[dir].waker = Some(w);
                return Poll::Pending;
            }
            // Wake the previous waker because it's going to get replaced.
            panic::catch_unwind(|| w.wake()).ok();
        }
        state[dir].waker = Some(cx.waker().clone());
        state[dir].ticks = Some((Reactor::get().ticker(), state[dir].tick));

        // Update interest in this I/O handle.
        if was_empty {
            Reactor::get().poller.modify(
                self.raw,
                Event {
                    key: self.key,
                    readable: !state[READ].is_empty(),
                    writable: !state[WRITE].is_empty(),
                },
            )?;
        }

        Poll::Pending
    }

    /// Waits until the I/O source is readable.
    pub(crate) fn readable<T>(handle: &crate::Async<T>) -> Readable<'_, T> {
        Readable(Self::ready(handle, READ))
    }

    /// Waits until the I/O source is readable.
    pub(crate) fn readable_owned<T>(handle: Arc<crate::Async<T>>) -> ReadableOwned<T> {
        ReadableOwned(Self::ready(handle, READ))
    }

    /// Waits until the I/O source is writable.
    pub(crate) fn writable<T>(handle: &crate::Async<T>) -> Writable<'_, T> {
        Writable(Self::ready(handle, WRITE))
    }

    /// Waits until the I/O source is writable.
    pub(crate) fn writable_owned<T>(handle: Arc<crate::Async<T>>) -> WritableOwned<T> {
        WritableOwned(Self::ready(handle, WRITE))
    }

    /// Waits until the I/O source is readable or writable.
    fn ready<H: Borrow<crate::Async<T>> + Clone, T>(handle: H, dir: usize) -> Ready<H, T> {
        Ready {
            handle,
            dir,
            ticks: None,
            index: None,
            _guard: None,
        }
    }
}

/// Future for [`Async::readable`](crate::Async::readable).
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Readable<'a, T>(Ready<&'a crate::Async<T>, T>);

impl<T> Future for Readable<'_, T> {
    type Output = io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        ready!(Pin::new(&mut self.0).poll(cx))?;
        log::trace!("readable: fd={}", self.0.handle.source.raw);
        Poll::Ready(Ok(()))
    }
}

impl<T> fmt::Debug for Readable<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Readable").finish()
    }
}

/// Future for [`Async::readable_owned`](crate::Async::readable_owned).
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ReadableOwned<T>(Ready<Arc<crate::Async<T>>, T>);

impl<T> Future for ReadableOwned<T> {
    type Output = io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        ready!(Pin::new(&mut self.0).poll(cx))?;
        log::trace!("readable_owned: fd={}", self.0.handle.source.raw);
        Poll::Ready(Ok(()))
    }
}

impl<T> fmt::Debug for ReadableOwned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadableOwned").finish()
    }
}

/// Future for [`Async::writable`](crate::Async::writable).
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Writable<'a, T>(Ready<&'a crate::Async<T>, T>);

impl<T> Future for Writable<'_, T> {
    type Output = io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        ready!(Pin::new(&mut self.0).poll(cx))?;
        log::trace!("writable: fd={}", self.0.handle.source.raw);
        Poll::Ready(Ok(()))
    }
}

impl<T> fmt::Debug for Writable<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Writable").finish()
    }
}

/// Future for [`Async::writable_owned`](crate::Async::writable_owned).
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct WritableOwned<T>(Ready<Arc<crate::Async<T>>, T>);

impl<T> Future for WritableOwned<T> {
    type Output = io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        ready!(Pin::new(&mut self.0).poll(cx))?;
        log::trace!("writable_owned: fd={}", self.0.handle.source.raw);
        Poll::Ready(Ok(()))
    }
}

impl<T> fmt::Debug for WritableOwned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WritableOwned").finish()
    }
}

struct Ready<H: Borrow<crate::Async<T>>, T> {
    handle: H,
    dir: usize,
    ticks: Option<(usize, usize)>,
    index: Option<usize>,
    _guard: Option<RemoveOnDrop<H, T>>,
}

impl<H: Borrow<crate::Async<T>>, T> Unpin for Ready<H, T> {}

impl<H: Borrow<crate::Async<T>> + Clone, T> Future for Ready<H, T> {
    type Output = io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self {
            ref handle,
            dir,
            ticks,
            index,
            _guard,
            ..
        } = &mut *self;

        let mut state = handle.borrow().source.state.lock().unwrap();

        // Check if the reactor has delivered an event.
        if let Some((a, b)) = *ticks {
            // If `state[dir].tick` has changed to a value other than the old reactor tick,
            // that means a newer reactor tick has delivered an event.
            if state[*dir].tick != a && state[*dir].tick != b {
                return Poll::Ready(Ok(()));
            }
        }

        let was_empty = state[*dir].is_empty();

        // Register the current task's waker.
        let i = match *index {
            Some(i) => i,
            None => {
                let i = state[*dir].wakers.insert(None);
                *_guard = Some(RemoveOnDrop {
                    handle: handle.clone(),
                    dir: *dir,
                    key: i,
                    _marker: PhantomData,
                });
                *index = Some(i);
                *ticks = Some((Reactor::get().ticker(), state[*dir].tick));
                i
            }
        };
        state[*dir].wakers[i] = Some(cx.waker().clone());

        // Update interest in this I/O handle.
        if was_empty {
            Reactor::get().poller.modify(
                handle.borrow().source.raw,
                Event {
                    key: handle.borrow().source.key,
                    readable: !state[READ].is_empty(),
                    writable: !state[WRITE].is_empty(),
                },
            )?;
        }

        Poll::Pending
    }
}

/// Remove waker when dropped.
struct RemoveOnDrop<H: Borrow<crate::Async<T>>, T> {
    handle: H,
    dir: usize,
    key: usize,
    _marker: PhantomData<fn() -> T>,
}

impl<H: Borrow<crate::Async<T>>, T> Drop for RemoveOnDrop<H, T> {
    fn drop(&mut self) {
        let mut state = self.handle.borrow().source.state.lock().unwrap();
        let wakers = &mut state[self.dir].wakers;
        if wakers.contains(self.key) {
            wakers.remove(self.key);
        }
    }
}
