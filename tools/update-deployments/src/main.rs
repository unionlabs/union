use std::{fmt::Display, path::PathBuf};

use alloy::{
    eips::BlockNumberOrTag, hex, network::AnyNetwork, primitives::keccak256, providers::Provider,
};
use anyhow::{bail, Context, Result};
use clap::Parser;
use cosmwasm_std::instantiate2_address;
use deployments::{
    Commit, DeployedContract, Deployments, IbcCosmwasmDeployedContractExtra, IbcCosmwasmUcs03Extra,
    Minter,
};
use protos::cosmwasm::wasm::v1::{
    QueryCodeRequest, QueryCodeResponse, QueryContractInfoRequest, QueryContractInfoResponse,
    QuerySmartContractStateRequest, QuerySmartContractStateResponse,
};
use tracing::info;
use ucs04::UniversalChainId;
use unionlabs::primitives::{Bech32, H160, H256};
use voyager_primitives::ClientType;

#[derive(clap::Parser)]
struct Args {
    path: PathBuf,
    id: UniversalChainId<'static>,
    #[arg(long, short = 'r')]
    rpc_url: String,
    #[arg(long)]
    lightclient: Vec<String>,
    #[arg(long)]
    ucs00: bool,
    #[arg(long)]
    ucs03: bool,
    #[arg(long)]
    ucs03_minter: Option<Ucs03Minter>,
    #[arg(long)]
    u: Option<String>,
    #[arg(long)]
    eu: Option<String>,
    #[arg(long)]
    lst: bool,
    #[arg(long)]
    on_zkgm_call_proxy: bool,
    #[arg(long, default_value_t = false)]
    update_deployment_heights: bool,
    #[arg(
        long,
        default_value_t = 0,
        required_if_eq("update_deployment_heights", "true")
    )]
    eth_get_logs_window: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
enum Ucs03Minter {
    Cw20,
    OsmosisTokenfactory,
}

const BYTECODE_BASE_CHECKSUM: &[u8] =
    &hex!("ec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1");

fn derive_cosmwasm(deployer: &Bech32<H160>, salt: String) -> Bech32<H256> {
    Bech32::new(
        deployer.hrp().to_string(),
        <Vec<u8>>::from(
            instantiate2_address(
                BYTECODE_BASE_CHECKSUM,
                &deployer.data().get().into(),
                salt.as_bytes(),
            )
            .unwrap(),
        )
        .try_into()
        .unwrap(),
    )
}

fn derive_evm(sender: H160, deployer: H160, namespace: &'static str, salt: &str) -> H160 {
    create3::predict_deterministic_address(
        deployer.into(),
        keccak256(
            sender
                .into_iter()
                .chain(format!("{namespace}/{salt}").bytes())
                .collect::<Vec<_>>(),
        ),
    )
    .into()
}

#[tokio::main]
async fn main() -> Result<()> {
    do_main().await
}

