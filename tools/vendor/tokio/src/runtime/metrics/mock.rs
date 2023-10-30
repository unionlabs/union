//! This file contains mocks of the types in src/runtime/metrics

pub(crate) struct SchedulerMetrics {}

pub(crate) struct WorkerMetrics {}

pub(crate) struct MetricsBatch {}

impl SchedulerMetrics {
    pub(crate) fn new() -> Self {
        Self {}
    }

    /// Increment the number of tasks scheduled externally
    pub(crate) fn inc_remote_schedule_count(&self) {}
}

impl WorkerMetrics {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn set_queue_depth(&self, _len: usize) {}
}

impl MetricsBatch {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn submit(&mut self, _to: &WorkerMetrics) {}
    pub(crate) fn about_to_park(&mut self) {}
    pub(crate) fn returned_from_park(&mut self) {}
    pub(crate) fn incr_poll_count(&mut self) {}
    pub(crate) fn inc_local_schedule_count(&mut self) {}
}

cfg_rt_multi_thread! {
    impl MetricsBatch {
        pub(crate) fn incr_steal_count(&mut self, _by: u16) {}
        pub(crate) fn incr_overflow_count(&mut self) {}
    }
}
