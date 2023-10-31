//! A scheduler is initialized with a fixed number of workers. Each worker is
//! driven by a thread. Each worker has a "core" which contains data such as the
//! run queue and other state. When `block_in_place` is called, the worker's
//! "core" is handed off to a new thread allowing the scheduler to continue to
//! make progress while the originating thread blocks.
//!
//! # Shutdown
//!
//! Shutting down the runtime involves the following steps:
//!
//!  1. The Shared::close method is called. This closes the inject queue and
//!     OwnedTasks instance and wakes up all worker threads.
//!
//!  2. Each worker thread observes the close signal next time it runs
//!     Core::maintenance by checking whether the inject queue is closed.
//!     The Core::is_shutdown flag is set to true.
//!
//!  3. The worker thread calls `pre_shutdown` in parallel. Here, the worker
//!     will keep removing tasks from OwnedTasks until it is empty. No new
//!     tasks can be pushed to the OwnedTasks during or after this step as it
//!     was closed in step 1.
//!
//!  5. The workers call Shared::shutdown to enter the single-threaded phase of
//!     shutdown. These calls will push their core to Shared::shutdown_cores,
//!     and the last thread to push its core will finish the shutdown procedure.
//!
//!  6. The local run queue of each core is emptied, then the inject queue is
//!     emptied.
//!
//! At this point, shutdown has completed. It is not possible for any of the
//! collections to contain any tasks at this point, as each collection was
//! closed first, then emptied afterwards.
//!
//! ## Spawns during shutdown
//!
//! When spawning tasks during shutdown, there are two cases:
//!
//!  * The spawner observes the OwnedTasks being open, and the inject queue is
//!    closed.
//!  * The spawner observes the OwnedTasks being closed and doesn't check the
//!    inject queue.
//!
//! The first case can only happen if the OwnedTasks::bind call happens before
//! or during step 1 of shutdown. In this case, the runtime will clean up the
//! task in step 3 of shutdown.
//!
//! In the latter case, the task was not spawned and the task is immediately
//! cancelled by the spawner.
//!
//! The correctness of shutdown requires both the inject queue and OwnedTasks
//! collection to have a closed bit. With a close bit on only the inject queue,
//! spawning could run in to a situation where a task is successfully bound long
//! after the runtime has shut down. With a close bit on only the OwnedTasks,
//! the first spawning situation could result in the notification being pushed
//! to the inject queue after step 6 of shutdown, which would leave a task in
//! the inject queue indefinitely. This would be a ref-count cycle and a memory
//! leak.

use crate::coop;
use crate::future::Future;
use crate::loom::rand::seed;
use crate::loom::sync::{Arc, Mutex};
use crate::park::{Park, Unpark};
use crate::runtime;
use crate::runtime::enter::EnterContext;
use crate::runtime::task::{Inject, JoinHandle, OwnedTasks};
use crate::runtime::thread_pool::{queue, Idle, Parker, Unparker};
use crate::runtime::{task, Callback, HandleInner, MetricsBatch, SchedulerMetrics, WorkerMetrics};
use crate::util::atomic_cell::AtomicCell;
use crate::util::FastRand;

use std::cell::RefCell;
use std::time::Duration;

/// A scheduler worker
pub(super) struct Worker {
    /// Reference to shared state
    shared: Arc<Shared>,

    /// Index holding this worker's remote state
    index: usize,

    /// Used to hand-off a worker's core to another thread.
    core: AtomicCell<Core>,
}

/// Core data
struct Core {
    /// Used to schedule bookkeeping tasks every so often.
    tick: u32,

    /// When a task is scheduled from a worker, it is stored in this slot. The
    /// worker will check this slot for a task **before** checking the run
    /// queue. This effectively results in the **last** scheduled task to be run
    /// next (LIFO). This is an optimization for message passing patterns and
    /// helps to reduce latency.
    lifo_slot: Option<Notified>,

    /// The worker-local run queue.
    run_queue: queue::Local<Arc<Shared>>,

    /// True if the worker is currently searching for more work. Searching
    /// involves attempting to steal from other workers.
    is_searching: bool,

    /// True if the scheduler is being shutdown
    is_shutdown: bool,

    /// Parker
    ///
    /// Stored in an `Option` as the parker is added / removed to make the
    /// borrow checker happy.
    park: Option<Parker>,

