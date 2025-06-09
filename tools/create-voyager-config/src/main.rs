#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines, clippy::match_wildcard_for_single_variants)]

use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Display,
    fs,
    path::PathBuf,
    str::FromStr,
    time::Duration,
};

use anyhow::{bail, Result};
use beacon_api_types::chain_spec::PresetBaseKind;
use chain_kitchen::{Endpoint, Protocol};
use clap::Parser;
use cliclack::{confirm, input, intro, log, outro, select};
use deployments::DEPLOYMENTS;
use heck::ToKebabCase;
use hex_literal::hex;
use pg_queue::{
    default_max_connections, default_min_connections, default_retryable_error_expo_backoff_max,
    default_retryable_error_expo_backoff_multiplier, PgQueueConfig,
};
use serde::{Deserialize, Serialize};
#[allow(clippy::enum_glob_use)]
use ucs04::Family::*;
use ucs04::{
    is_well_known,
    well_known::{
        self, BOB_60808, BOB_808813, CORN_21000000, CORN_21000001, ETHEREUM_1, ETHEREUM_11155111,
        SEI_1328, SEI_1329, SEI_ATLANTIC_2, SEI_PACIFIC_1,
    },
    Family, Id, UniversalChainId,
};
use unionlabs::{
    bech32::Bech32,
    primitives::{H160, H256},
};
use voyager_config::VoyagerConfig;
use voyager_core::{
    context::{ModuleConfig, ModulesConfig},
    default_metrics_endpoint, default_rest_laddr, default_rpc_laddr,
    equivalent_chain_ids::EquivalentChainIds,
};
use voyager_primitives::{ChainId, ConsensusType, IbcSpecId};
use voyager_rpc::types::{FinalityModuleInfo, ProofModuleInfo, StateModuleInfo};

