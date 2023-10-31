use crate::runtime::handle::Handle;
use crate::runtime::{blocking, driver, Callback, Runtime, Spawner};

use std::fmt;
use std::io;
use std::time::Duration;

/// Builds Tokio Runtime with custom configuration values.
///
/// Methods can be chained in order to set the configuration values. The
/// Runtime is constructed by calling [`build`].
///
/// New instances of `Builder` are obtained via [`Builder::new_multi_thread`]
/// or [`Builder::new_current_thread`].
///
/// See function level documentation for details on the various configuration
/// settings.
///
/// [`build`]: method@Self::build
/// [`Builder::new_multi_thread`]: method@Self::new_multi_thread
/// [`Builder::new_current_thread`]: method@Self::new_current_thread
///
/// # Examples
///
/// ```
/// use tokio::runtime::Builder;
///
/// fn main() {
///     // build runtime
///     let runtime = Builder::new_multi_thread()
///         .worker_threads(4)
///         .thread_name("my-custom-name")
///         .thread_stack_size(3 * 1024 * 1024)
///         .build()
///         .unwrap();
///
///     // use runtime ...
/// }
/// ```
pub struct Builder {
    /// Runtime type
    kind: Kind,

    /// Whether or not to enable the I/O driver
    enable_io: bool,

    /// Whether or not to enable the time driver
    enable_time: bool,

    /// Whether or not the clock should start paused.
    start_paused: bool,

    /// The number of worker threads, used by Runtime.
    ///
    /// Only used when not using the current-thread executor.
    worker_threads: Option<usize>,

    /// Cap on thread usage.
    max_blocking_threads: usize,

    /// Name fn used for threads spawned by the runtime.
    pub(super) thread_name: ThreadNameFn,

    /// Stack size used for threads spawned by the runtime.
    pub(super) thread_stack_size: Option<usize>,

    /// Callback to run after each thread starts.
    pub(super) after_start: Option<Callback>,

    /// To run before each worker thread stops
    pub(super) before_stop: Option<Callback>,

    /// To run before each worker thread is parked.
    pub(super) before_park: Option<Callback>,

    /// To run after each thread is unparked.
    pub(super) after_unpark: Option<Callback>,

    /// Customizable keep alive timeout for BlockingPool
    pub(super) keep_alive: Option<Duration>,

    /// How many ticks before pulling a task from the global/remote queue?
    pub(super) global_queue_interval: u32,

    /// How many ticks before yielding to the driver for timer and I/O events?
    pub(super) event_interval: u32,

    #[cfg(tokio_unstable)]
    pub(super) unhandled_panic: UnhandledPanic,
}

cfg_unstable! {
    /// How the runtime should respond to unhandled panics.
    ///
    /// Instances of `UnhandledPanic` are passed to `Builder::unhandled_panic`
    /// to configure the runtime behavior when a spawned task panics.
    ///
    /// See [`Builder::unhandled_panic`] for more details.
    #[derive(Debug, Clone)]
    #[non_exhaustive]
    pub enum UnhandledPanic {
        /// The runtime should ignore panics on spawned tasks.
        ///
        /// The panic is forwarded to the task's [`JoinHandle`] and all spawned
        /// tasks continue running normally.
        ///
        /// This is the default behavior.
        ///
        /// # Examples
        ///
        /// ```
        /// use tokio::runtime::{self, UnhandledPanic};
        ///
        /// # pub fn main() {
        /// let rt = runtime::Builder::new_current_thread()
        ///     .unhandled_panic(UnhandledPanic::Ignore)
        ///     .build()
        ///     .unwrap();
        ///
        /// let task1 = rt.spawn(async { panic!("boom"); });
        /// let task2 = rt.spawn(async {
        ///     // This task completes normally
        ///     "done"
        /// });
        ///
        /// rt.block_on(async {
        ///     // The panic on the first task is forwarded to the `JoinHandle`
        ///     assert!(task1.await.is_err());
        ///
        ///     // The second task completes normally
        ///     assert!(task2.await.is_ok());
        /// })
        /// # }
        /// ```
        ///
        /// [`JoinHandle`]: struct@crate::task::JoinHandle
        Ignore,

        /// The runtime should immediately shutdown if a spawned task panics.
        ///
        /// The runtime will immediately shutdown even if the panicked task's
        /// [`JoinHandle`] is still available. All further spawned tasks will be
        /// immediately dropped and call to [`Runtime::block_on`] will panic.
        ///
        /// # Examples
        ///
        /// ```should_panic
        /// use tokio::runtime::{self, UnhandledPanic};
        ///
        /// # pub fn main() {
        /// let rt = runtime::Builder::new_current_thread()
        ///     .unhandled_panic(UnhandledPanic::ShutdownRuntime)
        ///     .build()
        ///     .unwrap();
        ///
        /// rt.spawn(async { panic!("boom"); });
        /// rt.spawn(async {
        ///     // This task never completes.
        /// });
        ///
        /// rt.block_on(async {
        ///     // Do some work
        /// # loop { tokio::task::yield_now().await; }
        /// })
        /// # }
        /// ```
        ///
        /// [`JoinHandle`]: struct@crate::task::JoinHandle
        ShutdownRuntime,
    }
}

