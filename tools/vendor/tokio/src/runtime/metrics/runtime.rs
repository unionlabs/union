use crate::runtime::Handle;

use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

/// Handle to the runtime's metrics.
///
/// This handle is internally reference-counted and can be freely cloned. A
/// `RuntimeMetrics` handle is obtained using the [`Runtime::metrics`] method.
///
/// [`Runtime::metrics`]: crate::runtime::Runtime::metrics()
#[derive(Clone, Debug)]
pub struct RuntimeMetrics {
    handle: Handle,
}

impl RuntimeMetrics {
    pub(crate) fn new(handle: Handle) -> RuntimeMetrics {
        RuntimeMetrics { handle }
    }

    /// Returns the number of worker threads used by the runtime.
    ///
    /// The number of workers is set by configuring `worker_threads` on
    /// `runtime::Builder`. When using the `current_thread` runtime, the return
    /// value is always `1`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime::Handle;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let metrics = Handle::current().metrics();
    ///
    ///     let n = metrics.num_workers();
    ///     println!("Runtime is using {} workers", n);
    /// }
    /// ```
    pub fn num_workers(&self) -> usize {
        self.handle.spawner.num_workers()
    }

    /// Returns the number of tasks scheduled from **outside** of the runtime.
    ///
    /// The remote schedule count starts at zero when the runtime is created and
    /// increases by one each time a task is woken from **outside** of the
    /// runtime. This usually means that a task is spawned or notified from a
    /// non-runtime thread and must be queued using the Runtime's injection
    /// queue, which tends to be slower.
    ///
    /// The counter is monotonically increasing. It is never decremented or
    /// reset to zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime::Handle;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let metrics = Handle::current().metrics();
    ///
    ///     let n = metrics.remote_schedule_count();
    ///     println!("{} tasks were scheduled from outside the runtime", n);
    /// }
    /// ```
    pub fn remote_schedule_count(&self) -> u64 {
        self.handle
            .spawner
            .scheduler_metrics()
            .remote_schedule_count
            .load(Relaxed)
    }

    /// Returns the total number of times the given worker thread has parked.
    ///
    /// The worker park count starts at zero when the runtime is created and
    /// increases by one each time the worker parks the thread waiting for new
    /// inbound events to process. This usually means the worker has processed
    /// all pending work and is currently idle.
    ///
    /// The counter is monotonically increasing. It is never decremented or
    /// reset to zero.
    ///
    /// # Arguments
    ///
    /// `worker` is the index of the worker being queried. The given value must
    /// be between 0 and `num_workers()`. The index uniquely identifies a single
    /// worker and will continue to identify the worker throughout the lifetime
    /// of the runtime instance.
    ///
    /// # Panics
    ///
    /// The method panics when `worker` represents an invalid worker, i.e. is
    /// greater than or equal to `num_workers()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime::Handle;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let metrics = Handle::current().metrics();
    ///
    ///     let n = metrics.worker_park_count(0);
    ///     println!("worker 0 parked {} times", n);
    /// }
    /// ```
    pub fn worker_park_count(&self, worker: usize) -> u64 {
        self.handle
            .spawner
            .worker_metrics(worker)
            .park_count
            .load(Relaxed)
    }

    /// Returns the number of times the given worker thread unparked but
    /// performed no work before parking again.
    ///
    /// The worker no-op count starts at zero when the runtime is created and
    /// increases by one each time the worker unparks the thread but finds no
    /// new work and goes back to sleep. This indicates a false-positive wake up.
    ///
    /// The counter is monotonically increasing. It is never decremented or
    /// reset to zero.
    ///
    /// # Arguments
    ///
    /// `worker` is the index of the worker being queried. The given value must
    /// be between 0 and `num_workers()`. The index uniquely identifies a single
    /// worker and will continue to indentify the worker throughout the lifetime
    /// of the runtime instance.
    ///
    /// # Panics
    ///
    /// The method panics when `worker` represents an invalid worker, i.e. is
    /// greater than or equal to `num_workers()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime::Handle;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let metrics = Handle::current().metrics();
    ///
    ///     let n = metrics.worker_noop_count(0);
    ///     println!("worker 0 had {} no-op unparks", n);
    /// }
    /// ```
    pub fn worker_noop_count(&self, worker: usize) -> u64 {
        self.handle
            .spawner
            .worker_metrics(worker)
            .noop_count
            .load(Relaxed)
    }

    /// Returns the number of times the given worker thread stole tasks from
    /// another worker thread.
    ///
    /// This metric only applies to the **multi-threaded** runtime and will always return `0` when using the current thread runtime.
    ///
    /// The worker steal count starts at zero when the runtime is created and
    /// increases by one each time the worker has processed its scheduled queue
    /// and successfully steals more pending tasks from another worker.
    ///
    /// The counter is monotonically increasing. It is never decremented or
    /// reset to zero.
    ///
    /// # Arguments
    ///
    /// `worker` is the index of the worker being queried. The given value must
    /// be between 0 and `num_workers()`. The index uniquely identifies a single
    /// worker and will continue to identify the worker throughout the lifetime
    /// of the runtime instance.
    ///
    /// # Panics
    ///
    /// The method panics when `worker` represents an invalid worker, i.e. is
    /// greater than or equal to `num_workers()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime::Handle;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let metrics = Handle::current().metrics();
    ///
    ///     let n = metrics.worker_noop_count(0);
    ///     println!("worker 0 has stolen tasks {} times", n);
    /// }
    /// ```
    pub fn worker_steal_count(&self, worker: usize) -> u64 {
        self.handle
            .spawner
            .worker_metrics(worker)
            .steal_count
            .load(Relaxed)
    }

