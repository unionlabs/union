#![feature(trait_alias, try_find)]
#![warn(clippy::pedantic)]
#![allow(
    // required due to return_position_impl_trait_in_trait false positives
    clippy::manual_async_fn,
    clippy::single_match_else,
    clippy::module_name_repetitions,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]

use std::{
    collections::HashMap, fmt::Write, fs::read_to_string, iter, net::SocketAddr, process::ExitCode,
};

use anyhow::{anyhow, Context as _};
use clap::Parser;
use ibc_classic_spec::IbcClassic;
use ibc_union_spec::IbcUnion;
use pg_queue::PgQueueConfig;
use schemars::gen::{SchemaGenerator, SchemaSettings};
use serde::Serialize;
use tikv_jemallocator::Jemalloc;
use tracing::info;
use tracing_subscriber::EnvFilter;
use voyager_message::{
    call::FetchBlocks,
    context::{get_plugin_info, Context, IbcSpecHandler, ModulesConfig},
    core::{IbcSpec, QueryHeight},
    filter::{make_filter, run_filter, JaqInterestFilter},
    rpc::{IbcState, VoyagerRpcClient},
    VoyagerMessage,
};
use voyager_vm::{call, filter::FilterResult, Op, Queue};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use crate::{
    cli::{AppArgs, Command, ConfigCmd, ModuleCmd, MsgCmd, PluginCmd, QueueCmd, RpcCmd},
    config::{default_rest_laddr, default_rpc_laddr, Config, VoyagerConfig},
    queue::{QueueConfig, Voyager},
    utils::make_msg_create_client,
};

#[cfg(not(target_os = "linux"))]
compile_error!(
    "voyager interacts directly with subprocesses and has \
    not been tested on non-linux operating systems."
);