pub(crate) type ThreadNameFn = std::sync::Arc<dyn Fn() -> String + Send + Sync + 'static>;

pub(crate) enum Kind {
    CurrentThread,
    #[cfg(feature = "rt-multi-thread")]
    MultiThread,
}

impl Builder {
    /// Returns a new builder with the current thread scheduler selected.
    ///
    /// Configuration methods can be chained on the return value.
    ///
    /// To spawn non-`Send` tasks on the resulting runtime, combine it with a
    /// [`LocalSet`].
    ///
    /// [`LocalSet`]: crate::task::LocalSet
    pub fn new_current_thread() -> Builder {
        #[cfg(loom)]
        const EVENT_INTERVAL: u32 = 4;
        // The number `61` is fairly arbitrary. I believe this value was copied from golang.
        #[cfg(not(loom))]
        const EVENT_INTERVAL: u32 = 61;

        Builder::new(Kind::CurrentThread, 31, EVENT_INTERVAL)
    }

    /// Returns a new builder with the multi thread scheduler selected.
    ///
    /// Configuration methods can be chained on the return value.
    #[cfg(feature = "rt-multi-thread")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rt-multi-thread")))]
    pub fn new_multi_thread() -> Builder {
        // The number `61` is fairly arbitrary. I believe this value was copied from golang.
        Builder::new(Kind::MultiThread, 61, 61)
    }

    /// Returns a new runtime builder initialized with default configuration
    /// values.
    ///
    /// Configuration methods can be chained on the return value.
    pub(crate) fn new(kind: Kind, global_queue_interval: u32, event_interval: u32) -> Builder {
        Builder {
            kind,

            // I/O defaults to "off"
            enable_io: false,

            // Time defaults to "off"
            enable_time: false,

            // The clock starts not-paused
            start_paused: false,

            // Default to lazy auto-detection (one thread per CPU core)
            worker_threads: None,

            max_blocking_threads: 512,

            // Default thread name
            thread_name: std::sync::Arc::new(|| "tokio-runtime-worker".into()),

            // Do not set a stack size by default
            thread_stack_size: None,

            // No worker thread callbacks
            after_start: None,
            before_stop: None,
            before_park: None,
            after_unpark: None,

            keep_alive: None,

            // Defaults for these values depend on the scheduler kind, so we get them
            // as parameters.
            global_queue_interval,
            event_interval,

            #[cfg(tokio_unstable)]
            unhandled_panic: UnhandledPanic::Ignore,
        }
    }

    /// Enables both I/O and time drivers.
    ///
    /// Doing this is a shorthand for calling `enable_io` and `enable_time`
    /// individually. If additional components are added to Tokio in the future,
    /// `enable_all` will include these future components.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime;
    ///
    /// let rt = runtime::Builder::new_multi_thread()
    ///     .enable_all()
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn enable_all(&mut self) -> &mut Self {
        #[cfg(any(feature = "net", feature = "process", all(unix, feature = "signal")))]
        self.enable_io();
        #[cfg(feature = "time")]
        self.enable_time();

        self
    }

