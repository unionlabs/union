use crate::future::poll_fn;
use crate::loom::sync::atomic::AtomicBool;
use crate::loom::sync::{Arc, Mutex};
use crate::park::{Park, Unpark};
use crate::runtime::context::EnterGuard;
use crate::runtime::driver::Driver;
use crate::runtime::task::{self, JoinHandle, OwnedTasks, Schedule, Task};
use crate::runtime::{Callback, HandleInner};
use crate::runtime::{MetricsBatch, SchedulerMetrics, WorkerMetrics};
use crate::sync::notify::Notify;
use crate::util::atomic_cell::AtomicCell;
use crate::util::{waker_ref, Wake, WakerRef};

use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt;
use std::future::Future;
use std::sync::atomic::Ordering::{AcqRel, Release};
use std::task::Poll::{Pending, Ready};
use std::time::Duration;

/// Executes tasks on the current thread
pub(crate) struct BasicScheduler {
    /// Core scheduler data is acquired by a thread entering `block_on`.
    core: AtomicCell<Core>,

    /// Notifier for waking up other threads to steal the
    /// driver.
    notify: Notify,

    /// Sendable task spawner
    spawner: Spawner,

    /// This is usually None, but right before dropping the BasicScheduler, it
    /// is changed to `Some` with the context being the runtime's own context.
    /// This ensures that any tasks dropped in the `BasicScheduler`s destructor
    /// run in that runtime's context.
    context_guard: Option<EnterGuard>,
}

/// Data required for executing the scheduler. The struct is passed around to
/// a function that will perform the scheduling work and acts as a capability token.
struct Core {
    /// Scheduler run queue
    tasks: VecDeque<task::Notified<Arc<Shared>>>,

    /// Sendable task spawner
    spawner: Spawner,

    /// Current tick
    tick: u32,

    /// Runtime driver
    ///
    /// The driver is removed before starting to park the thread
    driver: Option<Driver>,

    /// Metrics batch
    metrics: MetricsBatch,

    /// True if a task panicked without being handled and the runtime is
    /// configured to shutdown on unhandled panic.
    unhandled_panic: bool,
}

#[derive(Clone)]
pub(crate) struct Spawner {
    shared: Arc<Shared>,
}

pub(crate) struct Config {
    /// How many ticks before pulling a task from the global/remote queue?
    pub(crate) global_queue_interval: u32,

    /// How many ticks before yielding to the driver for timer and I/O events?
    pub(crate) event_interval: u32,

    /// Callback for a worker parking itself
    pub(crate) before_park: Option<Callback>,

    /// Callback for a worker unparking itself
    pub(crate) after_unpark: Option<Callback>,

    #[cfg(tokio_unstable)]
    /// How to respond to unhandled task panics.
    pub(crate) unhandled_panic: crate::runtime::UnhandledPanic,
}

/// Scheduler state shared between threads.
struct Shared {
    /// Remote run queue. None if the `Runtime` has been dropped.
    queue: Mutex<Option<VecDeque<task::Notified<Arc<Shared>>>>>,

    /// Collection of all active tasks spawned onto this executor.
    owned: OwnedTasks<Arc<Shared>>,

    /// Unpark the blocked thread.
    unpark: <Driver as Park>::Unpark,

    /// Indicates whether the blocked on thread was woken.
    woken: AtomicBool,

    /// Handle to I/O driver, timer, blocking pool, ...
    handle_inner: HandleInner,

    /// Scheduler configuration options
    config: Config,

    /// Keeps track of various runtime metrics.
    scheduler_metrics: SchedulerMetrics,

    /// This scheduler only has one worker.
    worker_metrics: WorkerMetrics,
}

/// Thread-local context.
struct Context {
    /// Handle to the spawner
    spawner: Spawner,

    /// Scheduler core, enabling the holder of `Context` to execute the
    /// scheduler.
    core: RefCell<Option<Box<Core>>>,
}

/// Initial queue capacity.
const INITIAL_CAPACITY: usize = 64;

// Tracks the current BasicScheduler.
scoped_thread_local!(static CURRENT: Context);

