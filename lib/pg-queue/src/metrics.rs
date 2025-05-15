use opentelemetry::metrics::{Counter, Histogram};

#[derive(Debug, Clone)]
pub struct Metrics {
    pub item_processing_duration: Histogram<f64>,
    pub optimize_processing_duration: Histogram<f64>,
    pub optimize_item_count: Histogram<u64>,
    pub processed_item_count: Counter<u64>,
    pub fatal_errors_count: Counter<u64>,
    pub retryable_errors_count: Counter<u64>,
    pub unprocessable_count: Counter<u64>,
}

impl Metrics {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            item_processing_duration: opentelemetry::global::meter("pg_queue")
                .f64_histogram("pg_queue_item_processing_duration_seconds")
                .with_description("The time it takes to process an item in the queue.")
                .with_boundaries(vec![
                    0.0, 0.00001, 0.0001, 0.001, 0.01, 0.1, 0.5, 1.0, 1.5, 2.0, 5.0, 10.0, 20.0,
                    50.0,
                ])
                .build(),
            optimize_processing_duration: opentelemetry::global::meter("pg_queue")
                .f64_histogram("pg_queue_optimize_processing_duration_seconds")
                .with_description("The time it takes to run a pass over the optimize queue.")
                .with_boundaries(vec![
                    0.0, 0.00001, 0.0001, 0.001, 0.01, 0.1, 0.5, 1.0, 1.5, 2.0, 5.0, 10.0, 20.0,
                    50.0,
                ])
                .build(),
            optimize_item_count: opentelemetry::global::meter("pg_queue")
                .u64_histogram("pg_queue_optimize_item_count")
                .with_description("The amount of items processed in an optimize pass.")
                .with_boundaries(vec![
                    0.0, 0.00001, 0.0001, 0.001, 0.01, 0.1, 0.5, 1.0, 1.5, 2.0, 5.0, 10.0, 20.0,
                    50.0,
                ])
                .build(),
            processed_item_count: opentelemetry::global::meter("pg_queue")
                .u64_counter("pg_queue_processed_items_count")
                .with_description("Total count of successful messages processed.")
                .build(),
            fatal_errors_count: opentelemetry::global::meter("pg_queue")
                .u64_counter("pg_queue_fatal_error_count")
                .with_description("Total count of fatal errors encountered.")
                .build(),
            retryable_errors_count: opentelemetry::global::meter("pg_queue")
                .u64_counter("pg_queue_retryable_error_count")
                .with_description("Total count of retryable errors encountered.")
                .build(),
            unprocessable_count: opentelemetry::global::meter("pg_queue")
                .u64_counter("pg_queue_unprocessable_count")
                .with_description("Total count of unprocessable messages encountered.")
                .build(),
        }
    }
}
