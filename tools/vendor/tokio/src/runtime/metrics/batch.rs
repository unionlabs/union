use crate::runtime::WorkerMetrics;

use std::convert::TryFrom;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Instant;

pub(crate) struct MetricsBatch {
    /// Number of times the worker parked.
    park_count: u64,

    /// Number of times the worker woke w/o doing work.
    noop_count: u64,

    /// Number of times stolen.
    steal_count: u64,

    /// Number of tasks that were polled by the worker.
    poll_count: u64,

    /// Number of tasks polled when the worker entered park. This is used to
    /// track the noop count.
    poll_count_on_last_park: u64,

    /// Number of tasks that were scheduled locally on this worker.
    local_schedule_count: u64,

    /// Number of tasks moved to the global queue to make space in the local
    /// queue
    overflow_count: u64,

    /// The total busy duration in nanoseconds.
    busy_duration_total: u64,
    last_resume_time: Instant,
}

impl MetricsBatch {
    pub(crate) fn new() -> MetricsBatch {
        MetricsBatch {
            park_count: 0,
            noop_count: 0,
            steal_count: 0,
            poll_count: 0,
            poll_count_on_last_park: 0,
            local_schedule_count: 0,
            overflow_count: 0,
            busy_duration_total: 0,
            last_resume_time: Instant::now(),
        }
    }

    pub(crate) fn submit(&mut self, worker: &WorkerMetrics) {
        worker.park_count.store(self.park_count, Relaxed);
        worker.noop_count.store(self.noop_count, Relaxed);
        worker.steal_count.store(self.steal_count, Relaxed);
        worker.poll_count.store(self.poll_count, Relaxed);

        worker
            .busy_duration_total
            .store(self.busy_duration_total, Relaxed);

        worker
            .local_schedule_count
            .store(self.local_schedule_count, Relaxed);
        worker.overflow_count.store(self.overflow_count, Relaxed);
    }

    /// The worker is about to park.
    pub(crate) fn about_to_park(&mut self) {
        self.park_count += 1;

        if self.poll_count_on_last_park == self.poll_count {
            self.noop_count += 1;
        } else {
            self.poll_count_on_last_park = self.poll_count;
        }

        let busy_duration = self.last_resume_time.elapsed();
        let busy_duration = u64::try_from(busy_duration.as_nanos()).unwrap_or(u64::MAX);
        self.busy_duration_total += busy_duration;
    }

    pub(crate) fn returned_from_park(&mut self) {
        self.last_resume_time = Instant::now();
    }

    pub(crate) fn inc_local_schedule_count(&mut self) {
        self.local_schedule_count += 1;
    }

    pub(crate) fn incr_poll_count(&mut self) {
        self.poll_count += 1;
    }
}

cfg_rt_multi_thread! {
    impl MetricsBatch {
        pub(crate) fn incr_steal_count(&mut self, by: u16) {
            self.steal_count += by as u64;
        }

        pub(crate) fn incr_overflow_count(&mut self) {
            self.overflow_count += 1;
        }
    }
}
