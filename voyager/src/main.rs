#![feature(trait_alias, try_find)]
#![warn(
    clippy::pedantic,
    clippy::unwrap_used,
    closure_returning_async_block // TODO: Make this workspace-wide
)]
#![allow(
    clippy::single_match_else,
    clippy::module_name_repetitions,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]

use std::{collections::HashMap, time::Duration};

use anyhow::{Context as _, anyhow};
use clap::Parser;
use ibc_classic_spec::IbcClassic;
use ibc_union_spec::IbcUnion;
use pg_queue::{
    PgQueueConfig, Tables, default_max_connections, default_min_connections,
    default_retryable_error_expo_backoff_max, default_retryable_error_expo_backoff_multiplier,
};
use reqwest::Url;
use schemars::r#gen::{SchemaGenerator, SchemaSettings};
use serde::Serialize;
use serde_json::Value;
use tikv_jemallocator::Jemalloc;
use tracing::info;
use voyager_client::VoyagerClient;
use voyager_core::{
    Engine,
    context::ModulesConfig,
    default_rest_laddr, default_rpc_laddr, default_trace_ratio,
    equivalent_chain_ids::EquivalentChainIds,
    filter::{JaqFilterResult, make_filter, run_filter},
    get_plugin_info,
    ibc_spec_handlers::IbcSpecHandler,
};
use voyager_message::{
    VoyagerMessage,
    call::{FetchUpdateHeaders, Index, IndexRange, IndexRangeHeights},
    callback::AggregateSubmitTxFromOrderedHeaders,
};
use voyager_primitives::{IbcSpec, QueryHeight};
use voyager_rpc::{VoyagerRpcClient, types::IbcStateResponse};
use voyager_vm::{Op, Queue, call, promise};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use crate::{
    cli::{App, Command, ConfigCmd, MsgCmd, PluginCmd, QueueCmd, RpcCmd, get_voyager_config},
    config::{Config, VoyagerConfig},
    metrics::init_logging,
    queue::{QueueConfig, QueueImpl},
};

#[cfg(windows)]
compile_error!(
    "voyager interacts directly with subprocesses and has \
    not been tested on windows."
);

pub mod cli;
pub mod config;
pub mod metrics;
pub mod queue;

fn main() -> anyhow::Result<()> {
    let app = App::parse();

    init_logging(
        app.log_format,
        get_voyager_config(app.config_file_path.as_deref()).map_or(None, |c| c.voyager.trace_ratio),
    )?;

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(app.stack_size)
        .build()
        .expect("building the tokio runtime is infallible; qed;")
        .block_on(do_main(app))
}