impl BasicScheduler {
    pub(crate) fn new(driver: Driver, handle_inner: HandleInner, config: Config) -> BasicScheduler {
        let unpark = driver.unpark();

        let spawner = Spawner {
            shared: Arc::new(Shared {
                queue: Mutex::new(Some(VecDeque::with_capacity(INITIAL_CAPACITY))),
                owned: OwnedTasks::new(),
                unpark,
                woken: AtomicBool::new(false),
                handle_inner,
                config,
                scheduler_metrics: SchedulerMetrics::new(),
                worker_metrics: WorkerMetrics::new(),
            }),
        };

        let core = AtomicCell::new(Some(Box::new(Core {
            tasks: VecDeque::with_capacity(INITIAL_CAPACITY),
            spawner: spawner.clone(),
            tick: 0,
            driver: Some(driver),
            metrics: MetricsBatch::new(),
            unhandled_panic: false,
        })));

        BasicScheduler {
            core,
            notify: Notify::new(),
            spawner,
            context_guard: None,
        }
    }

    pub(crate) fn spawner(&self) -> &Spawner {
        &self.spawner
    }

    #[track_caller]
    pub(crate) fn block_on<F: Future>(&self, future: F) -> F::Output {
        pin!(future);

        // Attempt to steal the scheduler core and block_on the future if we can
        // there, otherwise, lets select on a notification that the core is
        // available or the future is complete.
        loop {
            if let Some(core) = self.take_core() {
                return core.block_on(future);
            } else {
                let mut enter = crate::runtime::enter(false);

                let notified = self.notify.notified();
                pin!(notified);

                if let Some(out) = enter
                    .block_on(poll_fn(|cx| {
                        if notified.as_mut().poll(cx).is_ready() {
                            return Ready(None);
                        }

                        if let Ready(out) = future.as_mut().poll(cx) {
                            return Ready(Some(out));
                        }

                        Pending
                    }))
                    .expect("Failed to `Enter::block_on`")
                {
                    return out;
                }
            }
        }
    }

    fn take_core(&self) -> Option<CoreGuard<'_>> {
        let core = self.core.take()?;

        Some(CoreGuard {
            context: Context {
                spawner: self.spawner.clone(),
                core: RefCell::new(Some(core)),
            },
            basic_scheduler: self,
        })
    }

    pub(super) fn set_context_guard(&mut self, guard: EnterGuard) {
        self.context_guard = Some(guard);
    }
}

impl Drop for BasicScheduler {
    fn drop(&mut self) {
        // Avoid a double panic if we are currently panicking and
        // the lock may be poisoned.

        let core = match self.take_core() {
            Some(core) => core,
            None if std::thread::panicking() => return,
            None => panic!("Oh no! We never placed the Core back, this is a bug!"),
        };

        core.enter(|mut core, context| {
            // Drain the OwnedTasks collection. This call also closes the
            // collection, ensuring that no tasks are ever pushed after this
            // call returns.
            context.spawner.shared.owned.close_and_shutdown_all();

            // Drain local queue
            // We already shut down every task, so we just need to drop the task.
            while let Some(task) = core.pop_task() {
                drop(task);
            }

            // Drain remote queue and set it to None
            let remote_queue = core.spawner.shared.queue.lock().take();

            // Using `Option::take` to replace the shared queue with `None`.
            // We already shut down every task, so we just need to drop the task.
            if let Some(remote_queue) = remote_queue {
                for task in remote_queue {
                    drop(task);
                }
            }

            assert!(context.spawner.shared.owned.is_empty());

            // Submit metrics
            core.metrics.submit(&core.spawner.shared.worker_metrics);

            (core, ())
        });
    }
}

impl fmt::Debug for BasicScheduler {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("BasicScheduler").finish()
    }
}

// ===== impl Core =====

impl Core {
    fn pop_task(&mut self) -> Option<task::Notified<Arc<Shared>>> {
        let ret = self.tasks.pop_front();
        self.spawner
            .shared
            .worker_metrics
            .set_queue_depth(self.tasks.len());
        ret
    }

    fn push_task(&mut self, task: task::Notified<Arc<Shared>>) {
        self.tasks.push_back(task);
        self.metrics.inc_local_schedule_count();
        self.spawner
            .shared
            .worker_metrics
            .set_queue_depth(self.tasks.len());
    }
}

// ===== impl Context =====

impl Context {
    /// Execute the closure with the given scheduler core stored in the
    /// thread-local context.
    fn run_task<R>(&self, mut core: Box<Core>, f: impl FnOnce() -> R) -> (Box<Core>, R) {
        core.metrics.incr_poll_count();
        self.enter(core, || crate::coop::budget(f))
    }