    /// Batching metrics so they can be submitted to RuntimeMetrics.
    metrics: MetricsBatch,

    /// Fast random number generator.
    rand: FastRand,

    /// How many ticks before pulling a task from the global/remote queue?
    global_queue_interval: u32,

    /// How many ticks before yielding to the driver for timer and I/O events?
    event_interval: u32,
}

/// State shared across all workers
pub(super) struct Shared {
    /// Handle to the I/O driver, timer, blocking spawner, ...
    handle_inner: HandleInner,

    /// Per-worker remote state. All other workers have access to this and is
    /// how they communicate between each other.
    remotes: Box<[Remote]>,

    /// Global task queue used for:
    ///  1. Submit work to the scheduler while **not** currently on a worker thread.
    ///  2. Submit work to the scheduler when a worker run queue is saturated
    inject: Inject<Arc<Shared>>,

    /// Coordinates idle workers
    idle: Idle,

    /// Collection of all active tasks spawned onto this executor.
    owned: OwnedTasks<Arc<Shared>>,

    /// Cores that have observed the shutdown signal
    ///
    /// The core is **not** placed back in the worker to avoid it from being
    /// stolen by a thread that was spawned as part of `block_in_place`.
    #[allow(clippy::vec_box)] // we're moving an already-boxed value
    shutdown_cores: Mutex<Vec<Box<Core>>>,

    /// Callback for a worker parking itself
    before_park: Option<Callback>,
    /// Callback for a worker unparking itself
    after_unpark: Option<Callback>,

    /// Collects metrics from the runtime.
    pub(super) scheduler_metrics: SchedulerMetrics,

    pub(super) worker_metrics: Box<[WorkerMetrics]>,
}

/// Used to communicate with a worker from other threads.
struct Remote {
    /// Steals tasks from this worker.
    steal: queue::Steal<Arc<Shared>>,

    /// Unparks the associated worker thread
    unpark: Unparker,
}

/// Thread-local context
struct Context {
    /// Worker
    worker: Arc<Worker>,

    /// Core data
    core: RefCell<Option<Box<Core>>>,
}

/// Starts the workers
pub(crate) struct Launch(Vec<Arc<Worker>>);

/// Running a task may consume the core. If the core is still available when
/// running the task completes, it is returned. Otherwise, the worker will need
/// to stop processing.
type RunResult = Result<Box<Core>, ()>;

/// A task handle
type Task = task::Task<Arc<Shared>>;

/// A notified task handle
type Notified = task::Notified<Arc<Shared>>;

// Tracks thread-local state
scoped_thread_local!(static CURRENT: Context);

pub(super) fn create(
    size: usize,
    park: Parker,
    handle_inner: HandleInner,
    before_park: Option<Callback>,
    after_unpark: Option<Callback>,
    global_queue_interval: u32,
    event_interval: u32,
) -> (Arc<Shared>, Launch) {
    let mut cores = Vec::with_capacity(size);
    let mut remotes = Vec::with_capacity(size);
    let mut worker_metrics = Vec::with_capacity(size);

    // Create the local queues
    for _ in 0..size {
        let (steal, run_queue) = queue::local();

        let park = park.clone();
        let unpark = park.unpark();

        cores.push(Box::new(Core {
            tick: 0,
            lifo_slot: None,
            run_queue,
            is_searching: false,
            is_shutdown: false,
            park: Some(park),
            metrics: MetricsBatch::new(),
            rand: FastRand::new(seed()),
            global_queue_interval,
            event_interval,
        }));

        remotes.push(Remote { steal, unpark });
        worker_metrics.push(WorkerMetrics::new());
    }

    let shared = Arc::new(Shared {
        handle_inner,
        remotes: remotes.into_boxed_slice(),
        inject: Inject::new(),
        idle: Idle::new(size),
        owned: OwnedTasks::new(),
        shutdown_cores: Mutex::new(vec![]),
        before_park,
        after_unpark,
        scheduler_metrics: SchedulerMetrics::new(),
        worker_metrics: worker_metrics.into_boxed_slice(),
    });

    let mut launch = Launch(vec![]);

    for (index, core) in cores.drain(..).enumerate() {
        launch.0.push(Arc::new(Worker {
            shared: shared.clone(),
            index,
            core: AtomicCell::new(Some(core)),
        }));
    }

    (shared, launch)
}

