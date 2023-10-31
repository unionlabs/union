//! Timer state structures.
//!
//! This module contains the heart of the intrusive timer implementation, and as
//! such the structures inside are full of tricky concurrency and unsafe code.
//!
//! # Ground rules
//!
//! The heart of the timer implementation here is the [`TimerShared`] structure,
//! shared between the [`TimerEntry`] and the driver. Generally, we permit access
//! to [`TimerShared`] ONLY via either 1) a mutable reference to [`TimerEntry`] or
//! 2) a held driver lock.
//!
//! It follows from this that any changes made while holding BOTH 1 and 2 will
//! be reliably visible, regardless of ordering. This is because of the acq/rel
//! fences on the driver lock ensuring ordering with 2, and rust mutable
//! reference rules for 1 (a mutable reference to an object can't be passed
//! between threads without an acq/rel barrier, and same-thread we have local
//! happens-before ordering).
//!
//! # State field
//!
//! Each timer has a state field associated with it. This field contains either
//! the current scheduled time, or a special flag value indicating its state.
//! This state can either indicate that the timer is on the 'pending' queue (and
//! thus will be fired with an `Ok(())` result soon) or that it has already been
//! fired/deregistered.
//!
//! This single state field allows for code that is firing the timer to
//! synchronize with any racing `reset` calls reliably.
//!
//! # Cached vs true timeouts
//!
//! To allow for the use case of a timeout that is periodically reset before
//! expiration to be as lightweight as possible, we support optimistically
//! lock-free timer resets, in the case where a timer is rescheduled to a later
//! point than it was originally scheduled for.
//!
//! This is accomplished by lazily rescheduling timers. That is, we update the
//! state field field with the true expiration of the timer from the holder of
//! the [`TimerEntry`]. When the driver services timers (ie, whenever it's
//! walking lists of timers), it checks this "true when" value, and reschedules
//! based on it.
//!
//! We do, however, also need to track what the expiration time was when we
//! originally registered the timer; this is used to locate the right linked
//! list when the timer is being cancelled. This is referred to as the "cached
//! when" internally.
//!
//! There is of course a race condition between timer reset and timer
//! expiration. If the driver fails to observe the updated expiration time, it
//! could trigger expiration of the timer too early. However, because
//! [`mark_pending`][mark_pending] performs a compare-and-swap, it will identify this race and
//! refuse to mark the timer as pending.
//!
//! [mark_pending]: TimerHandle::mark_pending

use crate::loom::cell::UnsafeCell;
use crate::loom::sync::atomic::AtomicU64;
use crate::loom::sync::atomic::Ordering;

use crate::sync::AtomicWaker;
use crate::time::Instant;
use crate::util::linked_list;

use super::Handle;

use std::cell::UnsafeCell as StdUnsafeCell;
use std::task::{Context, Poll, Waker};
use std::{marker::PhantomPinned, pin::Pin, ptr::NonNull};

type TimerResult = Result<(), crate::time::error::Error>;

const STATE_DEREGISTERED: u64 = u64::MAX;
const STATE_PENDING_FIRE: u64 = STATE_DEREGISTERED - 1;
const STATE_MIN_VALUE: u64 = STATE_PENDING_FIRE;

/// This structure holds the current shared state of the timer - its scheduled
/// time (if registered), or otherwise the result of the timer completing, as
/// well as the registered waker.
///
/// Generally, the StateCell is only permitted to be accessed from two contexts:
/// Either a thread holding the corresponding &mut TimerEntry, or a thread
/// holding the timer driver lock. The write actions on the StateCell amount to
/// passing "ownership" of the StateCell between these contexts; moving a timer
/// from the TimerEntry to the driver requires _both_ holding the &mut
/// TimerEntry and the driver lock, while moving it back (firing the timer)
/// requires only the driver lock.
pub(super) struct StateCell {
    /// Holds either the scheduled expiration time for this timer, or (if the
    /// timer has been fired and is unregistered), `u64::MAX`.
    state: AtomicU64,
    /// If the timer is fired (an Acquire order read on state shows
    /// `u64::MAX`), holds the result that should be returned from
    /// polling the timer. Otherwise, the contents are unspecified and reading
    /// without holding the driver lock is undefined behavior.
    result: UnsafeCell<TimerResult>,
    /// The currently-registered waker
    waker: CachePadded<AtomicWaker>,
}