    /// Blocks the current thread until an event is received by the driver,
    /// including I/O events, timer events, ...
    fn park(&self, mut core: Box<Core>) -> Box<Core> {
        let mut driver = core.driver.take().expect("driver missing");

        if let Some(f) = &self.spawner.shared.config.before_park {
            // Incorrect lint, the closures are actually different types so `f`
            // cannot be passed as an argument to `enter`.
            #[allow(clippy::redundant_closure)]
            let (c, _) = self.enter(core, || f());
            core = c;
        }

        // This check will fail if `before_park` spawns a task for us to run
        // instead of parking the thread
        if core.tasks.is_empty() {
            // Park until the thread is signaled
            core.metrics.about_to_park();
            core.metrics.submit(&core.spawner.shared.worker_metrics);

            let (c, _) = self.enter(core, || {
                driver.park().expect("failed to park");
            });

            core = c;
            core.metrics.returned_from_park();
        }

        if let Some(f) = &self.spawner.shared.config.after_unpark {
            // Incorrect lint, the closures are actually different types so `f`
            // cannot be passed as an argument to `enter`.
            #[allow(clippy::redundant_closure)]
            let (c, _) = self.enter(core, || f());
            core = c;
        }

        core.driver = Some(driver);
        core
    }

    /// Checks the driver for new events without blocking the thread.
    fn park_yield(&self, mut core: Box<Core>) -> Box<Core> {
        let mut driver = core.driver.take().expect("driver missing");

        core.metrics.submit(&core.spawner.shared.worker_metrics);
        let (mut core, _) = self.enter(core, || {
            driver
                .park_timeout(Duration::from_millis(0))
                .expect("failed to park");
        });

        core.driver = Some(driver);
        core
    }

    fn enter<R>(&self, core: Box<Core>, f: impl FnOnce() -> R) -> (Box<Core>, R) {
        // Store the scheduler core in the thread-local context
        //
        // A drop-guard is employed at a higher level.
        *self.core.borrow_mut() = Some(core);

        // Execute the closure while tracking the execution budget
        let ret = f();

        // Take the scheduler core back
        let core = self.core.borrow_mut().take().expect("core missing");
        (core, ret)
    }
}

// ===== impl Spawner =====

impl Spawner {
    /// Spawns a future onto the basic scheduler
    pub(crate) fn spawn<F>(&self, future: F, id: super::task::Id) -> JoinHandle<F::Output>
    where
        F: crate::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (handle, notified) = self.shared.owned.bind(future, self.shared.clone(), id);

        if let Some(notified) = notified {
            self.shared.schedule(notified);
        }

        handle
    }

    fn pop(&self) -> Option<task::Notified<Arc<Shared>>> {
        match self.shared.queue.lock().as_mut() {
            Some(queue) => queue.pop_front(),
            None => None,
        }
    }

    fn waker_ref(&self) -> WakerRef<'_> {
        // Set woken to true when enter block_on, ensure outer future
        // be polled for the first time when enter loop
        self.shared.woken.store(true, Release);
        waker_ref(&self.shared)
    }

    // reset woken to false and return original value
    pub(crate) fn reset_woken(&self) -> bool {
        self.shared.woken.swap(false, AcqRel)
    }

    pub(crate) fn as_handle_inner(&self) -> &HandleInner {
        &self.shared.handle_inner
    }
}

cfg_metrics! {
    impl Spawner {
        pub(crate) fn scheduler_metrics(&self) -> &SchedulerMetrics {
            &self.shared.scheduler_metrics
        }

        pub(crate) fn injection_queue_depth(&self) -> usize {
            // TODO: avoid having to lock. The multi-threaded injection queue
            // could probably be used here.
            self.shared.queue.lock()
                .as_ref()
                .map(|queue| queue.len())
                .unwrap_or(0)
        }

        pub(crate) fn worker_metrics(&self, worker: usize) -> &WorkerMetrics {
            assert_eq!(0, worker);
            &self.shared.worker_metrics
        }
    }
}

impl fmt::Debug for Spawner {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Spawner").finish()
    }
}

// ===== impl Shared =====

impl Schedule for Arc<Shared> {
    fn release(&self, task: &Task<Self>) -> Option<Task<Self>> {
        self.owned.remove(task)
    }

    fn schedule(&self, task: task::Notified<Self>) {
        CURRENT.with(|maybe_cx| match maybe_cx {
            Some(cx) if Arc::ptr_eq(self, &cx.spawner.shared) => {
                let mut core = cx.core.borrow_mut();

                // If `None`, the runtime is shutting down, so there is no need
                // to schedule the task.
                if let Some(core) = core.as_mut() {
                    core.push_task(task);
                }
            }
            _ => {
                // Track that a task was scheduled from **outside** of the runtime.
                self.scheduler_metrics.inc_remote_schedule_count();

                // If the queue is None, then the runtime has shut down. We
                // don't need to do anything with the notification in that case.
                let mut guard = self.queue.lock();
                if let Some(queue) = guard.as_mut() {
                    queue.push_back(task);
                    drop(guard);
                    self.unpark.unpark();
                }
            }
        });
    }