pub(crate) fn block_in_place<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // Try to steal the worker core back
    struct Reset(coop::Budget);

    impl Drop for Reset {
        fn drop(&mut self) {
            CURRENT.with(|maybe_cx| {
                if let Some(cx) = maybe_cx {
                    let core = cx.worker.core.take();
                    let mut cx_core = cx.core.borrow_mut();
                    assert!(cx_core.is_none());
                    *cx_core = core;

                    // Reset the task budget as we are re-entering the
                    // runtime.
                    coop::set(self.0);
                }
            });
        }
    }

    let mut had_entered = false;

    CURRENT.with(|maybe_cx| {
        match (crate::runtime::enter::context(), maybe_cx.is_some()) {
            (EnterContext::Entered { .. }, true) => {
                // We are on a thread pool runtime thread, so we just need to
                // set up blocking.
                had_entered = true;
            }
            (EnterContext::Entered { allow_blocking }, false) => {
                // We are on an executor, but _not_ on the thread pool.  That is
                // _only_ okay if we are in a thread pool runtime's block_on
                // method:
                if allow_blocking {
                    had_entered = true;
                    return;
                } else {
                    // This probably means we are on the basic_scheduler or in a
                    // LocalSet, where it is _not_ okay to block.
                    panic!("can call blocking only when running on the multi-threaded runtime");
                }
            }
            (EnterContext::NotEntered, true) => {
                // This is a nested call to block_in_place (we already exited).
                // All the necessary setup has already been done.
                return;
            }
            (EnterContext::NotEntered, false) => {
                // We are outside of the tokio runtime, so blocking is fine.
                // We can also skip all of the thread pool blocking setup steps.
                return;
            }
        }

        let cx = maybe_cx.expect("no .is_some() == false cases above should lead here");

        // Get the worker core. If none is set, then blocking is fine!
        let core = match cx.core.borrow_mut().take() {
            Some(core) => core,
            None => return,
        };

        // The parker should be set here
        assert!(core.park.is_some());

        // In order to block, the core must be sent to another thread for
        // execution.
        //
        // First, move the core back into the worker's shared core slot.
        cx.worker.core.set(core);

        // Next, clone the worker handle and send it to a new thread for
        // processing.
        //
        // Once the blocking task is done executing, we will attempt to
        // steal the core back.
        let worker = cx.worker.clone();
        runtime::spawn_blocking(move || run(worker));
    });

    if had_entered {
        // Unset the current task's budget. Blocking sections are not
        // constrained by task budgets.
        let _reset = Reset(coop::stop());

        crate::runtime::enter::exit(f)
    } else {
        f()
    }
}

impl Launch {
    pub(crate) fn launch(mut self) {
        for worker in self.0.drain(..) {
            runtime::spawn_blocking(move || run(worker));
        }
    }
}

fn run(worker: Arc<Worker>) {
    // Acquire a core. If this fails, then another thread is running this
    // worker and there is nothing further to do.
    let core = match worker.core.take() {
        Some(core) => core,
        None => return,
    };

    // Set the worker context.
    let cx = Context {
        worker,
        core: RefCell::new(None),
    };

    let _enter = crate::runtime::enter(true);

    CURRENT.set(&cx, || {
        // This should always be an error. It only returns a `Result` to support
        // using `?` to short circuit.
        assert!(cx.run(core).is_err());
    });
}

impl Context {
    fn run(&self, mut core: Box<Core>) -> RunResult {
        while !core.is_shutdown {
            // Increment the tick
            core.tick();

            // Run maintenance, if needed
            core = self.maintenance(core);

            // First, check work available to the current worker.
            if let Some(task) = core.next_task(&self.worker) {
                core = self.run_task(task, core)?;
                continue;
            }

            // There is no more **local** work to process, try to steal work
            // from other workers.
            if let Some(task) = core.steal_work(&self.worker) {
                core = self.run_task(task, core)?;
            } else {
                // Wait for work
                core = self.park(core);
            }
        }

        core.pre_shutdown(&self.worker);

        // Signal shutdown
        self.worker.shared.shutdown(core);
        Err(())
    }

