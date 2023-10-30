//! This module contains information need to view information about how the
//! runtime is performing.
//!
//! **Note**: This is an [unstable API][unstable]. The public API of types in
//! this module may break in 1.x releases. See [the documentation on unstable
//! features][unstable] for details.
//!
//! [unstable]: crate#unstable-features
#![allow(clippy::module_inception)]

cfg_metrics! {
    mod batch;
    pub(crate) use batch::MetricsBatch;

    mod runtime;
    #[allow(unreachable_pub)] // rust-lang/rust#57411
    pub use runtime::RuntimeMetrics;

    mod scheduler;
    pub(crate) use scheduler::SchedulerMetrics;

    mod worker;
    pub(crate) use worker::WorkerMetrics;

    cfg_net! {
        mod io;
        pub(crate) use io::IoDriverMetrics;
    }
}

cfg_not_metrics! {
    mod mock;

    pub(crate) use mock::{SchedulerMetrics, WorkerMetrics, MetricsBatch};
}
