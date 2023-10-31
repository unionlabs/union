//! The Tokio runtime.
//!
//! Unlike other Rust programs, asynchronous applications require runtime
//! support. In particular, the following runtime services are necessary:
//!
//! * An **I/O event loop**, called the driver, which drives I/O resources and
//!   dispatches I/O events to tasks that depend on them.
//! * A **scheduler** to execute [tasks] that use these I/O resources.
//! * A **timer** for scheduling work to run after a set period of time.
//!
//! Tokio's [`Runtime`] bundles all of these services as a single type, allowing
//! them to be started, shut down, and configured together. However, often it is
//! not required to configure a [`Runtime`] manually, and a user may just use the
//! [`tokio::main`] attribute macro, which creates a [`Runtime`] under the hood.
//!
//! # Usage
//!
//! When no fine tuning is required, the [`tokio::main`] attribute macro can be
//! used.
//!
//! ```no_run
//! use tokio::net::TcpListener;
//! use tokio::io::{AsyncReadExt, AsyncWriteExt};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let listener = TcpListener::bind("127.0.0.1:8080").await?;
//!
//!     loop {
//!         let (mut socket, _) = listener.accept().await?;
//!
//!         tokio::spawn(async move {
//!             let mut buf = [0; 1024];
//!
//!             // In a loop, read data from the socket and write the data back.
//!             loop {
//!                 let n = match socket.read(&mut buf).await {
//!                     // socket closed
//!                     Ok(n) if n == 0 => return,
//!                     Ok(n) => n,
//!                     Err(e) => {
//!                         println!("failed to read from socket; err = {:?}", e);
//!                         return;
//!                     }
//!                 };
//!
//!                 // Write the data back
//!                 if let Err(e) = socket.write_all(&buf[0..n]).await {
//!                     println!("failed to write to socket; err = {:?}", e);
//!                     return;
//!                 }
//!             }
//!         });
//!     }
//! }
//! ```
//!
//! From within the context of the runtime, additional tasks are spawned using
//! the [`tokio::spawn`] function. Futures spawned using this function will be
//! executed on the same thread pool used by the [`Runtime`].
//!
//! A [`Runtime`] instance can also be used directly.
//!
//! ```no_run
//! use tokio::net::TcpListener;
//! use tokio::io::{AsyncReadExt, AsyncWriteExt};
//! use tokio::runtime::Runtime;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create the runtime
//!     let rt  = Runtime::new()?;
//!
//!     // Spawn the root task
//!     rt.block_on(async {
//!         let listener = TcpListener::bind("127.0.0.1:8080").await?;
//!
//!         loop {
//!             let (mut socket, _) = listener.accept().await?;
//!
//!             tokio::spawn(async move {
//!                 let mut buf = [0; 1024];
//!
//!                 // In a loop, read data from the socket and write the data back.
//!                 loop {
//!                     let n = match socket.read(&mut buf).await {
//!                         // socket closed
//!                         Ok(n) if n == 0 => return,
//!                         Ok(n) => n,
//!                         Err(e) => {
//!                             println!("failed to read from socket; err = {:?}", e);
//!                             return;
//!                         }
//!                     };
//!
//!                     // Write the data back
//!                     if let Err(e) = socket.write_all(&buf[0..n]).await {
//!                         println!("failed to write to socket; err = {:?}", e);
//!                         return;
//!                     }
//!                 }
//!             });
//!         }
//!     })
//! }
//! ```
//!
//! ## Runtime Configurations
//!
//! Tokio provides multiple task scheduling strategies, suitable for different
//! applications. The [runtime builder] or `#[tokio::main]` attribute may be
//! used to select which scheduler to use.
//!
//! #### Multi-Thread Scheduler
//!
//! The multi-thread scheduler executes futures on a _thread pool_, using a
//! work-stealing strategy. By default, it will start a worker thread for each
//! CPU core available on the system. This tends to be the ideal configuration
//! for most applications. The multi-thread scheduler requires the `rt-multi-thread`
//! feature flag, and is selected by default:
//! ```
//! use tokio::runtime;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let threaded_rt = runtime::Runtime::new()?;
//! # Ok(()) }
//! ```
//!
//! Most applications should use the multi-thread scheduler, except in some
//! niche use-cases, such as when running only a single thread is required.
//!
//! #### Current-Thread Scheduler
//!
//! The current-thread scheduler provides a _single-threaded_ future executor.
//! All tasks will be created and executed on the current thread. This requires
//! the `rt` feature flag.
//! ```
//! use tokio::runtime;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let basic_rt = runtime::Builder::new_current_thread()
//!     .build()?;
//! # Ok(()) }
//! ```
//!
//! #### Resource drivers
//!
//! When configuring a runtime by hand, no resource drivers are enabled by
//! default. In this case, attempting to use networking types or time types will
//! fail. In order to enable these types, the resource drivers must be enabled.
//! This is done with [`Builder::enable_io`] and [`Builder::enable_time`]. As a
//! shorthand, [`Builder::enable_all`] enables both resource drivers.
//!
//! ## Lifetime of spawned threads
//!
//! The runtime may spawn threads depending on its configuration and usage. The
//! multi-thread scheduler spawns threads to schedule tasks and for `spawn_blocking`
//! calls.
//!
//! While the `Runtime` is active, threads may shutdown after periods of being
//! idle. Once `Runtime` is dropped, all runtime threads are forcibly shutdown.
//! Any tasks that have not yet completed will be dropped.
//!
//! [tasks]: crate::task
//! [`Runtime`]: Runtime
//! [`tokio::spawn`]: crate::spawn
//! [`tokio::main`]: ../attr.main.html
//! [runtime builder]: crate::runtime::Builder
//! [`Runtime::new`]: crate::runtime::Runtime::new
//! [`Builder::basic_scheduler`]: crate::runtime::Builder::basic_scheduler
//! [`Builder::threaded_scheduler`]: crate::runtime::Builder::threaded_scheduler
//! [`Builder::enable_io`]: crate::runtime::Builder::enable_io
//! [`Builder::enable_time`]: crate::runtime::Builder::enable_time
//! [`Builder::enable_all`]: crate::runtime::Builder::enable_all

