cfg_io_driver! {
    pub(crate) mod bit;
    pub(crate) mod slab;
}

#[cfg(feature = "rt")]
pub(crate) mod atomic_cell;

#[cfg(any(
    // io driver uses `WakeList` directly
    feature = "net",
    feature = "process",
    // `sync` enables `Notify` and `batch_semaphore`, which require `WakeList`.
    feature = "sync",
    // `fs` uses `batch_semaphore`, which requires `WakeList`.
    feature = "fs",
    // rt and signal use `Notify`, which requires `WakeList`.
    feature = "rt",
    feature = "signal",
))]
mod wake_list;
#[cfg(any(
    feature = "net",
    feature = "process",
    feature = "sync",
    feature = "fs",
    feature = "rt",
    feature = "signal",
))]
pub(crate) use wake_list::WakeList;

#[cfg(any(
    feature = "fs",
    feature = "net",
    feature = "process",
    feature = "rt",
    feature = "sync",
    feature = "signal",
    feature = "time",
))]
pub(crate) mod linked_list;

#[cfg(any(feature = "rt-multi-thread", feature = "macros"))]
mod rand;

cfg_rt! {
    cfg_unstable! {
        mod idle_notified_set;
        pub(crate) use idle_notified_set::IdleNotifiedSet;
    }

    mod wake;
    pub(crate) use wake::WakerRef;
    pub(crate) use wake::{waker_ref, Wake};

    mod sync_wrapper;
    pub(crate) use sync_wrapper::SyncWrapper;

    mod vec_deque_cell;
    pub(crate) use vec_deque_cell::VecDequeCell;
}

cfg_rt_multi_thread! {
    pub(crate) use self::rand::FastRand;

    mod try_lock;
    pub(crate) use try_lock::TryLock;
}

pub(crate) mod trace;

#[cfg(any(feature = "macros"))]
#[cfg_attr(not(feature = "macros"), allow(unreachable_pub))]
pub use self::rand::thread_rng_n;

#[cfg(any(
    feature = "rt",
    feature = "time",
    feature = "net",
    feature = "process",
    all(unix, feature = "signal")
))]
pub(crate) mod error;
