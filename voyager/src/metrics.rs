use std::time::Duration;

use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::{MetricExporter, Protocol, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{metrics::SdkMeterProvider, trace::SdkTracerProvider};

pub fn init(endpoint: &str) {
    // let otlp_exporter = SpanExporter::builder().with_tonic().build().unwrap();

    // let tracer = SdkTracerProvider::builder()
    //     .with_simple_exporter(otlp_exporter)
    //     .build()
    //     .tracer("trace_demo");

    // // Create a layer with the configured tracer
    // let otel_layer = OpenTelemetryLayer::new(tracer);

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

    // global::set_text_map_propagator(TraceContextPropagator::new());

    global::set_meter_provider(provider);
}
