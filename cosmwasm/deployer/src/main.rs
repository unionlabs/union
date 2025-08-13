use std::{collections::BTreeMap, num::NonZeroU64, ops::Deref, path::PathBuf, str::FromStr};

use anyhow::{bail, Context, Result};
use bip32::secp256k1::ecdsa::SigningKey;
use clap::Parser;
use cometbft_rpc::rpc_types::GrpcAbciQueryError;
use cosmos_client::{
    gas::any::GasFiller as AnyGasFiller,
    rpc::{Rpc, RpcT},
    wallet::{LocalSigner, WalletT},
    TxClient,
};
use cosmwasm_std::Addr;
use futures::{future::OptionFuture, stream::FuturesOrdered, TryStreamExt};
use hex_literal::hex;
use protos::cosmwasm::wasm::v1::{QuerySmartContractStateRequest, QuerySmartContractStateResponse};
use rand_chacha::{
    rand_core::{block::BlockRng, SeedableRng},
    ChaChaCore,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Digest;
use tracing::{info, instrument};
use tracing_subscriber::EnvFilter;
use ucs03_zkgm::msg::TokenMinterInitParams;
use unionlabs::{
    cosmos::{bank::msg_send::MsgSend, base::coin::Coin},
    cosmwasm::wasm::{
        access_config::AccessConfig, msg_execute_contract::MsgExecuteContract,
        msg_instantiate_contract2::MsgInstantiateContract2,
        msg_migrate_contract::MsgMigrateContract, msg_store_code::MsgStoreCode,
        msg_update_admin::MsgUpdateAdmin,
        msg_update_instantiate_config::MsgUpdateInstantiateConfig,
    },
    google::protobuf::any::Any,
    primitives::{Bech32, Bytes, H256},
    signer::CosmosSigner,
};

#[derive(clap::Parser)]
enum App {
    DeployFull {
        #[arg(long)]
        rpc_url: String,
        #[arg(long, env)]
        private_key: H256,
        #[arg(long)]
        contracts: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
        /// Marks this chain as permissioned.
        ///
        /// Permisioned cosmwasm chains require special handling of instantiate permissions in order to deploy the stack.
        #[arg(long)]
        permissioned: bool,
        #[command(flatten)]
        gas_config: GasFillerArgs,
        // #[arg(long)]
        // relayers_admin: Bech32<Bytes>,
        // #[arg(long)]
        // relayers: Vec<Bech32<Bytes>>,
        // #[arg(long)]
        // rate_limit_admin: Bech32<Bytes>,
        // #[arg(long)]
        // rate_limit_operators: Vec<Bech32<Bytes>>,
    },
    Migrate {
        #[arg(long)]
        rpc_url: String,
        #[arg(long, env)]
        private_key: H256,
        #[arg(long)]
        address: Bech32<H256>,
        #[arg(long)]
        new_bytecode: PathBuf,
        #[arg(
            long,
            // the autoref value parser selector chooses From<String> before FromStr, but Value's From<String> impl always returns Value::String(..), whereas FromStr actually parses the json contained within the string
            value_parser(serde_json::Value::from_str),
            default_value_t = serde_json::Value::Object(serde_json::Map::new())
        )]
        message: Value,
        #[arg(long, default_value_t = false)]
        force: bool,
        #[command(flatten)]
        gas_config: GasFillerArgs,
    },
    /// Calculate the addresses of the deployed stack given the deployer.
    ///
    /// The returned addresses will have the same bech32 prefix as the deployer address.
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
        #[arg(long, env)]
        private_key: H256,
        #[arg(long)]
        bech32_prefix: String,
    },
    MigrateAdmin {
        #[arg(long, env)]
        private_key: H256,
        #[arg(long)]
        addresses: PathBuf,
        #[arg(long)]
        rpc_url: String,
        #[arg(long)]
        new_admin: Bech32<Bytes>,
        #[command(flatten)]
        gas_config: GasFillerArgs,
    },
    StoreCode {
        #[arg(long, env)]
        private_key: H256,
        #[arg(long)]
        bytecode: PathBuf,
        #[arg(long)]
        rpc_url: String,
        #[command(flatten)]
        gas_config: GasFillerArgs,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    ProxyCodeId {
        #[arg(long)]
        rpc_url: String,
        #[arg(long, env)]
        private_key: H256,
        #[command(flatten)]
        gas_config: GasFillerArgs,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    #[command(subcommand)]
    Tx(TxCmd),
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

#[derive(clap::Subcommand)]
enum TxCmd {
    WhitelistRelayers {
        #[arg(long)]
        rpc_url: String,
        #[arg(long, env)]
        private_key: H256,
        #[arg(long)]
        contract: Bech32<H256>,
        /// The relayer(s) to whitelist.
        #[arg(trailing_var_arg = true)]
        relayer: Vec<Bech32<Bytes>>,
        #[command(flatten)]
        gas_config: GasFillerArgs,
    },
    CreateSigners {
        #[arg(long)]
        rpc_url: String,
        #[arg(long)]
        count: u32,
        #[arg(long)]
        output: PathBuf,
    },
    InitSigners {
        #[arg(long)]
        rpc_url: String,
        #[arg(long)]
        funder_private_key: H256,
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        amount: u128,
        #[command(flatten)]
        gas_config: GasFillerArgs,
    },
}

#[derive(Debug, Clone, PartialEq, Default, clap::Args)]
pub struct GasFillerArgs {
    #[arg(long, value_enum, default_value_t = GasFillerType::Fixed)]
    pub gas: GasFillerType,

    // Whether or not to simulate the transactions first.
    #[arg(long, default_value_t = false)]
    pub simulate: bool,

    #[arg(long, help_heading = "Gas filler args", default_value_t = 100_000_000)]
    pub max_gas: u64,
    #[arg(long, help_heading = "Gas filler args", default_value_t = 0)]
    pub min_gas: u64,
    #[arg(
        long,
        help_heading = "Gas filler args",
        required_if_eq("gas", "fixed"),
        default_value_t = 1.0
    )]
    pub gas_multiplier: f64,

    #[arg(
        long,
        help_heading = "--gas fixed",
        required_if_eq("gas", "fixed"),
        default_value_t
    )]
    pub gas_price: f64,
    #[arg(
        long,
        help_heading = "--gas fixed",
        required_if_eq("gas", "fixed"),
        default_value_t
    )]
    pub gas_denom: String,

    /// The denom to use for the feemarket gas token.
    ///
    /// If not set, the Params.FeeDenom value will be used.
    #[arg(
        long,
        help_heading = "--gas feemarket",
        // required_if_eq("gas", "feemarket"),
    )]
    pub fee_denom: Option<String>,

    /// The multiplier to use for the EIP-1559 fee calculation.
    ///
    /// This will be multiplied by the base fee as queried from the chain.
    #[arg(
        long,
        help_heading = "--gas osmosis-eip1559-feemarket",
        required_if_eq("gas", "osmosis_eip1559_feemarket")
    )]
    pub base_fee_multiplier: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Default, clap::ValueEnum)]