// At the top due to macros
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
#[macro_use]
mod tests;

pub(crate) mod enter;

pub(crate) mod task;

cfg_metrics! {
    mod metrics;
    pub use metrics::RuntimeMetrics;

    pub(crate) use metrics::{MetricsBatch, SchedulerMetrics, WorkerMetrics};

    cfg_net! {
       pub(crate) use metrics::IoDriverMetrics;
    }
}

cfg_not_metrics! {
    pub(crate) mod metrics;
    pub(crate) use metrics::{SchedulerMetrics, WorkerMetrics, MetricsBatch};
}

cfg_rt! {
    mod basic_scheduler;
    use basic_scheduler::BasicScheduler;

    mod blocking;
    use blocking::BlockingPool;
    pub(crate) use blocking::spawn_blocking;

    cfg_trace! {
        pub(crate) use blocking::Mandatory;
    }

    cfg_fs! {
        pub(crate) use blocking::spawn_mandatory_blocking;
    }

    mod builder;
    pub use self::builder::Builder;
    cfg_unstable! {
        pub use self::builder::UnhandledPanic;
    }

    pub(crate) mod context;
    mod driver;

    use self::enter::enter;

    mod handle;
    pub use handle::{EnterGuard, Handle, TryCurrentError};
    pub(crate) use handle::{HandleInner, ToHandle};

    mod spawner;
    use self::spawner::Spawner;
}

cfg_rt_multi_thread! {
    use driver::Driver;

    pub(crate) mod thread_pool;
    use self::thread_pool::ThreadPool;
}