    /// Sets the number of worker threads the `Runtime` will use.
    ///
    /// This can be any number above 0 though it is advised to keep this value
    /// on the smaller side.
    ///
    /// # Default
    ///
    /// The default value is the number of cores available to the system.
    ///
    /// # Panics
    ///
    /// When using the `current_thread` runtime this method will panic, since
    /// those variants do not allow setting worker thread counts.
    ///
    ///
    /// # Examples
    ///
    /// ## Multi threaded runtime with 4 threads
    ///
    /// ```
    /// use tokio::runtime;
    ///
    /// // This will spawn a work-stealing runtime with 4 worker threads.
    /// let rt = runtime::Builder::new_multi_thread()
    ///     .worker_threads(4)
    ///     .build()
    ///     .unwrap();
    ///
    /// rt.spawn(async move {});
    /// ```
    ///
    /// ## Current thread runtime (will only run on the current thread via `Runtime::block_on`)
    ///
    /// ```
    /// use tokio::runtime;
    ///
    /// // Create a runtime that _must_ be driven from a call
    /// // to `Runtime::block_on`.
    /// let rt = runtime::Builder::new_current_thread()
    ///     .build()
    ///     .unwrap();
    ///
    /// // This will run the runtime and future on the current thread
    /// rt.block_on(async move {});
    /// ```
    ///
    /// # Panics
    ///
    /// This will panic if `val` is not larger than `0`.
    #[track_caller]
    pub fn worker_threads(&mut self, val: usize) -> &mut Self {
        assert!(val > 0, "Worker threads cannot be set to 0");
        self.worker_threads = Some(val);
        self
    }

    /// Specifies the limit for additional threads spawned by the Runtime.
    ///
    /// These threads are used for blocking operations like tasks spawned
    /// through [`spawn_blocking`]. Unlike the [`worker_threads`], they are not
    /// always active and will exit if left idle for too long. You can change
    /// this timeout duration with [`thread_keep_alive`].
    ///
    /// The default value is 512.
    ///
    /// # Panics
    ///
    /// This will panic if `val` is not larger than `0`.
    ///
    /// # Upgrading from 0.x
    ///
    /// In old versions `max_threads` limited both blocking and worker threads, but the
    /// current `max_blocking_threads` does not include async worker threads in the count.
    ///
    /// [`spawn_blocking`]: fn@crate::task::spawn_blocking
    /// [`worker_threads`]: Self::worker_threads
    /// [`thread_keep_alive`]: Self::thread_keep_alive
    #[track_caller]
    #[cfg_attr(docsrs, doc(alias = "max_threads"))]
    pub fn max_blocking_threads(&mut self, val: usize) -> &mut Self {
        assert!(val > 0, "Max blocking threads cannot be set to 0");
        self.max_blocking_threads = val;
        self
    }

    /// Sets name of threads spawned by the `Runtime`'s thread pool.
    ///
    /// The default name is "tokio-runtime-worker".
    ///
    /// # Examples
    ///
    /// ```
    /// # use tokio::runtime;
    ///
    /// # pub fn main() {
    /// let rt = runtime::Builder::new_multi_thread()
    ///     .thread_name("my-pool")
    ///     .build();
    /// # }
    /// ```
    pub fn thread_name(&mut self, val: impl Into<String>) -> &mut Self {
        let val = val.into();
        self.thread_name = std::sync::Arc::new(move || val.clone());
        self
    }