pub mod api;
pub mod cli;
pub mod config;
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
        .unwrap()
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
        Some(config_file_path) => read_to_string(config_file_path)
            .with_context(|| {
                format!(
                    "unable to read the config file at `{}`",
                    config_file_path.to_string_lossy()
                )
            })
            .and_then(|s| {
                serde_json::from_str::<Config>(&s).with_context(|| {
                    format!(
                        "unable to parse the config file at `{}`",
                        config_file_path.to_string_lossy()
                    )
                })
            }),
        None => Err(anyhow!("config file must be specified")),
    };

    match args.command {
        Command::Config(cmd) => match cmd {
            ConfigCmd::Print => {
                print_json(&get_voyager_config()?);
            }
            ConfigCmd::Default => print_json(&Config {
                schema: None,
                plugins: vec![],
                modules: ModulesConfig {
                    state: vec![],
                    proof: vec![],
                    consensus: vec![],
                    client: vec![],
                },
                voyager: VoyagerConfig {
                    num_workers: 1,
                    rest_laddr: default_rest_laddr(),
                    rpc_laddr: default_rpc_laddr(),
                    queue: QueueConfig::PgQueue(PgQueueConfig {
                        database_url: String::new(),
                        max_connections: None,
                        min_connections: None,
                        idle_timeout: None,
                        max_lifetime: None,
                    }),
                    optimizer_delay_milliseconds: 100,
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
            let voyager = Voyager::new(get_voyager_config()?).await?;

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
            PluginCmd::Call { plugin_name, args } => match plugin_name {
                Some(module_name) => {
                    let plugin_config = get_voyager_config()?
                        .plugins
                        .into_iter()
                        .try_find(|plugin_config| {
                            <anyhow::Result<_>>::Ok(
                                module_name == get_plugin_info(plugin_config)?.name,
                            )
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
                None => {
                    println!("available plugins and modules");
                    for module_config in get_voyager_config()?.plugins {
                        println!("  {}", get_plugin_info(&module_config)?.name);
                    }
                }
            },
        },
        Command::Module(cmd) => match cmd {
            ModuleCmd::State(_) => todo!(),
            ModuleCmd::Proof(_) => todo!(),
            ModuleCmd::Consensus(_) => todo!(),
            ModuleCmd::Client(_) => todo!(),
        },
        // Command::Query { on, height, path } => {
        //     let voyager = Voyager::new(get_voyager_config()?).await?;

        //     let height = voyager.context.rpc_server.query_height(&on, height).await?;

        //     let state = voyager
        //         .context
        //         .rpc_server
        //         .query_ibc_state(&on, height, path.clone())
        //         .await?
        //         .state;

        //     let state = match &path {
        //         ics24::Path::ClientState(path) => {
        //             let client_info = voyager
        //                 .context
        //                 .rpc_server
        //                 .client_info(&on, path.client_id.clone())
        //                 .await?;

        //             voyager
        //                 .context
        //                 .rpc_server
        //                 .decode_client_state(
        //                     &client_info.client_type,
        //                     &client_info.ibc_interface,
        //                     serde_json::from_value::<Hex<Vec<u8>>>(state).unwrap().0,
        //                 )
        //                 .await?
        //         }
        //         ics24::Path::ClientConsensusState(path) => {
        //             let client_info = voyager
        //                 .context
        //                 .rpc_server
        //                 .client_info(&on, path.client_id.clone())
        //                 .await?;

        //             voyager
        //                 .context
        //                 .rpc_server
        //                 .decode_consensus_state(
        //                     &client_info.client_type,
        //                     &client_info.ibc_interface,
        //                     serde_json::from_value::<Hex<Vec<u8>>>(state).unwrap().0,
        //                 )
        //                 .await?
        //         }
        //         _ => state,
        //     };

        //     voyager.shutdown().await;

        //     print_json(&json!({
        //        "path": path.to_string(),
        //        "state": state,
        //     }));
        // }
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
                QueueCmd::Enqueue { op } => {
                    send_enqueue(&get_voyager_config()?.voyager.rest_laddr, op).await?;
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
                            q.enqueue(record, &JaqInterestFilter::new(vec![]).unwrap())
                                .await?;
                        }
                    }

                    print_json(&record);
                }
            }
        }
        // Command::Handshake(HandshakeCmd {
        //     chain_a,
        //     chain_b,
        //     ty,
        // }) => {
        //     let chain_a = voyager_config.get_chain(&chain_a).await?;
        //     let chain_b = voyager_config.get_chain(&chain_b).await?;

        //     let chains = Arc::new(chains_from_config(voyager_config.chain).await.unwrap());

        //     let all_msgs = match (chain_a, chain_b) {
        //         (AnyChain::Union(union), AnyChain::Cosmos(cosmos)) => {
        //             mk_handshake::<Union, Wasm<Cosmos>>(&union, &Wasm(cosmos), ty, chains).await
        //         }
        //         (AnyChain::Union(union), AnyChain::EthereumMainnet(ethereum)) => {
        //             mk_handshake::<Wasm<Union>, Ethereum<Mainnet>>(
        //                 &Wasm(union),
        //                 &ethereum,
        //                 ty,
        //                 chains,
        //             )
        //             .await
        //         }
        //         (AnyChain::Union(union), AnyChain::EthereumMinimal(ethereum)) => {
        //             mk_handshake::<Wasm<Union>, Ethereum<Minimal>>(
        //                 &Wasm(union),
        //                 &ethereum,
        //                 ty,
        //                 chains,
        //             )
        //             .await
        //         }
        //         (AnyChain::Union(union), AnyChain::Scroll(scroll)) => {
        //             mk_handshake::<Wasm<Union>, Scroll>(&Wasm(union), &scroll, ty, chains).await
        //         }
        //         (AnyChain::Union(union), AnyChain::Arbitrum(scroll)) => {
        //             mk_handshake::<Wasm<Union>, Arbitrum>(&Wasm(union), &scroll, ty, chains).await
        //         }
        //         (AnyChain::Union(union), AnyChain::Berachain(berachain)) => {
        //             mk_handshake::<Wasm<Union>, Berachain>(&Wasm(union), &berachain, ty, chains)
        //                 .await
        //         }
        //         (AnyChain::Cosmos(cosmos), AnyChain::Union(union)) => {
        //             mk_handshake::<Wasm<Cosmos>, Union>(&Wasm(cosmos), &union, ty, chains).await
        //         }
        //         (AnyChain::Cosmos(cosmos_a), AnyChain::Cosmos(cosmos_b)) => {
        //             mk_handshake::<Cosmos, Cosmos>(&cosmos_a, &cosmos_b, ty, chains).await
        //         }
        //         (AnyChain::EthereumMainnet(ethereum), AnyChain::Union(union)) => {
        //             mk_handshake::<Ethereum<Mainnet>, Wasm<Union>>(
        //                 &ethereum,
        //                 &Wasm(union),
        //                 ty,
        //                 chains,
        //             )
        //             .await
        //         }
        //         (AnyChain::EthereumMinimal(ethereum), AnyChain::Union(union)) => {
        //             mk_handshake::<Ethereum<Minimal>, Wasm<Union>>(
        //                 &ethereum,
        //                 &Wasm(union),
        //                 ty,
        //                 chains,
        //             )
        //             .await
        //         }
        //         (AnyChain::Scroll(scroll), AnyChain::Union(union)) => {
        //             mk_handshake::<Scroll, Wasm<Union>>(&scroll, &Wasm(union), ty, chains).await
        //         }
        //         (AnyChain::Arbitrum(scroll), AnyChain::Union(union)) => {
        //             mk_handshake::<Arbitrum, Wasm<Union>>(&scroll, &Wasm(union), ty, chains).await
        //         }
        //         (AnyChain::Berachain(berachain), AnyChain::Union(union)) => {
        //             mk_handshake::<Berachain, Wasm<Union>>(&berachain, &Wasm(union), ty, chains)
        //                 .await
        //         }
        //         _ => panic!("invalid"),
        //     };

        //     print_json(&all_msgs);
        // }
        Command::InitFetch {
            chain_id,
            height,
            enqueue,
        } => {
            let start_height = match height {
                QueryHeight::Latest => {
                    let config = get_voyager_config()?;

                    let context = Context::new(config.plugins, config.modules, |h| {
                        h.register::<IbcClassic>();
                        h.register::<IbcUnion>();
                    })
                    .await?;

                    let latest_height = context
                        .rpc_server
                        .query_latest_height(&chain_id, false)
                        .await?;

                    context.shutdown().await;

                    latest_height
                }
                QueryHeight::Finalized => {
                    let config = get_voyager_config()?;

                    let context = Context::new(config.plugins, config.modules, |h| {
                        h.register::<IbcClassic>();
                        h.register::<IbcUnion>();
                    })
                    .await?;

                    let latest_height = context
                        .rpc_server
                        .query_latest_height(&chain_id, true)
                        .await?;

                    context.shutdown().await;

                    latest_height
                }
                QueryHeight::Specific(height) => height,
            };

            let op = call::<VoyagerMessage>(FetchBlocks {
                chain_id: chain_id.clone(),
                start_height,
            });

            if enqueue {
                println!("enqueueing op for `{chain_id}` at `{start_height}`");
                send_enqueue(&get_voyager_config()?.voyager.rest_laddr, op).await?;
            } else {
                print_json(&op);
            }
        }
        Command::Rpc(rpc) => {
            let voyager_client = jsonrpsee::http_client::HttpClient::builder().build(format!(
                "http://{}",
                get_voyager_config()?.voyager.rpc_laddr
            ))?;

            match rpc {
                RpcCmd::Info => print_json(&voyager_client.info().await?),
                RpcCmd::ClientState {
                    on,
                    client_id,
                    ibc_spec_id,
                    height,
                    decode,
                } => {
                    let ibc_handlers = [
                            (IbcClassic::ID, IbcSpecHandler::new::<IbcClassic>()),
                            (IbcUnion::ID, IbcSpecHandler::new::<IbcUnion>())
                         ]
                        .into_iter()
                        .collect::<HashMap<_, _>>();

                    let ibc_state = voyager_client
                        .query_ibc_state(
                            on.clone(),
                            ibc_spec_id.clone(),
                            height,
                            (ibc_handlers.get(&ibc_spec_id).unwrap().client_state_path)(
                                client_id.clone(),
                            )?,
                        )
                        .await?;

                    if decode {
                        let client_info = voyager_client
                            .client_info(on, ibc_spec_id.clone(), client_id)
                            .await?;

                        let decoded = voyager_client
                            .decode_client_state(
                                client_info.client_type,
                                client_info.ibc_interface,
                                ibc_spec_id,
                                serde_json::from_value(ibc_state.state).unwrap(),
                            )
                            .await?;

                        print_json(&IbcState {
                            height: ibc_state.height,
                            state: decoded,
                        });
                    } else {
                        print_json(&ibc_state);
                    }
                }
                RpcCmd::ConsensusState {
                    // on,
                    // client_id,
                    // ibc_spec_id,
                    // trusted_height,
                    // height,
                    // decode,
                    ..
                } => {
                    // let ibc_state = voyager_client
                    //     .query_client_consensus_state(
                    //         on.clone(),
                    //         height,
                    //         client_id.clone(),
                    //         trusted_height,
                    //     )
                    //     .await?;

                    // if decode {
                    //     let client_info = voyager_client
                    //         .client_info(on, ibc_spec_id, client_id)
                    //         .await?;

                    //     let decoded = voyager_client
                    //         .decode_consensus_state(
                    //             client_info.client_type,
                    //             client_info.ibc_interface,
                    //             ibc_state.state,
                    //         )
                    //         .await?;

                    //     print_json(&IbcState {
                    //         height: ibc_state.height,
                    //         state: decoded,
                    //     });
                    // } else {
                    //     print_json(&ibc_state);
                    // }

                    todo!()
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
            } => {
                let voyager_config = get_voyager_config()?;

                let ctx = Context::new(voyager_config.plugins, voyager_config.modules, |h| {
                    h.register::<IbcClassic>();
                    h.register::<IbcUnion>();
                })
                .await?;

                // weird race condition in Context::new that i don't feel like debugging right now
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                let msg = make_msg_create_client(
                    &ctx,
                    tracking,
                    height,
                    on,
                    client_type,
                    ibc_interface,
                    ibc_spec_id,
                    metadata,
                )
                .await?;

                // let msg = call::<VoyagerMessage>(MakeMsgCreateClient {
                //     chain_id: on,
                //     height,
                //     metadata,
                //     counterparty_chain_id: tracking,
                //     ibc_interface,
                //     client_type,
                // });

                if enqueue {
                    println!("enqueueing msg");
                    send_enqueue(&get_voyager_config()?.voyager.rest_laddr, msg).await?;
                } else {
                    print_json(&msg);
                }
            }
        },
    }

    Ok(())
}

async fn send_enqueue(
    rest_laddr: &SocketAddr,
    op: Op<VoyagerMessage>,
) -> anyhow::Result<reqwest::Response> {
    Ok(reqwest::Client::new()
        .post(format!("http://{rest_laddr}/enqueue"))
        .json(&op)
        .send()
        .await?)
}

fn print_json<T: Serialize>(t: &T) {
    println!("{}", serde_json::to_string(&t).unwrap());
}

// TODO: Extract all logic here to a plugin
pub mod utils {
    use anyhow::{anyhow, bail};
    use ibc_classic_spec::IbcClassic;
    use ibc_union_spec::IbcUnion;
    use serde_json::Value;
    use tracing::trace;
    use voyager_message::{
        context::Context,
        core::{ChainId, ClientType, IbcInterface, IbcSpecId, QueryHeight},
        data::{IbcDatagram, WithChainId},
        module::{ClientModuleClient, ConsensusModuleClient},
        VoyagerMessage,
    };
    use voyager_vm::{data, Op};

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
    ) -> anyhow::Result<Op<VoyagerMessage>> {
        if height == QueryHeight::Latest {
            // TODO: Also check if a specific height was passed and ensure that that height is also finalized
            bail!("cannot create a client at a non-finalized height")
        }

        let height = ctx
            .rpc_server
            .query_height(&counterparty_chain_id, height)
            .await?;

        let counterparty_consensus_module = ctx
            .rpc_server
            .modules()?
            .consensus_module(&counterparty_chain_id)?;

        let self_client_state = counterparty_consensus_module
            .self_client_state(height)
            .await?;
        trace!(%self_client_state);

        let self_consensus_state = counterparty_consensus_module
            .self_consensus_state(height)
            .await?;
        trace!(%self_consensus_state);

        let consensus_type = ctx
            .rpc_server
            .modules()?
            .chain_consensus_type(&counterparty_chain_id)?;

        let client_consensus_type = ctx
            .rpc_server
            .modules()?
            .client_consensus_type(&client_type)?;

        if client_consensus_type != consensus_type {
            return Err(anyhow!(
                "attempted to create a {client_type} client on \
                {chain_id} tracking {counterparty_chain_id}, but \
                the consensus of that chain ({consensus_type}) is \
                not verifiable by a client of type {client_type} \
                (which instead verifies {client_consensus_type})."
            ));
        }

        let client_module =
            ctx.rpc_server
                .modules()?
                .client_module(&client_type, &ibc_interface, &ibc_spec_id)?;

        Ok(data(WithChainId {
            chain_id,
            message: match ibc_spec_id.as_str() {
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
                IbcSpecId::UNION => IbcDatagram::new::<IbcUnion>(ibc_union_spec::Datagram::from(
                    ibc_union_spec::MsgCreateClient {
                        client_type,
                        client_state_bytes: client_module
                            .encode_client_state(self_client_state, metadata)
                            .await?,
                        consensus_state_bytes: client_module
                            .encode_consensus_state(self_consensus_state)
                            .await?,
                    },
                )),
                _ => bail!("unknown IBC version id `{ibc_spec_id}`"),
            },
        }))
    }
}