    fn run_task(&self, task: Notified, mut core: Box<Core>) -> RunResult {
        let task = self.worker.shared.owned.assert_owner(task);

        // Make sure the worker is not in the **searching** state. This enables
        // another idle worker to try to steal work.
        core.transition_from_searching(&self.worker);

        // Make the core available to the runtime context
        core.metrics.incr_poll_count();
        *self.core.borrow_mut() = Some(core);

        // Run the task
        coop::budget(|| {
            task.run();

            // As long as there is budget remaining and a task exists in the
            // `lifo_slot`, then keep running.
            loop {
                // Check if we still have the core. If not, the core was stolen
                // by another worker.
                let mut core = match self.core.borrow_mut().take() {
                    Some(core) => core,
                    None => return Err(()),
                };

                // Check for a task in the LIFO slot
                let task = match core.lifo_slot.take() {
                    Some(task) => task,
                    None => return Ok(core),
                };

                if coop::has_budget_remaining() {
                    // Run the LIFO task, then loop
                    core.metrics.incr_poll_count();
                    *self.core.borrow_mut() = Some(core);
                    let task = self.worker.shared.owned.assert_owner(task);
                    task.run();
                } else {
                    // Not enough budget left to run the LIFO task, push it to
                    // the back of the queue and return.
                    core.run_queue
                        .push_back(task, self.worker.inject(), &mut core.metrics);
                    return Ok(core);
                }
            }
        })
    }

    fn maintenance(&self, mut core: Box<Core>) -> Box<Core> {
        if core.tick % core.event_interval == 0 {
            // Call `park` with a 0 timeout. This enables the I/O driver, timer, ...
            // to run without actually putting the thread to sleep.
            core = self.park_timeout(core, Some(Duration::from_millis(0)));

            // Run regularly scheduled maintenance
            core.maintenance(&self.worker);
        }

        core
    }

    /// Parks the worker thread while waiting for tasks to execute.
    ///
    /// This function checks if indeed there's no more work left to be done before parking.
    /// Also important to notice that, before parking, the worker thread will try to take
    /// ownership of the Driver (IO/Time) and dispatch any events that might have fired.
    /// Whenever a worker thread executes the Driver loop, all waken tasks are scheduled
    /// in its own local queue until the queue saturates (ntasks > LOCAL_QUEUE_CAPACITY).
    /// When the local queue is saturated, the overflow tasks are added to the injection queue
    /// from where other workers can pick them up.
    /// Also, we rely on the workstealing algorithm to spread the tasks amongst workers
    /// after all the IOs get dispatched
    fn park(&self, mut core: Box<Core>) -> Box<Core> {
        if let Some(f) = &self.worker.shared.before_park {
            f();
        }

        if core.transition_to_parked(&self.worker) {
            while !core.is_shutdown {
                core.metrics.about_to_park();
                core = self.park_timeout(core, None);
                core.metrics.returned_from_park();

                // Run regularly scheduled maintenance
                core.maintenance(&self.worker);

                if core.transition_from_parked(&self.worker) {
                    break;
                }
            }
        }

        if let Some(f) = &self.worker.shared.after_unpark {
            f();
        }
        core
    }

    fn park_timeout(&self, mut core: Box<Core>, duration: Option<Duration>) -> Box<Core> {
        // Take the parker out of core
        let mut park = core.park.take().expect("park missing");

        // Store `core` in context
        *self.core.borrow_mut() = Some(core);

        // Park thread
        if let Some(timeout) = duration {
            park.park_timeout(timeout).expect("park failed");
        } else {
            park.park().expect("park failed");
        }

        // Remove `core` from context
        core = self.core.borrow_mut().take().expect("core missing");

        // Place `park` back in `core`
        core.park = Some(park);

        // If there are tasks available to steal, but this worker is not
        // looking for tasks to steal, notify another worker.
        if !core.is_searching && core.run_queue.is_stealable() {
            self.worker.shared.notify_parked();
        }

        core
    }
}

impl Core {
    /// Increment the tick
    fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    /// Return the next notified task available to this worker.
    fn next_task(&mut self, worker: &Worker) -> Option<Notified> {
        if self.tick % self.global_queue_interval == 0 {
            worker.inject().pop().or_else(|| self.next_local_task())
        } else {
            self.next_local_task().or_else(|| worker.inject().pop())
        }
    }

    fn next_local_task(&mut self) -> Option<Notified> {
        self.lifo_slot.take().or_else(|| self.run_queue.pop())
    }