    /// Sets a function used to generate the name of threads spawned by the `Runtime`'s thread pool.
    ///
    /// The default name fn is `|| "tokio-runtime-worker".into()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tokio::runtime;
    /// # use std::sync::atomic::{AtomicUsize, Ordering};
    /// # pub fn main() {
    /// let rt = runtime::Builder::new_multi_thread()
    ///     .thread_name_fn(|| {
    ///        static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
    ///        let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
    ///        format!("my-pool-{}", id)
    ///     })
    ///     .build();
    /// # }
    /// ```
    pub fn thread_name_fn<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.thread_name = std::sync::Arc::new(f);
        self
    }

    /// Sets the stack size (in bytes) for worker threads.
    ///
    /// The actual stack size may be greater than this value if the platform
    /// specifies minimal stack size.
    ///
    /// The default stack size for spawned threads is 2 MiB, though this
    /// particular stack size is subject to change in the future.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tokio::runtime;
    ///
    /// # pub fn main() {
    /// let rt = runtime::Builder::new_multi_thread()
    ///     .thread_stack_size(32 * 1024)
    ///     .build();
    /// # }
    /// ```
    pub fn thread_stack_size(&mut self, val: usize) -> &mut Self {
        self.thread_stack_size = Some(val);
        self
    }

    /// Executes function `f` after each thread is started but before it starts
    /// doing work.
    ///
    /// This is intended for bookkeeping and monitoring use cases.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tokio::runtime;
    /// # pub fn main() {
    /// let runtime = runtime::Builder::new_multi_thread()
    ///     .on_thread_start(|| {
    ///         println!("thread started");
    ///     })
    ///     .build();
    /// # }
    /// ```
    #[cfg(not(loom))]
    pub fn on_thread_start<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.after_start = Some(std::sync::Arc::new(f));
        self
    }

    /// Executes function `f` before each thread stops.
    ///
    /// This is intended for bookkeeping and monitoring use cases.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tokio::runtime;
    /// # pub fn main() {
    /// let runtime = runtime::Builder::new_multi_thread()
    ///     .on_thread_stop(|| {
    ///         println!("thread stopping");
    ///     })
    ///     .build();
    /// # }
    /// ```
    #[cfg(not(loom))]
    pub fn on_thread_stop<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.before_stop = Some(std::sync::Arc::new(f));
        self
    }

    /// Executes function `f` just before a thread is parked (goes idle).
    /// `f` is called within the Tokio context, so functions like [`tokio::spawn`](crate::spawn)
    /// can be called, and may result in this thread being unparked immediately.
    ///
    /// This can be used to start work only when the executor is idle, or for bookkeeping
    /// and monitoring purposes.
    ///
    /// Note: There can only be one park callback for a runtime; calling this function
    /// more than once replaces the last callback defined, rather than adding to it.
    ///
    /// # Examples
    ///
    /// ## Multithreaded executor
    /// ```
    /// # use std::sync::Arc;
    /// # use std::sync::atomic::{AtomicBool, Ordering};
    /// # use tokio::runtime;
    /// # use tokio::sync::Barrier;
    /// # pub fn main() {
    /// let once = AtomicBool::new(true);
    /// let barrier = Arc::new(Barrier::new(2));
    ///
    /// let runtime = runtime::Builder::new_multi_thread()
    ///     .worker_threads(1)
    ///     .on_thread_park({
    ///         let barrier = barrier.clone();
    ///         move || {
    ///             let barrier = barrier.clone();
    ///             if once.swap(false, Ordering::Relaxed) {
    ///                 tokio::spawn(async move { barrier.wait().await; });
    ///            }
    ///         }
    ///     })
    ///     .build()
    ///     .unwrap();
    ///
    /// runtime.block_on(async {
    ///    barrier.wait().await;
    /// })
    /// # }
    /// ```
    /// ## Current thread executor
    /// ```
    /// # use std::sync::Arc;
    /// # use std::sync::atomic::{AtomicBool, Ordering};
    /// # use tokio::runtime;
    /// # use tokio::sync::Barrier;
    /// # pub fn main() {
    /// let once = AtomicBool::new(true);
    /// let barrier = Arc::new(Barrier::new(2));
    ///
    /// let runtime = runtime::Builder::new_current_thread()
    ///     .on_thread_park({
    ///         let barrier = barrier.clone();
    ///         move || {
    ///             let barrier = barrier.clone();
    ///             if once.swap(false, Ordering::Relaxed) {
    ///                 tokio::spawn(async move { barrier.wait().await; });
    ///            }
    ///         }
    ///     })
    ///     .build()
    ///     .unwrap();
    ///
    /// runtime.block_on(async {
    ///    barrier.wait().await;
    /// })
    /// # }
    /// ```
    #[cfg(not(loom))]
    pub fn on_thread_park<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.before_park = Some(std::sync::Arc::new(f));
        self
    }

    /// Executes function `f` just after a thread unparks (starts executing tasks).
    ///
    /// This is intended for bookkeeping and monitoring use cases; note that work
    /// in this callback will increase latencies when the application has allowed one or
    /// more runtime threads to go idle.
    ///
    /// Note: There can only be one unpark callback for a runtime; calling this function
    /// more than once replaces the last callback defined, rather than adding to it.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tokio::runtime;
    /// # pub fn main() {
    /// let runtime = runtime::Builder::new_multi_thread()
    ///     .on_thread_unpark(|| {
    ///         println!("thread unparking");
    ///     })
    ///     .build();
    ///
    /// runtime.unwrap().block_on(async {
    ///    tokio::task::yield_now().await;
    ///    println!("Hello from Tokio!");
    /// })
    /// # }
    /// ```
    #[cfg(not(loom))]
    pub fn on_thread_unpark<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.after_unpark = Some(std::sync::Arc::new(f));
        self
    }

    /// Creates the configured `Runtime`.
    ///
    /// The returned `Runtime` instance is ready to spawn tasks.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime::Builder;
    ///
    /// let rt  = Builder::new_multi_thread().build().unwrap();
    ///
    /// rt.block_on(async {
    ///     println!("Hello from the Tokio runtime");
    /// });
    /// ```
    pub fn build(&mut self) -> io::Result<Runtime> {
        match &self.kind {
            Kind::CurrentThread => self.build_basic_runtime(),
            #[cfg(feature = "rt-multi-thread")]
            Kind::MultiThread => self.build_threaded_runtime(),
        }
    }

    fn get_cfg(&self) -> driver::Cfg {
        driver::Cfg {
            enable_pause_time: match self.kind {
                Kind::CurrentThread => true,
                #[cfg(feature = "rt-multi-thread")]
                Kind::MultiThread => false,
            },
            enable_io: self.enable_io,
            enable_time: self.enable_time,
            start_paused: self.start_paused,
        }
    }

    /// Sets a custom timeout for a thread in the blocking pool.
    ///
    /// By default, the timeout for a thread is set to 10 seconds. This can
    /// be overridden using .thread_keep_alive().
    ///
    /// # Example
    ///
    /// ```
    /// # use tokio::runtime;
    /// # use std::time::Duration;
    /// # pub fn main() {
    /// let rt = runtime::Builder::new_multi_thread()
    ///     .thread_keep_alive(Duration::from_millis(100))
    ///     .build();
    /// # }
    /// ```
    pub fn thread_keep_alive(&mut self, duration: Duration) -> &mut Self {
        self.keep_alive = Some(duration);
        self
    }

    /// Sets the number of scheduler ticks after which the scheduler will poll the global
    /// task queue.
    ///
    /// A scheduler "tick" roughly corresponds to one `poll` invocation on a task.
    ///
    /// By default the global queue interval is:
    ///
    /// * `31` for the current-thread scheduler.
    /// * `61` for the multithreaded scheduler.
    ///
    /// Schedulers have a local queue of already-claimed tasks, and a global queue of incoming
    /// tasks. Setting the interval to a smaller value increases the fairness of the scheduler,
    /// at the cost of more synchronization overhead. That can be beneficial for prioritizing
    /// getting started on new work, especially if tasks frequently yield rather than complete
    /// or await on further I/O. Conversely, a higher value prioritizes existing work, and
    /// is a good choice when most tasks quickly complete polling.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tokio::runtime;
    /// # pub fn main() {
    /// let rt = runtime::Builder::new_multi_thread()
    ///     .global_queue_interval(31)
    ///     .build();
    /// # }
    /// ```
    pub fn global_queue_interval(&mut self, val: u32) -> &mut Self {
        self.global_queue_interval = val;
        self
    }

    /// Sets the number of scheduler ticks after which the scheduler will poll for
    /// external events (timers, I/O, and so on).
    ///
    /// A scheduler "tick" roughly corresponds to one `poll` invocation on a task.
    ///
    /// By default, the event interval is `61` for all scheduler types.
    ///
    /// Setting the event interval determines the effective "priority" of delivering
    /// these external events (which may wake up additional tasks), compared to
    /// executing tasks that are currently ready to run. A smaller value is useful
    /// when tasks frequently spend a long time in polling, or frequently yield,
    /// which can result in overly long delays picking up I/O events. Conversely,
    /// picking up new events requires extra synchronization and syscall overhead,
    /// so if tasks generally complete their polling quickly, a higher event interval
    /// will minimize that overhead while still keeping the scheduler responsive to
    /// events.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tokio::runtime;
    /// # pub fn main() {
    /// let rt = runtime::Builder::new_multi_thread()
    ///     .event_interval(31)
    ///     .build();
    /// # }
    /// ```
    pub fn event_interval(&mut self, val: u32) -> &mut Self {
        self.event_interval = val;
        self
    }

    cfg_unstable! {
        /// Configure how the runtime responds to an unhandled panic on a
        /// spawned task.
        ///
        /// By default, an unhandled panic (i.e. a panic not caught by
        /// [`std::panic::catch_unwind`]) has no impact on the runtime's
        /// execution. The panic is error value is forwarded to the task's
        /// [`JoinHandle`] and all other spawned tasks continue running.
        ///
        /// The `unhandled_panic` option enables configuring this behavior.
        ///
        /// * `UnhandledPanic::Ignore` is the default behavior. Panics on
        ///   spawned tasks have no impact on the runtime's execution.
        /// * `UnhandledPanic::ShutdownRuntime` will force the runtime to
        ///   shutdown immediately when a spawned task panics even if that
        ///   task's `JoinHandle` has not been dropped. All other spawned tasks
        ///   will immediatetly terminate and further calls to
        ///   [`Runtime::block_on`] will panic.
        ///
        /// # Unstable
        ///
        /// This option is currently unstable and its implementation is
        /// incomplete. The API may change or be removed in the future. See
        /// tokio-rs/tokio#4516 for more details.
        ///
        /// # Examples
        ///
        /// The following demonstrates a runtime configured to shutdown on
        /// panic. The first spawned task panics and results in the runtime
        /// shutting down. The second spawned task never has a chance to
        /// execute. The call to `block_on` will panic due to the runtime being
        /// forcibly shutdown.
        ///
        /// ```should_panic
        /// use tokio::runtime::{self, UnhandledPanic};
        ///
        /// # pub fn main() {
        /// let rt = runtime::Builder::new_current_thread()
        ///     .unhandled_panic(UnhandledPanic::ShutdownRuntime)
        ///     .build()
        ///     .unwrap();
        ///
        /// rt.spawn(async { panic!("boom"); });
        /// rt.spawn(async {
        ///     // This task never completes.
        /// });
        ///
        /// rt.block_on(async {
        ///     // Do some work
        /// # loop { tokio::task::yield_now().await; }
        /// })
        /// # }
        /// ```
        ///
        /// [`JoinHandle`]: struct@crate::task::JoinHandle
        pub fn unhandled_panic(&mut self, behavior: UnhandledPanic) -> &mut Self {
            self.unhandled_panic = behavior;
            self
        }
    }

    fn build_basic_runtime(&mut self) -> io::Result<Runtime> {
        use crate::runtime::basic_scheduler::Config;
        use crate::runtime::{BasicScheduler, HandleInner, Kind};

        let (driver, resources) = driver::Driver::new(self.get_cfg())?;

        // Blocking pool
        let blocking_pool = blocking::create_blocking_pool(self, self.max_blocking_threads);
        let blocking_spawner = blocking_pool.spawner().clone();

        let handle_inner = HandleInner {
            io_handle: resources.io_handle,
            time_handle: resources.time_handle,
            signal_handle: resources.signal_handle,
            clock: resources.clock,
            blocking_spawner,
        };

        // And now put a single-threaded scheduler on top of the timer. When
        // there are no futures ready to do something, it'll let the timer or
        // the reactor to generate some new stimuli for the futures to continue
        // in their life.
        let scheduler = BasicScheduler::new(
            driver,
            handle_inner,
            Config {
                before_park: self.before_park.clone(),
                after_unpark: self.after_unpark.clone(),
                global_queue_interval: self.global_queue_interval,
                event_interval: self.event_interval,
                #[cfg(tokio_unstable)]
                unhandled_panic: self.unhandled_panic.clone(),
            },
        );
        let spawner = Spawner::Basic(scheduler.spawner().clone());

        Ok(Runtime {
            kind: Kind::CurrentThread(scheduler),
            handle: Handle { spawner },
            blocking_pool,
        })
    }
}

