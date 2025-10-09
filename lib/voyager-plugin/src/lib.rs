use std::{env::VarError, time::Duration};

use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use serde::de::DeserializeOwned;
use tracing::{Instrument, debug_span, instrument};
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

#[allow(async_fn_in_trait)]
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
                metrics_endpoint,
            } => {
                init(metrics_endpoint);

                let config = must_parse::<Self::Config>(&config);

                let info = Self::info(config.clone());

                let name = info.name;

                worker_server(
                    name.clone(),
                    coordinator_socket,
                    worker_socket,
                    Self::new(config),
                    Self::into_rpc,
                )
                .instrument(debug_span!("main", %name))
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

#[allow(async_fn_in_trait)]
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
                metrics_endpoint,
            } => {
                init(metrics_endpoint);

                let config = must_parse::<Self::Config>(&config);

                let info = must_parse::<StateModuleInfo>(&info);

                let name = info.id();

                worker_server(
                    name.clone(),
                    coordinator_socket,
                    worker_socket,
                    Self::new(config, info),
                    Self::into_rpc,
                )
                .instrument(debug_span!("run_state_module_server", %name))
                .await
            }
        }
    }
}

#[allow(async_fn_in_trait)]
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
                metrics_endpoint,
            } => {
                init(metrics_endpoint);

                let config = must_parse::<Self::Config>(&config);

                let info = must_parse::<ProofModuleInfo>(&info);

                let name = info.id();

                worker_server(
                    name.clone(),
                    coordinator_socket,
                    worker_socket,
                    Self::new(config, info),
                    Self::into_rpc,
                )
                .instrument(debug_span!("run_proof_module_server", %name))
                .await
            }
        }
    }
}

#[allow(async_fn_in_trait)]
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
                metrics_endpoint,
            } => {
                init(metrics_endpoint);

                let config = must_parse::<Self::Config>(&config);

                let info = must_parse::<FinalityModuleInfo>(&info);

                let name = info.id();

                worker_server(
                    name.clone(),
                    coordinator_socket,
                    worker_socket,
                    Self::new(config, info),
                    Self::into_rpc,
                )
                .instrument(debug_span!("run_finality_module_server", %name))
                .await
            }
        }
    }
}

#[allow(async_fn_in_trait)]
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
                metrics_endpoint,
            } => {
                init(metrics_endpoint);

                let config = must_parse::<Self::Config>(&config);

                let info = must_parse::<ClientModuleInfo>(&info);

                let name = info.id();

                worker_server(
                    name.clone(),
                    coordinator_socket,
                    worker_socket,
                    Self::new(config, info),
                    Self::into_rpc,
                )
                .instrument(debug_span!("run_client_module_server", %name))
                .await
            }
        }
    }
}

#[allow(async_fn_in_trait)]
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
                metrics_endpoint,
            } => {
                init(metrics_endpoint);

                let config = must_parse::<Self::Config>(&config);

                let info = must_parse::<ClientBootstrapModuleInfo>(&info);

                let name = info.id();

                worker_server(
                    name.clone(),
                    coordinator_socket,
                    worker_socket,
                    Self::new(config, info),
                    Self::into_rpc,
                )
                .instrument(debug_span!("run_client_bootstrap_module_server", %name))
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
        metrics_endpoint: Option<String>,
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
        metrics_endpoint: Option<String>,
    },
}

// set up logging and metrics
fn init(metrics_endpoint: Option<String>) {
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

    if let Some(metrics_endpoint) = metrics_endpoint {
        let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .with_endpoint(&metrics_endpoint)
            .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
            .with_timeout(Duration::from_secs(3))
            .build()
            .expect("unable to build metrics exporter");

        let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
            .with_periodic_exporter(metric_exporter)
            .with_resource(
                opentelemetry_sdk::Resource::builder_empty()
                    .with_attributes([KeyValue::new("process.name", "voyager")])
                    .build(),
            )
            .build();

        opentelemetry::global::set_meter_provider(provider);
    };

    match format {
        LogFormat::Text => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_filter(EnvFilter::from_default_env()))
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

#[instrument(level = "debug", fields(%config_str))]
fn must_parse<T: DeserializeOwned>(config_str: &str) -> T {
    match serde_json::from_str::<T>(config_str) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("invalid config: {}", ErrorReporter(err));
            std::process::exit(INVALID_CONFIG_EXIT_CODE as i32);
        }
    }
}