    /// Function responsible for stealing tasks from another worker
    ///
    /// Note: Only if less than half the workers are searching for tasks to steal
    /// a new worker will actually try to steal. The idea is to make sure not all
    /// workers will be trying to steal at the same time.
    fn steal_work(&mut self, worker: &Worker) -> Option<Notified> {
        if !self.transition_to_searching(worker) {
            return None;
        }

        let num = worker.shared.remotes.len();
        // Start from a random worker
        let start = self.rand.fastrand_n(num as u32) as usize;

        for i in 0..num {
            let i = (start + i) % num;

            // Don't steal from ourself! We know we don't have work.
            if i == worker.index {
                continue;
            }

            let target = &worker.shared.remotes[i];
            if let Some(task) = target
                .steal
                .steal_into(&mut self.run_queue, &mut self.metrics)
            {
                return Some(task);
            }
        }

        // Fallback on checking the global queue
        worker.shared.inject.pop()
    }

    fn transition_to_searching(&mut self, worker: &Worker) -> bool {
        if !self.is_searching {
            self.is_searching = worker.shared.idle.transition_worker_to_searching();
        }

        self.is_searching
    }

    fn transition_from_searching(&mut self, worker: &Worker) {
        if !self.is_searching {
            return;
        }

        self.is_searching = false;
        worker.shared.transition_worker_from_searching();
    }

    /// Prepares the worker state for parking.
    ///
    /// Returns true if the transition happened, false if there is work to do first.
    fn transition_to_parked(&mut self, worker: &Worker) -> bool {
        // Workers should not park if they have work to do
        if self.lifo_slot.is_some() || self.run_queue.has_tasks() {
            return false;
        }

        // When the final worker transitions **out** of searching to parked, it
        // must check all the queues one last time in case work materialized
        // between the last work scan and transitioning out of searching.
        let is_last_searcher = worker
            .shared
            .idle
            .transition_worker_to_parked(worker.index, self.is_searching);

        // The worker is no longer searching. Setting this is the local cache
        // only.
        self.is_searching = false;

        if is_last_searcher {
            worker.shared.notify_if_work_pending();
        }

        true
    }

    /// Returns `true` if the transition happened.
    fn transition_from_parked(&mut self, worker: &Worker) -> bool {
        // If a task is in the lifo slot, then we must unpark regardless of
        // being notified
        if self.lifo_slot.is_some() {
            // When a worker wakes, it should only transition to the "searching"
            // state when the wake originates from another worker *or* a new task
            // is pushed. We do *not* want the worker to transition to "searching"
            // when it wakes when the I/O driver receives new events.
            self.is_searching = !worker.shared.idle.unpark_worker_by_id(worker.index);
            return true;
        }

        if worker.shared.idle.is_parked(worker.index) {
            return false;
        }

        // When unparked, the worker is in the searching state.
        self.is_searching = true;
        true
    }

    /// Runs maintenance work such as checking the pool's state.
    fn maintenance(&mut self, worker: &Worker) {
        self.metrics
            .submit(&worker.shared.worker_metrics[worker.index]);

        if !self.is_shutdown {
            // Check if the scheduler has been shutdown
            self.is_shutdown = worker.inject().is_closed();
        }
    }

    /// Signals all tasks to shut down, and waits for them to complete. Must run
    /// before we enter the single-threaded phase of shutdown processing.
    fn pre_shutdown(&mut self, worker: &Worker) {
        // Signal to all tasks to shut down.
        worker.shared.owned.close_and_shutdown_all();

        self.metrics
            .submit(&worker.shared.worker_metrics[worker.index]);
    }

    /// Shuts down the core.
    fn shutdown(&mut self) {
        // Take the core
        let mut park = self.park.take().expect("park missing");

        // Drain the queue
        while self.next_local_task().is_some() {}

        park.shutdown();
    }
}

impl Worker {
    /// Returns a reference to the scheduler's injection queue.
    fn inject(&self) -> &Inject<Arc<Shared>> {
        &self.shared.inject
    }
}

impl task::Schedule for Arc<Shared> {
    fn release(&self, task: &Task) -> Option<Task> {
        self.owned.remove(task)
    }

    fn schedule(&self, task: Notified) {
        (**self).schedule(task, false);
    }

    fn yield_now(&self, task: Notified) {
        (**self).schedule(task, true);
    }
}

impl Shared {
    pub(crate) fn as_handle_inner(&self) -> &HandleInner {
        &self.handle_inner
    }

