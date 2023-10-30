use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::usize;

use crate::registry::{Registry, WorkerThread};

/// We define various kinds of latches, which are all a primitive signaling
/// mechanism. A latch starts as false. Eventually someone calls `set()` and
/// it becomes true. You can test if it has been set by calling `probe()`.
///
/// Some kinds of latches, but not all, support a `wait()` operation
/// that will wait until the latch is set, blocking efficiently. That
/// is not part of the trait since it is not possibly to do with all
/// latches.
///
/// The intention is that `set()` is called once, but `probe()` may be
/// called any number of times. Once `probe()` returns true, the memory
/// effects that occurred before `set()` become visible.
///
/// It'd probably be better to refactor the API into two paired types,
/// but that's a bit of work, and this is not a public API.
///
/// ## Memory ordering
///
/// Latches need to guarantee two things:
///
/// - Once `probe()` returns true, all memory effects from the `set()`
///   are visible (in other words, the set should synchronize-with
///   the probe).
/// - Once `set()` occurs, the next `probe()` *will* observe it.  This
///   typically requires a seq-cst ordering. See [the "tickle-then-get-sleepy" scenario in the sleep
///   README](/src/sleep/README.md#tickle-then-get-sleepy) for details.
pub(super) trait Latch {
    /// Set the latch, signalling others.
    ///
    /// # WARNING
    ///
    /// Setting a latch triggers other threads to wake up and (in some
    /// cases) complete. This may, in turn, cause memory to be
    /// deallocated and so forth. One must be very careful about this,
    /// and it's typically better to read all the fields you will need
    /// to access *before* a latch is set!
    ///
    /// This function operates on `*const Self` instead of `&self` to allow it
    /// to become dangling during this call. The caller must ensure that the
    /// pointer is valid upon entry, and not invalidated during the call by any
    /// actions other than `set` itself.
    unsafe fn set(this: *const Self);
}

pub(super) trait AsCoreLatch {
    fn as_core_latch(&self) -> &CoreLatch;
}

/// Latch is not set, owning thread is awake
const UNSET: usize = 0;

/// Latch is not set, owning thread is going to sleep on this latch
/// (but has not yet fallen asleep).
const SLEEPY: usize = 1;

/// Latch is not set, owning thread is asleep on this latch and
/// must be awoken.
const SLEEPING: usize = 2;

/// Latch is set.
const SET: usize = 3;

/// Spin latches are the simplest, most efficient kind, but they do
/// not support a `wait()` operation. They just have a boolean flag
/// that becomes true when `set()` is called.
#[derive(Debug)]
pub(super) struct CoreLatch {
    state: AtomicUsize,
}

impl CoreLatch {
    #[inline]
    fn new() -> Self {
        Self {
            state: AtomicUsize::new(0),
        }
    }

    /// Returns the address of this core latch as an integer. Used
    /// for logging.
    #[inline]
    pub(super) fn addr(&self) -> usize {
        self as *const CoreLatch as usize
    }

