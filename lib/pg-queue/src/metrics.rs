use std::sync::LazyLock;

use prometheus::{register_histogram, Histogram};

pub static ITEM_PROCESSING_DURATION: LazyLock<Histogram> = LazyLock::new(|| {
    register_histogram!(
        "pg_queue_item_processing_duration_seconds",
        "The time it takes to process an item in the queue.",
    )
    .unwrap()
});

pub static OPTIMIZE_PROCESSING_DURATION: LazyLock<Histogram> = LazyLock::new(|| {
    register_histogram!(
        "pg_queue_optimize_processing_duration_seconds",
        "The time it takes to run a pass over the optimize queue.",
    )
    .unwrap()
});

pub static OPTIMIZE_ITEM_COUNT: LazyLock<Histogram> = LazyLock::new(|| {
    register_histogram!(
        "pg_queue_optimize_item_count",
        "The amount of items processed in an optimize pass.",
        vec![1.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0],
    )
    .unwrap()
});