async fn do_main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let file = std::fs::read_to_string(&args.path)?;
    let mut deployments = serde_json::from_str::<Deployments>(&file)?;

    let deployment = deployments
        .get_mut(&args.id)
        .context("deployment not found")?;

    match deployment {
        deployments::Deployment::IbcCosmwasm {
            core,
            lightclient,
            app,
            deployer,
            u,
            eu,
            lst,
            on_zkgm_call_proxy,
        } => {
            let client = cometbft_rpc::Client::new(args.rpc_url).await?;

            // always write core
            let contract_info = get_cosmwasm_contract_info(&client, &core.address).await?;
            core.height = contract_info.created.unwrap().block_height;
            core.commit = get_commit_wasm(&client, contract_info.code_id).await?;
            core.extra.code_id = contract_info.code_id;
            info!(
                address = %core.address,
                height = core.height,
                commit = %core.commit,
                code_id = core.extra.code_id,
                "updated core"
            );

            for (client_type, info) in lightclient {
                let contract_info = get_cosmwasm_contract_info(&client, &info.address).await?;
                info.height = contract_info.created.unwrap().block_height;
                info.commit = get_commit_wasm(&client, contract_info.code_id).await?;
                info.extra.code_id = contract_info.code_id;
                info!(
                    address = %info.address,
                    height = info.height,
                    commit = %info.commit,
                    code_id = info.extra.code_id,
                    "updated lightclient {client_type}"
                );
            }

            if let Some(u_address) = args.u {
                let contract_info = get_cosmwasm_contract_info(&client, &u_address).await?;
                let info = DeployedContract {
                    address: u_address.parse()?,
                    height: contract_info.created.unwrap().block_height,
                    commit: get_commit_wasm(&client, contract_info.code_id).await?,
                    extra: IbcCosmwasmDeployedContractExtra {
                        code_id: contract_info.code_id,
                    },
                };
                info!(
                    address = %info.address,
                    height = info.height,
                    commit = %info.commit,
                    code_id = info.extra.code_id,
                    "updated u"
                );
                *u = Some(info);
            } else {
                *u = None;
            }

            if let Some(eu_address) = args.eu {
                let contract_info = get_cosmwasm_contract_info(&client, &eu_address).await?;
                let info = DeployedContract {
                    address: eu_address.parse()?,
                    height: contract_info.created.unwrap().block_height,
                    commit: get_commit_wasm(&client, contract_info.code_id).await?,
                    extra: IbcCosmwasmDeployedContractExtra {
                        code_id: contract_info.code_id,
                    },
                };
                info!(
                    address = %info.address,
                    height = info.height,
                    commit = %info.commit,
                    code_id = info.extra.code_id,
                    "updated eu"
                );
                *eu = Some(info);
            } else {
                *eu = None;
            }

            if args.lst {
                let lst_address = derive_cosmwasm(deployer, "lst/eu".to_owned());
                let contract_info = get_cosmwasm_contract_info(&client, &lst_address).await?;
                let info = DeployedContract {
                    address: lst_address,
                    height: contract_info.created.unwrap().block_height,
                    commit: get_commit_wasm(&client, contract_info.code_id).await?,
                    extra: IbcCosmwasmDeployedContractExtra {
                        code_id: contract_info.code_id,
                    },
                };
                info!(
                    address = %info.address,
                    height = info.height,
                    commit = %info.commit,
                    code_id = info.extra.code_id,
                    "updated lst"
                );
                *lst = Some(info);
            } else {
                *lst = None;
            }

            if args.on_zkgm_call_proxy {
                let on_zkgm_call_proxy_address =
                    derive_cosmwasm(deployer, "on-zkgm-call-proxy".to_owned());
                info!(address = %on_zkgm_call_proxy_address, "on-zkgm-call-proxy");
                let contract_info =
                    get_cosmwasm_contract_info(&client, &on_zkgm_call_proxy_address).await?;
                let info = DeployedContract {
                    address: on_zkgm_call_proxy_address,
                    height: contract_info.created.unwrap().block_height,
                    commit: get_commit_wasm(&client, contract_info.code_id).await?,
                    extra: IbcCosmwasmDeployedContractExtra {
                        code_id: contract_info.code_id,
                    },
                };
                info!(
                    address = %info.address,
                    height = info.height,
                    commit = %info.commit,
                    code_id = info.extra.code_id,
                    "updated on_zkgm_call_proxy"
                );
                *on_zkgm_call_proxy = Some(info);
            } else {
                *on_zkgm_call_proxy = None;
            }

            if args.ucs00 {
                let ucs00 = app.ucs00.get_or_insert(DeployedContract {
                    address: derive_cosmwasm(deployer, "protocols/ucs00".to_owned()),
                    height: 0,
                    commit: Commit::Unknown,
                    extra: IbcCosmwasmDeployedContractExtra { code_id: 0 },
                });

                let contract_info = get_cosmwasm_contract_info(&client, &ucs00.address).await?;
                ucs00.height = contract_info.created.unwrap().block_height;
                ucs00.commit = get_commit_wasm(&client, contract_info.code_id).await?;
                ucs00.extra.code_id = contract_info.code_id;
                info!(
                    address = %ucs00.address,
                    height = ucs00.height,
                    commit = %ucs00.commit,
                    code_id = ucs00.extra.code_id,
                    "updated app ucs00"
                );
            } else {
                app.ucs00 = None;
            }

            if args.ucs03 {
                let ucs03 = match app.ucs03.as_mut() {
                    Some(k) => k,
                    None => {
                        let address = derive_cosmwasm(deployer, "protocols/ucs03".to_owned());
                        let minter_address = client
                            .grpc_abci_query::<_, QuerySmartContractStateResponse>(
                                "/cosmwasm.wasm.v1.Query/SmartContractState",
                                &QuerySmartContractStateRequest {
                                    address: address.to_string(),
                                    query_data: serde_json::to_vec(
                                        &ucs03_zkgm::msg::QueryMsg::GetMinter {},
                                    )
                                    .unwrap(),
                                },
                                None,
                                false,
                            )
                            .await?
                            .into_result()?
                            .map(|x| serde_json::from_slice::<Bech32<H256>>(&x.data))
                            .transpose()?
                            .unwrap();

                        info!(%minter_address, "ucs03 minter");

                        let minter = match args.ucs03_minter.unwrap() {
                            Ucs03Minter::Cw20 => Minter::Cw20 {
                                address: minter_address,
                                commit: Commit::Unknown,
                                code_id: 0,
                            },
                            Ucs03Minter::OsmosisTokenfactory => Minter::OsmosisTokenfactory {
                                address: minter_address,
                                commit: Commit::Unknown,
                                code_id: 0,
                            },
                        };
                        app.ucs03 = Some(DeployedContract {
                            address,
                            height: 0,
                            commit: Commit::Unknown,
                            extra: IbcCosmwasmUcs03Extra { code_id: 0, minter },
                        });

                        app.ucs03.as_mut().unwrap()
                    }
                };

                let contract_info = get_cosmwasm_contract_info(&client, &ucs03.address).await?;
                ucs03.height = contract_info.created.unwrap().block_height;
                ucs03.commit = get_commit_wasm(&client, contract_info.code_id).await?;
                ucs03.extra.code_id = contract_info.code_id;
                info!(
                    address = %ucs03.address,
                    height = ucs03.height,
                    commit = %ucs03.commit,
                    code_id = ucs03.extra.code_id,
                    "updated app ucs03"
                );
            } else {
                app.ucs03 = None;
            }
        }
        deployments::Deployment::IbcSolidity {
            deployer,
            sender,
            manager: _,
            multicall,
            core,
            lightclient,
            app,
            u,
            eu,
        } => {
            let provider = alloy::providers::ProviderBuilder::new_with_network::<AnyNetwork>()
                .connect(&args.rpc_url)
                .await?;

            if args.update_deployment_heights {
                multicall.height =
                    get_init_height(&provider, multicall.address, args.eth_get_logs_window).await?;
            }
            core.commit = get_commit_evm(&provider, core.address).await?;
            info!(
                address = %core.address,
                height = core.height,
                commit = %core.commit,
                "updated multicall"
            );

            // always write core
            if args.update_deployment_heights {
                core.height =
                    get_init_height(&provider, core.address, args.eth_get_logs_window).await?;
            }
            core.commit = get_commit_evm(&provider, core.address).await?;
            info!(
                address = %core.address,
                height = core.height,
                commit = %core.commit,
                "updated core"
            );

            for client_type in &args.lightclient {
                let info = lightclient
                    .entry(ClientType::new(client_type.clone()))
                    .or_insert(DeployedContract {
                        address: derive_evm(*sender, *deployer, "lightclients", client_type),
                        height: 0,
                        commit: Commit::Unknown,
                        extra: (),
                    });
                if args.update_deployment_heights {
                    info.height =
                        get_init_height(&provider, info.address, args.eth_get_logs_window).await?;
                }
                info.commit = get_commit_evm(&provider, info.address).await?;
                info!(
                    address = %info.address,
                    height = info.height,
                    commit = %info.commit,
                    "updated lightclient {client_type}"
                );
            }

            lightclient.retain(|k, _| args.lightclient.iter().any(|s| s == k.as_str()));

            if args.ucs00 {
                let ucs00 = app.ucs00.get_or_insert(DeployedContract {
                    address: derive_evm(*sender, *deployer, "protocols", "ucs00"),
                    height: 0,
                    commit: Commit::Unknown,
                    extra: (),
                });
                if args.update_deployment_heights {
                    ucs00.height =
                        get_init_height(&provider, ucs00.address, args.eth_get_logs_window).await?;
                }
                ucs00.commit = get_commit_evm(&provider, ucs00.address).await?;
                info!(
                    address = %ucs00.address,
                    height = ucs00.height,
                    commit = %ucs00.commit,
                    "updated app ucs00"
                );
            } else {
                app.ucs00 = None;
            }

            if args.ucs03 {
                let ucs03 = app.ucs03.get_or_insert(DeployedContract {
                    address: derive_evm(*sender, *deployer, "protocols", "ucs03"),
                    height: 0,
                    commit: Commit::Unknown,
                    extra: (),
                });
                if args.update_deployment_heights {
                    ucs03.height =
                        get_init_height(&provider, ucs03.address, args.eth_get_logs_window).await?;
                }
                ucs03.commit = get_commit_evm(&provider, ucs03.address).await?;
                info!(
                    address = %ucs03.address,
                    height = ucs03.height,
                    commit = %ucs03.commit,
                    "updated app ucs03"
                );
            } else {
                app.ucs03 = None;
            }

            if let Some(u_address) = args.u {
                let mut info = DeployedContract {
                    address: u_address.parse()?,
                    height: 0,
                    commit: get_commit_evm(&provider, u_address.parse()?).await?,
                    extra: (),
                };
                if args.update_deployment_heights {
                    info.height =
                        get_init_height(&provider, info.address, args.eth_get_logs_window).await?;
                }
                info!(
                    address = %info.address,
                    height = info.height,
                    commit = %info.commit,
                    "updated u"
                );
                *u = Some(info);
            } else {
                *u = None;
            }

            if let Some(eu_address) = args.eu {
                let mut info = DeployedContract {
                    address: eu_address.parse()?,
                    height: 0,
                    commit: get_commit_evm(&provider, eu_address.parse()?).await?,
                    extra: (),
                };
                if args.update_deployment_heights {
                    info.height =
                        get_init_height(&provider, info.address, args.eth_get_logs_window).await?;
                }
                info!(
                    address = %info.address,
                    height = info.height,
                    commit = %info.commit,
                    "updated eu"
                );
                *eu = Some(info);
            } else {
                *eu = None;
            }
        }
    }

    std::fs::write(
        &args.path,
        serde_json::to_string_pretty(&deployments).unwrap(),
    )?;

    Ok(())
}

