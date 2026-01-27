use std::{env::VarError, time::Duration};

use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_otlp::{MetricExporter, Protocol, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    metrics::{MeterProviderBuilder, PeriodicReader, Temporality},
    propagation::TraceContextPropagator,
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
};
use serde::de::DeserializeOwned;
use tracing::{Instrument, error, instrument, trace_span};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};
use unionlabs::ErrorReporter;
pub use voyager_plugin_protocol as protocol;
use voyager_plugin_protocol::{INVALID_CONFIG_EXIT_CODE, worker_server};
use voyager_primitives::IbcSpec;
use voyager_rpc::{
    ClientBootstrapModuleServer, ClientModuleServer, FinalityModuleServer, Member, PluginServer,
    ProofModuleServer, StateModuleServer,
    types::{
        ClientBootstrapModuleInfo, ClientModuleInfo, FinalityModuleInfo, PluginInfo,
        ProofModuleInfo, StateModuleInfo,
    },
};

#[expect(async_fn_in_trait)]
pub trait Plugin: PluginServer<Self::Call, Self::Callback> + Sized {
    type Call: Member;
    type Callback: Member;

    type Config: DeserializeOwned + Clone;
    type Cmd: clap::Subcommand;

    async fn new(config: Self::Config) -> anyhow::Result<Self>;

    fn info(config: Self::Config) -> PluginInfo;

    async fn cmd(config: Self::Config, cmd: Self::Cmd);

    async fn run() {
        let app = <PluginApp<Self::Cmd> as clap::Parser>::parse();

        match app {
            PluginApp::Run {
                worker_socket,
                coordinator_socket,
                config,
                trace_ratio,
            } => {
                let config = must_parse::<Self::Config>(&config);

                let info = Self::info(config.clone());

                let name = info.name;

                init(trace_ratio, &name);

                worker_server(
                    name.clone(),
                    coordinator_socket,
                    worker_socket,
                    Self::new(config),
                    Self::into_rpc,
                )
                .instrument(trace_span!("main", %name))
                .await;
            }
            PluginApp::Info { config } => {
                let info = Self::info(must_parse(&config));

                print!("{}", serde_json::to_string(&info).unwrap())
            }
            PluginApp::Cmd { cmd, config } => Self::cmd(must_parse(&config), cmd).await,
        }
    }
}

#[expect(async_fn_in_trait)]
pub trait StateModule<V: IbcSpec>: StateModuleServer<V> + Sized {
    type Config: DeserializeOwned + Clone;

    async fn new(config: Self::Config, info: StateModuleInfo) -> anyhow::Result<Self>;

    async fn run() {
        match <ModuleApp as clap::Parser>::parse() {
            ModuleApp::Run {
                worker_socket,
                coordinator_socket,
                config,
                info,
                trace_ratio,
            } => {
                let config = must_parse::<Self::Config>(&config);

                let info = must_parse::<StateModuleInfo>(&info);

                let name = info.id();

                init(trace_ratio, &name);

                worker_server(
                    name.clone(),
                    coordinator_socket,
                    worker_socket,
                    Self::new(config, info),
                    Self::into_rpc,
                )
                .instrument(trace_span!("run_state_module_server", %name))
                .await
            }
        }
    }
}

#[expect(async_fn_in_trait)]
pub trait ProofModule<V: IbcSpec>: ProofModuleServer<V> + Sized {
    type Config: DeserializeOwned + Clone;

    async fn new(config: Self::Config, info: ProofModuleInfo) -> anyhow::Result<Self>;

    async fn run() {
        match <ModuleApp as clap::Parser>::parse() {
            ModuleApp::Run {
                worker_socket,
                coordinator_socket,
                config,
                info,
                trace_ratio,
            } => {
                let config = must_parse::<Self::Config>(&config);

                let info = must_parse::<ProofModuleInfo>(&info);

                let name = info.id();

                init(trace_ratio, &name);

                worker_server(
                    name.clone(),
                    coordinator_socket,
                    worker_socket,
                    Self::new(config, info),
                    Self::into_rpc,
                )
                .instrument(trace_span!("run_proof_module_server", %name))
                .await
            }
        }
    }
}

#[expect(async_fn_in_trait)]
pub trait FinalityModule: FinalityModuleServer + Sized {
    type Config: DeserializeOwned + Clone;

    async fn new(config: Self::Config, info: FinalityModuleInfo) -> anyhow::Result<Self>;

    async fn run() {
        match <ModuleApp as clap::Parser>::parse() {
            ModuleApp::Run {
                worker_socket,
                coordinator_socket,
                config,
                info,
                trace_ratio,
            } => {
                let config = must_parse::<Self::Config>(&config);

                let info = must_parse::<FinalityModuleInfo>(&info);

                let name = info.id();

                init(trace_ratio, &name);

                worker_server(
                    name.clone(),
                    coordinator_socket,
                    worker_socket,
                    Self::new(config, info),
                    Self::into_rpc,
                )
                .instrument(trace_span!("run_finality_module_server", %name))
                .await
            }
        }
    }
}

#[expect(async_fn_in_trait)]
pub trait ClientModule: ClientModuleServer + Sized {
    type Config: DeserializeOwned + Clone;

    async fn new(config: Self::Config, info: ClientModuleInfo) -> anyhow::Result<Self>;

