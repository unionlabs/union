use opentelemetry::metrics::Histogram;

#[derive(Debug, Clone)]
pub struct Metrics {
    pub item_processing_duration: Histogram<f64>,
    pub optimize_processing_duration: Histogram<f64>,
    pub optimize_item_count: Histogram<u64>,
}

impl Metrics {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            item_processing_duration: opentelemetry::global::meter("pg_queue")
                .f64_histogram("pg_queue_item_processing_duration_seconds")
                .with_description("The time it takes to process an item in the queue.")
                .build(),
            optimize_processing_duration: opentelemetry::global::meter("pg_queue")
                .f64_histogram("pg_queue_optimize_processing_duration_seconds")
                .with_description("The time it takes to run a pass over the optimize queue.")
                .build(),
            optimize_item_count: opentelemetry::global::meter("pg_queue")
                .u64_histogram("pg_queue_optimize_item_count")
                .with_description("The amount of items processed in an optimize pass.")
                .with_boundaries(vec![
                    1.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0,
                ])
                .build(),
        }
    }
}