    /// Invoked by owning thread as it prepares to sleep. Returns true
    /// if the owning thread may proceed to fall asleep, false if the
    /// latch was set in the meantime.
    #[inline]
    pub(super) fn get_sleepy(&self) -> bool {
        self.state
            .compare_exchange(UNSET, SLEEPY, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
    }

    /// Invoked by owning thread as it falls asleep sleep. Returns
    /// true if the owning thread should block, or false if the latch
    /// was set in the meantime.
    #[inline]
    pub(super) fn fall_asleep(&self) -> bool {
        self.state
            .compare_exchange(SLEEPY, SLEEPING, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
    }

    /// Invoked by owning thread as it falls asleep sleep. Returns
    /// true if the owning thread should block, or false if the latch
    /// was set in the meantime.
    #[inline]
    pub(super) fn wake_up(&self) {
        if !self.probe() {
            let _ =
                self.state
                    .compare_exchange(SLEEPING, UNSET, Ordering::SeqCst, Ordering::Relaxed);
        }
    }

    /// Set the latch. If this returns true, the owning thread was sleeping
    /// and must be awoken.
    ///
    /// This is private because, typically, setting a latch involves
    /// doing some wakeups; those are encapsulated in the surrounding
    /// latch code.
    #[inline]
    unsafe fn set(this: *const Self) -> bool {
        let old_state = (*this).state.swap(SET, Ordering::AcqRel);
        old_state == SLEEPING
    }

    /// Test if this latch has been set.
    #[inline]
    pub(super) fn probe(&self) -> bool {
        self.state.load(Ordering::Acquire) == SET
    }
}

/// Spin latches are the simplest, most efficient kind, but they do
/// not support a `wait()` operation. They just have a boolean flag
/// that becomes true when `set()` is called.
pub(super) struct SpinLatch<'r> {
    core_latch: CoreLatch,
    registry: &'r Arc<Registry>,
    target_worker_index: usize,
    cross: bool,
}

impl<'r> SpinLatch<'r> {
    /// Creates a new spin latch that is owned by `thread`. This means
    /// that `thread` is the only thread that should be blocking on
    /// this latch -- it also means that when the latch is set, we
    /// will wake `thread` if it is sleeping.
    #[inline]
    pub(super) fn new(thread: &'r WorkerThread) -> SpinLatch<'r> {
        SpinLatch {
            core_latch: CoreLatch::new(),
            registry: thread.registry(),
            target_worker_index: thread.index(),
            cross: false,
        }
    }

    /// Creates a new spin latch for cross-threadpool blocking.  Notably, we
    /// need to make sure the registry is kept alive after setting, so we can
    /// safely call the notification.
    #[inline]
    pub(super) fn cross(thread: &'r WorkerThread) -> SpinLatch<'r> {
        SpinLatch {
            cross: true,
            ..SpinLatch::new(thread)
        }
    }

    #[inline]
    pub(super) fn probe(&self) -> bool {
        self.core_latch.probe()
    }
}

impl<'r> AsCoreLatch for SpinLatch<'r> {
    #[inline]
    fn as_core_latch(&self) -> &CoreLatch {
        &self.core_latch
    }
}

impl<'r> Latch for SpinLatch<'r> {
    #[inline]
    unsafe fn set(this: *const Self) {
        let cross_registry;

        let registry: &Registry = if (*this).cross {
            // Ensure the registry stays alive while we notify it.
            // Otherwise, it would be possible that we set the spin
            // latch and the other thread sees it and exits, causing
            // the registry to be deallocated, all before we get a
            // chance to invoke `registry.notify_worker_latch_is_set`.
            cross_registry = Arc::clone((*this).registry);
            &cross_registry
        } else {
            // If this is not a "cross-registry" spin-latch, then the
            // thread which is performing `set` is itself ensuring
            // that the registry stays alive. However, that doesn't
            // include this *particular* `Arc` handle if the waiting
            // thread then exits, so we must completely dereference it.
            (*this).registry
        };
        let target_worker_index = (*this).target_worker_index;

        // NOTE: Once we `set`, the target may proceed and invalidate `this`!
        if CoreLatch::set(&(*this).core_latch) {
            // Subtle: at this point, we can no longer read from
            // `self`, because the thread owning this spin latch may
            // have awoken and deallocated the latch. Therefore, we
            // only use fields whose values we already read.
            registry.notify_worker_latch_is_set(target_worker_index);
        }
    }
}

/// A Latch starts as false and eventually becomes true. You can block
/// until it becomes true.
#[derive(Debug)]
pub(super) struct LockLatch {
    m: Mutex<bool>,
    v: Condvar,
}

impl LockLatch {
    #[inline]
    pub(super) fn new() -> LockLatch {
        LockLatch {
            m: Mutex::new(false),
            v: Condvar::new(),
        }
    }

    /// Block until latch is set, then resets this lock latch so it can be reused again.
    pub(super) fn wait_and_reset(&self) {
        let mut guard = self.m.lock().unwrap();
        while !*guard {
            guard = self.v.wait(guard).unwrap();
        }
        *guard = false;
    }

    /// Block until latch is set.
    pub(super) fn wait(&self) {
        let mut guard = self.m.lock().unwrap();
        while !*guard {
            guard = self.v.wait(guard).unwrap();
        }
    }
}

impl Latch for LockLatch {
    #[inline]
    unsafe fn set(this: *const Self) {
        let mut guard = (*this).m.lock().unwrap();
        *guard = true;
        (*this).v.notify_all();
    }
}

/// Counting latches are used to implement scopes. They track a
/// counter. Unlike other latches, calling `set()` does not
/// necessarily make the latch be considered `set()`; instead, it just
/// decrements the counter. The latch is only "set" (in the sense that
/// `probe()` returns true) once the counter reaches zero.
///
/// Note: like a `SpinLatch`, count laches are always associated with
/// some registry that is probing them, which must be tickled when
/// they are set. *Unlike* a `SpinLatch`, they don't themselves hold a
/// reference to that registry. This is because in some cases the
/// registry owns the count-latch, and that would create a cycle. So a
/// `CountLatch` must be given a reference to its owning registry when
/// it is set. For this reason, it does not implement the `Latch`
/// trait (but it doesn't have to, as it is not used in those generic
/// contexts).
#[derive(Debug)]
pub(super) struct CountLatch {
    core_latch: CoreLatch,
    counter: AtomicUsize,
}

impl CountLatch {
    #[inline]
    pub(super) fn new() -> CountLatch {
        Self::with_count(1)
    }

    #[inline]
    pub(super) fn with_count(n: usize) -> CountLatch {
        CountLatch {
            core_latch: CoreLatch::new(),
            counter: AtomicUsize::new(n),
        }
    }

    #[inline]
    pub(super) fn increment(&self) {
        debug_assert!(!self.core_latch.probe());
        self.counter.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrements the latch counter by one. If this is the final
    /// count, then the latch is **set**, and calls to `probe()` will
    /// return true. Returns whether the latch was set.
    #[inline]
    pub(super) unsafe fn set(this: *const Self) -> bool {
        if (*this).counter.fetch_sub(1, Ordering::SeqCst) == 1 {
            CoreLatch::set(&(*this).core_latch);
            true
        } else {
            false
        }
    }

    /// Decrements the latch counter by one and possibly set it.  If
    /// the latch is set, then the specific worker thread is tickled,
    /// which should be the one that owns this latch.
    #[inline]
    pub(super) unsafe fn set_and_tickle_one(
        this: *const Self,
        registry: &Registry,
        target_worker_index: usize,
    ) {
        if Self::set(this) {
            registry.notify_worker_latch_is_set(target_worker_index);
        }
    }
}

impl AsCoreLatch for CountLatch {
    #[inline]
    fn as_core_latch(&self) -> &CoreLatch {
        &self.core_latch
    }
}

#[derive(Debug)]
pub(super) struct CountLockLatch {
    lock_latch: LockLatch,
    counter: AtomicUsize,
}

impl CountLockLatch {
    #[inline]
    pub(super) fn with_count(n: usize) -> CountLockLatch {
        CountLockLatch {
            lock_latch: LockLatch::new(),
            counter: AtomicUsize::new(n),
        }
    }

    #[inline]
    pub(super) fn increment(&self) {
        let old_counter = self.counter.fetch_add(1, Ordering::Relaxed);
        debug_assert!(old_counter != 0);
    }

    pub(super) fn wait(&self) {
        self.lock_latch.wait();
    }
}

impl Latch for CountLockLatch {
    #[inline]
    unsafe fn set(this: *const Self) {
        if (*this).counter.fetch_sub(1, Ordering::SeqCst) == 1 {
            LockLatch::set(&(*this).lock_latch);
        }
    }
}

/// `&L` without any implication of `dereferenceable` for `Latch::set`
pub(super) struct LatchRef<'a, L> {
    inner: *const L,
    marker: PhantomData<&'a L>,
}

impl<L> LatchRef<'_, L> {
    pub(super) fn new(inner: &L) -> LatchRef<'_, L> {
        LatchRef {
            inner,
            marker: PhantomData,
        }
    }
}

unsafe impl<L: Sync> Sync for LatchRef<'_, L> {}

impl<L> Deref for LatchRef<'_, L> {
    type Target = L;

    fn deref(&self) -> &L {
        // SAFETY: if we have &self, the inner latch is still alive
        unsafe { &*self.inner }
    }
}

impl<L: Latch> Latch for LatchRef<'_, L> {
    #[inline]
    unsafe fn set(this: *const Self) {
        L::set((*this).inner);
    }
}
