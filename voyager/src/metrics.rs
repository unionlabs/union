use std::sync::LazyLock;

use prometheus::{Histogram, HistogramOpts, Opts, Registry, DEFAULT_BUCKETS};

pub static REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);
pub static QUEUE_PROCESSING_TIME_HISTOGRAM: LazyLock<Histogram> = LazyLock::new(|| {
    Histogram::with_opts(HistogramOpts {
        common_opts: Opts {
            namespace: "".to_owned(),
            subsystem: "".to_owned(),
            name: "voyager_queue_item_processing_seconds".to_owned(),
            help: "Time spent processing a queue item.".to_owned(),
            const_labels: Default::default(),
            variable_labels: Default::default(),
        },
        buckets: DEFAULT_BUCKETS.into(),
    })
    .unwrap()
});

pub fn register_custom_metrics() {
    REGISTRY
        .register(Box::new(QUEUE_PROCESSING_TIME_HISTOGRAM.clone()))
        .unwrap();
}
