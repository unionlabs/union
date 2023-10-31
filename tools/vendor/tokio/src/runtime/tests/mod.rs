use self::unowned_wrapper::unowned;

mod unowned_wrapper {
    use crate::runtime::blocking::NoopSchedule;
    use crate::runtime::task::{Id, JoinHandle, Notified};

    #[cfg(all(tokio_unstable, feature = "tracing"))]
    pub(crate) fn unowned<T>(task: T) -> (Notified<NoopSchedule>, JoinHandle<T::Output>)
    where
        T: std::future::Future + Send + 'static,
        T::Output: Send + 'static,
    {
        use tracing::Instrument;
        let span = tracing::trace_span!("test_span");
        let task = task.instrument(span);
        let (task, handle) = crate::runtime::task::unowned(task, NoopSchedule, Id::next());
        (task.into_notified(), handle)
    }

    #[cfg(not(all(tokio_unstable, feature = "tracing")))]
    pub(crate) fn unowned<T>(task: T) -> (Notified<NoopSchedule>, JoinHandle<T::Output>)
    where
        T: std::future::Future + Send + 'static,
        T::Output: Send + 'static,
    {
        let (task, handle) = crate::runtime::task::unowned(task, NoopSchedule, Id::next());
        (task.into_notified(), handle)
    }
}

cfg_loom! {
    mod loom_basic_scheduler;
    mod loom_blocking;
    mod loom_local;
    mod loom_oneshot;
    mod loom_pool;
    mod loom_queue;
    mod loom_shutdown_join;
    mod loom_join_set;
}

cfg_not_loom! {
    mod queue;

    #[cfg(not(miri))]
    mod task_combinations;

    #[cfg(miri)]
    mod task;
}