cfg_io_driver! {
    impl Builder {
        /// Enables the I/O driver.
        ///
        /// Doing this enables using net, process, signal, and some I/O types on
        /// the runtime.
        ///
        /// # Examples
        ///
        /// ```
        /// use tokio::runtime;
        ///
        /// let rt = runtime::Builder::new_multi_thread()
        ///     .enable_io()
        ///     .build()
        ///     .unwrap();
        /// ```
        pub fn enable_io(&mut self) -> &mut Self {
            self.enable_io = true;
            self
        }
    }
}

cfg_time! {
    impl Builder {
        /// Enables the time driver.
        ///
        /// Doing this enables using `tokio::time` on the runtime.
        ///
        /// # Examples
        ///
        /// ```
        /// use tokio::runtime;
        ///
        /// let rt = runtime::Builder::new_multi_thread()
        ///     .enable_time()
        ///     .build()
        ///     .unwrap();
        /// ```
        pub fn enable_time(&mut self) -> &mut Self {
            self.enable_time = true;
            self
        }
    }
}

cfg_test_util! {
    impl Builder {
        /// Controls if the runtime's clock starts paused or advancing.
        ///
        /// Pausing time requires the current-thread runtime; construction of
        /// the runtime will panic otherwise.
        ///
        /// # Examples
        ///
        /// ```
        /// use tokio::runtime;
        ///
        /// let rt = runtime::Builder::new_current_thread()
        ///     .enable_time()
        ///     .start_paused(true)
        ///     .build()
        ///     .unwrap();
        /// ```
        pub fn start_paused(&mut self, start_paused: bool) -> &mut Self {
            self.start_paused = start_paused;
            self
        }
    }
}