impl Default for StateCell {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for StateCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StateCell({:?})", self.read_state())
    }
}

impl StateCell {
    fn new() -> Self {
        Self {
            state: AtomicU64::new(STATE_DEREGISTERED),
            result: UnsafeCell::new(Ok(())),
            waker: CachePadded(AtomicWaker::new()),
        }
    }

    fn is_pending(&self) -> bool {
        self.state.load(Ordering::Relaxed) == STATE_PENDING_FIRE
    }

    /// Returns the current expiration time, or None if not currently scheduled.
    fn when(&self) -> Option<u64> {
        let cur_state = self.state.load(Ordering::Relaxed);

        if cur_state == u64::MAX {
            None
        } else {
            Some(cur_state)
        }
    }

    /// If the timer is completed, returns the result of the timer. Otherwise,
    /// returns None and registers the waker.
    fn poll(&self, waker: &Waker) -> Poll<TimerResult> {
        // We must register first. This ensures that either `fire` will
        // observe the new waker, or we will observe a racing fire to have set
        // the state, or both.
        self.waker.0.register_by_ref(waker);

        self.read_state()
    }

    fn read_state(&self) -> Poll<TimerResult> {
        let cur_state = self.state.load(Ordering::Acquire);

        if cur_state == STATE_DEREGISTERED {
            // SAFETY: The driver has fired this timer; this involves writing
            // the result, and then writing (with release ordering) the state
            // field.
            Poll::Ready(unsafe { self.result.with(|p| *p) })
        } else {
            Poll::Pending
        }
    }

    /// Marks this timer as being moved to the pending list, if its scheduled
    /// time is not after `not_after`.
    ///
    /// If the timer is scheduled for a time after not_after, returns an Err
    /// containing the current scheduled time.
    ///
    /// SAFETY: Must hold the driver lock.
    unsafe fn mark_pending(&self, not_after: u64) -> Result<(), u64> {
        // Quick initial debug check to see if the timer is already fired. Since
        // firing the timer can only happen with the driver lock held, we know
        // we shouldn't be able to "miss" a transition to a fired state, even
        // with relaxed ordering.
        let mut cur_state = self.state.load(Ordering::Relaxed);

        loop {
            debug_assert!(cur_state < STATE_MIN_VALUE);

            if cur_state > not_after {
                break Err(cur_state);
            }

            match self.state.compare_exchange(
                cur_state,
                STATE_PENDING_FIRE,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    break Ok(());
                }
                Err(actual_state) => {
                    cur_state = actual_state;
                }
            }
        }
    }

    /// Fires the timer, setting the result to the provided result.
    ///
    /// Returns:
    /// * `Some(waker) - if fired and a waker needs to be invoked once the
    ///   driver lock is released
    /// * `None` - if fired and a waker does not need to be invoked, or if
    ///   already fired
    ///
    /// SAFETY: The driver lock must be held.
    unsafe fn fire(&self, result: TimerResult) -> Option<Waker> {
        // Quick initial check to see if the timer is already fired. Since
        // firing the timer can only happen with the driver lock held, we know
        // we shouldn't be able to "miss" a transition to a fired state, even
        // with relaxed ordering.
        let cur_state = self.state.load(Ordering::Relaxed);
        if cur_state == STATE_DEREGISTERED {
            return None;
        }

        // SAFETY: We assume the driver lock is held and the timer is not
        // fired, so only the driver is accessing this field.
        //
        // We perform a release-ordered store to state below, to ensure this
        // write is visible before the state update is visible.
        unsafe { self.result.with_mut(|p| *p = result) };

        self.state.store(STATE_DEREGISTERED, Ordering::Release);

        self.waker.0.take_waker()
    }

    /// Marks the timer as registered (poll will return None) and sets the
    /// expiration time.
    ///
    /// While this function is memory-safe, it should only be called from a
    /// context holding both `&mut TimerEntry` and the driver lock.
    fn set_expiration(&self, timestamp: u64) {
        debug_assert!(timestamp < STATE_MIN_VALUE);

        // We can use relaxed ordering because we hold the driver lock and will
        // fence when we release the lock.
        self.state.store(timestamp, Ordering::Relaxed);
    }

    /// Attempts to adjust the timer to a new timestamp.
    ///
    /// If the timer has already been fired, is pending firing, or the new
    /// timestamp is earlier than the old timestamp, (or occasionally
    /// spuriously) returns Err without changing the timer's state. In this
    /// case, the timer must be deregistered and re-registered.
    fn extend_expiration(&self, new_timestamp: u64) -> Result<(), ()> {
        let mut prior = self.state.load(Ordering::Relaxed);
        loop {
            if new_timestamp < prior || prior >= STATE_MIN_VALUE {
                return Err(());
            }

            match self.state.compare_exchange_weak(
                prior,
                new_timestamp,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    return Ok(());
                }
                Err(true_prior) => {
                    prior = true_prior;
                }
            }
        }
    }

    /// Returns true if the state of this timer indicates that the timer might
    /// be registered with the driver. This check is performed with relaxed
    /// ordering, but is conservative - if it returns false, the timer is
    /// definitely _not_ registered.
    pub(super) fn might_be_registered(&self) -> bool {
        self.state.load(Ordering::Relaxed) != u64::MAX
    }
}

