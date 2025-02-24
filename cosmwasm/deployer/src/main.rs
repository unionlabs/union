use std::{collections::BTreeMap, ops::Deref, path::PathBuf};

use anyhow::{bail, Context, Result};
use clap::Parser;
use cometbft_rpc::rpc_types::GrpcAbciQueryError;
use cosmos_client::{Ctx, GasConfig};
use cosmwasm_std::Addr;
use hex_literal::hex;
use protos::cosmwasm::wasm::v1::{
    AccessConfig, AccessType, MsgExecuteContract, MsgExecuteContractResponse,
    MsgInstantiateContract2, MsgInstantiateContract2Response, MsgMigrateContract,
    MsgMigrateContractResponse, MsgStoreCode, MsgStoreCodeResponse, QuerySmartContractStateRequest,
    QuerySmartContractStateResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Digest;
use tracing::{info, instrument};
use tracing_subscriber::EnvFilter;
use ucs03_zkgm::msg::TokenMinterInitMsg;
use unionlabs::{
    bech32::Bech32,
    cosmos::{base::coin::Coin, tx::fee::Fee},
    primitives::{Bytes, H256},
    signer::CosmosSigner,
};

#[derive(clap::Parser)]
enum App {
    DeployFull {
        #[arg(long)]
        rpc_url: String,
        #[arg(long)]
        private_key: H256,
        #[arg(long)]
        contracts: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        permissioned: bool,
        #[command(flatten)]
        gas_config: GasConfigArgs,
    },
    Migrate {
        #[arg(long)]
        rpc_url: String,
        #[arg(long)]
        private_key: H256,
        #[arg(long)]
        address: Bech32<H256>,
        #[arg(long)]
        new_bytecode: PathBuf,
        #[command(flatten)]
        gas_config: GasConfigArgs,
    },
    /// Calculate the addresses of the deployed stack given the deployer. The returned addresses will have the same bech32 prefix as the deployer address.
    Addresses {
        #[arg(long)]
        deployer: Bech32<Bytes>,
        #[arg(long)]
        lightclient: Vec<String>,
        #[command(flatten)]
        apps: AppFlags,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    InitHeights {
        #[arg(long)]
        rpc_url: String,
        /// output of `cosmwasm-deployer addresses`
        #[arg(long)]
        addresses: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    AddressOfPrivateKey {
        #[arg(long)]
        private_key: H256,
        #[arg(long)]
        bech32_prefix: String,
    },
}

#[derive(Debug, Clone, PartialEq, Default, clap::Args)]
pub struct AppFlags {
    #[arg(long)]
    ucs00: bool,
    #[arg(long)]
    ucs03: bool,
}

#[derive(clap::Subcommand)]
enum QueryCmd {
    CodeInfo {
        #[arg(long)]
        rpc_url: String,
        #[arg(long)]
        code_id: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    do_main().await
}

const BYTECODE_BASE_BYTECODE: &[u8] = &hex_literal::hex!("0061736d0100000001110360037f7f7f017f60017f017f60017f000304030001020503010001074605066d656d6f7279020013696e746572666163655f76657273696f6e5f3800000b696e7374616e7469617465000008616c6c6f6361746500010a6465616c6c6f6361746500020a0f03040041330b0400413f0b0300010b0b4e010041010b487b226f6b223a7b226d65737361676573223a5b5d2c2261747472696275746573223a5b5d2c226576656e7473223a5b5d7d7d0100000032000000320000004b000000000200000002");

fn sha2(bz: impl AsRef<[u8]>) -> H256 {
    ::sha2::Sha256::new().chain_update(bz).finalize().into()
}

const CORE: &str = "core";
const LIGHTCLIENT: &str = "lightclient";
const APP: &str = "app";

const UCS03: &str = "ucs03";

const BYTECODE_BASE: &str = "bytecode-base";

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
struct ContractPaths {
    core: PathBuf,
    // salt -> wasm path
    lightclient: BTreeMap<String, PathBuf>,
    app: AppPaths,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
struct AppPaths {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ucs00: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ucs03: Option<Ucs03Config>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct Ucs03Config {
    path: PathBuf,
    token_minter_path: PathBuf,
    token_minter_config: TokenMinterConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum TokenMinterConfig {
    Cw20 {
        /// The path to the cw20-base contract code.
        ///
        /// This MUST be the unionlabs fork of cw20-base, which forces instantiation through the migrate entrypoint, such that it can have a deterministic address.
        cw20_base: PathBuf,
    },
    Native,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
struct ContractAddresses {
    core: String,
    lightclient: BTreeMap<String, String>,
    app: AppAddresses,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
struct AppAddresses {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ucs00: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ucs03: Option<String>,
}

async fn do_main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let app = App::parse();

    match app {
        App::Addresses {
            deployer,
            lightclient,
            apps,
            output,
        } => {
            let core = instantiate2_address(deployer.clone(), sha2(BYTECODE_BASE_BYTECODE), CORE)
                .unwrap()
                .to_string();

            let lightclient = lightclient
                .into_iter()
                .map(|salt| {
                    (
                        salt.clone(),
                        instantiate2_address(
                            deployer.clone(),
                            sha2(BYTECODE_BASE_BYTECODE),
                            &format!("{LIGHTCLIENT}/{salt}"),
                        )
                        .unwrap()
                        .to_string(),
                    )
                })
                .collect();

            let mut app = AppAddresses::default();

            if apps.ucs00 {
                todo!()
            }

            if apps.ucs03 {
                app.ucs03 = Some(
                    instantiate2_address(
                        deployer,
                        sha2(BYTECODE_BASE_BYTECODE),
                        &format!("{APP}/{UCS03}"),
                    )
                    .unwrap()
                    .to_string(),
                );
            }

            write_output(
                output,
                ContractAddresses {
                    core,
                    lightclient,
                    app,
                },
            )?;
        }
        App::DeployFull {
            rpc_url,
            private_key,
            contracts,
            output,
            permissioned,
            gas_config,
        } => {
            let contracts = serde_json::from_slice::<ContractPaths>(
                &std::fs::read(contracts).context("reading contracts path")?,
            )?;

            let ctx = Deployer(Ctx::new(rpc_url, private_key, gas_config.into()).await?);

            let bytecode_base_address = ctx
                .instantiate2_address(sha2(BYTECODE_BASE_BYTECODE), BYTECODE_BASE)
                .await?;

            let bytecode_base_contract = ctx.contract_info(bytecode_base_address.clone()).await?;

            let bytecode_base_code_id = match bytecode_base_contract {
                Some(_) => ctx
                    .instantiate_code_id_of_contract(bytecode_base_address)
                    .await?
                    .unwrap(),
                // contract does not exist on chain
                None => {
                    let (_, response) = ctx
                        .tx::<_, MsgStoreCodeResponse>(
                            MsgStoreCode {
                                sender: ctx.signer().to_string(),
                                wasm_byte_code: BYTECODE_BASE_BYTECODE.to_vec(),
                                ..Default::default()
                            },
                            "",
                        )
                        .await
                        .context("store code")?;

                    ctx.tx::<_, MsgInstantiateContract2Response>(
                        MsgInstantiateContract2 {
                            sender: ctx.signer().to_string(),
                            admin: ctx.signer().to_string(),
                            code_id: response.code_id,
                            label: BYTECODE_BASE.to_string(),
                            msg: b"{}".to_vec(),
                            salt: BYTECODE_BASE.as_bytes().to_vec(),
                            ..Default::default()
                        },
                        "",
                    )
                    .await
                    .context("instantiate2")?;

                    response.code_id
                }
            };

            info!("bytecode-base code_id is {bytecode_base_code_id}");

            let core_address = ctx
                .deploy_and_initiate(
                    std::fs::read(contracts.core)?,
                    bytecode_base_code_id,
                    ibc_union_msg::msg::InitMsg {},
                    CORE.to_owned(),
                )
                .await?;

            let mut contract_addresses = ContractAddresses {
                core: core_address.clone(),
                lightclient: BTreeMap::default(),
                app: AppAddresses {
                    ucs00: None,
                    ucs03: None,
                },
            };

            for (client_type, path) in contracts.lightclient {
                let address = ctx
                    .deploy_and_initiate(
                        std::fs::read(path)?,
                        bytecode_base_code_id,
                        ibc_union_light_client::msg::InitMsg {
                            ibc_host: Addr::unchecked(core_address.clone()),
                        },
                        format!("{LIGHTCLIENT}/{client_type}"),
                    )
                    .await?;

                let response = ctx
                    .client()
                    .grpc_abci_query::<_, QuerySmartContractStateResponse>(
                        "/cosmwasm.wasm.v1.Query/SmartContractState",
                        &QuerySmartContractStateRequest {
                            address: core_address.clone(),
                            query_data: serde_json::to_vec(
                                &ibc_union_msg::query::QueryMsg::GetRegisteredClientType {
                                    client_type: client_type.clone(),
                                },
                            )
                            .unwrap(),
                        },
                        None,
                        false,
                    )
                    .await?;

                if let Some(addr) = response
                    .value
                    .map(|value| serde_json::from_slice::<Addr>(&value.data).unwrap())
                {
                    assert_eq!(addr.to_string(), address);
                    info!("client {client_type} has already been registered");
                } else {
                    ctx.tx::<_, MsgExecuteContractResponse>(
                        MsgExecuteContract {
                            sender: ctx.signer().to_string(),
                            contract: core_address.clone(),
                            msg: serde_json::to_vec(
                                &ibc_union_msg::msg::ExecuteMsg::RegisterClient(
                                    ibc_union_msg::msg::MsgRegisterClient {
                                        client_type: client_type.clone(),
                                        client_address: address.clone(),
                                    },
                                ),
                            )
                            .unwrap(),
                            funds: vec![],
                        },
                        "",
                    )
                    .await?;

                    info!("registered client type {client_type} to {address}");
                };

                contract_addresses.lightclient.insert(client_type, address);
            }

            if let Some(_ucs00) = contracts.app.ucs00 {}

            if let Some(ucs03_config) = contracts.app.ucs03 {
                let salt = format!("{APP}/{UCS03}");

                let ucs03_address = instantiate2_address(
                    ctx.signer().to_string().parse().unwrap(),
                    sha2(BYTECODE_BASE_BYTECODE),
                    &salt,
                )
                .unwrap();

                let state = ctx.contract_deploy_state(ucs03_address.clone()).await?;

                if let ContractDeployState::None | ContractDeployState::Instantiated = state {
                    let (tx_hash, response) = ctx
                        .tx::<_, MsgStoreCodeResponse>(
                            MsgStoreCode {
                                sender: ctx.signer().to_string(),
                                wasm_byte_code: std::fs::read(ucs03_config.token_minter_path)?,
                                // on permissioned cosmwasm, we must specify that this contract can be instantiated by the ucs03 contract
                                instantiate_permission: if permissioned {
                                    Some(AccessConfig {
                                        permission: AccessType::AnyOfAddresses.into(),
                                        addresses: vec![ucs03_address.clone()],
                                    })
                                } else {
                                    None
                                },
                            },
                            "",
                        )
                        .await
                        .context("store minter code")?;

                    let minter_code_id = response.code_id;

                    info!(%tx_hash, minter_code_id, "minter stored");

                    let minter_init_msg = match ucs03_config.token_minter_config {
                        TokenMinterConfig::Cw20 { cw20_base } => {
                            let (tx_hash, response) = ctx
                                .tx::<_, MsgStoreCodeResponse>(
                                    MsgStoreCode {
                                        sender: ctx.signer().to_string(),
                                        wasm_byte_code: std::fs::read(cw20_base)?,
                                        // on permissioned cosmwasm, we must specify that this contract can be instantiated by the minter contract
                                        instantiate_permission: if permissioned {
                                            Some(AccessConfig {
                                                permission: AccessType::AnyOfAddresses.into(),
                                                addresses: vec![ucs03_address.clone()],
                                            })
                                        } else {
                                            None
                                        },
                                    },
                                    "",
                                )
                                .await
                                .context("store minter code")?;

                            let code_id = response.code_id;

                            info!(%tx_hash, code_id, "cw20-base stored");

                            TokenMinterInitMsg::Cw20 {
                                cw20_base_code_id: code_id,
                                dummy_code_id: bytecode_base_code_id,
                            }
                        }
                        TokenMinterConfig::Native => TokenMinterInitMsg::Native,
                    };

                    ctx.deploy_and_initiate(
                        std::fs::read(ucs03_config.path)?,
                        bytecode_base_code_id,
                        ucs03_zkgm::msg::InitMsg {
                            config: ucs03_zkgm::msg::Config {
                                admin: Addr::unchecked(ctx.signer().to_string()),
                                ibc_host: Addr::unchecked(core_address.clone()),
                                token_minter_code_id: minter_code_id,
                            },
                            minter_init_msg,
                        },
                        salt,
                    )
                    .await?;
                }

                contract_addresses.app.ucs03 = Some(ucs03_address);
            }

            write_output(output, contract_addresses)?;
        }
        App::InitHeights {
            rpc_url,
            addresses,
            output,
        } => {
            let addresses = serde_json::from_slice::<ContractAddresses>(
                &std::fs::read(addresses).context("reading addresses path")?,
            )?;

            let ctx = Deployer(
                Ctx::new(
                    rpc_url,
                    hex!("9a95f0bb285a5d81415a0571cebbb63dbef2c7a0c90f9b60a40572552da3eac3").into(),
                    GasConfig::default(),
                )
                .await?,
            );

            let mut heights = BTreeMap::new();

            heights.insert(
                addresses.core.clone(),
                ctx.contract_history(addresses.core.clone())
                    .await??
                    .unwrap()
                    .entries
                    .pop()
                    .unwrap()
                    .updated
                    .unwrap()
                    .block_height,
            );

            for (client_type, address) in addresses.lightclient {
                if let Some(entry) = ctx
                    .contract_history(address.clone())
                    .await??
                    .unwrap()
                    .entries
                    .pop()
                {
                    let height = entry.updated.unwrap().block_height;

                    info!(
                        "lightclient contract for client type \
                        {client_type} was initiated at {height}"
                    );

                    heights.insert(address, height);
                } else {
                    info!(
                        "lightclient contract for client type \
                        {client_type} has not been stored yet"
                    )
                };
            }

            if let Some(_ucs00) = addresses.app.ucs00 {}

            if let Some(address) = addresses.app.ucs03 {
                if let Some(entry) = ctx
                    .contract_history(address.clone())
                    .await??
                    .unwrap()
                    .entries
                    .first()
                {
                    let height = entry.updated.as_ref().unwrap().block_height;

                    info!("app ucs03 was initiated at {height}");

                    heights.insert(address, height);
                } else {
                    info!("app ucs03 has not been stored yet");
                };
            }

            write_output(output, heights)?;
        }
        App::Migrate {
            rpc_url,
            private_key,
            address,
            new_bytecode,
            gas_config,
        } => {
            let new_bytecode = std::fs::read(new_bytecode).context("reading new bytecode")?;

            let ctx = Deployer(Ctx::new(rpc_url, private_key, gas_config.into()).await?);

            let contract_info = ctx
                .contract_info(address.to_string())
                .await?
                .with_context(|| format!("contract {address} does not exist"))?;

            let checksum = ctx.code_checksum(contract_info.code_id).await?.unwrap();

            if checksum == sha2(BYTECODE_BASE_BYTECODE) {
                bail!("contract {address} has not yet been initiated, it must be fully deployed before it can be migrated")
            } else if checksum == sha2(&new_bytecode) {
                info!("contract {address} has already been migrated to this bytecode");
                return Ok(());
            }

            let (tx_hash, store_code_response) = ctx
                .tx::<_, MsgStoreCodeResponse>(
                    MsgStoreCode {
                        sender: ctx.signer().to_string(),
                        wasm_byte_code: new_bytecode,
                        ..Default::default()
                    },
                    "",
                )
                .await
                .context("store code")?;

            info!(
                %tx_hash,
                code_id = store_code_response.code_id,
                "code stored"
            );

            let (tx_hash, _migrate_response) = ctx
                .tx::<_, MsgMigrateContractResponse>(
                    MsgMigrateContract {
                        sender: ctx.signer().to_string(),
                        contract: address.to_string(),
                        code_id: store_code_response.code_id,
                        msg: json!({ "migrate": {} }).to_string().into_bytes(),
                    },
                    "",
                )
                .await
                .context("migrate")?;

            info!(%tx_hash, "migrated");
        }
        App::AddressOfPrivateKey {
            private_key,
            bech32_prefix,
        } => println!(
            "{}",
            CosmosSigner::from_raw(*private_key.get(), bech32_prefix).unwrap(),
        ),
    }

    Ok(())
}

fn write_output(path: Option<PathBuf>, data: impl Serialize) -> Result<()> {
    let data = serde_json::to_string(&data).unwrap();

    match path {
        Some(output) => {
            std::fs::write(output, data)?;
        }
        None => println!("{data}"),
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Default, clap::Args)]
pub struct GasConfigArgs {
    #[arg(long)]
    pub gas_price: f64,
    #[arg(long)]
    pub gas_denom: String,
    #[arg(long)]
    pub gas_multiplier: f64,
    #[arg(long)]
    pub max_gas: u64,
    #[arg(long, default_value_t = 0)]
    pub min_gas: u64,
}

impl From<GasConfigArgs> for GasConfig {
    fn from(value: GasConfigArgs) -> Self {
        GasConfig {
            gas_price: value.gas_price,
            gas_denom: value.gas_denom,
            gas_multiplier: value.gas_multiplier,
            max_gas: value.max_gas,
            min_gas: value.min_gas,
        }
    }
}

impl GasConfigArgs {
    pub fn mk_fee(&self, gas: u64) -> Fee {
        // gas limit = provided gas * multiplier, clamped between min_gas and max_gas
        let gas_limit = u128_saturating_mul_f64(gas.into(), self.gas_multiplier)
            .clamp(self.min_gas.into(), self.max_gas.into());

        let amount = u128_saturating_mul_f64(gas.into(), self.gas_price);

        Fee {
            amount: vec![Coin {
                amount,
                denom: self.gas_denom.clone(),
            }],
            gas_limit: gas_limit.try_into().unwrap_or(u64::MAX),
            payer: String::new(),
            granter: String::new(),
        }
    }
}

fn u128_saturating_mul_f64(u: u128, f: f64) -> u128 {
    (num_rational::BigRational::from_integer(u.into())
        * num_rational::BigRational::from_float(f).expect("finite"))
    .to_integer()
    .try_into()
    .unwrap_or(u128::MAX)
    // .expect("overflow")
}

struct Deployer(Ctx);

impl Deref for Deployer {
    type Target = Ctx;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deployer {
    async fn contract_info(
        &self,
        address: String,
    ) -> Result<Option<protos::cosmwasm::wasm::v1::ContractInfo>> {
        let result = self
            .0
            .client()
            .grpc_abci_query::<_, protos::cosmwasm::wasm::v1::QueryContractInfoResponse>(
                "/cosmwasm.wasm.v1.Query/ContractInfo",
                &(protos::cosmwasm::wasm::v1::QueryContractInfoRequest { address }),
                None,
                false,
            )
            .await?
            .into_result();

        match result {
            Ok(ok) => Ok(Some(ok.unwrap().contract_info.unwrap())),
            Err(err) => {
                if err.error_code.get() == 6 && err.codespace == "sdk" {
                    Ok(None)
                } else {
                    Err(err.into())
                }
            }
        }
    }

    async fn code_checksum(&self, code_id: u64) -> Result<Option<H256>> {
        let result = self
            .client()
            .grpc_abci_query::<_, protos::cosmwasm::wasm::v1::QueryCodeResponse>(
                "/cosmwasm.wasm.v1.Query/Code",
                &protos::cosmwasm::wasm::v1::QueryCodeRequest { code_id },
                None,
                false,
            )
            .await?
            .into_result();

        match result {
            Ok(ok) => Ok(Some(
                ok.unwrap().code_info.unwrap().data_hash.try_into().unwrap(),
            )),
            Err(err) => Err(err.into()),
        }
    }

    async fn instantiate_code_id_of_contract(&self, address: String) -> Result<Option<u64>> {
        let result = self.contract_history(address).await?;

        match result {
            Ok(ok) => {
                let contract_code_history_entry = &ok.unwrap().entries[0];

                if contract_code_history_entry.operation
                    != protos::cosmwasm::wasm::v1::ContractCodeHistoryOperationType::Init as i32
                {
                    bail!(
                        "invalid state {} for first history entry",
                        contract_code_history_entry.operation
                    )
                }

                Ok(Some(contract_code_history_entry.code_id))
            }
            Err(err) => {
                // if err.error_code.get() == 6 && err.codespace == "sdk" {
                //     Ok(None)
                // } else {
                Err(err.into())
                // }
            }
        }
    }

    async fn contract_history(
        &self,
        address: String,
    ) -> Result<
        Result<
            Option<protos::cosmwasm::wasm::v1::QueryContractHistoryResponse>,
            GrpcAbciQueryError,
        >,
    > {
        Ok(self
            .client()
            .grpc_abci_query::<_, protos::cosmwasm::wasm::v1::QueryContractHistoryResponse>(
                "/cosmwasm.wasm.v1.Query/ContractHistory",
                &protos::cosmwasm::wasm::v1::QueryContractHistoryRequest {
                    address,
                    ..Default::default()
                },
                None,
                false,
            )
            .await?
            .into_result())
    }

    async fn instantiate2_address(&self, checksum: H256, salt: &str) -> Result<String> {
        let bech32 = self
            .signer()
            .to_string()
            .parse::<Bech32<Vec<u8>>>()
            .unwrap();

        let addr = cosmwasm_std::instantiate2_address(
            checksum.get(),
            &bech32.data().as_slice().into(),
            salt.as_bytes(),
        )?;

        Ok(Bech32::new(bech32.hrp(), &*addr).to_string())
    }

    #[instrument(skip_all, fields(%salt))]
    async fn deploy_and_initiate(
        &self,
        wasm_byte_code: Vec<u8>,
        bytecode_base_code_id: u64,
        msg: impl Serialize,
        salt: String,
    ) -> Result<String, anyhow::Error> {
        let address = self
            .instantiate2_address(sha2(BYTECODE_BASE_BYTECODE), &salt)
            .await?;

        info!("{salt} address is {address}");

        match self.contract_deploy_state(address.clone()).await? {
            // only need to instantiate if the contract has not yet been instantiated with the base code
            ContractDeployState::None => {
                let (_, instantiate2_response) = self
                    .tx::<_, MsgInstantiateContract2Response>(
                        MsgInstantiateContract2 {
                            sender: self.signer().to_string(),
                            admin: self.signer().to_string(),
                            code_id: bytecode_base_code_id,
                            label: salt.clone(),
                            msg: json!({}).to_string().into_bytes(),
                            salt: salt.into_bytes(),
                            ..Default::default()
                        },
                        "",
                    )
                    .await
                    .context("instantiate2")?;

                assert_eq!(address, instantiate2_response.address);
            }
            ContractDeployState::Instantiated => {}
            // fully initiated, nothing to do here
            ContractDeployState::Initiated => return Ok(address),
        }

        let (tx_hash, store_code_response) = self
            .tx::<_, MsgStoreCodeResponse>(
                MsgStoreCode {
                    sender: self.signer().to_string(),
                    wasm_byte_code,
                    ..Default::default()
                },
                "",
            )
            .await
            .context("store code")?;

        info!(
            %tx_hash,
            code_id = store_code_response.code_id,
        );

        let (_, _migrate_response) = self
            .tx::<_, MsgMigrateContractResponse>(
                MsgMigrateContract {
                    sender: self.signer().to_string(),
                    contract: address.clone(),
                    code_id: store_code_response.code_id,
                    msg: json!({ "init": msg }).to_string().into_bytes(),
                },
                "",
            )
            .await
            .context("init")?;

        // info!(%tx_hash, );

        Ok(address)
    }

    async fn contract_deploy_state(&self, address: String) -> Result<ContractDeployState> {
        match self.contract_info(address.clone()).await? {
            Some(_) => {
                let contract_history = self.contract_history(address.clone()).await??.unwrap();
                match contract_history.entries.len().cmp(&1) {
                    std::cmp::Ordering::Less => panic!("impossible"),
                    std::cmp::Ordering::Equal => {
                        info!(
                            "contract {address} has already been instantaited with the base bytecode but not yet initiated"
                        );
                        Ok(ContractDeployState::Instantiated)
                    }
                    std::cmp::Ordering::Greater => {
                        info!("contract {address} has already been instantiated and initiated");
                        Ok(ContractDeployState::Initiated)
                    }
                }
            }
            None => Ok(ContractDeployState::None),
        }
    }
}

enum ContractDeployState {
    None,
    Instantiated,
    Initiated,
}

fn instantiate2_address(address: Bech32<Bytes>, checksum: H256, salt: &str) -> Result<String> {
    let addr = cosmwasm_std::instantiate2_address(
        checksum.get(),
        &address.data()[..].into(),
        salt.as_bytes(),
    )?;

    Ok(Bech32::new(address.hrp(), &*addr).to_string())
}