cfg_rt_multi_thread! {
    impl Builder {
        fn build_threaded_runtime(&mut self) -> io::Result<Runtime> {
            use crate::loom::sys::num_cpus;
            use crate::runtime::{HandleInner, Kind, ThreadPool};

            let core_threads = self.worker_threads.unwrap_or_else(num_cpus);

            let (driver, resources) = driver::Driver::new(self.get_cfg())?;

            // Create the blocking pool
            let blocking_pool =
                blocking::create_blocking_pool(self, self.max_blocking_threads + core_threads);
            let blocking_spawner = blocking_pool.spawner().clone();

            let handle_inner = HandleInner {
                io_handle: resources.io_handle,
                time_handle: resources.time_handle,
                signal_handle: resources.signal_handle,
                clock: resources.clock,
                blocking_spawner,
            };

            let (scheduler, launch) = ThreadPool::new(
                core_threads,
                driver,
                handle_inner,
                self.before_park.clone(),
                self.after_unpark.clone(),
                self.global_queue_interval,
                self.event_interval,
            );
            let spawner = Spawner::ThreadPool(scheduler.spawner().clone());

            // Create the runtime handle
            let handle = Handle { spawner };

            // Spawn the thread pool workers
            let _enter = crate::runtime::context::enter(handle.clone());
            launch.launch();

            Ok(Runtime {
                kind: Kind::ThreadPool(scheduler),
                handle,
                blocking_pool,
            })
        }
    }
}

impl fmt::Debug for Builder {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Builder")
            .field("worker_threads", &self.worker_threads)
            .field("max_blocking_threads", &self.max_blocking_threads)
            .field(
                "thread_name",
                &"<dyn Fn() -> String + Send + Sync + 'static>",
            )
            .field("thread_stack_size", &self.thread_stack_size)
            .field("after_start", &self.after_start.as_ref().map(|_| "..."))
            .field("before_stop", &self.before_stop.as_ref().map(|_| "..."))
            .field("before_park", &self.before_park.as_ref().map(|_| "..."))
            .field("after_unpark", &self.after_unpark.as_ref().map(|_| "..."))
            .finish()
    }
}
