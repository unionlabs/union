use super::{Interest, Ready, ReadyEvent, Tick};
use crate::loom::sync::atomic::AtomicUsize;
use crate::loom::sync::Mutex;
use crate::util::bit;
use crate::util::slab::Entry;
use crate::util::WakeList;

use std::sync::atomic::Ordering::{AcqRel, Acquire, Release};
use std::task::{Context, Poll, Waker};

use super::Direction;

cfg_io_readiness! {
    use crate::util::linked_list::{self, LinkedList};

    use std::cell::UnsafeCell;
    use std::future::Future;
    use std::marker::PhantomPinned;
    use std::pin::Pin;
    use std::ptr::NonNull;
}

/// Stored in the I/O driver resource slab.
#[derive(Debug)]
pub(crate) struct ScheduledIo {
    /// Packs the resource's readiness with the resource's generation.
    readiness: AtomicUsize,

    waiters: Mutex<Waiters>,
}

cfg_io_readiness! {
    type WaitList = LinkedList<Waiter, <Waiter as linked_list::Link>::Target>;
}

#[derive(Debug, Default)]
struct Waiters {
    #[cfg(feature = "net")]
    /// List of all current waiters.
    list: WaitList,

    /// Waker used for AsyncRead.
    reader: Option<Waker>,

    /// Waker used for AsyncWrite.
    writer: Option<Waker>,

    /// True if this ScheduledIo has been killed due to IO driver shutdown.
    is_shutdown: bool,
}

cfg_io_readiness! {
    #[derive(Debug)]
    struct Waiter {
        pointers: linked_list::Pointers<Waiter>,

        /// The waker for this task.
        waker: Option<Waker>,

        /// The interest this waiter is waiting on.
        interest: Interest,

        is_ready: bool,

        /// Should never be `!Unpin`.
        _p: PhantomPinned,
    }

    /// Future returned by `readiness()`.
    struct Readiness<'a> {
        scheduled_io: &'a ScheduledIo,

        state: State,

        /// Entry in the waiter `LinkedList`.
        waiter: UnsafeCell<Waiter>,
    }

    enum State {
        Init,
        Waiting,
        Done,
    }
}

// The `ScheduledIo::readiness` (`AtomicUsize`) is packed full of goodness.
//
// | reserved | generation |  driver tick | readiness |
// |----------+------------+--------------+-----------|
// |   1 bit  |   7 bits   +    8 bits    +   16 bits |

const READINESS: bit::Pack = bit::Pack::least_significant(16);

const TICK: bit::Pack = READINESS.then(8);

const GENERATION: bit::Pack = TICK.then(7);

#[test]
fn test_generations_assert_same() {
    assert_eq!(super::GENERATION, GENERATION);
}

// ===== impl ScheduledIo =====

impl Entry for ScheduledIo {
    fn reset(&self) {
        let state = self.readiness.load(Acquire);

        let generation = GENERATION.unpack(state);
        let next = GENERATION.pack_lossy(generation + 1, 0);

        self.readiness.store(next, Release);
    }
}

impl Default for ScheduledIo {
    fn default() -> ScheduledIo {
        ScheduledIo {
            readiness: AtomicUsize::new(0),
            waiters: Mutex::new(Default::default()),
        }
    }
}

impl ScheduledIo {
    pub(crate) fn generation(&self) -> usize {
        GENERATION.unpack(self.readiness.load(Acquire))
    }

    /// Invoked when the IO driver is shut down; forces this ScheduledIo into a
    /// permanently ready state.
    pub(super) fn shutdown(&self) {
        self.wake0(Ready::ALL, true)
    }

    /// Sets the readiness on this `ScheduledIo` by invoking the given closure on
    /// the current value, returning the previous readiness value.
    ///
    /// # Arguments
    /// - `token`: the token for this `ScheduledIo`.
    /// - `tick`: whether setting the tick or trying to clear readiness for a
    ///    specific tick.
    /// - `f`: a closure returning a new readiness value given the previous
    ///   readiness.
    ///
    /// # Returns
    ///
    /// If the given token's generation no longer matches the `ScheduledIo`'s
    /// generation, then the corresponding IO resource has been removed and
    /// replaced with a new resource. In that case, this method returns `Err`.
    /// Otherwise, this returns the previous readiness.
    pub(super) fn set_readiness(
        &self,
        token: Option<usize>,
        tick: Tick,
        f: impl Fn(Ready) -> Ready,
    ) -> Result<(), ()> {
        let mut current = self.readiness.load(Acquire);

        loop {
            let current_generation = GENERATION.unpack(current);

            if let Some(token) = token {
                // Check that the generation for this access is still the
                // current one.
                if GENERATION.unpack(token) != current_generation {
                    return Err(());
                }
            }

            // Mask out the tick/generation bits so that the modifying
            // function doesn't see them.
            let current_readiness = Ready::from_usize(current);
            let new = f(current_readiness);

            let packed = match tick {
                Tick::Set(t) => TICK.pack(t as usize, new.as_usize()),
                Tick::Clear(t) => {
                    if TICK.unpack(current) as u8 != t {
                        // Trying to clear readiness with an old event!
                        return Err(());
                    }

                    TICK.pack(t as usize, new.as_usize())
                }
            };

            let next = GENERATION.pack(current_generation, packed);

            match self
                .readiness
                .compare_exchange(current, next, AcqRel, Acquire)
            {
                Ok(_) => return Ok(()),
                // we lost the race, retry!
                Err(actual) => current = actual,
            }
        }
    }