    async fn run() {
        match <ModuleApp as clap::Parser>::parse() {
            ModuleApp::Run {
                worker_socket,
                coordinator_socket,
                config,
                info,
                trace_ratio,
            } => {
                let config = must_parse::<Self::Config>(&config);

                let info = must_parse::<ClientModuleInfo>(&info);

                let name = info.id();

                init(trace_ratio, &name);

                worker_server(
                    name.clone(),
                    coordinator_socket,
                    worker_socket,
                    Self::new(config, info),
                    Self::into_rpc,
                )
                .instrument(trace_span!("run_client_module_server", %name))
                .await
            }
        }
    }
}

#[expect(async_fn_in_trait)]
pub trait ClientBootstrapModule: ClientBootstrapModuleServer + Sized {
    type Config: DeserializeOwned + Clone;

    async fn new(config: Self::Config, info: ClientBootstrapModuleInfo) -> anyhow::Result<Self>;

    async fn run() {
        match <ModuleApp as clap::Parser>::parse() {
            ModuleApp::Run {
                worker_socket,
                coordinator_socket,
                config,
                info,
                trace_ratio,
            } => {
                let config = must_parse::<Self::Config>(&config);

                let info = must_parse::<ClientBootstrapModuleInfo>(&info);

                let name = info.id();

                init(trace_ratio, &name);

                worker_server(
                    name.clone(),
                    coordinator_socket,
                    worker_socket,
                    Self::new(config, info),
                    Self::into_rpc,
                )
                .instrument(trace_span!("run_client_bootstrap_module_server", %name))
                .await
            }
        }
    }
}

#[derive(clap::Parser)]
enum PluginApp<Cmd: clap::Subcommand> {
    Run {
        worker_socket: String,
        coordinator_socket: String,
        config: String,
        trace_ratio: Option<f64>,
    },
    Info {
        config: String,
    },
    Cmd {
        #[command(subcommand)]
        cmd: Cmd,
        #[arg(long)]
        config: String,
    },
}

#[derive(clap::Parser)]
enum ModuleApp {
    Run {
        worker_socket: String,
        coordinator_socket: String,
        config: String,
        info: String,
        trace_ratio: Option<f64>,
    },
}

// set up logging and metrics
fn init(trace_ratio: Option<f64>, name: &str) {
    enum LogFormat {
        Text,
        Json,
    }

    let format = match std::env::var("RUST_LOG_FORMAT").as_deref() {
        Err(VarError::NotPresent) | Ok("text") => LogFormat::Text,
        Ok("json") => LogFormat::Json,
        Err(VarError::NotUnicode(invalid)) => {
            eprintln!("invalid non-utf8 log format {invalid:?}, defaulting to text");
            LogFormat::Text
        }
        Ok(invalid) => {
            eprintln!("invalid log format {invalid}, defaulting to text");
            LogFormat::Text
        }
    };

    if let Some(trace_ratio) = trace_ratio {
        let resource = Resource::builder()
            .with_attribute(KeyValue::new("service.name", name.to_owned()))
            .build();

        let meter_provider = MeterProviderBuilder::default()
            .with_reader(
                PeriodicReader::builder(
                    MetricExporter::builder()
                        .with_http()
                        .with_protocol(Protocol::HttpBinary)
                        .with_temporality(Temporality::default())
                        .build()
                        .unwrap(),
                )
                .with_interval(Duration::from_secs(30))
                .build(),
            )
            .with_resource(resource.clone())
            .build();

        let tracer_provider = SdkTracerProvider::builder()
            .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
                trace_ratio,
            ))))
            .with_id_generator(RandomIdGenerator::default())
            .with_resource(resource)
            .with_batch_exporter(
                SpanExporter::builder()
                    .with_http()
                    .with_protocol(Protocol::HttpBinary)
                    .build()
                    .unwrap(),
            )
            .build();

        global::set_text_map_propagator(TraceContextPropagator::new());
        global::set_meter_provider(meter_provider.clone());
        global::set_tracer_provider(tracer_provider.clone());

        let registry = tracing_subscriber::registry()
            .with(tracing_opentelemetry::MetricsLayer::new(
                meter_provider.clone(),
            ))
            .with(
                OpenTelemetryLayer::new(tracer_provider.tracer("voyager")).with_filter(
                    // prevent reentrant tracing
                    EnvFilter::from_default_env()
                        .add_directive("tower=off".parse().expect("valid directive; qed;"))
                        .add_directive("hyper=off".parse().expect("valid directive; qed;"))
                        .add_directive("h2=off".parse().expect("valid directive; qed;"))
                        .add_directive("rustls=off".parse().expect("valid directive; qed;"))
                        .add_directive("reqwest=off".parse().expect("valid directive; qed;")),
                ),
            );

        match format {
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
        match format {
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
}

#[instrument(level = "info", fields(%config_str, current_exe = ?std::env::current_exe().unwrap_or_default()))]
fn must_parse<T: DeserializeOwned>(config_str: &str) -> T {
    match serde_json::from_str::<T>(config_str) {
        Ok(ok) => ok,
        Err(err) => {
            error!("invalid config: {}", ErrorReporter(err));
            std::process::exit(INVALID_CONFIG_EXIT_CODE as i32);
        }
    }
}