async fn get_commit_wasm(client: &cometbft_rpc::Client, code_id: u64) -> Result<Commit> {
    let data = client
        .grpc_abci_query::<_, QueryCodeResponse>(
            "/cosmwasm.wasm.v1.Query/Code",
            &QueryCodeRequest { code_id },
            None,
            false,
        )
        .await?
        .into_result()?
        .context("empty response")?
        .data;

    Ok(
        embed_commit_verifier::extract_wasm(&data)?.map_or(Commit::Unknown, |rev| match rev {
            embed_commit::Rev::Unknown => Commit::Unknown,
            embed_commit::Rev::Dirty => Commit::Dirty,
            embed_commit::Rev::Hash(hash) => Commit::Hash(hash.into()),
        }),
    )
}

async fn get_cosmwasm_contract_info(
    client: &cometbft_rpc::Client,
    address: impl Display,
) -> Result<protos::cosmwasm::wasm::v1::ContractInfo> {
    client
        .grpc_abci_query::<_, QueryContractInfoResponse>(
            "/cosmwasm.wasm.v1.Query/ContractInfo",
            &QueryContractInfoRequest {
                address: address.to_string(),
            },
            None,
            false,
        )
        .await?
        .into_result()?
        .context("empty response")?
        .contract_info
        .context("empty response")
}