    pub(super) fn bind_new_task<T>(
        me: &Arc<Self>,
        future: T,
        id: crate::runtime::task::Id,
    ) -> JoinHandle<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        let (handle, notified) = me.owned.bind(future, me.clone(), id);

        if let Some(notified) = notified {
            me.schedule(notified, false);
        }

        handle
    }

    pub(super) fn schedule(&self, task: Notified, is_yield: bool) {
        CURRENT.with(|maybe_cx| {
            if let Some(cx) = maybe_cx {
                // Make sure the task is part of the **current** scheduler.
                if self.ptr_eq(&cx.worker.shared) {
                    // And the current thread still holds a core
                    if let Some(core) = cx.core.borrow_mut().as_mut() {
                        self.schedule_local(core, task, is_yield);
                        return;
                    }
                }
            }

            // Otherwise, use the inject queue.
            self.inject.push(task);
            self.scheduler_metrics.inc_remote_schedule_count();
            self.notify_parked();
        })
    }

    fn schedule_local(&self, core: &mut Core, task: Notified, is_yield: bool) {
        core.metrics.inc_local_schedule_count();

        // Spawning from the worker thread. If scheduling a "yield" then the
        // task must always be pushed to the back of the queue, enabling other
        // tasks to be executed. If **not** a yield, then there is more
        // flexibility and the task may go to the front of the queue.
        let should_notify = if is_yield {
            core.run_queue
                .push_back(task, &self.inject, &mut core.metrics);
            true
        } else {
            // Push to the LIFO slot
            let prev = core.lifo_slot.take();
            let ret = prev.is_some();

            if let Some(prev) = prev {
                core.run_queue
                    .push_back(prev, &self.inject, &mut core.metrics);
            }

            core.lifo_slot = Some(task);

            ret
        };

        // Only notify if not currently parked. If `park` is `None`, then the
        // scheduling is from a resource driver. As notifications often come in
        // batches, the notification is delayed until the park is complete.
        if should_notify && core.park.is_some() {
            self.notify_parked();
        }
    }

    pub(super) fn close(&self) {
        if self.inject.close() {
            self.notify_all();
        }
    }

    fn notify_parked(&self) {
        if let Some(index) = self.idle.worker_to_notify() {
            self.remotes[index].unpark.unpark();
        }
    }

    fn notify_all(&self) {
        for remote in &self.remotes[..] {
            remote.unpark.unpark();
        }
    }

    fn notify_if_work_pending(&self) {
        for remote in &self.remotes[..] {
            if !remote.steal.is_empty() {
                self.notify_parked();
                return;
            }
        }

        if !self.inject.is_empty() {
            self.notify_parked();
        }
    }

    fn transition_worker_from_searching(&self) {
        if self.idle.transition_worker_from_searching() {
            // We are the final searching worker. Because work was found, we
            // need to notify another worker.
            self.notify_parked();
        }
    }

    /// Signals that a worker has observed the shutdown signal and has replaced
    /// its core back into its handle.
    ///
    /// If all workers have reached this point, the final cleanup is performed.
    fn shutdown(&self, core: Box<Core>) {
        let mut cores = self.shutdown_cores.lock();
        cores.push(core);

        if cores.len() != self.remotes.len() {
            return;
        }

        debug_assert!(self.owned.is_empty());

        for mut core in cores.drain(..) {
            core.shutdown();
        }

        // Drain the injection queue
        //
        // We already shut down every task, so we can simply drop the tasks.
        while let Some(task) = self.inject.pop() {
            drop(task);
        }
    }

    fn ptr_eq(&self, other: &Shared) -> bool {
        std::ptr::eq(self, other)
    }
}

impl crate::runtime::ToHandle for Arc<Shared> {
    fn to_handle(&self) -> crate::runtime::Handle {
        use crate::runtime::thread_pool::Spawner;
        use crate::runtime::{self, Handle};

        Handle {
            spawner: runtime::Spawner::ThreadPool(Spawner {
                shared: self.clone(),
            }),
        }
    }
}

cfg_metrics! {
    impl Shared {
        pub(super) fn injection_queue_depth(&self) -> usize {
            self.inject.len()
        }

        pub(super) fn worker_local_queue_depth(&self, worker: usize) -> usize {
            self.remotes[worker].steal.len()
        }
    }
}