cfg_rt! {
    use crate::task::JoinHandle;

    use std::future::Future;
    use std::time::Duration;

    /// The Tokio runtime.
    ///
    /// The runtime provides an I/O driver, task scheduler, [timer], and
    /// blocking pool, necessary for running asynchronous tasks.
    ///
    /// Instances of `Runtime` can be created using [`new`], or [`Builder`].
    /// However, most users will use the `#[tokio::main]` annotation on their
    /// entry point instead.
    ///
    /// See [module level][mod] documentation for more details.
    ///
    /// # Shutdown
    ///
    /// Shutting down the runtime is done by dropping the value. The current
    /// thread will block until the shut down operation has completed.
    ///
    /// * Drain any scheduled work queues.
    /// * Drop any futures that have not yet completed.
    /// * Drop the reactor.
    ///
    /// Once the reactor has dropped, any outstanding I/O resources bound to
    /// that reactor will no longer function. Calling any method on them will
    /// result in an error.
    ///
    /// # Sharing
    ///
    /// The Tokio runtime implements `Sync` and `Send` to allow you to wrap it
    /// in a `Arc`. Most fn take `&self` to allow you to call them concurrently
    /// across multiple threads.
    ///
    /// Calls to `shutdown` and `shutdown_timeout` require exclusive ownership of
    /// the runtime type and this can be achieved via `Arc::try_unwrap` when only
    /// one strong count reference is left over.
    ///
    /// [timer]: crate::time
    /// [mod]: index.html
    /// [`new`]: method@Self::new
    /// [`Builder`]: struct@Builder
    #[derive(Debug)]
    pub struct Runtime {
        /// Task executor
        kind: Kind,

        /// Handle to runtime, also contains driver handles
        handle: Handle,

        /// Blocking pool handle, used to signal shutdown
        blocking_pool: BlockingPool,
    }

    /// The runtime executor is either a thread-pool or a current-thread executor.
    #[derive(Debug)]
    enum Kind {
        /// Execute all tasks on the current-thread.
        CurrentThread(BasicScheduler),

        /// Execute tasks across multiple threads.
        #[cfg(feature = "rt-multi-thread")]
        ThreadPool(ThreadPool),
    }

    /// After thread starts / before thread stops
    type Callback = std::sync::Arc<dyn Fn() + Send + Sync>;

    impl Runtime {
        /// Creates a new runtime instance with default configuration values.
        ///
        /// This results in the multi threaded scheduler, I/O driver, and time driver being
        /// initialized.
        ///
        /// Most applications will not need to call this function directly. Instead,
        /// they will use the  [`#[tokio::main]` attribute][main]. When a more complex
        /// configuration is necessary, the [runtime builder] may be used.
        ///
        /// See [module level][mod] documentation for more details.
        ///
        /// # Examples
        ///
        /// Creating a new `Runtime` with default configuration values.
        ///
        /// ```
        /// use tokio::runtime::Runtime;
        ///
        /// let rt = Runtime::new()
        ///     .unwrap();
        ///
        /// // Use the runtime...
        /// ```
        ///
        /// [mod]: index.html
        /// [main]: ../attr.main.html
        /// [threaded scheduler]: index.html#threaded-scheduler
        /// [basic scheduler]: index.html#basic-scheduler
        /// [runtime builder]: crate::runtime::Builder
        #[cfg(feature = "rt-multi-thread")]
        #[cfg_attr(docsrs, doc(cfg(feature = "rt-multi-thread")))]
        pub fn new() -> std::io::Result<Runtime> {
            Builder::new_multi_thread().enable_all().build()
        }

        /// Returns a handle to the runtime's spawner.
        ///
        /// The returned handle can be used to spawn tasks that run on this runtime, and can
        /// be cloned to allow moving the `Handle` to other threads.
        ///
        /// # Examples
        ///
        /// ```
        /// use tokio::runtime::Runtime;
        ///
        /// let rt = Runtime::new()
        ///     .unwrap();
        ///
        /// let handle = rt.handle();
        ///
        /// // Use the handle...
        /// ```
        pub fn handle(&self) -> &Handle {
            &self.handle
        }

        /// Spawns a future onto the Tokio runtime.
        ///
        /// This spawns the given future onto the runtime's executor, usually a
        /// thread pool. The thread pool is then responsible for polling the future
        /// until it completes.
        ///
        /// See [module level][mod] documentation for more details.
        ///
        /// [mod]: index.html
        ///
        /// # Examples
        ///
        /// ```
        /// use tokio::runtime::Runtime;
        ///
        /// # fn dox() {
        /// // Create the runtime
        /// let rt = Runtime::new().unwrap();
        ///
        /// // Spawn a future onto the runtime
        /// rt.spawn(async {
        ///     println!("now running on a worker thread");
        /// });
        /// # }
        /// ```
        #[track_caller]
        pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
        where
            F: Future + Send + 'static,
            F::Output: Send + 'static,
        {
            self.handle.spawn(future)
        }

        /// Runs the provided function on an executor dedicated to blocking operations.
        ///
        /// # Examples
        ///
        /// ```
        /// use tokio::runtime::Runtime;
        ///
        /// # fn dox() {
        /// // Create the runtime
        /// let rt = Runtime::new().unwrap();
        ///
        /// // Spawn a blocking function onto the runtime
        /// rt.spawn_blocking(|| {
        ///     println!("now running on a worker thread");
        /// });
        /// # }
        #[track_caller]
        pub fn spawn_blocking<F, R>(&self, func: F) -> JoinHandle<R>
        where
            F: FnOnce() -> R + Send + 'static,
            R: Send + 'static,
        {
            self.handle.spawn_blocking(func)
        }

        /// Runs a future to completion on the Tokio runtime. This is the
        /// runtime's entry point.
        ///
        /// This runs the given future on the current thread, blocking until it is
        /// complete, and yielding its resolved result. Any tasks or timers
        /// which the future spawns internally will be executed on the runtime.
        ///
        /// # Multi thread scheduler
        ///
        /// When the multi thread scheduler is used this will allow futures
        /// to run within the io driver and timer context of the overall runtime.
        ///
        /// Any spawned tasks will continue running after `block_on` returns.
        ///
        /// # Current thread scheduler
        ///
        /// When the current thread scheduler is enabled `block_on`
        /// can be called concurrently from multiple threads. The first call
        /// will take ownership of the io and timer drivers. This means
        /// other threads which do not own the drivers will hook into that one.
        /// When the first `block_on` completes, other threads will be able to
        /// "steal" the driver to allow continued execution of their futures.
        ///
        /// Any spawned tasks will be suspended after `block_on` returns. Calling
        /// `block_on` again will resume previously spawned tasks.
        ///
        /// # Panics
        ///
        /// This function panics if the provided future panics, or if called within an
        /// asynchronous execution context.
        ///
        /// # Examples
        ///
        /// ```no_run
        /// use tokio::runtime::Runtime;
        ///
        /// // Create the runtime
        /// let rt  = Runtime::new().unwrap();
        ///
        /// // Execute the future, blocking the current thread until completion
        /// rt.block_on(async {
        ///     println!("hello");
        /// });
        /// ```
        ///
        /// [handle]: fn@Handle::block_on
        pub fn block_on<F: Future>(&self, future: F) -> F::Output {
            #[cfg(all(tokio_unstable, feature = "tracing"))]
            let future = crate::util::trace::task(future, "block_on", None, task::Id::next().as_u64());

            let _enter = self.enter();

            match &self.kind {
                Kind::CurrentThread(exec) => exec.block_on(future),
                #[cfg(feature = "rt-multi-thread")]
                Kind::ThreadPool(exec) => exec.block_on(future),
            }
        }

        /// Enters the runtime context.
        ///
        /// This allows you to construct types that must have an executor
        /// available on creation such as [`Sleep`] or [`TcpStream`]. It will
        /// also allow you to call methods such as [`tokio::spawn`].
        ///
        /// [`Sleep`]: struct@crate::time::Sleep
        /// [`TcpStream`]: struct@crate::net::TcpStream
        /// [`tokio::spawn`]: fn@crate::spawn
        ///
        /// # Example
        ///
        /// ```
        /// use tokio::runtime::Runtime;
        ///
        /// fn function_that_spawns(msg: String) {
        ///     // Had we not used `rt.enter` below, this would panic.
        ///     tokio::spawn(async move {
        ///         println!("{}", msg);
        ///     });
        /// }
        ///
        /// fn main() {
        ///     let rt = Runtime::new().unwrap();
        ///
        ///     let s = "Hello World!".to_string();
        ///
        ///     // By entering the context, we tie `tokio::spawn` to this executor.
        ///     let _guard = rt.enter();
        ///     function_that_spawns(s);
        /// }
        /// ```
        pub fn enter(&self) -> EnterGuard<'_> {
            self.handle.enter()
        }

        /// Shuts down the runtime, waiting for at most `duration` for all spawned
        /// task to shutdown.
        ///
        /// Usually, dropping a `Runtime` handle is sufficient as tasks are able to
        /// shutdown in a timely fashion. However, dropping a `Runtime` will wait
        /// indefinitely for all tasks to terminate, and there are cases where a long
        /// blocking task has been spawned, which can block dropping `Runtime`.
        ///
        /// In this case, calling `shutdown_timeout` with an explicit wait timeout
        /// can work. The `shutdown_timeout` will signal all tasks to shutdown and
        /// will wait for at most `duration` for all spawned tasks to terminate. If
        /// `timeout` elapses before all tasks are dropped, the function returns and
        /// outstanding tasks are potentially leaked.
        ///
        /// # Examples
        ///
        /// ```
        /// use tokio::runtime::Runtime;
        /// use tokio::task;
        ///
        /// use std::thread;
        /// use std::time::Duration;
        ///
        /// fn main() {
        ///    let runtime = Runtime::new().unwrap();
        ///
        ///    runtime.block_on(async move {
        ///        task::spawn_blocking(move || {
        ///            thread::sleep(Duration::from_secs(10_000));
        ///        });
        ///    });
        ///
        ///    runtime.shutdown_timeout(Duration::from_millis(100));
        /// }
        /// ```
        pub fn shutdown_timeout(mut self, duration: Duration) {
            // Wakeup and shutdown all the worker threads
            self.handle.clone().shutdown();
            self.blocking_pool.shutdown(Some(duration));
        }

        /// Shuts down the runtime, without waiting for any spawned tasks to shutdown.
        ///
        /// This can be useful if you want to drop a runtime from within another runtime.
        /// Normally, dropping a runtime will block indefinitely for spawned blocking tasks
        /// to complete, which would normally not be permitted within an asynchronous context.
        /// By calling `shutdown_background()`, you can drop the runtime from such a context.
        ///
        /// Note however, that because we do not wait for any blocking tasks to complete, this
        /// may result in a resource leak (in that any blocking tasks are still running until they
        /// return.
        ///
        /// This function is equivalent to calling `shutdown_timeout(Duration::of_nanos(0))`.
        ///
        /// ```
        /// use tokio::runtime::Runtime;
        ///
        /// fn main() {
        ///    let runtime = Runtime::new().unwrap();
        ///
        ///    runtime.block_on(async move {
        ///        let inner_runtime = Runtime::new().unwrap();
        ///        // ...
        ///        inner_runtime.shutdown_background();
        ///    });
        /// }
        /// ```
        pub fn shutdown_background(self) {
            self.shutdown_timeout(Duration::from_nanos(0))
        }
    }

    #[allow(clippy::single_match)] // there are comments in the error branch, so we don't want if-let
    impl Drop for Runtime {
        fn drop(&mut self) {
            match &mut self.kind {
                Kind::CurrentThread(basic) => {
                    // This ensures that tasks spawned on the basic runtime are dropped inside the
                    // runtime's context.
                    match self::context::try_enter(self.handle.clone()) {
                        Some(guard) => basic.set_context_guard(guard),
                        None => {
                            // The context thread-local has already been destroyed.
                            //
                            // We don't set the guard in this case. Calls to tokio::spawn in task
                            // destructors would fail regardless if this happens.
                        },
                    }
                },
                #[cfg(feature = "rt-multi-thread")]
                Kind::ThreadPool(_) => {
                    // The threaded scheduler drops its tasks on its worker threads, which is
                    // already in the runtime's context.
                },
            }
        }
    }

    cfg_metrics! {
        impl Runtime {
            /// TODO
            pub fn metrics(&self) -> RuntimeMetrics {
                self.handle.metrics()
            }
        }
    }
}