/// A timer entry.
///
/// This is the handle to a timer that is controlled by the requester of the
/// timer. As this participates in intrusive data structures, it must be pinned
/// before polling.
#[derive(Debug)]
pub(super) struct TimerEntry {
    /// Arc reference to the driver. We can only free the driver after
    /// deregistering everything from their respective timer wheels.
    driver: Handle,
    /// Shared inner structure; this is part of an intrusive linked list, and
    /// therefore other references can exist to it while mutable references to
    /// Entry exist.
    ///
    /// This is manipulated only under the inner mutex. TODO: Can we use loom
    /// cells for this?
    inner: StdUnsafeCell<TimerShared>,
    /// Initial deadline for the timer. This is used to register on the first
    /// poll, as we can't register prior to being pinned.
    initial_deadline: Option<Instant>,
    /// Ensure the type is !Unpin
    _m: std::marker::PhantomPinned,
}

unsafe impl Send for TimerEntry {}
unsafe impl Sync for TimerEntry {}

/// An TimerHandle is the (non-enforced) "unique" pointer from the driver to the
/// timer entry. Generally, at most one TimerHandle exists for a timer at a time
/// (enforced by the timer state machine).
///
/// SAFETY: An TimerHandle is essentially a raw pointer, and the usual caveats
/// of pointer safety apply. In particular, TimerHandle does not itself enforce
/// that the timer does still exist; however, normally an TimerHandle is created
/// immediately before registering the timer, and is consumed when firing the
/// timer, to help minimize mistakes. Still, because TimerHandle cannot enforce
/// memory safety, all operations are unsafe.
#[derive(Debug)]
pub(crate) struct TimerHandle {
    inner: NonNull<TimerShared>,
}

pub(super) type EntryList = crate::util::linked_list::LinkedList<TimerShared, TimerShared>;

/// The shared state structure of a timer. This structure is shared between the
/// frontend (`Entry`) and driver backend.
///
/// Note that this structure is located inside the `TimerEntry` structure.
#[derive(Debug)]
#[repr(C)] // required by `link_list::Link` impl
pub(crate) struct TimerShared {
    /// Data manipulated by the driver thread itself, only.
    driver_state: CachePadded<TimerSharedPadded>,

    /// Current state. This records whether the timer entry is currently under
    /// the ownership of the driver, and if not, its current state (not
    /// complete, fired, error, etc).
    state: StateCell,

    _p: PhantomPinned,
}

impl TimerShared {
    pub(super) fn new() -> Self {
        Self {
            state: StateCell::default(),
            driver_state: CachePadded(TimerSharedPadded::new()),
            _p: PhantomPinned,
        }
    }

    /// Gets the cached time-of-expiration value.
    pub(super) fn cached_when(&self) -> u64 {
        // Cached-when is only accessed under the driver lock, so we can use relaxed
        self.driver_state.0.cached_when.load(Ordering::Relaxed)
    }

    /// Gets the true time-of-expiration value, and copies it into the cached
    /// time-of-expiration value.
    ///
    /// SAFETY: Must be called with the driver lock held, and when this entry is
    /// not in any timer wheel lists.
    pub(super) unsafe fn sync_when(&self) -> u64 {
        let true_when = self.true_when();

        self.driver_state
            .0
            .cached_when
            .store(true_when, Ordering::Relaxed);

        true_when
    }