    /// Notifies all pending waiters that have registered interest in `ready`.
    ///
    /// There may be many waiters to notify. Waking the pending task **must** be
    /// done from outside of the lock otherwise there is a potential for a
    /// deadlock.
    ///
    /// A stack array of wakers is created and filled with wakers to notify, the
    /// lock is released, and the wakers are notified. Because there may be more
    /// than 32 wakers to notify, if the stack array fills up, the lock is
    /// released, the array is cleared, and the iteration continues.
    pub(super) fn wake(&self, ready: Ready) {
        self.wake0(ready, false);
    }

    fn wake0(&self, ready: Ready, shutdown: bool) {
        let mut wakers = WakeList::new();

        let mut waiters = self.waiters.lock();

        waiters.is_shutdown |= shutdown;

        // check for AsyncRead slot
        if ready.is_readable() {
            if let Some(waker) = waiters.reader.take() {
                wakers.push(waker);
            }
        }

        // check for AsyncWrite slot
        if ready.is_writable() {
            if let Some(waker) = waiters.writer.take() {
                wakers.push(waker);
            }
        }

        #[cfg(feature = "net")]
        'outer: loop {
            let mut iter = waiters.list.drain_filter(|w| ready.satisfies(w.interest));

            while wakers.can_push() {
                match iter.next() {
                    Some(waiter) => {
                        let waiter = unsafe { &mut *waiter.as_ptr() };

                        if let Some(waker) = waiter.waker.take() {
                            waiter.is_ready = true;
                            wakers.push(waker);
                        }
                    }
                    None => {
                        break 'outer;
                    }
                }
            }

            drop(waiters);

            wakers.wake_all();

            // Acquire the lock again.
            waiters = self.waiters.lock();
        }

        // Release the lock before notifying
        drop(waiters);

        wakers.wake_all();
    }

    pub(super) fn ready_event(&self, interest: Interest) -> ReadyEvent {
        let curr = self.readiness.load(Acquire);

        ReadyEvent {
            tick: TICK.unpack(curr) as u8,
            ready: interest.mask() & Ready::from_usize(READINESS.unpack(curr)),
        }
    }

    /// Polls for readiness events in a given direction.
    ///
    /// These are to support `AsyncRead` and `AsyncWrite` polling methods,
    /// which cannot use the `async fn` version. This uses reserved reader
    /// and writer slots.
    pub(super) fn poll_readiness(
        &self,
        cx: &mut Context<'_>,
        direction: Direction,
    ) -> Poll<ReadyEvent> {
        let curr = self.readiness.load(Acquire);

        let ready = direction.mask() & Ready::from_usize(READINESS.unpack(curr));

        if ready.is_empty() {
            // Update the task info
            let mut waiters = self.waiters.lock();
            let slot = match direction {
                Direction::Read => &mut waiters.reader,
                Direction::Write => &mut waiters.writer,
            };

            // Avoid cloning the waker if one is already stored that matches the
            // current task.
            match slot {
                Some(existing) => {
                    if !existing.will_wake(cx.waker()) {
                        *existing = cx.waker().clone();
                    }
                }
                None => {
                    *slot = Some(cx.waker().clone());
                }
            }

            // Try again, in case the readiness was changed while we were
            // taking the waiters lock
            let curr = self.readiness.load(Acquire);
            let ready = direction.mask() & Ready::from_usize(READINESS.unpack(curr));
            if waiters.is_shutdown {
                Poll::Ready(ReadyEvent {
                    tick: TICK.unpack(curr) as u8,
                    ready: direction.mask(),
                })
            } else if ready.is_empty() {
                Poll::Pending
            } else {
                Poll::Ready(ReadyEvent {
                    tick: TICK.unpack(curr) as u8,
                    ready,
                })
            }
        } else {
            Poll::Ready(ReadyEvent {
                tick: TICK.unpack(curr) as u8,
                ready,
            })
        }
    }

    pub(crate) fn clear_readiness(&self, event: ReadyEvent) {
        // This consumes the current readiness state **except** for closed
        // states. Closed states are excluded because they are final states.
        let mask_no_closed = event.ready - Ready::READ_CLOSED - Ready::WRITE_CLOSED;

        // result isn't important
        let _ = self.set_readiness(None, Tick::Clear(event.tick), |curr| curr - mask_no_closed);
    }

    pub(crate) fn clear_wakers(&self) {
        let mut waiters = self.waiters.lock();
        waiters.reader.take();
        waiters.writer.take();
    }
}

impl Drop for ScheduledIo {
    fn drop(&mut self) {
        self.wake(Ready::ALL);
    }
}

unsafe impl Send for ScheduledIo {}
unsafe impl Sync for ScheduledIo {}