alloy::sol! {
    #![sol(rpc)]

    contract Versioned {
        event Initialized(uint64);

        function gitRev() public pure returns (string memory);
    }
}

async fn get_commit_evm(provider: &impl Provider<AnyNetwork>, address: H160) -> Result<Commit> {
    let client = Versioned::new(address.get().into(), provider);

    Ok(match client.gitRev().call().await?.as_str() {
        "dirty" => Commit::Dirty,
        "unknown" => Commit::Unknown,
        hash => Commit::Hash(hash.parse()?),
    })
}

async fn get_init_height(
    provider: &impl Provider<AnyNetwork>,
    address: H160,
    window: u64,
) -> Result<u64> {
    let client = Versioned::new(address.get().into(), provider);

    let mut latest_height = provider.get_block_number().await?;

    for (from, to) in std::iter::from_fn(|| -> Option<(BlockNumberOrTag, BlockNumberOrTag)> {
        if latest_height == 0 {
            None
        } else {
            let upper_bound = latest_height;
            let lower_bound = latest_height.saturating_sub(window);
            latest_height = lower_bound;
            Some((lower_bound.into(), upper_bound.into()))
        }
    }) {
        info!(%from, %to, "querying range for init event");

        let query = client.Initialized_filter();

        let batch_logs = query.from_block(from).to_block(to).query().await?;

        if batch_logs.is_empty() {
            info!(%from, %to, "event not found in range");
            continue;
        } else {
            return batch_logs
                .into_iter()
                .find_map(|(_, log)| log.block_number)
                .context("no height found");
        }
    }

    bail!("init height not found for {address}");
}