    /// Sets the cached time-of-expiration value.
    ///
    /// SAFETY: Must be called with the driver lock held, and when this entry is
    /// not in any timer wheel lists.
    unsafe fn set_cached_when(&self, when: u64) {
        self.driver_state
            .0
            .cached_when
            .store(when, Ordering::Relaxed);
    }

    /// Returns the true time-of-expiration value, with relaxed memory ordering.
    pub(super) fn true_when(&self) -> u64 {
        self.state.when().expect("Timer already fired")
    }

    /// Sets the true time-of-expiration value, even if it is less than the
    /// current expiration or the timer is deregistered.
    ///
    /// SAFETY: Must only be called with the driver lock held and the entry not
    /// in the timer wheel.
    pub(super) unsafe fn set_expiration(&self, t: u64) {
        self.state.set_expiration(t);
        self.driver_state.0.cached_when.store(t, Ordering::Relaxed);
    }

    /// Sets the true time-of-expiration only if it is after the current.
    pub(super) fn extend_expiration(&self, t: u64) -> Result<(), ()> {
        self.state.extend_expiration(t)
    }

    /// Returns a TimerHandle for this timer.
    pub(super) fn handle(&self) -> TimerHandle {
        TimerHandle {
            inner: NonNull::from(self),
        }
    }

    /// Returns true if the state of this timer indicates that the timer might
    /// be registered with the driver. This check is performed with relaxed
    /// ordering, but is conservative - if it returns false, the timer is
    /// definitely _not_ registered.
    pub(super) fn might_be_registered(&self) -> bool {
        self.state.might_be_registered()
    }
}

/// Additional shared state between the driver and the timer which is cache
/// padded. This contains the information that the driver thread accesses most
/// frequently to minimize contention. In particular, we move it away from the
/// waker, as the waker is updated on every poll.
#[repr(C)] // required by `link_list::Link` impl
struct TimerSharedPadded {
    /// A link within the doubly-linked list of timers on a particular level and
    /// slot. Valid only if state is equal to Registered.
    ///
    /// Only accessed under the entry lock.
    pointers: linked_list::Pointers<TimerShared>,

    /// The expiration time for which this entry is currently registered.
    /// Generally owned by the driver, but is accessed by the entry when not
    /// registered.
    cached_when: AtomicU64,

    /// The true expiration time. Set by the timer future, read by the driver.
    true_when: AtomicU64,
}

impl std::fmt::Debug for TimerSharedPadded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimerSharedPadded")
            .field("when", &self.true_when.load(Ordering::Relaxed))
            .field("cached_when", &self.cached_when.load(Ordering::Relaxed))
            .finish()
    }
}

impl TimerSharedPadded {
    fn new() -> Self {
        Self {
            cached_when: AtomicU64::new(0),
            true_when: AtomicU64::new(0),
            pointers: linked_list::Pointers::new(),
        }
    }
}

unsafe impl Send for TimerShared {}
unsafe impl Sync for TimerShared {}

unsafe impl linked_list::Link for TimerShared {
    type Handle = TimerHandle;

    type Target = TimerShared;

    fn as_raw(handle: &Self::Handle) -> NonNull<Self::Target> {
        handle.inner
    }

    unsafe fn from_raw(ptr: NonNull<Self::Target>) -> Self::Handle {
        TimerHandle { inner: ptr }
    }

    unsafe fn pointers(
        target: NonNull<Self::Target>,
    ) -> NonNull<linked_list::Pointers<Self::Target>> {
        target.cast()
    }
}

// ===== impl Entry =====

impl TimerEntry {
    pub(crate) fn new(handle: &Handle, deadline: Instant) -> Self {
        let driver = handle.clone();

        Self {
            driver,
            inner: StdUnsafeCell::new(TimerShared::new()),
            initial_deadline: Some(deadline),
            _m: std::marker::PhantomPinned,
        }
    }

    fn inner(&self) -> &TimerShared {
        unsafe { &*self.inner.get() }
    }

    pub(crate) fn is_elapsed(&self) -> bool {
        !self.inner().state.might_be_registered() && self.initial_deadline.is_none()
    }

