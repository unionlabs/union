use crate::loom::sync::Arc;
use crate::time::driver::ClockTime;
use std::fmt;

/// Handle to time driver instance.
#[derive(Clone)]
pub(crate) struct Handle {
    time_source: ClockTime,
    inner: Arc<super::Inner>,
}

impl Handle {
    /// Creates a new timer `Handle` from a shared `Inner` timer state.
    pub(super) fn new(inner: Arc<super::Inner>) -> Self {
        let time_source = inner.state.lock().time_source.clone();
        Handle { time_source, inner }
    }

    /// Returns the time source associated with this handle.
    pub(super) fn time_source(&self) -> &ClockTime {
        &self.time_source
    }

    /// Access the driver's inner structure.
    pub(super) fn get(&self) -> &super::Inner {
        &*self.inner
    }

    /// Checks whether the driver has been shutdown.
    pub(super) fn is_shutdown(&self) -> bool {
        self.inner.is_shutdown()
    }
}

cfg_rt! {
    impl Handle {
        /// Tries to get a handle to the current timer.
        ///
        /// # Panics
        ///
        /// This function panics if there is no current timer set.
        ///
        /// It can be triggered when [`Builder::enable_time`] or
        /// [`Builder::enable_all`] are not included in the builder.
        ///
        /// It can also panic whenever a timer is created outside of a
        /// Tokio runtime. That is why `rt.block_on(sleep(...))` will panic,
        /// since the function is executed outside of the runtime.
        /// Whereas `rt.block_on(async {sleep(...).await})` doesn't panic.
        /// And this is because wrapping the function on an async makes it lazy,
        /// and so gets executed inside the runtime successfully without
        /// panicking.
        ///
        /// [`Builder::enable_time`]: crate::runtime::Builder::enable_time
        /// [`Builder::enable_all`]: crate::runtime::Builder::enable_all
        #[track_caller]
        pub(crate) fn current() -> Self {
            crate::runtime::context::time_handle()
                .expect("A Tokio 1.x context was found, but timers are disabled. Call `enable_time` on the runtime builder to enable timers.")
        }
    }
}

cfg_not_rt! {
    impl Handle {
        /// Tries to get a handle to the current timer.
        ///
        /// # Panics
        ///
        /// This function panics if there is no current timer set.
        ///
        /// It can be triggered when [`Builder::enable_time`] or
        /// [`Builder::enable_all`] are not included in the builder.
        ///
        /// It can also panic whenever a timer is created outside of a
        /// Tokio runtime. That is why `rt.block_on(sleep(...))` will panic,
        /// since the function is executed outside of the runtime.
        /// Whereas `rt.block_on(async {sleep(...).await})` doesn't panic.
        /// And this is because wrapping the function on an async makes it lazy,
        /// and so gets executed inside the runtime successfully without
        /// panicking.
        ///
        /// [`Builder::enable_time`]: crate::runtime::Builder::enable_time
        /// [`Builder::enable_all`]: crate::runtime::Builder::enable_all
        #[track_caller]
        pub(crate) fn current() -> Self {
            panic!("{}", crate::util::error::CONTEXT_MISSING_ERROR)
        }
    }
}

impl fmt::Debug for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Handle")
    }
}
