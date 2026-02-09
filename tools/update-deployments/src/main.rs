use std::{fmt::Display, path::PathBuf};

use alloy::{hex, network::AnyNetwork, primitives::keccak256, providers::Provider};
use anyhow::{Context, Result};
use clap::Parser;
use cosmwasm_std::instantiate2_address;
use deployments::{DeployedContract, Deployments, IbcCosmwasmDeployedContractExtra, Rev};
use protos::cosmwasm::wasm::v1::{
    QueryCodeRequest, QueryCodeResponse, QueryContractInfoRequest, QueryContractInfoResponse,
};
use tracing::{info, instrument};
use ucs04::UniversalChainId;
use unionlabs::primitives::{Bech32, H160, H256};

// TODO: Change this to take just a list of salts and/or addresses
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

fn derive_evm(sender: H160, deployer: H160, namespace: &'static str, salt: Option<&str>) -> H160 {
    create3::predict_deterministic_address(
        deployer.into(),
        keccak256(format!(
            "{sender}/{}",
            match salt {
                None => namespace.to_owned(),
                Some(salt) => format!("{namespace}/{salt}"),
            }
        )),
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
    let mut deployments =
        serde_json::from_str::<Deployments>(&file).context("reading deployments file")?;

    let deployment = deployments
        .get_mut(&args.id)
        .context("deployment not found")?;

    match deployment {
        deployments::Deployment::IbcCosmwasm {
            deployer,
            contracts,
        } => {
            let client = cometbft_rpc::Client::new(args.rpc_url).await?;

            contracts
                .entry(derive_cosmwasm(deployer, "ibc-is-based".to_owned()))
                .or_insert(DeployedContract {
                    name: "core".to_owned(),
                    salt: Some(b"ibc-is-based".into()),
                    height: 0,
                    commit: Rev::Unknown,
                    extra: IbcCosmwasmDeployedContractExtra { code_id: 0 },
                });

            for client_type in &args.lightclient {
                contracts
                    .entry(derive_cosmwasm(
                        deployer,
                        format!("lightclients/{client_type}"),
                    ))
                    .or_insert(DeployedContract {
                        name: format!("lightclients/{client_type}"),
                        salt: Some(format!("lightclients/{client_type}").into_bytes().into()),
                        height: 0,
                        commit: Rev::Unknown,
                        extra: IbcCosmwasmDeployedContractExtra { code_id: 0 },
                    });
            }

            for (address, deployment) in contracts {
                info!(%address, "updating {}", deployment.name);
                let contract_info = get_cosmwasm_contract_info(&client, &address).await?;
                deployment.height = contract_info.created.unwrap().block_height;
                deployment.commit = get_commit_wasm(&client, contract_info.code_id).await?;
                deployment.extra.code_id = contract_info.code_id;
                info!(
                    %address,
                    height = deployment.height,
                    commit = %deployment.commit,
                    code_id = deployment.extra.code_id,
                    "updated {}",
                    deployment.name
                );
            }
        }
        deployments::Deployment::IbcSolidity {
            deployer,
            sender,
            contracts,
        } => {
            let provider = alloy::providers::ProviderBuilder::new_with_network::<AnyNetwork>()
                .connect(&args.rpc_url)
                .await?;

            contracts
                .entry(derive_evm(*sender, *deployer, "ibc-is-based", None))
                .or_insert(DeployedContract {
                    name: "core".to_owned(),
                    salt: Some(b"ibc-is-based".into()),
                    height: 0,
                    commit: Rev::Unknown,
                    extra: (),
                });

            for client_type in &args.lightclient {
                contracts
                    .entry(derive_evm(
                        *sender,
                        *deployer,
                        "lightclients",
                        Some(client_type),
                    ))
                    .or_insert(DeployedContract {
                        name: format!("lightclients/{client_type}"),
                        salt: Some(format!("lightclients/{client_type}").into_bytes().into()),
                        height: 0,
                        commit: Rev::Unknown,
                        extra: (),
                    });
            }

            if let Some(u_address) = args.u {
                contracts
                    .entry(u_address.parse()?)
                    .or_insert(DeployedContract {
                        height: 0,
                        commit: get_commit_evm(&provider, u_address.parse()?).await?,
                        extra: (),
                        name: "u".to_owned(),
                        salt: Some(
                            "0x12c206e42a6e7773c97d1f1b855d7848492f9e4e396b33fcf0172d6758e9b047"
                                .parse()
                                .unwrap(),
                        ),
                    });
            }

            if let Some(eu_address) = args.eu {
                contracts
                    .entry(eu_address.parse()?)
                    .or_insert(DeployedContract {
                        height: 0,
                        commit: get_commit_evm(&provider, eu_address.parse()?).await?,
                        extra: (),
                        name: "eu".to_owned(),
                        salt: Some(
                            "0x0dec0db7b56214f189bc3d33052145c6d7558c6a7ee0da79e34bdd8a29d569c2"
                                .parse()
                                .unwrap(),
                        ),
                    });
            }

            for (address, deployment) in contracts {
                deployment.height = get_init_height(&provider, *address, deployment.height).await?;
                deployment.commit = get_commit_evm(&provider, *address).await?;
                info!(
                    %address,
                    height = deployment.height,
                    commit = %deployment.commit,
                    "updated {}",
                    deployment.name
                );
            }
        }
        _ => todo!(),
    }

    std::fs::write(
        &args.path,
        serde_json::to_string_pretty(&deployments).unwrap() + "\n",
    )?;

    Ok(())
}

async fn get_commit_wasm(client: &cometbft_rpc::Client, code_id: u64) -> Result<Rev> {
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

    Ok(embed_commit_verifier::extract_wasm(&data)?.unwrap_or_default())
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

async fn get_commit_evm(provider: &impl Provider<AnyNetwork>, address: H160) -> Result<Rev> {
    let client = Versioned::new(address.get().into(), provider);

    match client.gitRev().call().await {
        Ok(rev) => rev.as_str().parse().context("parsing gitRev() response"),
        Err(err) => {
            if err.to_string().contains("execution reverted") {
                Ok(Rev::Unknown)
            } else {
                Err(err).context("calling gitRev()")
            }
        }
    }
}

#[instrument(skip_all, fields(%address))]
async fn get_init_height(
    provider: &impl Provider<AnyNetwork>,
    address: H160,
    current_height: u64,
) -> Result<u64> {
    let client = Versioned::new(address.get().into(), provider);

    let latest_height = provider.get_block_number().await?;

    if (1..=latest_height).contains(&current_height) {
        info!("checking if current height {current_height} is correct");

        let query = client.Initialized_filter();

        let block_logs = query
            .from_block(current_height)
            .to_block(current_height)
            .query()
            .await?;

        if block_logs.is_empty() {
            info!(
                "event not found at current deployed height {current_height}, will scan from latest"
            );
        } else {
            info!("current height {current_height} is already correct");

            return block_logs
                .into_iter()
                .find_map(|(_, log)| log.block_number)
                .context("no height found");
        }
    }

    // bisect to find deployment height
    let mut low = 1;
    let mut high = latest_height;
    while low < high {
        let mid = (low + high) / 2;

        info!("checking between {low} and {high}, mid = {mid}");

        let code = provider
            .get_code_at(address.into())
            .block_id(mid.into())
            .await?;

        if code.is_empty() {
            low = mid + 1;
        } else {
            high = mid;
        }
    }

    let query = client.Initialized_filter();

    let block_logs = query.from_block(low).to_block(low).query().await?;

    return block_logs
        .into_iter()
        .find_map(|(_, log)| log.block_number)
        .context("no height found");
}
