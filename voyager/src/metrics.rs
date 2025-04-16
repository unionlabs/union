use std::time::Duration;

use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;

pub fn init(endpoint: &str) {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_endpoint(endpoint)
        .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
        .with_timeout(Duration::from_secs(3))
        .build()
        .expect("unable to build metrics exporter");

    let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder_empty()
                .with_attributes([KeyValue::new("process.name", "voyager")])
                .build(),
        )
        .build();

    opentelemetry::global::set_meter_provider(provider);
}