cfg_io_readiness! {
    impl ScheduledIo {
        /// An async version of `poll_readiness` which uses a linked list of wakers.
        pub(crate) async fn readiness(&self, interest: Interest) -> ReadyEvent {
            self.readiness_fut(interest).await
        }

        // This is in a separate function so that the borrow checker doesn't think
        // we are borrowing the `UnsafeCell` possibly over await boundaries.
        //
        // Go figure.
        fn readiness_fut(&self, interest: Interest) -> Readiness<'_> {
            Readiness {
                scheduled_io: self,
                state: State::Init,
                waiter: UnsafeCell::new(Waiter {
                    pointers: linked_list::Pointers::new(),
                    waker: None,
                    is_ready: false,
                    interest,
                    _p: PhantomPinned,
                }),
            }
        }
    }

    unsafe impl linked_list::Link for Waiter {
        type Handle = NonNull<Waiter>;
        type Target = Waiter;

        fn as_raw(handle: &NonNull<Waiter>) -> NonNull<Waiter> {
            *handle
        }

        unsafe fn from_raw(ptr: NonNull<Waiter>) -> NonNull<Waiter> {
            ptr
        }

        unsafe fn pointers(mut target: NonNull<Waiter>) -> NonNull<linked_list::Pointers<Waiter>> {
            NonNull::from(&mut target.as_mut().pointers)
        }
    }

    // ===== impl Readiness =====

    impl Future for Readiness<'_> {
        type Output = ReadyEvent;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            use std::sync::atomic::Ordering::SeqCst;

            let (scheduled_io, state, waiter) = unsafe {
                let me = self.get_unchecked_mut();
                (&me.scheduled_io, &mut me.state, &me.waiter)
            };

            loop {
                match *state {
                    State::Init => {
                        // Optimistically check existing readiness
                        let curr = scheduled_io.readiness.load(SeqCst);
                        let ready = Ready::from_usize(READINESS.unpack(curr));

                        // Safety: `waiter.interest` never changes
                        let interest = unsafe { (*waiter.get()).interest };
                        let ready = ready.intersection(interest);

                        if !ready.is_empty() {
                            // Currently ready!
                            let tick = TICK.unpack(curr) as u8;
                            *state = State::Done;
                            return Poll::Ready(ReadyEvent { tick, ready });
                        }

                        // Wasn't ready, take the lock (and check again while locked).
                        let mut waiters = scheduled_io.waiters.lock();

                        let curr = scheduled_io.readiness.load(SeqCst);
                        let mut ready = Ready::from_usize(READINESS.unpack(curr));

                        if waiters.is_shutdown {
                            ready = Ready::ALL;
                        }

                        let ready = ready.intersection(interest);

                        if !ready.is_empty() {
                            // Currently ready!
                            let tick = TICK.unpack(curr) as u8;
                            *state = State::Done;
                            return Poll::Ready(ReadyEvent { tick, ready });
                        }

                        // Not ready even after locked, insert into list...

                        // Safety: called while locked
                        unsafe {
                            (*waiter.get()).waker = Some(cx.waker().clone());
                        }

                        // Insert the waiter into the linked list
                        //
                        // safety: pointers from `UnsafeCell` are never null.
                        waiters
                            .list
                            .push_front(unsafe { NonNull::new_unchecked(waiter.get()) });
                        *state = State::Waiting;
                    }
                    State::Waiting => {
                        // Currently in the "Waiting" state, implying the caller has
                        // a waiter stored in the waiter list (guarded by
                        // `notify.waiters`). In order to access the waker fields,
                        // we must hold the lock.

                        let waiters = scheduled_io.waiters.lock();

                        // Safety: called while locked
                        let w = unsafe { &mut *waiter.get() };

                        if w.is_ready {
                            // Our waker has been notified.
                            *state = State::Done;
                        } else {
                            // Update the waker, if necessary.
                            if !w.waker.as_ref().unwrap().will_wake(cx.waker()) {
                                w.waker = Some(cx.waker().clone());
                            }

                            return Poll::Pending;
                        }

                        // Explicit drop of the lock to indicate the scope that the
                        // lock is held. Because holding the lock is required to
                        // ensure safe access to fields not held within the lock, it
                        // is helpful to visualize the scope of the critical
                        // section.
                        drop(waiters);
                    }
                    State::Done => {
                        let tick = TICK.unpack(scheduled_io.readiness.load(Acquire)) as u8;

                        // Safety: State::Done means it is no longer shared
                        let w = unsafe { &mut *waiter.get() };

                        return Poll::Ready(ReadyEvent {
                            tick,
                            ready: Ready::from_interest(w.interest),
                        });
                    }
                }
            }
        }
    }

    impl Drop for Readiness<'_> {
        fn drop(&mut self) {
            let mut waiters = self.scheduled_io.waiters.lock();

            // Safety: `waiter` is only ever stored in `waiters`
            unsafe {
                waiters
                    .list
                    .remove(NonNull::new_unchecked(self.waiter.get()))
            };
        }
    }

    unsafe impl Send for Readiness<'_> {}
    unsafe impl Sync for Readiness<'_> {}
}