pub const SUPPORTED_FAMILIES: &[Family] = &[
    Family::Babylon,
    Family::Bob,
    Family::Corn,
    Family::Ethereum,
    Family::Osmosis,
    Family::Sei,
    Family::Union,
    Family::Xion,
];

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    derive_more::Display,
)]
#[serde(rename_all = "snake_case")]
pub enum Key {
    #[display(fmt = "cometbft rpc url")]
    CometbftRpcUrl,
    #[display(fmt = "eth rpc url")]
    EthRpcUrl,
    #[display(fmt = "cosmos chain id")]
    CosmosChainId,
    #[display(fmt = "evm chain id")]
    EvmChainId,
    #[display(fmt = "beacon rpc url")]
    BeaconRpcUrl,
    #[display(fmt = "chain spec")]
    ChainSpec,
    #[display(fmt = "L1 contract address")]
    L1ContractAddress,
    #[display(fmt = "L2 oracle address")]
    L2OracleAddress,
    #[display(fmt = "L1 chain id")]
    L1ChainId,
    #[display(fmt = "ibc host contract address")]
    IbcHostContractAddress,
    #[display(fmt = "ibc handler address")]
    IbcHandlerAddress,
    #[display(fmt = "max query window")]
    MaxQueryWindow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum QueueConfig {
    PgQueue(PgQueueConfig),
}

#[derive(Parser)]
enum Args {
    Bootstrap,
}

fn main() -> Result<()> {
    match Args::parse() {
        Args::Bootstrap => bootstrap(),
    }
}

fn bootstrap() -> Result<()> {
    intro("create-voyager-config")?;

    let mut context = Context::new(input("plugins base path").interact()?);

    let mut config = voyager_config::Config::<QueueConfig> {
        schema: None,
        equivalent_chain_ids: EquivalentChainIds::default(),
        modules: ModulesConfig::default(),
        plugins: vec![],
        voyager: VoyagerConfig {
            num_workers: 50,
            rest_laddr: default_rest_laddr(),
            rpc_laddr: default_rpc_laddr(),
            metrics_endpoint: default_metrics_endpoint(),
            queue: QueueConfig::PgQueue(PgQueueConfig {
                database_url: input("database url")
                    .default_input("postgres://postgres:postgrespassword@127.0.0.1:5432/default")
                    .interact()?,
                max_connections: default_max_connections(),
                min_connections: default_min_connections(),
                idle_timeout: None,
                max_lifetime: None,
                optimize_batch_limit: None,
                retryable_error_expo_backoff_max: default_retryable_error_expo_backoff_max(),
                retryable_error_expo_backoff_multiplier:
                    default_retryable_error_expo_backoff_multiplier(),
            }),
            optimizer_delay_milliseconds: 100,
            ipc_client_request_timeout: Duration::new(60, 0),
            cache: voyager_core::cache::Config::default(),
        },
    };

    loop {
        log::step("chain pair")?;

        log::info("first chain")?;

        let a = UniversalChainId::new(read_chain_family()?, &read_chain_id()?).into_owned();

        log::info(format!("chain pair [{a}]"))?;

        log::info("second chain")?;

        let b = UniversalChainId::new(read_chain_family()?, &read_chain_id()?).into_owned();

        if a == b {
            log::error("cannot connect chain to itself")?;
            continue;
        }

        log::info(format!("chain pair [{a}, {b}]"))?;

        let modules = context.build_chain_pair(&a, &b)?;

        context.dump_receipt()?;

        merge_modules(&mut config.modules, modules);

        if !confirm("add another chain pair?").interact()? {
            break;
        }
    }

    fs::write(
        "voyager.json",
        serde_json::to_string_pretty(&config).unwrap(),
    )?;

    outro("create-voyager-config")?;

    Ok(())
}

fn merge_modules(base: &mut ModulesConfig, new: ModulesConfig) {
    base.state.extend(new.state);
    base.state.dedup_by_key(|s| s.info.id());

    base.proof.extend(new.proof);
    base.proof.dedup_by_key(|s| s.info.id());

    base.consensus.extend(new.consensus);
    base.consensus.dedup_by_key(|s| s.info.id());

    base.client.extend(new.client);
    base.client.dedup_by_key(|s| s.info.id());

    base.client_bootstrap.extend(new.client_bootstrap);
    base.client_bootstrap.dedup_by_key(|s| s.info.id());
}

macro_rules! module_config {
    (
        $this:ident {
            info: $info:expr,
            config: $path:ident::Config $init:tt,
        }
    ) => {
        ModuleConfig {
            path: $this.make_path(stringify!($path).to_kebab_case()),
            info: $info,
            config: serde_json::to_value($path::Config $init).unwrap(),
            enabled: true,
        }
    };
}

struct Context<'a> {
    // id -> key -> value
    defaults: BTreeMap<UniversalChainId<'a>, BTreeMap<Key, String>>,
    base_path: String,
}

