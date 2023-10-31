cfg_macros! {
    pub use crate::future::poll_fn;
    pub use crate::future::maybe_done::maybe_done;
    pub use crate::util::thread_rng_n;
}

pub use std::future::Future;
pub use std::pin::Pin;
pub use std::task::Poll;