#[allow(clippy::too_many_lines)]
// NOTE: This function is a mess, will be cleaned up
async fn do_main(app: cli::App) -> anyhow::Result<()> {
    let get_voyager_config = || get_voyager_config(app.config_file_path.as_deref());

    let get_rest_url = |rest_url: Option<String>| match (get_voyager_config(), rest_url) {
        (Ok(config), None) => format!("http://{}", config.voyager.rest_laddr),
        (_, Some(rest_url)) => rest_url,
        (Err(_), None) => format!("http://{}", default_rest_laddr()),
    };

    let get_rpc_url = |rpc_url: Option<String>| match (get_voyager_config(), rpc_url) {
        (Ok(config), None) => format!("http://{}", config.voyager.rpc_laddr),
        (_, Some(rpc_url)) => {
            Url::parse(&rpc_url).map_or_else(|_| format!("https://{rpc_url}"), |e| e.to_string())
        }
        (Err(_), None) => format!("http://{}", default_rpc_laddr()),
    };

    match app.command {
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
                    trace_ratio: default_trace_ratio(),
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
                        vacuum_on_boot: false,
                    }),
                    optimizer_delay_milliseconds: 100,
                    ipc_client_request_timeout: Duration::new(60, 0),
                    cache: voyager_core::cache::Config::default(),
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

            // metrics::init(&config.voyager.metrics_endpoint);

            let voyager = Engine::builder()
                .with_equivalent_chain_ids(config.equivalent_chain_ids)
                .with_plugins(config.plugins)
                .with_modules(config.modules)
                .with_ipc_client_request_timeout(config.voyager.ipc_client_request_timeout)
                .with_cache_config(config.voyager.cache)
                .with_trace_ratio(config.voyager.trace_ratio)
                .with_num_workers(config.voyager.num_workers.into())
                .with_rest_laddr(config.voyager.rest_laddr)
                .with_rpc_laddr(config.voyager.rpc_laddr)
                .with_optimizer_delay_milliseconds(config.voyager.optimizer_delay_milliseconds)
                .with_queue::<QueueImpl>(config.voyager.queue)
                .register_ibc_spec_handler::<IbcUnion>()
                .register_ibc_spec_handler::<IbcClassic>()
                .build()
                .await?;

            info!("starting relay service");

            voyager.run().await;
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
                    plugin_name,
                    serde_json::from_str::<serde_json::Value>(&message)?.into(),
                );

                match result {
                    Ok(JaqFilterResult::Take(tag)) => {
                        println!("interest (take, {tag})");
                    }
                    Ok(JaqFilterResult::Copy(tag)) => {
                        println!("interest (copy, {tag})");
                    }
                    Ok(JaqFilterResult::NoInterest) => println!("no interest"),
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
        Command::Queue(cli_msg) => {
            let db = || {
                Ok(match get_voyager_config()?.voyager.queue {
                    QueueConfig::PgQueue(cfg) => {
                        pg_queue::PgQueue::<VoyagerMessage>::new(PgQueueConfig {
                            // only one connection is needed for the queue commands
                            min_connections: 1,
                            max_connections: 1,
                            ..cfg
                        })
                    }
                    QueueConfig::InMemory => {
                        return Err(anyhow!(
                            "no database set in config, queue commands \
                            require the `pg-queue` database backend"
                        ));
                    }
                })
            };

            match cli_msg {
                QueueCmd::Enqueue { op, rest_url } => {
                    let rest_url = get_rest_url(rest_url);

                    send_enqueue(&rest_url, op).await?;
                }
                QueueCmd::Stats => {
                    let stats = db()?.await?.stats().await?;

                    print_json(&stats);
                }
                QueueCmd::Truncate {
                    queue,
                    optimize,
                    done,
                    failed,
                } => {
                    db()?
                        .await?
                        .truncate(Tables {
                            queue,
                            optimize,
                            done,
                            failed,
                        })
                        .await?;
                }
                QueueCmd::Vacuum {
                    queue,
                    optimize,
                    done,
                    failed,
                } => {
                    db()?
                        .await?
                        .vacuum(Tables {
                            queue,
                            optimize,
                            done,
                            failed,
                        })
                        .await?;
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
                QueueCmd::QueryFailedById {
                    id,
                    requeue,
                    rest_url,
                } => {
                    let rest_url = get_rest_url(rest_url);

                    let q = db()?.await?;

                    let record = q.query_failed_by_id(id.inner()).await?;

                    if requeue {
                        if let Some(op) = record.as_ref().map(|r| r.item.0.clone()) {
                            send_enqueue(&rest_url, op).await?;
                            println!("requeued");
                        }
                    } else {
                        print_json(&record);
                    }
                }
            }
        }
        Command::Index {
            chain_id,
            from,
            to,
            exact,
            enqueue,
            rpc_url,
            rest_url,
        } => {
            let rpc_url = get_rpc_url(rpc_url);
            let rest_url = get_rest_url(rest_url);

            let voyager_client = jsonrpsee::http_client::HttpClient::builder().build(rpc_url)?;

            let op = if let Some(exact) = exact {
                call::<VoyagerMessage>(IndexRange {
                    chain_id: chain_id.clone(),
                    range: IndexRangeHeights::new(exact, exact).expect("valid"),
                })
            } else {
                let start_height = match from {
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

                if let Some(to) = to {
                    call(IndexRange {
                        chain_id: chain_id.clone(),
                        range: IndexRangeHeights::new(start_height, to)?,
                    })
                } else {
                    call(Index {
                        chain_id: chain_id.clone(),
                        start_height,
                    })
                }
            };

            print_json(&op);

            if enqueue {
                send_enqueue(&rest_url, op).await?;
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
                RpcCmd::ConsensusMeta {
                    on,
                    ibc_spec_id,
                    client_id,
                    trusted_height,
                    height,
                } => {
                    let consensus_state_meta = voyager_client
                        .consensus_state_meta(
                            on.clone(),
                            ibc_spec_id.clone(),
                            height,
                            client_id.clone(),
                            trusted_height,
                        )
                        .await?;

                    print_json(&consensus_state_meta);
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

                            print_json(&IbcStateResponse {
                                height: ibc_state.height,
                                state: Some(decoded),
                            });
                        }
                        (state, _) => {
                            print_json(&IbcStateResponse {
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

                            print_json(&IbcStateResponse {
                                height: ibc_state.height,
                                state: Some(decoded),
                            });
                        }
                        (state, _) => {
                            print_json(&IbcStateResponse {
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
                RpcCmd::Query {
                    on,
                    ibc_spec_id,
                    query,
                } => {
                    let response = voyager_client.query(on, ibc_spec_id, query).await?;
                    print_json(&response);
                }
                RpcCmd::IbcState {
                    on,
                    ibc_spec_id,
                    height,
                    path,
                } => {
                    let response = voyager_client
                        .query_ibc_state(on, ibc_spec_id, height, path)
                        .await?;
                    print_json(&response);
                }
                RpcCmd::IbcProof {
                    on,
                    ibc_spec_id,
                    height,
                    path,
                    encode,
                    ibc_interface,
                    client_type,
                } => {
                    let response = voyager_client
                        .query_ibc_proof(on, ibc_spec_id.clone(), height, path)
                        .await?
                        .into_result()
                        .map_err(|e| anyhow!("{e}"))?;

                    if encode {
                        let encoded = voyager_client
                            .encode_proof(
                                client_type.expect("guaranteed to exist by clap; qed;"),
                                ibc_interface.expect("guaranteed to exist by clap; qed;"),
                                ibc_spec_id,
                                response.proof,
                            )
                            .await?;
                        print_json(&encoded);
                    } else {
                        print_json(&response);
                    }
                }
                RpcCmd::Plugin { name, method, args } => {
                    let response = voyager_client
                        .plugin_custom(
                            name,
                            method,
                            args.into_iter()
                                .map(|arg| arg.parse::<Value>().unwrap_or(Value::String(arg)))
                                .collect(),
                        )
                        .await?;
                    print_json(&response);
                }
            }
        }
        Command::Msg(msg) => match msg {
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
                rpc_url,
                client_state_config,
                consensus_state_config,
                config,
            } => {
                let client_state_config = if client_state_config.is_null() {
                    config.clone()
                } else {
                    client_state_config
                };
                let consensus_state_config = if consensus_state_config.is_null() {
                    config
                } else {
                    consensus_state_config
                };

                let voyager_client = VoyagerClient::new(
                    jsonrpsee::http_client::HttpClient::builder().build(get_rpc_url(rpc_url))?,
                );

                let op = utils::make_msg_create_client(
                    &voyager_client,
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
                    send_enqueue(&get_rest_url(rest_url), op).await?;
                } else {
                    print_json(&op);
                }
            }
            MsgCmd::UpdateClient {
                on,
                client_id,
                ibc_spec_id,
                update_to,
                update_from,
                enqueue,
                rest_url,
                rpc_url,
            } => {
                let voyager_client = VoyagerClient::new(
                    jsonrpsee::http_client::HttpClient::builder().build(get_rpc_url(rpc_url))?,
                );

                let client_info = voyager_client
                    .client_info_raw(on.clone(), ibc_spec_id.clone(), client_id.clone())
                    .await
                    .map_err(|e| anyhow!("{e}"))?;

                let client_state_meta = voyager_client
                    .client_state_meta_raw(
                        on.clone(),
                        ibc_spec_id.clone(),
                        QueryHeight::Latest,
                        client_id.clone(),
                    )
                    .await
                    .map_err(|e| anyhow!("{e}"))?;

                let update_to = match update_to {
                    Some(update_to) => update_to,
                    None => voyager_client
                        .query_latest_height(client_state_meta.counterparty_chain_id.clone(), true)
                        .await
                        .map_err(|e| anyhow!("{e}"))?,
                };

                let update_from = update_from.unwrap_or(client_state_meta.counterparty_height);

                let op = promise::<VoyagerMessage>(
                    [call(FetchUpdateHeaders {
                        client_type: client_info.client_type,
                        chain_id: client_state_meta.counterparty_chain_id,
                        counterparty_chain_id: on.clone(),
                        client_id: client_id.clone(),
                        update_from,
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
                    send_enqueue(&get_rest_url(rest_url), op).await?;
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

// // TODO: Extract all logic here to a plugin
pub mod utils {
    use anyhow::{anyhow, bail};
    use ibc_classic_spec::IbcClassic;
    use ibc_union_spec::IbcUnion;
    use jsonrpsee::core::client::ClientT;
    use serde_json::Value;
    use tracing::trace;
    use voyager_client::VoyagerClient;
    use voyager_message::{VoyagerMessage, call::SubmitTx, data::IbcDatagram};
    use voyager_primitives::{ChainId, ClientType, IbcInterface, IbcSpecId, QueryHeight};
    use voyager_vm::{Op, call};

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn make_msg_create_client<C: ClientT + Send + Sync>(
        voyager_client: &VoyagerClient<C>,
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

        let height = match height {
            QueryHeight::Latest => voyager_client
                .query_latest_height(counterparty_chain_id.clone(), false)
                .await
                .map_err(|e| anyhow!("{e}"))?,
            QueryHeight::Finalized => voyager_client
                .query_latest_height(counterparty_chain_id.clone(), true)
                .await
                .map_err(|e| anyhow!("{e}"))?,
            QueryHeight::Specific(height) => height,
        };

        let self_client_state = voyager_client
            .self_client_state(
                counterparty_chain_id.clone(),
                client_type.clone(),
                QueryHeight::Specific(height),
                client_state_config,
            )
            .await
            .map_err(|e| anyhow!("{e}"))?
            .state;
        trace!(%self_client_state);

        let self_consensus_state = voyager_client
            .self_consensus_state(
                counterparty_chain_id.clone(),
                client_type.clone(),
                QueryHeight::Specific(height),
                consensus_state_config,
            )
            .await
            .map_err(|e| anyhow!("{e}"))?
            .state;
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

        Ok(call(SubmitTx {
            chain_id,
            datagrams: vec![match ibc_spec_id.as_str() {
                IbcSpecId::CLASSIC => IbcDatagram::new::<IbcClassic>(
                    ibc_classic_spec::Datagram::from(ibc_classic_spec::MsgCreateClientData {
                        msg: unionlabs::ibc::core::client::msg_create_client::MsgCreateClient {
                            client_state: voyager_client
                                .encode_client_state::<IbcClassic>(
                                    client_type.clone(),
                                    ibc_interface.clone(),
                                    self_client_state,
                                    metadata,
                                )
                                .await
                                .map_err(|e| anyhow!("{e}"))?,
                            consensus_state: voyager_client
                                .encode_consensus_state::<IbcClassic>(
                                    client_type.clone(),
                                    ibc_interface.clone(),
                                    self_consensus_state,
                                )
                                .await
                                .map_err(|e| anyhow!("{e}"))?,
                        },
                        client_type: client_type.clone(),
                    }),
                ),
                IbcSpecId::UNION => {
                    IbcDatagram::new::<IbcUnion>(ibc_union_spec::datagram::Datagram::from(
                        ibc_union_spec::datagram::MsgCreateClient {
                            client_type: client_type.clone(),
                            client_state_bytes: voyager_client
                                .encode_client_state::<IbcUnion>(
                                    client_type.clone(),
                                    ibc_interface.clone(),
                                    self_client_state,
                                    metadata,
                                )
                                .await
                                .map_err(|e| anyhow!("{e}"))?,
                            consensus_state_bytes: voyager_client
                                .encode_consensus_state::<IbcUnion>(
                                    client_type.clone(),
                                    ibc_interface.clone(),
                                    self_consensus_state,
                                )
                                .await
                                .map_err(|e| anyhow!("{e}"))?,
                        },
                    ))
                }
                _ => bail!("unknown IBC version id `{ibc_spec_id}`"),
            }],
        }))
    }
}
