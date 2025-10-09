use std::time::Duration;

use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::{MetricExporter, Protocol, WithExportConfig};
use opentelemetry_sdk::metrics::SdkMeterProvider;

pub fn init(endpoint: &str) {
    let metric_exporter = MetricExporter::builder()
        .with_http()
        .with_endpoint(endpoint)
        .with_protocol(Protocol::HttpBinary)
        .with_timeout(Duration::from_secs(3))
        .build()
        .expect("unable to build metrics exporter");

    let provider = SdkMeterProvider::builder()
        .with_periodic_exporter(metric_exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder_empty()
                .with_attributes([KeyValue::new("process.name", "voyager")])
                .build(),
        )
        .build();

    global::set_meter_provider(provider);
}