    cfg_unstable! {
        fn unhandled_panic(&self) {
            use crate::runtime::UnhandledPanic;

            match self.config.unhandled_panic {
                UnhandledPanic::Ignore => {
                    // Do nothing
                }
                UnhandledPanic::ShutdownRuntime => {
                    // This hook is only called from within the runtime, so
                    // `CURRENT` should match with `&self`, i.e. there is no
                    // opportunity for a nested scheduler to be called.
                    CURRENT.with(|maybe_cx| match maybe_cx {
                        Some(cx) if Arc::ptr_eq(self, &cx.spawner.shared) => {
                            let mut core = cx.core.borrow_mut();

                            // If `None`, the runtime is shutting down, so there is no need to signal shutdown
                            if let Some(core) = core.as_mut() {
                                core.unhandled_panic = true;
                                self.owned.close_and_shutdown_all();
                            }
                        }
                        _ => unreachable!("runtime core not set in CURRENT thread-local"),
                    })
                }
            }
        }
    }
}

impl Wake for Shared {
    fn wake(arc_self: Arc<Self>) {
        Wake::wake_by_ref(&arc_self)
    }

    /// Wake by reference
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.woken.store(true, Release);
        arc_self.unpark.unpark();
    }
}

// ===== CoreGuard =====

/// Used to ensure we always place the `Core` value back into its slot in
/// `BasicScheduler`, even if the future panics.
struct CoreGuard<'a> {
    context: Context,
    basic_scheduler: &'a BasicScheduler,
}

impl CoreGuard<'_> {
    #[track_caller]
    fn block_on<F: Future>(self, future: F) -> F::Output {
        let ret = self.enter(|mut core, context| {
            let _enter = crate::runtime::enter(false);
            let waker = context.spawner.waker_ref();
            let mut cx = std::task::Context::from_waker(&waker);

            pin!(future);

            'outer: loop {
                if core.spawner.reset_woken() {
                    let (c, res) = context.enter(core, || {
                        crate::coop::budget(|| future.as_mut().poll(&mut cx))
                    });

                    core = c;

                    if let Ready(v) = res {
                        return (core, Some(v));
                    }
                }

                for _ in 0..core.spawner.shared.config.event_interval {
                    // Make sure we didn't hit an unhandled_panic
                    if core.unhandled_panic {
                        return (core, None);
                    }

                    // Get and increment the current tick
                    let tick = core.tick;
                    core.tick = core.tick.wrapping_add(1);

                    let entry = if tick % core.spawner.shared.config.global_queue_interval == 0 {
                        core.spawner.pop().or_else(|| core.tasks.pop_front())
                    } else {
                        core.tasks.pop_front().or_else(|| core.spawner.pop())
                    };

                    let task = match entry {
                        Some(entry) => entry,
                        None => {
                            core = context.park(core);

                            // Try polling the `block_on` future next
                            continue 'outer;
                        }
                    };

                    let task = context.spawner.shared.owned.assert_owner(task);

                    let (c, _) = context.run_task(core, || {
                        task.run();
                    });

                    core = c;
                }

                // Yield to the driver, this drives the timer and pulls any
                // pending I/O events.
                core = context.park_yield(core);
            }
        });

        match ret {
            Some(ret) => ret,
            None => {
                // `block_on` panicked.
                panic!("a spawned task panicked and the runtime is configured to shut down on unhandled panic");
            }
        }
    }

    /// Enters the scheduler context. This sets the queue and other necessary
    /// scheduler state in the thread-local.
    fn enter<F, R>(self, f: F) -> R
    where
        F: FnOnce(Box<Core>, &Context) -> (Box<Core>, R),
    {
        // Remove `core` from `context` to pass into the closure.
        let core = self.context.core.borrow_mut().take().expect("core missing");

        // Call the closure and place `core` back
        let (core, ret) = CURRENT.set(&self.context, || f(core, &self.context));

        *self.context.core.borrow_mut() = Some(core);

        ret
    }
}

impl Drop for CoreGuard<'_> {
    fn drop(&mut self) {
        if let Some(core) = self.context.core.borrow_mut().take() {
            // Replace old scheduler back into the state to allow
            // other threads to pick it up and drive it.
            self.basic_scheduler.core.set(core);

            // Wake up other possible threads that could steal the driver.
            self.basic_scheduler.notify.notify_one()
        }
    }
}