    /// Returns the number of tasks the given worker thread has polled.
    ///
    /// The worker poll count starts at zero when the runtime is created and
    /// increases by one each time the worker polls a scheduled task.
    ///
    /// The counter is monotonically increasing. It is never decremented or
    /// reset to zero.
    ///
    /// # Arguments
    ///
    /// `worker` is the index of the worker being queried. The given value must
    /// be between 0 and `num_workers()`. The index uniquely identifies a single
    /// worker and will continue to identify the worker throughout the lifetime
    /// of the runtime instance.
    ///
    /// # Panics
    ///
    /// The method panics when `worker` represents an invalid worker, i.e. is
    /// greater than or equal to `num_workers()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime::Handle;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let metrics = Handle::current().metrics();
    ///
    ///     let n = metrics.worker_poll_count(0);
    ///     println!("worker 0 has polled {} tasks", n);
    /// }
    /// ```
    pub fn worker_poll_count(&self, worker: usize) -> u64 {
        self.handle
            .spawner
            .worker_metrics(worker)
            .poll_count
            .load(Relaxed)
    }

    /// Returns the amount of time the given worker thread has been busy.
    ///
    /// The worker busy duration starts at zero when the runtime is created and
    /// increases whenever the worker is spending time processing work. Using
    /// this value can indicate the load of the given worker. If a lot of time
    /// is spent busy, then the worker is under load and will check for inbound
    /// events less often.
    ///
    /// The timer is monotonically increasing. It is never decremented or reset
    /// to zero.
    ///
    /// # Arguments
    ///
    /// `worker` is the index of the worker being queried. The given value must
    /// be between 0 and `num_workers()`. The index uniquely identifies a single
    /// worker and will continue to identify the worker throughout the lifetime
    /// of the runtime instance.
    ///
    /// # Panics
    ///
    /// The method panics when `worker` represents an invalid worker, i.e. is
    /// greater than or equal to `num_workers()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime::Handle;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let metrics = Handle::current().metrics();
    ///
    ///     let n = metrics.worker_poll_count(0);
    ///     println!("worker 0 has polled {} tasks", n);
    /// }
    /// ```
    pub fn worker_total_busy_duration(&self, worker: usize) -> Duration {
        let nanos = self
            .handle
            .spawner
            .worker_metrics(worker)
            .busy_duration_total
            .load(Relaxed);
        Duration::from_nanos(nanos)
    }

    /// Returns the number of tasks scheduled from **within** the runtime on the
    /// given worker's local queue.
    ///
    /// The local schedule count starts at zero when the runtime is created and
    /// increases by one each time a task is woken from **inside** of the
    /// runtime on the given worker. This usually means that a task is spawned
    /// or notified from within a runtime thread and will be queued on the
    /// worker-local queue.
    ///
    /// The counter is monotonically increasing. It is never decremented or
    /// reset to zero.
    ///
    /// # Arguments
    ///
    /// `worker` is the index of the worker being queried. The given value must
    /// be between 0 and `num_workers()`. The index uniquely identifies a single
    /// worker and will continue to identify the worker throughout the lifetime
    /// of the runtime instance.
    ///
    /// # Panics
    ///
    /// The method panics when `worker` represents an invalid worker, i.e. is
    /// greater than or equal to `num_workers()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime::Handle;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let metrics = Handle::current().metrics();
    ///
    ///     let n = metrics.worker_local_schedule_count(0);
    ///     println!("{} tasks were scheduled on the worker's local queue", n);
    /// }
    /// ```
    pub fn worker_local_schedule_count(&self, worker: usize) -> u64 {
        self.handle
            .spawner
            .worker_metrics(worker)
            .local_schedule_count
            .load(Relaxed)
    }

    /// Returns the number of times the given worker thread saturated its local
    /// queue.
    ///
    /// This metric only applies to the **multi-threaded** scheduler.
    ///
    /// The worker steal count starts at zero when the runtime is created and
    /// increases by one each time the worker attempts to schedule a task
    /// locally, but its local queue is full. When this happens, half of the
    /// local queue is moved to the injection queue.
    ///
    /// The counter is monotonically increasing. It is never decremented or
    /// reset to zero.
    ///
    /// # Arguments
    ///
    /// `worker` is the index of the worker being queried. The given value must
    /// be between 0 and `num_workers()`. The index uniquely identifies a single
    /// worker and will continue to identify the worker throughout the lifetime
    /// of the runtime instance.
    ///
    /// # Panics
    ///
    /// The method panics when `worker` represents an invalid worker, i.e. is
    /// greater than or equal to `num_workers()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime::Handle;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let metrics = Handle::current().metrics();
    ///
    ///     let n = metrics.worker_overflow_count(0);
    ///     println!("worker 0 has overflowed its queue {} times", n);
    /// }
    /// ```
    pub fn worker_overflow_count(&self, worker: usize) -> u64 {
        self.handle
            .spawner
            .worker_metrics(worker)
            .overflow_count
            .load(Relaxed)
    }

