use crate::future::Future;
use crate::runtime::task::Id;
use crate::runtime::{basic_scheduler, HandleInner};
use crate::task::JoinHandle;

cfg_rt_multi_thread! {
    use crate::runtime::thread_pool;
}

#[derive(Debug, Clone)]
pub(crate) enum Spawner {
    Basic(basic_scheduler::Spawner),
    #[cfg(feature = "rt-multi-thread")]
    ThreadPool(thread_pool::Spawner),
}

impl Spawner {
    pub(crate) fn shutdown(&mut self) {
        #[cfg(feature = "rt-multi-thread")]
        {
            if let Spawner::ThreadPool(spawner) = self {
                spawner.shutdown();
            }
        }
    }

    pub(crate) fn spawn<F>(&self, future: F, id: Id) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        match self {
            Spawner::Basic(spawner) => spawner.spawn(future, id),
            #[cfg(feature = "rt-multi-thread")]
            Spawner::ThreadPool(spawner) => spawner.spawn(future, id),
        }
    }

    pub(crate) fn as_handle_inner(&self) -> &HandleInner {
        match self {
            Spawner::Basic(spawner) => spawner.as_handle_inner(),
            #[cfg(feature = "rt-multi-thread")]
            Spawner::ThreadPool(spawner) => spawner.as_handle_inner(),
        }
    }
}

cfg_metrics! {
    use crate::runtime::{SchedulerMetrics, WorkerMetrics};

    impl Spawner {
        pub(crate) fn num_workers(&self) -> usize {
            match self {
                Spawner::Basic(_) => 1,
                #[cfg(feature = "rt-multi-thread")]
                Spawner::ThreadPool(spawner) => spawner.num_workers(),
            }
        }

        pub(crate) fn scheduler_metrics(&self) -> &SchedulerMetrics {
            match self {
                Spawner::Basic(spawner) => spawner.scheduler_metrics(),
                #[cfg(feature = "rt-multi-thread")]
                Spawner::ThreadPool(spawner) => spawner.scheduler_metrics(),
            }
        }

        pub(crate) fn worker_metrics(&self, worker: usize) -> &WorkerMetrics {
            match self {
                Spawner::Basic(spawner) => spawner.worker_metrics(worker),
                #[cfg(feature = "rt-multi-thread")]
                Spawner::ThreadPool(spawner) => spawner.worker_metrics(worker),
            }
        }

        pub(crate) fn injection_queue_depth(&self) -> usize {
            match self {
                Spawner::Basic(spawner) => spawner.injection_queue_depth(),
                #[cfg(feature = "rt-multi-thread")]
                Spawner::ThreadPool(spawner) => spawner.injection_queue_depth(),
            }
        }

        pub(crate) fn worker_local_queue_depth(&self, worker: usize) -> usize {
            match self {
                Spawner::Basic(spawner) => spawner.worker_metrics(worker).queue_depth(),
                #[cfg(feature = "rt-multi-thread")]
                Spawner::ThreadPool(spawner) => spawner.worker_local_queue_depth(worker),
            }
        }
    }
}
