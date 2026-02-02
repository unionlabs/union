use std::time::Duration;

use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_otlp::{MetricExporter, Protocol, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    metrics::{MeterProviderBuilder, PeriodicReader, Temporality},
    propagation::TraceContextPropagator,
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
};
use tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer};
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::LogFormat;

pub fn init_logging(log_format: LogFormat, trace_ratio: Option<f64>) -> anyhow::Result<()> {
    if let Some(trace_ratio) = trace_ratio {
        let resource = Resource::builder()
            .with_attribute(KeyValue::new("service.name", "voyager"))
            .build();

        let tracer_provider = SdkTracerProvider::builder()
            .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
                trace_ratio,
            ))))
            .with_id_generator(RandomIdGenerator::default())
            .with_resource(resource.clone())
            .with_batch_exporter(
                SpanExporter::builder()
                    .with_http()
                    .with_protocol(Protocol::HttpBinary)
                    .build()?,
            )
            .build();

        let meter_provider = MeterProviderBuilder::default()
            .with_reader(
                PeriodicReader::builder(
                    MetricExporter::builder()
                        .with_http()
                        .with_protocol(Protocol::HttpBinary)
                        .with_temporality(Temporality::default())
                        .build()?,
                )
                .with_interval(Duration::from_secs(30))
                .build(),
            )
            .build();

        global::set_meter_provider(meter_provider.clone());
        global::set_text_map_propagator(TraceContextPropagator::new());
        global::set_tracer_provider(tracer_provider.clone());

        let tracer = tracer_provider.tracer("voyager");

        let registry = tracing_subscriber::registry()
            .with(MetricsLayer::new(meter_provider))
            .with(
                OpenTelemetryLayer::new(tracer).with_filter(
                    EnvFilter::from_default_env()
                        // prevent reentrant tracing
                        .add_directive("tower=off".parse().expect("valid directive; qed;"))
                        .add_directive("hyper=off".parse().expect("valid directive; qed;"))
                        .add_directive("h2=off".parse().expect("valid directive; qed;"))
                        .add_directive("rustls=off".parse().expect("valid directive; qed;"))
                        .add_directive("reqwest=off".parse().expect("valid directive; qed;")),
                ),
            );

        match log_format {
            LogFormat::Text => {
                registry
                    .with(
                        tracing_subscriber::fmt::layer().with_filter(EnvFilter::from_default_env()),
                    )
                    .init();
            }
            LogFormat::Json => {
                registry
                    .with(
                        tracing_subscriber::fmt::layer()
                            .json()
                            .with_filter(EnvFilter::from_default_env()),
                    )
                    .init();
            }
        }
    } else {
        match log_format {
            LogFormat::Text => {
                tracing_subscriber::registry()
                    .with(
                        tracing_subscriber::fmt::layer().with_filter(EnvFilter::from_default_env()),
                    )
                    .init();
            }
            LogFormat::Json => {
                tracing_subscriber::registry()
                    .with(
                        tracing_subscriber::fmt::layer()
                            .json()
                            .with_filter(EnvFilter::from_default_env()),
                    )
                    .init();
            }
        }
    }

    Ok(())
}
