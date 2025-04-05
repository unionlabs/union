#![feature(trait_alias, try_find)]
#![warn(clippy::pedantic, clippy::unwrap_used)]
#![allow(
    // required due to return_position_impl_trait_in_trait false positives
    clippy::manual_async_fn,
    clippy::single_match_else,
    clippy::module_name_repetitions,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]

use std::{
    collections::HashMap, ffi::OsStr, fmt::Write, fs::read_to_string, iter, path::PathBuf,
    process::ExitCode, time::Duration,
};

use anyhow::{anyhow, Context as _};
use clap::Parser;
use ibc_classic_spec::IbcClassic;
use ibc_union_spec::IbcUnion;
use pg_queue::{
    default_max_connections, default_min_connections, default_retryable_error_expo_backoff_max,
    default_retryable_error_expo_backoff_multiplier, PgQueueConfig,
};
use schemars::gen::{SchemaGenerator, SchemaSettings};
use serde::Serialize;
use tikv_jemallocator::Jemalloc;
use tracing::info;
use tracing_subscriber::EnvFilter;
use voyager_message::{
    call::{FetchBlocks, FetchUpdateHeaders},
    callback::AggregateSubmitTxFromOrderedHeaders,
    context::{
        equivalent_chain_ids::EquivalentChainIds, get_plugin_info,
        ibc_spec_handler::IbcSpecHandler, Context, ModulesConfig,
    },
    filter::{make_filter, run_filter, JaqInterestFilter},
    primitives::{IbcSpec, QueryHeight},
    rpc::{server::cache, IbcState, VoyagerRpcClient},
    VoyagerMessage,
};
use voyager_vm::{call, filter::FilterResult, promise, Op, Queue};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use crate::{
    cli::{AppArgs, Command, ConfigCmd, ModuleCmd, MsgCmd, PluginCmd, QueueCmd, RpcCmd},
    config::{
        default_metrics_endpoint, default_rest_laddr, default_rpc_laddr, Config, VoyagerConfig,
    },
    queue::{QueueConfig, Voyager},
    utils::make_msg_create_client,
};

#[cfg(windows)]
compile_error!(
    "voyager interacts directly with subprocesses and has \
    not been tested on windows."
);

pub mod api;
pub mod cli;
pub mod config;
pub mod metrics;
pub mod queue;

fn main() -> ExitCode {
    let args = AppArgs::parse();

    match args.log_format {
        cli::LogFormat::Text => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                // .with_span_events(FmtSpan::CLOSE)
                .init();
        }
        cli::LogFormat::Json => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                // .with_span_events(FmtSpan::CLOSE)
                .json()
                .init();
        }
    }

    let res = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(args.stack_size)
        .build()
        .expect("building the tokio runtime is infallible; qed;")
        .block_on(do_main(args));

    match res {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            let errs = iter::successors(err.source(), |e| (*e).source())
                .enumerate()
                .fold(format!("0: {err}\n"), |mut acc, (i, e)| {
                    writeln!(acc, "{}: {e}", i + 1).unwrap();
                    acc
                });

            eprintln!("{errs}");

            ExitCode::FAILURE
        }
    }
}