    /// Cancels and deregisters the timer. This operation is irreversible.
    pub(crate) fn cancel(self: Pin<&mut Self>) {
        // We need to perform an acq/rel fence with the driver thread, and the
        // simplest way to do so is to grab the driver lock.
        //
        // Why is this necessary? We're about to release this timer's memory for
        // some other non-timer use. However, we've been doing a bunch of
        // relaxed (or even non-atomic) writes from the driver thread, and we'll
        // be doing more from _this thread_ (as this memory is interpreted as
        // something else).
        //
        // It is critical to ensure that, from the point of view of the driver,
        // those future non-timer writes happen-after the timer is fully fired,
        // and from the purpose of this thread, the driver's writes all
        // happen-before we drop the timer. This in turn requires us to perform
        // an acquire-release barrier in _both_ directions between the driver
        // and dropping thread.
        //
        // The lock acquisition in clear_entry serves this purpose. All of the
        // driver manipulations happen with the lock held, so we can just take
        // the lock and be sure that this drop happens-after everything the
        // driver did so far and happens-before everything the driver does in
        // the future. While we have the lock held, we also go ahead and
        // deregister the entry if necessary.
        unsafe { self.driver.clear_entry(NonNull::from(self.inner())) };
    }

    pub(crate) fn reset(mut self: Pin<&mut Self>, new_time: Instant) {
        unsafe { self.as_mut().get_unchecked_mut() }.initial_deadline = None;

        let tick = self.driver.time_source().deadline_to_tick(new_time);

        if self.inner().extend_expiration(tick).is_ok() {
            return;
        }

        unsafe {
            self.driver.reregister(tick, self.inner().into());
        }
    }

    pub(crate) fn poll_elapsed(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), super::Error>> {
        if self.driver.is_shutdown() {
            panic!("{}", crate::util::error::RUNTIME_SHUTTING_DOWN_ERROR);
        }

        if let Some(deadline) = self.initial_deadline {
            self.as_mut().reset(deadline);
        }

        let this = unsafe { self.get_unchecked_mut() };

        this.inner().state.poll(cx.waker())
    }
}

impl TimerHandle {
    pub(super) unsafe fn cached_when(&self) -> u64 {
        unsafe { self.inner.as_ref().cached_when() }
    }

    pub(super) unsafe fn sync_when(&self) -> u64 {
        unsafe { self.inner.as_ref().sync_when() }
    }

    pub(super) unsafe fn is_pending(&self) -> bool {
        unsafe { self.inner.as_ref().state.is_pending() }
    }

    /// Forcibly sets the true and cached expiration times to the given tick.
    ///
    /// SAFETY: The caller must ensure that the handle remains valid, the driver
    /// lock is held, and that the timer is not in any wheel linked lists.
    pub(super) unsafe fn set_expiration(&self, tick: u64) {
        self.inner.as_ref().set_expiration(tick);
    }

    /// Attempts to mark this entry as pending. If the expiration time is after
    /// `not_after`, however, returns an Err with the current expiration time.
    ///
    /// If an `Err` is returned, the `cached_when` value will be updated to this
    /// new expiration time.
    ///
    /// SAFETY: The caller must ensure that the handle remains valid, the driver
    /// lock is held, and that the timer is not in any wheel linked lists.
    /// After returning Ok, the entry must be added to the pending list.
    pub(super) unsafe fn mark_pending(&self, not_after: u64) -> Result<(), u64> {
        match self.inner.as_ref().state.mark_pending(not_after) {
            Ok(()) => {
                // mark this as being on the pending queue in cached_when
                self.inner.as_ref().set_cached_when(u64::MAX);
                Ok(())
            }
            Err(tick) => {
                self.inner.as_ref().set_cached_when(tick);
                Err(tick)
            }
        }
    }

    /// Attempts to transition to a terminal state. If the state is already a
    /// terminal state, does nothing.
    ///
    /// Because the entry might be dropped after the state is moved to a
    /// terminal state, this function consumes the handle to ensure we don't
    /// access the entry afterwards.
    ///
    /// Returns the last-registered waker, if any.
    ///
    /// SAFETY: The driver lock must be held while invoking this function, and
    /// the entry must not be in any wheel linked lists.
    pub(super) unsafe fn fire(self, completed_state: TimerResult) -> Option<Waker> {
        self.inner.as_ref().state.fire(completed_state)
    }
}

impl Drop for TimerEntry {
    fn drop(&mut self) {
        unsafe { Pin::new_unchecked(self) }.as_mut().cancel()
    }
}

#[cfg_attr(target_arch = "x86_64", repr(align(128)))]
#[cfg_attr(not(target_arch = "x86_64"), repr(align(64)))]
#[derive(Debug, Default)]
struct CachePadded<T>(T);
