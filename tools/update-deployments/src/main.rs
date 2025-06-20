use std::{fmt::Display, path::PathBuf};

use alloy::{eips::BlockNumberOrTag, network::AnyNetwork, providers::Provider};
use anyhow::{bail, Context, Result};
use clap::Parser;
use deployments::{Commit, Deployments, Minter};
use protos::cosmwasm::wasm::v1::{
    QueryCodeRequest, QueryCodeResponse, QueryContractInfoRequest, QueryContractInfoResponse,
};
use tracing::info;
use ucs04::UniversalChainId;
use unionlabs::primitives::H160;

#[derive(clap::Parser)]
struct Args {
    path: PathBuf,
    id: UniversalChainId<'static>,
    #[arg(long, short = 'r')]
    rpc_url: String,
    #[arg(long, default_value_t = false)]
    update_deployment_heights: bool,
    #[arg(
        long,
        default_value_t = 0,
        required_if_eq("update_deployment_heights", "true")
    )]
    eth_get_logs_window: u64,
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
            deployer: _,
        } => {
            let client = cometbft_rpc::Client::new(args.rpc_url).await?;

            let contract_info = get_cosmwasm_contract_info(&client, &core.address).await?;
            core.height = contract_info.created.unwrap().block_height;
            core.commit = get_commit_wasm(&client, contract_info.code_id).await?;
            core.extra.code_id = contract_info.code_id;
            info!(
                address = %core.address,
                height = core.height,
                commit = %core.commit,
                code_id = core.extra.code_id,
                "updating core"
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
                    "updating lightclient {client_type}"
                );
            }

            if let Some(ucs00) = &mut app.ucs00 {
                let contract_info = get_cosmwasm_contract_info(&client, &ucs00.address).await?;
                ucs00.height = contract_info.created.unwrap().block_height;
                ucs00.commit = get_commit_wasm(&client, contract_info.code_id).await?;
                ucs00.extra.code_id = contract_info.code_id;
                info!(
                    address = %ucs00.address,
                    height = ucs00.height,
                    commit = %ucs00.commit,
                    code_id = ucs00.extra.code_id,
                    "updating app ucs00"
                );
            }

            if let Some(ucs03) = &mut app.ucs03 {
                let contract_info = get_cosmwasm_contract_info(&client, &ucs03.address).await?;
                ucs03.height = contract_info.created.unwrap().block_height;
                ucs03.commit = get_commit_wasm(&client, contract_info.code_id).await?;
                ucs03.extra.code_id = contract_info.code_id;
                info!(
                    address = %ucs03.address,
                    height = ucs03.height,
                    commit = %ucs03.commit,
                    code_id = ucs03.extra.code_id,
                    "updating app ucs03"
                );
                match &mut ucs03.extra.minter {
                    Minter::Cw20 {
                        address,
                        commit,
                        code_id,
                    }
                    | Minter::OsmosisTokenfactory {
                        address,
                        commit,
                        code_id,
                    } => {
                        let contract_info = get_cosmwasm_contract_info(&client, &address).await?;
                        *commit = get_commit_wasm(&client, contract_info.code_id).await?;
                        *code_id = contract_info.code_id;
                        info!(
                            %address,
                            %commit,
                            code_id,
                            "updating ucs03 minter"
                        );
                    }
                }
            }
        }
        deployments::Deployment::IbcSolidity {
            deployer: _,
            sender: _,
            manager: _,
            multicall: _,
            core,
            lightclient,
            app,
        } => {
            let provider = alloy::providers::ProviderBuilder::new_with_network::<AnyNetwork>()
                .connect(&args.rpc_url)
                .await?;

            if args.update_deployment_heights {
                core.height =
                    get_init_height(&provider, core.address, args.eth_get_logs_window).await?;
            }
            core.commit = get_commit_evm(&provider, core.address).await?;
            info!(
                address = %core.address,
                height = core.height,
                commit = %core.commit,
                "updating core"
            );

            for (client_type, info) in lightclient {
                if args.update_deployment_heights {
                    info.height =
                        get_init_height(&provider, info.address, args.eth_get_logs_window).await?;
                }
                info.commit = get_commit_evm(&provider, info.address).await?;
                info!(
                    address = %info.address,
                    height = info.height,
                    commit = %info.commit,
                    "updating lightclient {client_type}"
                );
            }

            if let Some(ucs00) = &mut app.ucs00 {
                if args.update_deployment_heights {
                    ucs00.height =
                        get_init_height(&provider, ucs00.address, args.eth_get_logs_window).await?;
                }
                ucs00.commit = get_commit_evm(&provider, ucs00.address).await?;
                info!(
                    address = %ucs00.address,
                    height = ucs00.height,
                    commit = %ucs00.commit,
                    "updating app ucs00"
                );
            }

            if let Some(ucs03) = &mut app.ucs03 {
                if args.update_deployment_heights {
                    ucs03.height =
                        get_init_height(&provider, ucs03.address, args.eth_get_logs_window).await?;
                }
                ucs03.commit = get_commit_evm(&provider, ucs03.address).await?;
                info!(
                    address = %ucs03.address,
                    height = ucs03.height,
                    commit = %ucs03.commit,
                    "updating app ucs03"
                );
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