pub enum GasFillerType {
    #[default]
    Fixed,
    Feemarket,
    OsmosisEip1559Feemarket,
}

async fn any_gas_filler_from_args(args: GasFillerArgs, rpc_url: String) -> Result<AnyGasFiller> {
    Ok(match args.gas {
        GasFillerType::Fixed => AnyGasFiller::Fixed(cosmos_client::gas::fixed::GasFiller {
            gas_price: args.gas_price,
            gas_denom: args.gas_denom.clone(),
            gas_multiplier: args.gas_multiplier,
            max_gas: args.max_gas,
            min_gas: args.min_gas,
        }),
        GasFillerType::Feemarket => AnyGasFiller::Feemarket(
            cosmos_client::gas::feemarket::GasFiller::new(cosmos_client::gas::feemarket::Config {
                rpc_url,
                max_gas: args.max_gas,
                gas_multiplier: Some(args.gas_multiplier),
                denom: args.fee_denom,
            })
            .await?,
        ),
        GasFillerType::OsmosisEip1559Feemarket => AnyGasFiller::OsmosisEip1559Feemarket(
            cosmos_client::gas::osmosis_eip1559_feemarket::GasFiller::new(
                cosmos_client::gas::osmosis_eip1559_feemarket::Config {
                    rpc_url,
                    max_gas: args.max_gas,
                    gas_multiplier: Some(args.gas_multiplier),
                    base_fee_multiplier: args.base_fee_multiplier,
                    denom: args.fee_denom,
                },
            )
            .await?,
        ),
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    do_main().await
}

const BYTECODE_BASE_BYTECODE: &[u8] = &hex_literal::hex!("0061736d0100000001110360037f7f7f017f60017f017f60017f000304030001020503010001074605066d656d6f7279020013696e746572666163655f76657273696f6e5f3800000b696e7374616e7469617465000008616c6c6f6361746500010a6465616c6c6f6361746500020a0f03040041330b0400413f0b0300010b0b4e010041010b487b226f6b223a7b226d65737361676573223a5b5d2c2261747472696275746573223a5b5d2c226576656e7473223a5b5d7d7d0100000032000000320000004b000000000200000002");

fn sha2(bz: impl AsRef<[u8]>) -> H256 {
    ::sha2::Sha256::new().chain_update(bz).finalize().into()
}

const CORE: &str = "ibc-is-based";
const LIGHTCLIENT: &str = "lightclients";
const APP: &str = "protocols";

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
#[serde(rename_all = "snake_case")]
pub struct Ucs03Config {
    path: PathBuf,
    token_minter_path: PathBuf,
    cw_account_path: PathBuf,
    token_minter_config: TokenMinterConfig,
    rate_limit_disabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum TokenMinterConfig {
    Cw20 {
        /// The path to the cw20 implementation contract code.
        ///
        /// This contract MUST be compatible with [`frissitheto`], allowing instantiation through the migrate entrypoint, such that it can have a deterministic address.
        cw20_impl: PathBuf,
    },
    OsmosisTokenfactory {},
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
struct ContractAddresses {
    core: Bech32<H256>,
    lightclient: BTreeMap<String, Bech32<H256>>,
    app: AppAddresses,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
struct AppAddresses {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ucs00: Option<Bech32<H256>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ucs03: Option<Bech32<H256>>,
}

async fn do_main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let app = App::parse();

    match app {
        App::ProxyCodeId {
            rpc_url,
            private_key,
            gas_config,
            output,
        } => {
            let ctx = Deployer::new(rpc_url, private_key, &gas_config).await?;
            let bytecode_base_address = ctx
                .instantiate2_address(sha2(BYTECODE_BASE_BYTECODE), BYTECODE_BASE)
                .await?;
            let code_id = ctx
                .instantiate_code_id_of_contract(bytecode_base_address)
                .await?
                .unwrap();
            write_output(output, code_id)?;
        }
        App::Addresses {
            deployer,
            lightclient,
            apps,
            output,
        } => {
            let addresses = calculate_contract_addresses(deployer, lightclient, apps)?;

            write_output(output, addresses)?;
        }
        App::DeployFull {
            rpc_url,
            private_key,
            contracts,
            output,
            permissioned,
            gas_config,
            // relayers_admin,
            // relayers,
            // rate_limit_admin,
            // rate_limit_operators,
        } => {
            let contracts = serde_json::from_slice::<ContractPaths>(
                &std::fs::read(contracts).context("reading contracts path")?,
            )?;

            let ctx = Deployer::new(rpc_url, private_key, &gas_config).await?;

            let bytecode_base_address = ctx
                .instantiate2_address(sha2(BYTECODE_BASE_BYTECODE), BYTECODE_BASE)
                .await?;

            let bytecode_base_contract = ctx.contract_info(&bytecode_base_address).await?;

            let bytecode_base_code_id = match bytecode_base_contract {
                Some(_) => ctx
                    .instantiate_code_id_of_contract(bytecode_base_address)
                    .await?
                    .unwrap(),
                // contract does not exist on chain
                None => {
                    info!("bytecode-base has not yet been stored");

                    let (tx_hash, store_code_response) = ctx
                        .tx(
                            MsgStoreCode {
                                sender: ctx.wallet().address().map_data(Into::into),
                                wasm_byte_code: BYTECODE_BASE_BYTECODE.into(),
                                instantiate_permission: Some(AccessConfig::Everybody),
                            },
                            "",
                            gas_config.simulate,
                        )
                        .await
                        .context("store code")?;

                    info!(%tx_hash, code_id = store_code_response.code_id, "stored bytecode-base");

                    let (tx_hash, instantiate_response) = ctx
                        .tx(
                            MsgInstantiateContract2 {
                                sender: ctx.wallet().address().map_data(Into::into),
                                admin: ctx.wallet().address().map_data(Into::into),
                                code_id: store_code_response.code_id,
                                label: BYTECODE_BASE.to_string(),
                                msg: b"{}".into(),
                                salt: BYTECODE_BASE.as_bytes().into(),
                                funds: vec![],
                                fix_msg: false,
                            },
                            "",
                            gas_config.simulate,
                        )
                        .await
                        .context("instantiate2")?;

                    info!(%tx_hash, address = %instantiate_response.address, "instantiated bytecode-base");

                    store_code_response.code_id
                }
            };

            info!("bytecode-base code_id is {bytecode_base_code_id}");

            let core_address = ctx
                .deploy_and_initiate(
                    std::fs::read(contracts.core)?,
                    bytecode_base_code_id,
                    ibc_union_msg::msg::InitMsg {
                        relayers_admin: Some(ctx.wallet().address().to_string()),
                        relayers: vec![ctx.wallet().address().to_string()],
                    },
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
                            ibc_host: Addr::unchecked(core_address.to_string()),
                        },
                        format!("{LIGHTCLIENT}/{client_type}"),
                    )
                    .await?;

                let response = ctx
                    .rpc()
                    .client()
                    .grpc_abci_query::<_, QuerySmartContractStateResponse>(
                        "/cosmwasm.wasm.v1.Query/SmartContractState",
                        &QuerySmartContractStateRequest {
                            address: core_address.to_string(),
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
                    .map(|value| serde_json::from_slice::<Bech32<H256>>(&value.data))
                    .transpose()?
                {
                    assert_eq!(addr, address);
                    info!("client {client_type} has already been registered");
                } else {
                    ctx.tx(
                        MsgExecuteContract {
                            sender: ctx.wallet().address().map_data(Into::into),
                            contract: core_address.clone(),
                            msg: serde_json::to_vec(
                                &ibc_union_msg::msg::ExecuteMsg::RegisterClient(
                                    ibc_union_msg::msg::MsgRegisterClient {
                                        client_type: client_type.clone(),
                                        client_address: address.to_string(),
                                    },
                                ),
                            )
                            .unwrap()
                            .into(),
                            funds: vec![],
                        },
                        "",
                        gas_config.simulate,
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
                    ctx.wallet().address(),
                    sha2(BYTECODE_BASE_BYTECODE),
                    &salt,
                )
                .unwrap();

                info!("ucs03 address is {ucs03_address}");

                let state = ctx.contract_deploy_state(ucs03_address.clone()).await?;

                if let ContractDeployState::None | ContractDeployState::Instantiated = state {
                    let (tx_hash, response) = ctx
                        .tx(
                            MsgStoreCode {
                                sender: ctx.wallet().address().map_data(Into::into),
                                wasm_byte_code: std::fs::read(ucs03_config.token_minter_path)?
                                    .into(),
                                instantiate_permission: None,
                            },
                            "",
                            gas_config.simulate,
                        )
                        .await
                        .context("store minter code")?;

                    let minter_code_id = response.code_id;

                    info!(%tx_hash, minter_code_id, "minter stored");

                    // on permissioned cosmwasm, we must specify that this code can be instantiated by the ucs03 contract
                    if permissioned {
                        let (tx_hash, _) = ctx
                            .tx(
                                MsgUpdateInstantiateConfig {
                                    sender: ctx.wallet().address().map_data(Into::into),
                                    code_id: minter_code_id,
                                    new_instantiate_permission: Some(
                                        AccessConfig::AnyOfAddresses {
                                            addresses: vec![ucs03_address
                                                .clone()
                                                .map_data(Into::into)],
                                        },
                                    ),
                                },
                                "",
                                gas_config.simulate,
                            )
                            .await
                            .context("update instantiate perms of cw20 impl")?;

                        info!(%tx_hash, "cw20 impl instantiate permissions updated");

                        // the bytecode base code id must be instantiable by ucs03
                        ctx.update_bytecode_base_instantiate_permissions(
                            bytecode_base_code_id,
                            ucs03_address.clone(),
                        )
                        .await?;
                    }

                    let token_minter_address = instantiate2_address(
                        ucs03_address.clone(),
                        response.checksum,
                        &ucs03_zkgm::contract::minter_salt(),
                    )
                    .unwrap();

                    let minter_init_params = match ucs03_config.token_minter_config {
                        TokenMinterConfig::Cw20 {
                            cw20_impl: cw20_base,
                        } => {
                            let (tx_hash, response) = ctx
                                .tx(
                                    MsgStoreCode {
                                        sender: ctx.wallet().address().map_data(Into::into),
                                        wasm_byte_code: std::fs::read(cw20_base)?.into(),
                                        instantiate_permission: None,
                                    },
                                    "",
                                    gas_config.simulate,
                                )
                                .await
                                .context("store cw20 impl code")?;

                            let code_id = response.code_id;

                            info!(%tx_hash, code_id, "cw20 impl stored");

                            // on permissioned cosmwasm, we must specify that this code can be instantiated by the token minter
                            if permissioned {
                                let (tx_hash, _) = ctx
                                    .tx(
                                        MsgUpdateInstantiateConfig {
                                            sender: ctx.wallet().address().map_data(Into::into),
                                            code_id,
                                            new_instantiate_permission: Some(
                                                AccessConfig::AnyOfAddresses {
                                                    addresses: vec![token_minter_address
                                                        .clone()
                                                        .map_data(Into::into)
                                                        .clone()],
                                                },
                                            ),
                                        },
                                        "",
                                        gas_config.simulate,
                                    )
                                    .await
                                    .context("update instantiate perms of cw20 impl")?;

                                info!(%tx_hash, "cw20 impl instantiate permissions updated");

                                // the bytecode-base code must also be instantiable by the token minter
                                ctx.update_bytecode_base_instantiate_permissions(
                                    bytecode_base_code_id,
                                    token_minter_address,
                                )
                                .await?;
                            }

                            TokenMinterInitParams::Cw20 {
                                cw20_impl_code_id: code_id.get(),
                                dummy_code_id: bytecode_base_code_id.get(),
                            }
                        }
                        TokenMinterConfig::OsmosisTokenfactory {} => {
                            TokenMinterInitParams::OsmosisTokenFactory {}
                        }
                    };

                    let (tx_hash, response) = ctx
                        .tx(
                            MsgStoreCode {
                                sender: ctx.wallet().address().map_data(Into::into),
                                wasm_byte_code: std::fs::read(ucs03_config.cw_account_path)?.into(),
                                instantiate_permission: None,
                            },
                            "",
                            gas_config.simulate,
                        )
                        .await
                        .context("store minter code")?;

                    let cw_account_code_id = response.code_id;

                    info!(%tx_hash, cw_account_code_id, "cw_account stored");

                    // on permissioned cosmwasm, we must specify that this code can be instantiated by the ucs03 contract
                    if permissioned {
                        let (tx_hash, _) = ctx
                            .tx(
                                MsgUpdateInstantiateConfig {
                                    sender: ctx.wallet().address().map_data(Into::into),
                                    code_id: cw_account_code_id,
                                    new_instantiate_permission: Some(
                                        AccessConfig::AnyOfAddresses {
                                            addresses: vec![ucs03_address
                                                .clone()
                                                .map_data(Into::into)],
                                        },
                                    ),
                                },
                                "",
                                gas_config.simulate,
                            )
                            .await
                            .context("update instantiate perms of cw-account")?;

                        info!(%tx_hash, "cw-account instantiate permissions updated");
                    }

                    ctx.deploy_and_initiate(
                        std::fs::read(ucs03_config.path)?,
                        bytecode_base_code_id,
                        ucs03_zkgm::msg::InitMsg {
                            config: ucs03_zkgm::msg::Config {
                                admin: Addr::unchecked(ctx.wallet().address().to_string()),
                                ibc_host: Addr::unchecked(core_address.to_string()),
                                token_minter_code_id: minter_code_id.into(),
                                rate_limit_admin: Addr::unchecked(
                                    ctx.wallet().address().to_string(),
                                ),
                                rate_limit_operators: vec![Addr::unchecked(
                                    ctx.wallet().address().to_string(),
                                )],
                                rate_limit_disabled: ucs03_config.rate_limit_disabled,
                                dummy_code_id: bytecode_base_code_id.get(),
                                cw_account_code_id: cw_account_code_id.get(),
                            },
                            minter_init_params,
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

            let ctx = Deployer::new(
                rpc_url,
                hex!("9a95f0bb285a5d81415a0571cebbb63dbef2c7a0c90f9b60a40572552da3eac3").into(),
                &GasFillerArgs::default(),
            )
            .await?;

            let mut heights = BTreeMap::new();

            heights.insert(
                addresses.core.clone(),
                ctx.contract_history(addresses.core.clone())
                    .await??
                    .unwrap()
                    .entries
                    .first()
                    .as_ref()
                    .unwrap()
                    .updated
                    .as_ref()
                    .unwrap()
                    .block_height,
            );

            for (client_type, address) in addresses.lightclient {
                if let Some(entry) = ctx
                    .contract_history(address.clone())
                    .await??
                    .unwrap()
                    .entries
                    .first()
                {
                    let height = entry.updated.as_ref().unwrap().block_height;

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
            message,
            force,
            gas_config,
        } => {
            let new_bytecode = std::fs::read(new_bytecode).context("reading new bytecode")?;

            info!("migrating address {address}");

            let ctx = Deployer::new(rpc_url, private_key, &gas_config).await?;

            let contract_info = ctx
                .contract_info(&address)
                .await?
                .with_context(|| format!("contract {address} does not exist"))?;

            let checksum = ctx.code_checksum(contract_info.code_id).await?.unwrap();

            if checksum == sha2(BYTECODE_BASE_BYTECODE) {
                bail!("contract {address} has not yet been initiated, it must be fully deployed before it can be migrated")
            } else if checksum == sha2(&new_bytecode) {
                if force {
                    info!("contract {address} has already been migrated to this bytecode, migrating anyways since --force was passed");
                } else {
                    info!("contract {address} has already been migrated to this bytecode");
                    return Ok(());
                }
            }

            let (tx_hash, store_code_response) = ctx
                .tx(
                    MsgStoreCode {
                        sender: ctx.wallet().address().map_data(Into::into),
                        wasm_byte_code: new_bytecode.into(),
                        instantiate_permission: None,
                    },
                    "",
                    gas_config.simulate,
                )
                .await
                .context("store code")?;

            info!(
                %tx_hash,
                code_id = store_code_response.code_id,
                "code stored"
            );

            let message = json!({ "migrate": message });

            info!("migrate message: {message}");

            let (tx_hash, _migrate_response) = ctx
                .tx(
                    MsgMigrateContract {
                        sender: ctx.wallet().address().map_data(Into::into),
                        contract: address,
                        code_id: store_code_response.code_id,
                        msg: message.to_string().into_bytes().into(),
                    },
                    "",
                    gas_config.simulate,
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
        App::MigrateAdmin {
            private_key,
            addresses,
            rpc_url,
            new_admin,
            gas_config,
        } => {
            let addresses = serde_json::from_slice::<ContractAddresses>(
                &std::fs::read(addresses).context("reading addresses path")?,
            )?;

            let ctx = Deployer::new(rpc_url, private_key, &gas_config).await?;

            let check_contract = async |salt, address| {
                let Some(core_info) = ctx.contract_info(&address).await? else {
                    return Ok(None);
                };

                if core_info.admin == new_admin.to_string() {
                    info!("{salt} already migrated to admin {new_admin}");
                    Ok(None)
                } else if core_info.admin != ctx.wallet().address().to_string() {
                    bail!(
                        "the admin of {salt} is not {}, found {}",
                        ctx.wallet().address(),
                        core_info.admin
                    );
                } else {
                    Ok(Some(address))
                }
            };

            let mut messages = [check_contract(CORE.to_owned(), addresses.core).await?]
                .into_iter()
                // not sure why i have to collect here but whatever
                .chain(
                    addresses
                        .lightclient
                        .into_iter()
                        .map(|(client_type, address)| {
                            check_contract(format!("{LIGHTCLIENT}/{client_type}"), address)
                        })
                        .collect::<FuturesOrdered<_>>()
                        .try_collect::<Vec<_>>()
                        .await?,
                )
                // .chain(check_contract(addresses.app.ucs00))
                .chain(
                    OptionFuture::from(
                        addresses
                            .app
                            .ucs03
                            .map(|address| check_contract(format!("{APP}/{UCS03}"), address)),
                    )
                    .await
                    .transpose()?,
                )
                .flatten()
                .map(|contract| {
                    Any(MsgUpdateAdmin {
                        sender: ctx.wallet().address().map_data(Into::into),
                        new_admin: new_admin.clone(),
                        contract,
                    })
                })
                .peekable();

            if messages.peek().is_none() {
                info!("all contracts already migrated");
            } else {
                let result = ctx.broadcast_tx_commit(messages, "", true).await?;

                info!(tx_hash = %result.hash, "admin migrated to {new_admin}");
            }
        }
        App::StoreCode {
            private_key,
            bytecode,
            rpc_url,
            gas_config,
            output,
        } => {
            let bytecode = std::fs::read(bytecode).context("reading bytecode path")?;

            let deployer = Deployer::new(rpc_url.clone(), private_key, &gas_config).await?;

            let (tx_hash, response) = deployer
                .tx(
                    MsgStoreCode {
                        sender: deployer.wallet().address().map_data(Into::into),
                        wasm_byte_code: bytecode.into(),
                        // TODO: Support permissions
                        instantiate_permission: None,
                    },
                    "",
                    gas_config.simulate,
                )
                .await
                .context("store code")?;

            let code_id = response.code_id;

            info!(%tx_hash, %code_id, "stored code");

            write_output(output, code_id)?;
        }
        App::Tx(tx_cmd) => match tx_cmd {
            TxCmd::WhitelistRelayers {
                rpc_url,
                private_key,
                contract,
                relayer,
                gas_config,
            } => {
                let ctx = Deployer::new(rpc_url, private_key, &gas_config).await?;

                info!("whitelisting {} relayers", relayer.len());

                for relayer in relayer {
                    let (tx_hash, _) = ctx
                        .tx(
                            MsgExecuteContract {
                                sender: ctx.wallet().address().map_data(Into::into),
                                contract: contract.clone(),
                                msg: serde_json::to_vec(
                                    &ibc_union_msg::msg::ExecuteMsg::AddRelayer(
                                        relayer.to_string(),
                                    ),
                                )
                                .unwrap()
                                .into(),
                                funds: vec![],
                            },
                            "",
                            gas_config.simulate,
                        )
                        .await?;

                    info!(%tx_hash, "registered relayer {relayer}");
                }
            }
            TxCmd::CreateSigners {
                rpc_url,
                count,
                output,
            } => {
                let mut rng = BlockRng::new(ChaChaCore::from_rng(rand_chacha::rand_core::OsRng)?);

                let client = cometbft_rpc::Client::new(rpc_url).await?;

                let bech32_prefix = client
                    .grpc_abci_query::<_, protos::cosmos::auth::v1beta1::Bech32PrefixResponse>(
                        "/cosmos.auth.v1beta1.Query/Bech32Prefix",
                        &protos::cosmos::auth::v1beta1::Bech32PrefixRequest {},
                        None,
                        false,
                    )
                    .await
                    .context("querying bech32 prefix")?
                    .into_result()?
                    .unwrap()
                    .bech32_prefix;

                let signers = (0..count)
                    .map(|_| {
                        let signer =
                            CosmosSigner::new(SigningKey::random(&mut rng), bech32_prefix.clone());

                        info!("signer created: {signer}");

                        signer
                    })
                    .collect::<Vec<_>>();

                write_output(
                    Some(output),
                    signers.iter().map(|s| s.private_key()).collect::<Vec<_>>(),
                )?;
            }
            TxCmd::InitSigners {
                rpc_url,
                funder_private_key,
                path,
                amount,
                gas_config,
            } => {
                let funder =
                    Deployer::new(rpc_url.clone(), funder_private_key, &gas_config).await?;

                let bech32_prefix = funder
                    .rpc()
                    .client()
                    .grpc_abci_query::<_, protos::cosmos::auth::v1beta1::Bech32PrefixResponse>(
                        "/cosmos.auth.v1beta1.Query/Bech32Prefix",
                        &protos::cosmos::auth::v1beta1::Bech32PrefixRequest {},
                        None,
                        false,
                    )
                    .await
                    .context("querying bech32 prefix")?
                    .into_result()?
                    .unwrap()
                    .bech32_prefix;

                let signers = serde_json::from_slice::<Vec<H256>>(
                    &std::fs::read(path).context("reading signers path")?,
                )?
                .into_iter()
                .map(|s| CosmosSigner::new_from_bytes(s, bech32_prefix.clone()).unwrap());

                for (idx, signer) in signers.clone().enumerate() {
                    info!("signer {signer} ({idx})");
                    if funder.account_info(signer.address()).await?.is_none() {
                        info!("funding signer {signer}");

                        funder
                            .tx(
                                MsgSend {
                                    from_address: funder.wallet().address().map_data(Into::into),
                                    to_address: signer.address().map_data(Into::into),
                                    amount: vec![Coin {
                                        denom: "TODO".to_owned(),
                                        amount,
                                    }],
                                },
                                "",
                                gas_config.simulate,
                            )
                            .await?;

                        info!("funded signer {signer}");

                        let signer =
                            Deployer::new(rpc_url.clone(), signer.private_key(), &gas_config)
                                .await?;

                        signer
                            .tx(
                                MsgSend {
                                    from_address: signer.wallet().address().map_data(Into::into),
                                    to_address: signer.wallet().address().map_data(Into::into),
                                    amount: vec![Coin {
                                        denom: "TODO".to_owned(),
                                        amount: 1,
                                    }],
                                },
                                "",
                                gas_config.simulate,
                            )
                            .await?;

                        info!("initiated signer {}", signer.wallet().address());
                    } else {
                        info!("signer {signer} already funded")
                    }
                }

                write_output(
                    None,
                    signers.into_iter().map(|s| s.address()).collect::<Vec<_>>(),
                )?;
            }
        },
    }

    Ok(())
}

fn calculate_contract_addresses(
    deployer: Bech32,
    lightclient: Vec<String>,
    apps: AppFlags,
) -> Result<ContractAddresses> {
    let core = instantiate2_address(deployer.clone(), sha2(BYTECODE_BASE_BYTECODE), CORE)?;

    let lightclient = lightclient
        .into_iter()
        .map(|salt| {
            Result::Ok((
                salt.clone(),
                instantiate2_address(
                    deployer.clone(),
                    sha2(BYTECODE_BASE_BYTECODE),
                    &format!("{LIGHTCLIENT}/{salt}"),
                )?,
            ))
        })
        .collect::<Result<_>>()?;

    let mut app = AppAddresses::default();

    if apps.ucs00 {
        todo!()
    }

    if apps.ucs03 {
        app.ucs03 = Some(instantiate2_address(
            deployer,
            sha2(BYTECODE_BASE_BYTECODE),
            &format!("{APP}/{UCS03}"),
        )?);
    }

    Ok(ContractAddresses {
        core,
        lightclient,
        app,
    })
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

struct Deployer {
    client: TxClient<LocalSigner, Rpc, AnyGasFiller>,
    simulate: bool,
}

impl Deref for Deployer {
    type Target = TxClient<LocalSigner, Rpc, AnyGasFiller>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl Deployer {
    async fn new(
        rpc_url: String,
        private_key: H256,
        gas_config: &GasFillerArgs,
    ) -> Result<Deployer> {
        let rpc = Rpc::new(rpc_url.clone()).await?;

        let bech32_prefix = rpc
            .client()
            .grpc_abci_query::<_, protos::cosmos::auth::v1beta1::Bech32PrefixResponse>(
                "/cosmos.auth.v1beta1.Query/Bech32Prefix",
                &protos::cosmos::auth::v1beta1::Bech32PrefixRequest {},
                None,
                false,
            )
            .await
            .context("querying bech32 prefix")?
            .into_result()?
            .unwrap()
            .bech32_prefix;

        let ctx = TxClient::new(
            LocalSigner::new(private_key, bech32_prefix),
            rpc,
            any_gas_filler_from_args(gas_config.clone(), rpc_url).await?,
        );

        let ctx = Deployer {
            simulate: gas_config.simulate,
            client: ctx,
        };

        Ok(ctx)
    }

    async fn contract_info(
        &self,
        address: &Bech32<H256>,
    ) -> Result<Option<protos::cosmwasm::wasm::v1::ContractInfo>> {
        let result = self
            .rpc()
            .client()
            .grpc_abci_query::<_, protos::cosmwasm::wasm::v1::QueryContractInfoResponse>(
                "/cosmwasm.wasm.v1.Query/ContractInfo",
                &protos::cosmwasm::wasm::v1::QueryContractInfoRequest {
                    address: address.to_string(),
                },
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
            .rpc()
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

    async fn instantiate_code_id_of_contract(
        &self,
        address: Bech32<H256>,
    ) -> Result<Option<NonZeroU64>> {
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

                Ok(Some(
                    contract_code_history_entry.code_id.try_into().unwrap(),
                ))
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
        address: Bech32<H256>,
    ) -> Result<
        Result<
            Option<protos::cosmwasm::wasm::v1::QueryContractHistoryResponse>,
            GrpcAbciQueryError,
        >,
    > {
        Ok(self
            .rpc()
            .client()
            .grpc_abci_query::<_, protos::cosmwasm::wasm::v1::QueryContractHistoryResponse>(
                "/cosmwasm.wasm.v1.Query/ContractHistory",
                &protos::cosmwasm::wasm::v1::QueryContractHistoryRequest {
                    address: address.to_string(),
                    ..Default::default()
                },
                None,
                false,
            )
            .await?
            .into_result())
    }

    async fn instantiate2_address(&self, checksum: H256, salt: &str) -> Result<Bech32<H256>> {
        let bech32 = self.wallet().address();

        let addr = cosmwasm_std::instantiate2_address(
            checksum.get(),
            &(bech32.data().get().as_slice().into()),
            salt.as_bytes(),
        )?;

        Ok(Bech32::new(
            bech32.hrp().to_owned(),
            addr.as_slice().try_into().unwrap(),
        ))
    }

    #[instrument(skip_all, fields(%salt))]
    async fn deploy_and_initiate(
        &self,
        wasm_byte_code: Vec<u8>,
        bytecode_base_code_id: NonZeroU64,
        msg: impl Serialize,
        salt: String,
    ) -> Result<Bech32<H256>> {
        let address = self
            .instantiate2_address(sha2(BYTECODE_BASE_BYTECODE), &salt)
            .await?;

        info!("{salt} address is {address}");

        match self.contract_deploy_state(address.clone()).await? {
            // only need to instantiate if the contract has not yet been instantiated with the base code
            ContractDeployState::None => {
                let (_, instantiate2_response) = self
                    .tx(
                        MsgInstantiateContract2 {
                            sender: self.wallet().address().map_data(Into::into),
                            admin: self.wallet().address().map_data(Into::into),
                            code_id: bytecode_base_code_id,
                            label: salt.clone(),
                            msg: json!({}).to_string().into_bytes().into(),
                            salt: salt.into_bytes().into(),
                            funds: vec![],
                            fix_msg: false,
                        },
                        "",
                        self.simulate,
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
            .tx(
                MsgStoreCode {
                    sender: self.wallet().address().map_data(Into::into),
                    wasm_byte_code: wasm_byte_code.into(),
                    instantiate_permission: None,
                },
                "",
                self.simulate,
            )
            .await
            .context("store code")?;

        info!(
            %tx_hash,
            code_id = store_code_response.code_id,
        );

        let (_, _migrate_response) = self
            .tx(
                MsgMigrateContract {
                    sender: self.wallet().address().map_data(Into::into),
                    contract: address.clone(),
                    code_id: store_code_response.code_id,
                    msg: json!({ "init": msg }).to_string().into_bytes().into(),
                },
                "",
                self.simulate,
            )
            .await
            .context("init")?;

        // info!(%tx_hash, );

        Ok(address)
    }

    async fn contract_deploy_state(&self, address: Bech32<H256>) -> Result<ContractDeployState> {
        match self.contract_info(&address).await? {
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

    #[instrument(skip_all, fields(bytecode_base_code_id, address))]
    async fn update_bytecode_base_instantiate_permissions(
        &self,
        bytecode_base_code_id: NonZeroU64,
        address: Bech32<H256>,
    ) -> Result<()> {
        let bytecode_base_code_info = self
            .rpc()
            .client()
            .grpc_abci_query::<_, protos::cosmwasm::wasm::v1::QueryCodeResponse>(
                "/cosmwasm.wasm.v1.Query/Code",
                &protos::cosmwasm::wasm::v1::QueryCodeRequest {
                    code_id: bytecode_base_code_id.get(),
                },
                None,
                false,
            )
            .await?
            .into_result()
            .context("querying bytecode base code id")?
            .expect("must exist");

        assert_eq!(
            bytecode_base_code_info.data, BYTECODE_BASE_BYTECODE,
            "invalid bytecode-base code"
        );

        if !bytecode_base_code_info
            .code_info
            .as_ref()
            .unwrap()
            .instantiate_permission
            .as_ref()
            .unwrap()
            .addresses
            .iter()
            .any(|a| a == &address.to_string())
        {
            let (tx_hash, _) = self
                .tx(
                    MsgUpdateInstantiateConfig {
                        sender: self.wallet().address().map_data(Into::into),
                        code_id: bytecode_base_code_id,
                        new_instantiate_permission: Some(AccessConfig::AnyOfAddresses {
                            addresses: bytecode_base_code_info
                                .code_info
                                .unwrap()
                                .instantiate_permission
                                .unwrap()
                                .addresses
                                .into_iter()
                                .map(|a| a.parse().unwrap())
                                .chain([address.clone().map_data(Into::into)])
                                .collect(),
                        }),
                    },
                    "",
                    self.simulate,
                )
                .await
                .context("update bytecode-base instantiate permissions")?;

            info!(%tx_hash, "{address} added to bytecode-base instantiate permissions");
        } else {
            info!("{address} is already in bytecode-base instantiate permissions");
        };

        Ok(())
    }
}

enum ContractDeployState {
    None,
    Instantiated,
    Initiated,
}

fn instantiate2_address(
    address: Bech32<impl AsRef<[u8]>>,
    checksum: H256,
    salt: &str,
) -> Result<Bech32<H256>> {
    let addr = cosmwasm_std::instantiate2_address(
        checksum.get(),
        &address.data().as_ref().into(),
        salt.as_bytes(),
    )?;

    Ok(Bech32::new(
        address.hrp().to_owned(),
        addr.as_slice().try_into().unwrap(),
    ))
}
