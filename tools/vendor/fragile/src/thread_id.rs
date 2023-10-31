use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};

fn next() -> NonZeroUsize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    NonZeroUsize::new(COUNTER.fetch_add(1, Ordering::SeqCst)).expect("more than usize::MAX threads")
}

pub(crate) fn get() -> NonZeroUsize {
    thread_local!(static THREAD_ID: NonZeroUsize = next());
    THREAD_ID.with(|&x| x)
}