#[allow(clippy::too_many_lines)]
// NOTE: This function is a mess, will be cleaned up
async fn do_main(args: cli::AppArgs) -> anyhow::Result<()> {
    let get_voyager_config = || match &args.config_file_path {
        Some(config_file_path) => {
            let config_file_path = PathBuf::from(config_file_path);
            let ext = config_file_path.extension();
            read_to_string(&config_file_path)
                .with_context(|| {
                    format!(
                        "unable to read the config file at `{}`",
                        config_file_path.to_string_lossy()
                    )
                })
                .and_then(|s| match ext.map(OsStr::as_encoded_bytes) {
                    Some(b"jsonc") => serde_jsonc::from_str::<Config>(&s).with_context(|| {
                        format!(
                            "unable to parse the config file at `{}`",
                            config_file_path.to_string_lossy()
                        )
                    }),
                    _ => serde_json::from_str::<Config>(&s).with_context(|| {
                        format!(
                            "unable to parse the config file at `{}`",
                            config_file_path.to_string_lossy()
                        )
                    }),
                })
        }
        None => Err(anyhow!("config file must be specified")),
    };

    let get_rest_url = |rest_url: Option<String>| match (get_voyager_config(), rest_url) {
        (Ok(config), None) => format!("http://{}", config.voyager.rest_laddr),
        (_, Some(rest_url)) => rest_url,
        (Err(_), None) => format!("http://{}", default_rest_laddr()),
    };

    let get_rpc_url = |rpc_url: Option<String>| match (get_voyager_config(), rpc_url) {
        (Ok(config), None) => format!("http://{}", config.voyager.rpc_laddr),
        (_, Some(rpc_url)) => rpc_url,
        (Err(_), None) => format!("http://{}", default_rpc_laddr()),
    };

    match args.command {
        Command::Config(cmd) => match cmd {
            ConfigCmd::Print => {
                print_json(&get_voyager_config()?);
            }
            ConfigCmd::Default => print_json(&Config {
                schema: None,
                equivalent_chain_ids: EquivalentChainIds::default(),
                modules: ModulesConfig {
                    state: vec![],
                    proof: vec![],
                    consensus: vec![],
                    client: vec![],
                    client_bootstrap: vec![],
                },
                plugins: vec![],
                voyager: VoyagerConfig {
                    num_workers: 1,
                    rest_laddr: default_rest_laddr(),
                    rpc_laddr: default_rpc_laddr(),
                    metrics_endpoint: default_metrics_endpoint(),
                    queue: QueueConfig::PgQueue(PgQueueConfig {
                        database_url: "postgres://postgres:postgrespassword@127.0.0.1:5432/default"
                            .into(),
                        max_connections: default_max_connections(),
                        min_connections: default_min_connections(),
                        idle_timeout: None,
                        max_lifetime: None,
                        optimize_batch_limit: None,
                        retryable_error_expo_backoff_max: default_retryable_error_expo_backoff_max(
                        ),
                        retryable_error_expo_backoff_multiplier:
                            default_retryable_error_expo_backoff_multiplier(),
                    }),
                    optimizer_delay_milliseconds: 100,
                    ipc_client_request_timeout: Duration::new(60, 0),
                    cache: voyager_message::rpc::server::cache::Config::default(),
                },
            }),
            ConfigCmd::Schema => print_json(
                &SchemaGenerator::new(SchemaSettings::draft2019_09().with(|s| {
                    s.option_nullable = true;
                    s.option_add_null_type = false;
                }))
                .into_root_schema_for::<Config>(),
            ),
        },
        Command::Start => {
            let config = get_voyager_config()?;

            metrics::init(&config.voyager.metrics_endpoint);

            let voyager = Voyager::new(config).await?;

            info!("starting relay service");

            voyager.run().await?;
        }
        Command::Plugin(cmd) => match cmd {
            PluginCmd::Interest {
                plugin_name,
                message,
            } => {
                let plugin_config = get_voyager_config()?
                    .plugins
                    .into_iter()
                    .try_find(|plugin_config| {
                        <anyhow::Result<_>>::Ok(plugin_name == get_plugin_info(plugin_config)?.name)
                    })?
                    .ok_or(anyhow!("plugin not found"))?;

                let (filter, plugin_name) = make_filter(get_plugin_info(&plugin_config)?)?;

                let result = run_filter(
                    &filter,
                    &plugin_name,
                    serde_json::from_str::<serde_json::Value>(&message)?.into(),
                );

                match result {
                    Ok(FilterResult::Interest(tag)) => println!("interest ({tag})"),
                    Ok(FilterResult::NoInterest) => println!("no interest"),
                    Err(()) => println!("failed"),
                }
            }
            PluginCmd::Info { plugin_name } => {
                let plugin_config = get_voyager_config()?
                    .plugins
                    .into_iter()
                    .try_find(|plugin_config| {
                        <anyhow::Result<_>>::Ok(plugin_name == get_plugin_info(plugin_config)?.name)
                    })?
                    .ok_or(anyhow!("plugin not found"))?;

                print_json(&get_plugin_info(&plugin_config)?);
            }
            PluginCmd::Call { plugin_name, args } => {
                let plugin_config = get_voyager_config()?
                    .plugins
                    .into_iter()
                    .try_find(|plugin_config| {
                        <anyhow::Result<_>>::Ok(plugin_name == get_plugin_info(plugin_config)?.name)
                    })?
                    .ok_or(anyhow!("plugin not found"))?;

                tokio::process::Command::new(&plugin_config.path)
                    .arg("cmd")
                    .arg("--config")
                    .arg(plugin_config.config.to_string())
                    .args(args)
                    .spawn()?
                    .wait()
                    .await?;
            }
            PluginCmd::List => {
                let list = get_voyager_config()?
                    .plugins
                    .into_iter()
                    .map(|module_config| get_plugin_info(&module_config).map(|p| p.name))
                    .collect::<Result<Vec<_>, _>>()?;

                print_json(&list);
            }
        },
        Command::Module(cmd) => match cmd {
            ModuleCmd::State(_) => todo!(),
            ModuleCmd::Proof(_) => todo!(),
            ModuleCmd::Consensus(_) => todo!(),
            ModuleCmd::Client(_) => todo!(),
        },
        Command::Queue(cli_msg) => {
            let db = || {
                Ok(match get_voyager_config()?.voyager.queue {
                    QueueConfig::PgQueue(cfg) => pg_queue::PgQueue::<VoyagerMessage>::new(cfg),
                    QueueConfig::InMemory => {
                        return Err(anyhow!(
                            "no database set in config, queue commands \
                            require the `pg-queue` database backend"
                        ))
                    }
                })
            };

            match cli_msg {
                QueueCmd::Enqueue { op, rest_url } => {
                    let rest_url = get_rest_url(rest_url);

                    send_enqueue(&rest_url, op).await?;
                }
                // NOTE: Temporarily disabled until i figure out a better way to implement this with the new queue design
                // cli::QueueCmd::History { id, max_depth } => {
                //     // let results = query_as!(
                //     //     Record,
                //     //     r#"SELECT id as "id!", parent, item as "item!: Item" FROM get_list($1, $2) ORDER BY id ASC"#,
                //     //     id.inner(),
                //     //     max_depth.inner()
                //     // )
                //     // .fetch_all(&db)
                //     // .await
                //     // .unwrap();

                //     // println!("{}", serde_json::to_string_pretty(&results).unwrap());

                //     todo!();
                // }
                QueueCmd::QueryFailed {
                    page,
                    per_page,
                    item_filters,
                    message_filters,
                } => {
                    let record = db()?
                        .await?
                        .query_failed(page.into(), per_page.into(), item_filters, message_filters)
                        .await?;

                    print_json(&record);
                }
                QueueCmd::QueryFailedById { id, requeue } => {
                    let q = db()?.await?;

                    let record = q.query_failed_by_id(id.inner()).await?;

                    if requeue {
                        if let Some(record) = record.as_ref().map(|r| r.item.0.clone()) {
                            let res = q
                                .enqueue(
                                    record,
                                    &JaqInterestFilter::new(vec![])
                                        .expect("empty filter can be built"),
                                )
                                .await?;
                            print_json(&res);
                        }
                    } else {
                        print_json(&record);
                    }
                }
            }
        }
        Command::InitFetch {
            chain_id,
            height,
            enqueue,
            rpc_url,
            rest_url,
        } => {
            let rpc_url = get_rpc_url(rpc_url);
            let rest_url = get_rest_url(rest_url);

            let voyager_client = jsonrpsee::http_client::HttpClient::builder().build(rpc_url)?;

            let start_height = match height {
                QueryHeight::Latest => {
                    voyager_client
                        .query_latest_height(chain_id.clone(), false)
                        .await?
                }
                QueryHeight::Finalized => {
                    voyager_client
                        .query_latest_height(chain_id.clone(), true)
                        .await?
                }
                QueryHeight::Specific(height) => height,
            };

            let op = call::<VoyagerMessage>(FetchBlocks {
                chain_id: chain_id.clone(),
                start_height,
            });

            if enqueue {
                println!("enqueueing op for {chain_id} at {start_height}");
                send_enqueue(&rest_url, op).await?;
            } else {
                print_json(&op);
            }
        }
        Command::Rpc { cmd, rpc_url } => {
            let rpc_url = get_rpc_url(rpc_url);

            let voyager_client = jsonrpsee::http_client::HttpClient::builder().build(rpc_url)?;

            let ibc_handlers = [
                (IbcClassic::ID, IbcSpecHandler::new::<IbcClassic>()),
                (IbcUnion::ID, IbcSpecHandler::new::<IbcUnion>()),
            ]
            .into_iter()
            .collect::<HashMap<_, _>>();

            match cmd {
                RpcCmd::Info => print_json(&voyager_client.info().await?),
                RpcCmd::ClientMeta {
                    on,
                    client_id,
                    ibc_spec_id,
                    height,
                } => {
                    let client_state_meta = voyager_client
                        .client_state_meta(
                            on.clone(),
                            ibc_spec_id.clone(),
                            height,
                            client_id.clone(),
                        )
                        .await?;

                    print_json(&client_state_meta);
                }
                RpcCmd::ClientInfo {
                    on,
                    client_id,
                    ibc_spec_id,
                } => {
                    let client_info = voyager_client
                        .client_info(on.clone(), ibc_spec_id.clone(), client_id.clone())
                        .await?;

                    print_json(&client_info);
                }
                RpcCmd::ClientState {
                    on,
                    client_id,
                    ibc_spec_id,
                    height,
                    decode,
                } => {
                    let ibc_state = voyager_client
                        .query_ibc_state(
                            on.clone(),
                            ibc_spec_id.clone(),
                            height,
                            (ibc_handlers
                                .get(&ibc_spec_id)
                                .context(anyhow!("unknown IBC spec `{ibc_spec_id}`"))?
                                .client_state_path)(client_id.clone())?,
                        )
                        .await?;

                    match (ibc_state.state, decode) {
                        (Some(state), true) => {
                            let client_info = voyager_client
                                .client_info(on, ibc_spec_id.clone(), client_id)
                                .await?
                                .ok_or(anyhow!("client info not found"))?;

                            let decoded = voyager_client
                                .decode_client_state(
                                    client_info.client_type,
                                    client_info.ibc_interface,
                                    ibc_spec_id,
                                    serde_json::from_value(state)
                                        .expect("serialization is infallible; qed;"),
                                )
                                .await?;

                            print_json(&IbcState {
                                height: ibc_state.height,
                                state: Some(decoded),
                            });
                        }
                        (state, _) => {
                            print_json(&IbcState {
                                height: ibc_state.height,
                                state,
                            });
                        }
                    }
                }
                RpcCmd::ConsensusState {
                    on,
                    client_id,
                    ibc_spec_id,
                    trusted_height,
                    height,
                    decode,
                    ..
                } => {
                    let ibc_state = voyager_client
                        .query_ibc_state(
                            on.clone(),
                            ibc_spec_id.clone(),
                            height,
                            (ibc_handlers
                                .get(&ibc_spec_id)
                                .context(anyhow!("unknown IBC spec `{ibc_spec_id}`"))?
                                .consensus_state_path)(
                                client_id.clone(),
                                trusted_height.to_string(),
                            )?,
                        )
                        .await?;

                    match (ibc_state.state, decode) {
                        (Some(state), true) => {
                            let client_info = voyager_client
                                .client_info(on, ibc_spec_id.clone(), client_id)
                                .await?
                                .ok_or(anyhow!("client info not found"))?;

                            let decoded = voyager_client
                                .decode_consensus_state(
                                    client_info.client_type,
                                    client_info.ibc_interface,
                                    ibc_spec_id,
                                    serde_json::from_value(state)
                                        .expect("serialization is infallible; qed;"),
                                )
                                .await?;

                            print_json(&IbcState {
                                height: ibc_state.height,
                                state: Some(decoded),
                            });
                        }
                        (state, _) => {
                            print_json(&IbcState {
                                height: ibc_state.height,
                                state,
                            });
                        }
                    }
                }
                RpcCmd::LatestHeight { on, finalized } => {
                    let height = voyager_client.query_latest_height(on, finalized).await?;
                    print_json(&height);
                }
                RpcCmd::LatestTimestamp { on, finalized } => {
                    let timestamp = voyager_client.query_latest_timestamp(on, finalized).await?;
                    print_json(&timestamp);
                }
            }
        }
        Command::Msg(msg) => match msg {
            // TODO: Do this all with rpc calls instead of spinning up a full voyager instance
            MsgCmd::CreateClient {
                on,
                tracking,
                ibc_interface,
                ibc_spec_id,
                client_type,
                height,
                metadata,
                enqueue,
                rest_url,
                client_state_config,
                consensus_state_config,
            } => {
                let rest_url = get_rest_url(rest_url);

                let voyager_config = get_voyager_config()?;

                let ctx = Context::new(
                    voyager_config.plugins,
                    voyager_config.modules,
                    voyager_config.equivalent_chain_ids,
                    |h| {
                        h.register::<IbcClassic>();
                        h.register::<IbcUnion>();
                    },
                    Duration::new(60, 0),
                    cache::Config::default(),
                )
                .await?;

                // weird race condition in Context::new that i don't feel like debugging right now
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                let op = make_msg_create_client(
                    &ctx,
                    tracking,
                    height,
                    on,
                    client_type,
                    ibc_interface,
                    ibc_spec_id,
                    metadata,
                    client_state_config,
                    consensus_state_config,
                )
                .await?;

                if enqueue {
                    send_enqueue(&rest_url, op).await?;
                } else {
                    print_json(&op);
                }
            }
            // TODO: Do this all with rpc calls instead of spinning up a full voyager instance
            MsgCmd::UpdateClient {
                on,
                client_id,
                ibc_spec_id,
                update_to,
                enqueue,
                rest_url,
            } => {
                let rest_url = get_rest_url(rest_url);

                let voyager_config = get_voyager_config()?;

                let ctx = Context::new(
                    voyager_config.plugins,
                    voyager_config.modules,
                    voyager_config.equivalent_chain_ids,
                    |h| {
                        h.register::<IbcClassic>();
                        h.register::<IbcUnion>();
                    },
                    Duration::new(60, 0),
                    cache::Config::default(),
                )
                .await?;

                // weird race condition in Context::new that i don't feel like debugging right now
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                let client_info = ctx
                    .rpc_server
                    .client_info(&on, &ibc_spec_id, client_id.clone())
                    .await?
                    .ok_or(anyhow!("client info not found"))?;

                let client_state_meta = ctx
                    .rpc_server
                    .client_state_meta(&on, &ibc_spec_id, QueryHeight::Latest, client_id.clone())
                    .await?
                    .ok_or(anyhow!("client info not found"))?;

                let update_to = match update_to {
                    Some(update_to) => update_to,
                    None => {
                        ctx.rpc_server
                            .query_latest_height(&client_state_meta.counterparty_chain_id, true)
                            .await?
                    }
                };

                let op = promise::<VoyagerMessage>(
                    [call(FetchUpdateHeaders {
                        client_type: client_info.client_type,
                        chain_id: client_state_meta.counterparty_chain_id,
                        counterparty_chain_id: on.clone(),
                        client_id: client_id.clone(),
                        update_from: client_state_meta.counterparty_height,
                        update_to,
                    })],
                    [],
                    AggregateSubmitTxFromOrderedHeaders {
                        ibc_spec_id: ibc_spec_id.clone(),
                        chain_id: on.clone(),
                        client_id: client_id.clone(),
                    },
                );

                if enqueue {
                    send_enqueue(&rest_url, op).await?;
                } else {
                    print_json(&op);
                }
            }
        },
    }

    Ok(())
}

