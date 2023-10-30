#![cfg_attr(not(feature = "net"), allow(dead_code))]

use crate::loom::sync::atomic::{AtomicU64, Ordering::Relaxed};

#[derive(Default)]
pub(crate) struct IoDriverMetrics {
    pub(super) fd_registered_count: AtomicU64,
    pub(super) fd_deregistered_count: AtomicU64,
    pub(super) ready_count: AtomicU64,
}

impl IoDriverMetrics {
    pub(crate) fn incr_fd_count(&self) {
        self.fd_registered_count.fetch_add(1, Relaxed);
    }

    pub(crate) fn dec_fd_count(&self) {
        self.fd_deregistered_count.fetch_add(1, Relaxed);
    }

    pub(crate) fn incr_ready_count_by(&self, amt: u64) {
        self.ready_count.fetch_add(amt, Relaxed);
    }
}