impl<'a> Context<'a> {
    fn new(base_path: String) -> Self {
        Self {
            defaults:
                serde_json::from_str::<BTreeMap<UniversalChainId<'_>, BTreeMap<Key, String>>>(
                    &fs::read_to_string("./receipt.json").unwrap(),
                )
                .unwrap()
                .into_iter()
                .map(|(k, v)| (k.into_owned(), v))
                .collect(),
            base_path,
        }
    }

    fn with_chain(&mut self, id: &UniversalChainId<'a>) -> Result<()> {
        self.defaults.entry(id.clone()).or_default();

        // load deployment info
        match id.family() {
            Babylon | Osmosis | Stargaze | Stride | Union | Xion => {
                let entry = self
                    .defaults
                    .get_mut(id)
                    .unwrap()
                    .entry(Key::IbcHostContractAddress);

                if let Entry::Vacant(vacant_entry) = entry {
                    if let Some(deployment) = DEPLOYMENTS.get(id) {
                        vacant_entry.insert(deployment.deployments.core.address.to_string());
                    } else {
                        vacant_entry.insert(
                            input(format!("{id} {}", Key::IbcHostContractAddress))
                                .interact::<Bech32<H256>>()?
                                .to_string(),
                        );
                    }
                }
            }
            Arbitrum | Berachain | Bob | Corn | Ethereum | Movement | Scroll | Sei => {
                let entry = self
                    .defaults
                    .get_mut(id)
                    .unwrap()
                    .entry(Key::IbcHandlerAddress);

                if let Entry::Vacant(vacant_entry) = entry {
                    if let Some(deployment) = DEPLOYMENTS.get(id) {
                        vacant_entry.insert(deployment.deployments.core.address.to_string());
                    } else {
                        vacant_entry.insert(
                            input(format!("{id} {}", Key::IbcHandlerAddress))
                                .interact::<H160>()?
                                .to_string(),
                        );
                    }
                }
            }
            family => bail!("{family} is not currently supported"),
        }

        // load chain-specific values
        match id.family() {
            Arbitrum => todo!(),
            Babylon | Osmosis | Stargaze | Stride | Xion => {
                if is_well_known(id) {
                    self.defaults
                        .get_mut(id)
                        .unwrap()
                        .entry(Key::CometbftRpcUrl)
                        .or_insert(Endpoint::from_ucs04(id, Protocol::RPC).to_string());
                }
            }
            Berachain => todo!(),
            Bob => {
                if is_well_known(id) {
                    self.defaults
                        .get_mut(id)
                        .unwrap()
                        .entry(Key::EthRpcUrl)
                        .or_insert(Endpoint::from_ucs04(id, Protocol::RPC).to_string());
                }

                if let Entry::Vacant(vacant_entry) =
                    self.defaults.get_mut(id).unwrap().entry(Key::L1ChainId)
                {
                    vacant_entry.insert(
                        if id == BOB_60808 {
                            ETHEREUM_1
                        } else if id == BOB_808813 {
                            ETHEREUM_11155111
                        } else {
                            input(Key::L1ChainId)
                                .validate(|id: &String| {
                                    id.parse::<UniversalChainId>()
                                        .map_err(|e| e.to_string())
                                        .and_then(|id| {
                                            if id.family() == Ethereum {
                                                Ok(())
                                            } else {
                                                Err("bob must settle on an ethereum chain"
                                                    .to_owned())
                                            }
                                        })
                                })
                                .interact()?
                        }
                        .to_string(),
                    );
                }

                if let Entry::Vacant(vacant_entry) = self
                    .defaults
                    .get_mut(id)
                    .unwrap()
                    .entry(Key::L2OracleAddress)
                {
                    vacant_entry.insert(if id == BOB_60808 {
                        "0xdDa53E23f8a32640b04D7256e651C1db98dB11C1".to_owned()
                    } else if id == BOB_808813 {
                        "0xd1cBBC06213B7E14e99aDFfFeF1C249E6f9537e0".to_owned()
                    } else {
                        input(Key::L2OracleAddress).interact::<H160>()?.to_string()
                    });
                }

                self.with_chain(&self.get_required_default(id, Key::L1ChainId))?;
            }
            Corn => {
                if is_well_known(id) {
                    self.defaults
                        .get_mut(id)
                        .unwrap()
                        .entry(Key::EthRpcUrl)
                        .or_insert(Endpoint::from_ucs04(id, Protocol::RPC).to_string());
                }

                if let Entry::Vacant(vacant_entry) =
                    self.defaults.get_mut(id).unwrap().entry(Key::L1ChainId)
                {
                    vacant_entry.insert(
                        if id == CORN_21000000 {
                            ETHEREUM_1
                        } else if id == CORN_21000001 {
                            ETHEREUM_11155111
                        } else {
                            input(Key::L1ChainId)
                                .validate(|id: &String| {
                                    id.parse::<UniversalChainId>()
                                        .map_err(|e| e.to_string())
                                        .and_then(|id| {
                                            if id.family() == Ethereum {
                                                Ok(())
                                            } else {
                                                Err("corn must settle on an ethereum chain"
                                                    .to_owned())
                                            }
                                        })
                                })
                                .interact()?
                        }
                        .to_string(),
                    );
                }

                if let Entry::Vacant(vacant_entry) = self
                    .defaults
                    .get_mut(id)
                    .unwrap()
                    .entry(Key::L1ContractAddress)
                {
                    vacant_entry.insert(if id == CORN_21000000 {
                        "0x828C71bc1D7A34F32FfA624240633b6B7272C3D6".to_owned()
                    } else if id == CORN_21000001 {
                        "0xD318638594A5B17b50a1389B0c0580576226C0AE".to_owned()
                    } else {
                        input(Key::L1ContractAddress)
                            .interact::<H160>()?
                            .to_string()
                    });
                }

                self.with_chain(&self.get_required_default(id, Key::L1ChainId))?;
            }
            Ethereum => {
                if is_well_known(id) {
                    self.defaults
                        .get_mut(id)
                        .unwrap()
                        .entry(Key::EthRpcUrl)
                        .or_insert(Endpoint::from_ucs04(id, Protocol::RPC).to_string());
                    self.defaults
                        .get_mut(id)
                        .unwrap()
                        .entry(Key::BeaconRpcUrl)
                        .or_insert(Endpoint::from_ucs04(id, Protocol::BEACON).to_string());
                }

                if let Entry::Vacant(vacant_entry) =
                    self.defaults.get_mut(id).unwrap().entry(Key::ChainSpec)
                {
                    if [
                        well_known::ETHEREUM_1,
                        well_known::ETHEREUM_11155111,
                        well_known::ETHEREUM_17000,
                    ]
                    .iter()
                    .any(|wk| id == wk)
                    {
                        vacant_entry.insert(PresetBaseKind::Mainnet.to_string());
                    } else {
                        vacant_entry.insert(
                            select(Key::ChainSpec)
                                .items(&[
                                    (PresetBaseKind::Mainnet, "mainnet", ""),
                                    (PresetBaseKind::Minimal, "minimal", ""),
                                ])
                                .filter_mode()
                                .interact()?
                                .to_string(),
                        );
                    }
                }
            }
            Movement => todo!(),
            Scroll => todo!(),
            Sei => {
                let chain = self.defaults.get_mut(id).unwrap();

                let (evm, cosmos) = if id == SEI_1328 {
                    (SEI_1328, SEI_ATLANTIC_2)
                } else if id == SEI_1329 {
                    (SEI_1329, SEI_PACIFIC_1)
                } else if id == SEI_ATLANTIC_2 {
                    (SEI_1328, SEI_ATLANTIC_2)
                } else if id == SEI_PACIFIC_1 {
                    (SEI_1329, SEI_PACIFIC_1)
                } else {
                    let ty = select(format!("is {id} the cosmos or evm chain id?"))
                        .item(CosmosOrEvm::Cosmos, "cosmos", "")
                        .item(CosmosOrEvm::Evm, "evm", "")
                        .interact()?;

                    let other_id = input(format!(
                        "{} chain id",
                        match ty {
                            CosmosOrEvm::Cosmos => "evm",
                            CosmosOrEvm::Evm => "cosmos",
                        }
                    ))
                    .validate(|s: &String| Id::new(s).ok_or("invalid chain id").map(|_| ()))
                    .interact::<String>()?;

                    let other_id =
                        UniversalChainId::new_owned(Sei, Id::new_owned(other_id).unwrap());

                    match ty {
                        CosmosOrEvm::Cosmos => (other_id, id.clone()),
                        CosmosOrEvm::Evm => (id.clone(), other_id),
                    }
                };

                chain
                    .entry(Key::CosmosChainId)
                    .or_insert(cosmos.to_string());
                chain.entry(Key::EvmChainId).or_insert(evm.to_string());

                if is_well_known(id) {
                    chain
                        .entry(Key::CometbftRpcUrl)
                        .or_insert(Endpoint::from_ucs04(id, Protocol::RPC).to_string());
                    chain
                        .entry(Key::EthRpcUrl)
                        .or_insert(Endpoint::from_ucs04(id, Protocol::RPC).to_string());
                }
            }
            Union => {
                self.defaults
                    .get_mut(id)
                    .unwrap()
                    .entry(Key::CometbftRpcUrl)
                    .or_insert(Endpoint::from_ucs04(id, Protocol::RPC).to_string());
            }
            family => bail!("{family} is not currently supported"),
        }

        Ok(())
    }

    fn make_path(&self, path: impl Display) -> PathBuf {
        format!("{}/{path}", self.base_path).into()
    }

    #[track_caller]
    fn read_value<T: Display + FromStr>(
        &mut self,
        title: impl Display,
        id: &UniversalChainId<'a>,
        key: Key,
    ) -> Result<T> {
        let mut i = input(title);
        if let Some(default) = self.defaults[id].get(&key) {
            i = i.default_input(default);
        }
        let res = i.interact::<T>()?;

        self.defaults
            .get_mut(id)
            .unwrap()
            .insert(key, res.to_string());

        Ok(res)
    }

    #[track_caller]
    fn read_value_select<T: Clone + Eq + Display + FromStr>(
        &mut self,
        title: &str,
        items: &[(T, impl Display, impl Display)],
        id: &UniversalChainId<'a>,
        key: Key,
    ) -> Result<T> {
        let mut s = select::<T>(title).items(items).filter_mode();
        if let Some(default) = self.defaults[id].get(&key) {
            s = s.initial_value(default.parse().ok().unwrap());
        }
        let res = s.interact()?;

        self.defaults
            .get_mut(id)
            .unwrap()
            .insert(key, res.to_string());

        Ok(res)
    }

    fn finality_module(
        &mut self,
        id: &UniversalChainId<'a>,
    ) -> Result<ModuleConfig<FinalityModuleInfo>> {
        Ok(match id.family() {
            Babylon | Osmosis | Sei | Stargaze | Stride | Xion => module_config!(self {
                info: FinalityModuleInfo {
                    chain_id: ChainId::new(id.id().to_string()),
                    consensus_type: ConsensusType::new(ConsensusType::TENDERMINT),
                },
                config: voyager_finality_module_tendermint::Config {
                    rpc_url: self.read_value(
                        format!("{id} finality rpc url"),
                        id,
                        Key::CometbftRpcUrl
                    )?,
                },
            }),
            Bob => {
                let l1_chain_id = self.get_required_default::<UniversalChainId>(id, Key::L1ChainId);

                module_config!(self {
                    info: FinalityModuleInfo {
                        chain_id: ChainId::new(id.id().to_string()),
                        consensus_type: ConsensusType::new(ConsensusType::ARBITRUM),
                    },
                    config: voyager_finality_module_bob::Config {
                        l1_chain_id: ChainId::new(l1_chain_id.id().to_string()),
                        l2_oracle_address: self.get_required_default(id, Key::L2OracleAddress),
                        l1_rpc_url: self.read_value(
                            format!("{l1_chain_id} finality rpc url"),
                            &l1_chain_id,
                            Key::EthRpcUrl
                        )?,
                        l2_rpc_url: self.read_value(
                            format!("{id} finality rpc url"),
                            id,
                            Key::EthRpcUrl
                        )?,
                        max_cache_size: 1000,
                    },
                })
            }
            Arbitrum | Corn => {
                let l1_chain_id = self.get_required_default::<UniversalChainId>(id, Key::L1ChainId);

                module_config!(self {
                    info: FinalityModuleInfo {
                        chain_id: ChainId::new(id.id().to_string()),
                        consensus_type: ConsensusType::new(ConsensusType::ARBITRUM),
                    },
                    config: voyager_finality_module_arbitrum::Config {
                        l1_chain_id: ChainId::new(l1_chain_id.id().to_string()),
                        l1_contract_address: self.get_required_default(id, Key::L1ContractAddress),
                        l1_rpc_url: self.read_value(
                            format!("{l1_chain_id} finality rpc url"),
                            &l1_chain_id,
                            Key::EthRpcUrl
                        )?,
                        l2_rpc_url: self.read_value(
                            format!("{id} finality rpc url"),
                            id,
                            Key::EthRpcUrl
                        )?,
                        max_cache_size: 1000,
                    },
                })
            }
            Ethereum => module_config!(self {
                info: FinalityModuleInfo {
                    chain_id: ChainId::new(id.id().to_string()),
                    consensus_type: ConsensusType::new(ConsensusType::ETHEREUM),
                },
                config: voyager_finality_module_ethereum::Config {
                    rpc_url: self.read_value(
                        format!("{id} finality rpc url"),
                        id,
                        Key::EthRpcUrl
                    )?,
                    beacon_rpc_url: self.read_value(
                        format!("{id} finality beacon rpc url"),
                        id,
                        Key::BeaconRpcUrl
                    )?,
                    chain_spec: self.get_required_default(id, Key::ChainSpec),
                    max_cache_size: 1000
                },
            }),
            Union => module_config!(self {
                info: FinalityModuleInfo {
                    chain_id: ChainId::new(id.id().to_string()),
                    consensus_type: ConsensusType::new(ConsensusType::COMETBLS),
                },
                config: voyager_finality_module_cometbls::Config {
                    rpc_url: self.read_value(
                        format!("{id} finality rpc url"),
                        id,
                        Key::CometbftRpcUrl
                    )?,
                },
            }),
            family => bail!("{family} is not currently supported"),
        })
    }

    fn state_module(
        &mut self,
        id: &UniversalChainId<'a>,
        ibc_spec_id: IbcSpecId,
    ) -> Result<ModuleConfig<StateModuleInfo>> {
        Ok(match (id.family(), ibc_spec_id.as_str()) {
            (Babylon | Osmosis | Stargaze | Stride | Union | Xion, IbcSpecId::UNION) => {
                module_config!(self {
                    info: StateModuleInfo {
                        chain_id: ChainId::new(id.id().to_string()),
                        ibc_spec_id
                    },
                    config: voyager_state_module_cosmos_sdk_union::Config {
                        rpc_url: self.read_value(
                            format!("{id} state rpc url"),
                            id,
                            Key::CometbftRpcUrl
                        )?,
                        ibc_host_contract_address: self
                            .get_required_default(id, Key::IbcHostContractAddress)
                    },
                })
            }
            (Arbitrum | Bob | Corn | Ethereum | Sei, IbcSpecId::UNION) => {
                module_config!(self {
                    info: StateModuleInfo {
                        chain_id: ChainId::new(id.id().to_string()),
                        ibc_spec_id
                    },
                    config: voyager_state_module_ethereum::Config {
                        ibc_handler_address: self.get_required_default(id, Key::IbcHandlerAddress),
                        rpc_url: self.read_value(
                            format!("{id} state rpc url"),
                            id,
                            Key::EthRpcUrl
                        )?,
                        // TODO: Make optional?
                        max_query_window: Some(self.read_value(
                            format!("{id} state rpc url max eth_getLogs query window"),
                            id,
                            Key::MaxQueryWindow
                        )?),
                        max_cache_size: 1000
                    },
                })
            }
            (family, spec) => {
                bail!("chain family {family} and ibc spec {spec} is not currently supported")
            }
        })
    }

    fn proof_module(
        &mut self,
        id: &UniversalChainId<'a>,
        ibc_spec_id: IbcSpecId,
    ) -> Result<ModuleConfig<ProofModuleInfo>> {
        Ok(match (id.family(), ibc_spec_id.as_str()) {
            (Babylon | Osmosis | Stargaze | Stride | Union | Xion, IbcSpecId::UNION) => {
                module_config!(self {
                    info: ProofModuleInfo {
                        chain_id: ChainId::new(id.id().to_string()),
                        ibc_spec_id
                    },
                    config: voyager_proof_module_cosmos_sdk_union::Config {
                        rpc_url: self.read_value(
                            format!("{id} proof rpc url"),
                            id,
                            Key::CometbftRpcUrl
                        )?,
                        ibc_host_contract_address: self
                            .get_required_default(id, Key::IbcHostContractAddress)
                    },
                })
            }
            (Arbitrum | Bob | Corn | Ethereum, IbcSpecId::UNION) => {
                module_config!(self {
                    info: ProofModuleInfo {
                        chain_id: ChainId::new(id.id().to_string()),
                        ibc_spec_id
                    },
                    config: voyager_proof_module_ethereum::Config {
                        ibc_handler_address: self.get_required_default(id, Key::IbcHandlerAddress),
                        rpc_url: self.read_value(
                            format!("{id} proof rpc url"),
                            id,
                            Key::EthRpcUrl
                        )?,
                        max_cache_size: 1000
                    },
                })
            }
            (Sei, IbcSpecId::UNION) => {
                module_config!(self {
                    info: ProofModuleInfo {
                        chain_id: ChainId::new(id.id().to_string()),
                        ibc_spec_id
                    },
                    config: voyager_proof_module_ethermint::Config {
                        ibc_handler_address: self.get_required_default(id, Key::IbcHandlerAddress),
                        rpc_url: self.read_value(
                            format!("{id} proof rpc url"),
                            id,
                            Key::EthRpcUrl
                        )?,
                        key_prefix_storage: hex!("03").into(),
                        store_key: hex!("65766D").into()
                    },
                })
            }
            (family, spec) => bail!("{family} on {spec} is not currently supported"),
        })
    }

    #[track_caller]
    fn get_required_default<T: FromStr>(&self, id: &UniversalChainId<'a>, key: Key) -> T {
        self.defaults[id][&key].parse().ok().unwrap()
    }

    fn build_chain_pair(
        &mut self,
        a: &UniversalChainId<'a>,
        b: &UniversalChainId<'a>,
    ) -> Result<ModulesConfig> {
        self.with_chain(a)?;
        self.with_chain(b)?;

        let mut config = ModulesConfig::default();

        // load finality modules
        config.consensus.push(self.finality_module(a)?);
        self.dump_receipt()?;
        config.consensus.push(self.finality_module(b)?);
        self.dump_receipt()?;

        // load state modules
        config
            .state
            .push(self.state_module(a, IbcSpecId::new(IbcSpecId::UNION))?);
        self.dump_receipt()?;
        config
            .state
            .push(self.state_module(b, IbcSpecId::new(IbcSpecId::UNION))?);
        self.dump_receipt()?;

        // load proof modules
        config
            .proof
            .push(self.proof_module(a, IbcSpecId::new(IbcSpecId::UNION))?);
        self.dump_receipt()?;
        config
            .proof
            .push(self.proof_module(b, IbcSpecId::new(IbcSpecId::UNION))?);
        self.dump_receipt()?;

        match (a.parts(), b.parts()) {
            ((Babylon, babylon), (Ethereum, ethereum))
            | ((Ethereum, ethereum), (Babylon, babylon)) => {
                let babylon = UniversalChainId::new(Babylon, babylon).into_owned();
                let ethereum = UniversalChainId::new(Ethereum, ethereum).into_owned();
            }
            // (Babylon, Bob) => todo!(),
            // (Babylon, Corn) => todo!(),
            // (Babylon, Ethereum) => todo!(),
            // (Babylon, Osmosis) => todo!(),
            // (Babylon, Sei) => todo!(),
            // (Babylon, Stride) => todo!(),
            // (Babylon, Union) => todo!(),
            // (Babylon, Xion) => todo!(),
            // (Bob, Babylon) => todo!(),
            // (Bob, Bob) => todo!(),
            // (Bob, Corn) => todo!(),
            // (Bob, Ethereum) => todo!(),
            // (Bob, Osmosis) => todo!(),
            // (Bob, Sei) => todo!(),
            // (Bob, Stride) => todo!(),
            // (Bob, Union) => todo!(),
            // (Bob, Xion) => todo!(),
            // (Corn, Babylon) => todo!(),
            // (Corn, Bob) => todo!(),
            // (Corn, Corn) => todo!(),
            // (Corn, Ethereum) => todo!(),
            // (Corn, Osmosis) => todo!(),
            // (Corn, Sei) => todo!(),
            // (Corn, Stride) => todo!(),
            // (Corn, Union) => todo!(),
            // (Corn, Xion) => todo!(),
            // (Ethereum, Babylon) => todo!(),
            // (Ethereum, Bob) => todo!(),
            // (Ethereum, Corn) => todo!(),
            // (Ethereum, Ethereum) => todo!(),
            // (Ethereum, Osmosis) => todo!(),
            // (Ethereum, Sei) => todo!(),
            // (Ethereum, Stride) => todo!(),
            // (Ethereum, Union) => todo!(),
            // (Ethereum, Xion) => todo!(),
            // (Osmosis, Babylon) => todo!(),
            // (Osmosis, Bob) => todo!(),
            // (Osmosis, Corn) => todo!(),
            // (Osmosis, Ethereum) => todo!(),
            // (Osmosis, Osmosis) => todo!(),
            // (Osmosis, Sei) => todo!(),
            // (Osmosis, Stride) => todo!(),
            // (Osmosis, Union) => todo!(),
            // (Osmosis, Xion) => todo!(),
            // (Sei, Babylon) => todo!(),
            // (Sei, Bob) => todo!(),
            // (Sei, Corn) => todo!(),
            // (Sei, Ethereum) => todo!(),
            // (Sei, Osmosis) => todo!(),
            // (Sei, Sei) => todo!(),
            // (Sei, Stride) => todo!(),
            // (Sei, Union) => todo!(),
            // (Sei, Xion) => todo!(),
            // (Stride, Babylon) => todo!(),
            // (Stride, Bob) => todo!(),
            // (Stride, Corn) => todo!(),
            // (Stride, Ethereum) => todo!(),
            // (Stride, Osmosis) => todo!(),
            // (Stride, Sei) => todo!(),
            // (Stride, Stride) => todo!(),
            // (Stride, Union) => todo!(),
            // (Stride, Xion) => todo!(),
            // (Union, Babylon) => todo!(),
            // (Union, Bob) => todo!(),
            // (Union, Corn) => todo!(),
            // (Union, Ethereum) => todo!(),
            // (Union, Osmosis) => todo!(),
            // (Union, Sei) => todo!(),
            // (Union, Stride) => todo!(),
            // (Union, Union) => todo!(),
            // (Union, Xion) => todo!(),
            // (Xion, Babylon) => todo!(),
            // (Xion, Bob) => todo!(),
            // (Xion, Corn) => todo!(),
            // (Xion, Ethereum) => todo!(),
            // (Xion, Osmosis) => todo!(),
            // (Xion, Sei) => todo!(),
            // (Xion, Stride) => todo!(),
            // (Xion, Union) => todo!(),
            // (Xion, Xion) => todo!(),
            ((a, _), (b, _)) => bail!("{a}<->{b} is not currently supported"),
        }

        Ok(config)
    }

    fn dump_receipt(&self) -> Result<()> {
        Ok(fs::write(
            "./receipt.json",
            serde_json::to_string_pretty(&self.defaults).unwrap(),
        )?)
    }
}

fn read_chain_family() -> Result<Family> {
    Ok(select("family")
        .filter_mode()
        .items(
            &SUPPORTED_FAMILIES
                .iter()
                .map(|f| (*f, f, ""))
                .collect::<Vec<_>>(),
        )
        .interact()?)
}

fn read_chain_id() -> Result<Box<Id>> {
    Ok(Id::new_owned(input("chain id").interact::<String>()?).expect("valid"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CosmosOrEvm {
    Cosmos,
    Evm,
}