async fn send_enqueue(
    rest_laddr: &str,
    op: Op<VoyagerMessage>,
) -> anyhow::Result<reqwest::Response> {
    Ok(reqwest::Client::new()
        .post(format!("{rest_laddr}/enqueue"))
        .json(&op)
        .send()
        .await?)
}

fn print_json<T: Serialize>(t: &T) {
    println!(
        "{}",
        serde_json::to_string(&t).expect("serialization is infallible; qed;")
    );
}

// TODO: Extract all logic here to a plugin
pub mod utils {
    use anyhow::bail;
    use ibc_classic_spec::IbcClassic;
    use ibc_union_spec::IbcUnion;
    use serde_json::Value;
    use tracing::trace;
    use voyager_message::{
        call::SubmitTx,
        context::Context,
        primitives::{ChainId, ClientType, IbcInterface, IbcSpecId, QueryHeight},
        data::IbcDatagram,
        module::{ClientBootstrapModuleClient, ClientModuleClient},
        VoyagerMessage,
    };
    use voyager_vm::{call, Op};

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn make_msg_create_client(
        ctx: &Context,
        counterparty_chain_id: ChainId,
        height: QueryHeight,
        chain_id: ChainId,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        metadata: Value,
        client_state_config: Value,
        consensus_state_config: Value,
    ) -> anyhow::Result<Op<VoyagerMessage>> {
        if height == QueryHeight::Latest {
            // TODO: Also check if a specific height was passed and ensure that that height is also finalized
            bail!("cannot create a client at a non-finalized height")
        }

        let height = ctx
            .rpc_server
            .query_height(&counterparty_chain_id, height)
            .await?;

        let counterparty_client_bootstrap_module = ctx
            .rpc_server
            .modules()?
            .client_bootstrap_module(&counterparty_chain_id, &client_type)?;

        let self_client_state = counterparty_client_bootstrap_module
            .self_client_state(height, client_state_config)
            .await?;
        trace!(%self_client_state);

        let self_consensus_state = counterparty_client_bootstrap_module
            .self_consensus_state(height, consensus_state_config)
            .await?;
        trace!(%self_consensus_state);

        // let consensus_type = ctx
        //     .rpc_server
        //     .modules()?
        //     .chain_consensus_type(&counterparty_chain_id)?;

        // let client_consensus_type = ctx
        //     .rpc_server
        //     .modules()?
        //     .client_consensus_type(&client_type)?;

        // if client_consensus_type != consensus_type {
        //     return Err(anyhow!(
        //         "attempted to create a {client_type} client on \
        //         {chain_id} tracking {counterparty_chain_id}, but \
        //         the consensus of that chain ({consensus_type}) is \
        //         not verifiable by a client of type {client_type} \
        //         (which instead verifies {client_consensus_type})."
        //     ));
        // }

        let client_module =
            ctx.rpc_server
                .modules()?
                .client_module(&client_type, &ibc_interface, &ibc_spec_id)?;

        Ok(call(SubmitTx {
            chain_id,
            datagrams: vec![match ibc_spec_id.as_str() {
                IbcSpecId::CLASSIC => IbcDatagram::new::<IbcClassic>(
                    ibc_classic_spec::Datagram::from(ibc_classic_spec::MsgCreateClientData {
                        msg: unionlabs::ibc::core::client::msg_create_client::MsgCreateClient {
                            client_state: client_module
                                .encode_client_state(self_client_state, metadata)
                                .await?,
                            consensus_state: client_module
                                .encode_consensus_state(self_consensus_state)
                                .await?,
                        },
                        client_type: client_type.clone(),
                    }),
                ),
                IbcSpecId::UNION => {
                    IbcDatagram::new::<IbcUnion>(ibc_union_spec::datagram::Datagram::from(
                        ibc_union_spec::datagram::MsgCreateClient {
                            client_type,
                            client_state_bytes: client_module
                                .encode_client_state(self_client_state, metadata)
                                .await?,
                            consensus_state_bytes: client_module
                                .encode_consensus_state(self_consensus_state)
                                .await?,
                        },
                    ))
                }
                _ => bail!("unknown IBC version id `{ibc_spec_id}`"),
            }],
        }))
    }
}