    /// Returns the number of tasks currently scheduled in the runtime's
    /// injection queue.
    ///
    /// Tasks that are spawned or notified from a non-runtime thread are
    /// scheduled using the runtime's injection queue. This metric returns the
    /// **current** number of tasks pending in the injection queue. As such, the
    /// returned value may increase or decrease as new tasks are scheduled and
    /// processed.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime::Handle;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let metrics = Handle::current().metrics();
    ///
    ///     let n = metrics.injection_queue_depth();
    ///     println!("{} tasks currently pending in the runtime's injection queue", n);
    /// }
    /// ```
    pub fn injection_queue_depth(&self) -> usize {
        self.handle.spawner.injection_queue_depth()
    }

    /// Returns the number of tasks currently scheduled in the given worker's
    /// local queue.
    ///
    /// Tasks that are spawned or notified from within a runtime thread are
    /// scheduled using that worker's local queue. This metric returns the
    /// **current** number of tasks pending in the worker's local queue. As
    /// such, the returned value may increase or decrease as new tasks are
    /// scheduled and processed.
    ///
    /// # Arguments
    ///
    /// `worker` is the index of the worker being queried. The given value must
    /// be between 0 and `num_workers()`. The index uniquely identifies a single
    /// worker and will continue to identify the worker throughout the lifetime
    /// of the runtime instance.
    ///
    /// # Panics
    ///
    /// The method panics when `worker` represents an invalid worker, i.e. is
    /// greater than or equal to `num_workers()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime::Handle;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let metrics = Handle::current().metrics();
    ///
    ///     let n = metrics.worker_local_queue_depth(0);
    ///     println!("{} tasks currently pending in worker 0's local queue", n);
    /// }
    /// ```
    pub fn worker_local_queue_depth(&self, worker: usize) -> usize {
        self.handle.spawner.worker_local_queue_depth(worker)
    }
}

cfg_net! {
    impl RuntimeMetrics {
        /// Returns the number of file descriptors that have been registered with the
        /// runtime's I/O driver.
        ///
        /// # Examples
        ///
        /// ```
        /// use tokio::runtime::Handle;
        ///
        /// #[tokio::main]
        /// async fn main() {
        ///     let metrics = Handle::current().metrics();
        ///
        ///     let registered_fds = metrics.io_driver_fd_registered_count();
        ///     println!("{} fds have been registered with the runtime's I/O driver.", registered_fds);
        ///
        ///     let deregistered_fds = metrics.io_driver_fd_deregistered_count();
        ///
        ///     let current_fd_count = registered_fds - deregistered_fds;
        ///     println!("{} fds are currently registered by the runtime's I/O driver.", current_fd_count);
        /// }
        /// ```
        pub fn io_driver_fd_registered_count(&self) -> u64 {
            self.with_io_driver_metrics(|m| {
                m.fd_registered_count.load(Relaxed)
            })
        }

        /// Returns the number of file descriptors that have been deregistered by the
        /// runtime's I/O driver.
        ///
        /// # Examples
        ///
        /// ```
        /// use tokio::runtime::Handle;
        ///
        /// #[tokio::main]
        /// async fn main() {
        ///     let metrics = Handle::current().metrics();
        ///
        ///     let n = metrics.io_driver_fd_deregistered_count();
        ///     println!("{} fds have been deregistered by the runtime's I/O driver.", n);
        /// }
        /// ```
        pub fn io_driver_fd_deregistered_count(&self) -> u64 {
            self.with_io_driver_metrics(|m| {
                m.fd_deregistered_count.load(Relaxed)
            })
        }

        /// Returns the number of ready events processed by the runtime's
        /// I/O driver.
        ///
        /// # Examples
        ///
        /// ```
        /// use tokio::runtime::Handle;
        ///
        /// #[tokio::main]
        /// async fn main() {
        ///     let metrics = Handle::current().metrics();
        ///
        ///     let n = metrics.io_driver_ready_count();
        ///     println!("{} ready events processed by the runtime's I/O driver.", n);
        /// }
        /// ```
        pub fn io_driver_ready_count(&self) -> u64 {
            self.with_io_driver_metrics(|m| m.ready_count.load(Relaxed))
        }

        fn with_io_driver_metrics<F>(&self, f: F) -> u64
        where
            F: Fn(&super::IoDriverMetrics) -> u64,
        {
            // TODO: Investigate if this should return 0, most of our metrics always increase
            // thus this breaks that guarantee.
            self.handle
                .as_inner()
                .io_handle
                .as_ref()
                .map(|h| f(h.metrics()))
                .unwrap_or(0)
        }
    }
}
