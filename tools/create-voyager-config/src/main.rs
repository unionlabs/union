#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines)]

use std::{
    collections::{hash_map::Entry, HashMap},
    env::temp_dir,
    fmt::Display,
    fs,
    path::PathBuf,
    str::FromStr,
};

use anyhow::{bail, Result};
use beacon_api_types::chain_spec::PresetBaseKind;
use chain_kitchen::{Endpoint, Protocol};
use clap::Parser;
use cliclack::{input, intro, log, outro, select};
use heck::ToKebabCase;
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
use unionlabs_primitives::H160;
use voyager_core::context::{ModuleConfig, ModulesConfig};
use voyager_primitives::{ChainId, ConsensusType, IbcSpecId};
use voyager_rpc::types::FinalityModuleInfo;

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

pub mod keys {
    pub const COSMOS_RPC_URL: &str = "cosmos_rpc_url";
    pub const EVM_RPC_URL: &str = "evm_rpc_url";
    pub const COSMOS_CHAIN_ID: &str = "cosmos_chain_id";
    pub const EVM_CHAIN_ID: &str = "evm_chain_id";
    pub const BEACON_RPC_URL: &str = "beacon_rpc_url";
    pub const CHAIN_SPEC: &str = "chain_spec";
    pub const L1_CONTRACT_ADDRESS: &str = "l1_contract_address";
    pub const L2_ORACLE_ADDRESS: &str = "l2_oracle_address";
    pub const L1_CHAIN_ID: &str = "l1_chain_id";
    pub const IBC_HOST_CONTRACT_ADDRESS: &str = "ibc_host_contract_address";
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

        let config = context.build_chain_pair(&a, &b)?;

        dbg!(&config);

        context.dump_receipt()?;
    }

    outro("create-voyager-config")?;

    Ok(())
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
    defaults: HashMap<UniversalChainId<'a>, HashMap<&'static str, String>>,
    base_path: String,
}

impl<'a> Context<'a> {
    fn new(base_path: String) -> Self {
        Self {
            defaults: HashMap::default(),
            base_path,
        }
    }

    fn with_chain(&mut self, id: &UniversalChainId<'a>) -> Result<()> {
        self.defaults.entry(id.clone()).or_default();

        match id.family() {
            Arbitrum => todo!(),
            Babylon | Osmosis | Stargaze | Stride | Xion => {
                if is_well_known(id) {
                    self.defaults
                        .get_mut(id)
                        .unwrap()
                        .entry(keys::COSMOS_RPC_URL)
                        .or_insert(Endpoint::from_ucs04(id, Protocol::RPC).to_string());
                }
            }
            Berachain => todo!(),
            Bob => {
                if is_well_known(id) {
                    self.defaults
                        .get_mut(id)
                        .unwrap()
                        .entry(keys::EVM_RPC_URL)
                        .or_insert(Endpoint::from_ucs04(id, Protocol::RPC).to_string());
                }

                if let Entry::Vacant(vacant_entry) =
                    self.defaults.get_mut(id).unwrap().entry(keys::L1_CHAIN_ID)
                {
                    vacant_entry.insert(
                        if id == BOB_60808 {
                            ETHEREUM_1
                        } else if id == BOB_808813 {
                            ETHEREUM_11155111
                        } else {
                            input(keys::L1_CHAIN_ID)
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
                    .entry(keys::L2_ORACLE_ADDRESS)
                {
                    vacant_entry.insert(if id == BOB_60808 {
                        "0xdDa53E23f8a32640b04D7256e651C1db98dB11C1".to_owned()
                    } else if id == BOB_808813 {
                        "0xd1cBBC06213B7E14e99aDFfFeF1C249E6f9537e0".to_owned()
                    } else {
                        input(keys::L2_ORACLE_ADDRESS)
                            .interact::<H160>()?
                            .to_string()
                    });
                }

                self.with_chain(&self.get_required_default(id, keys::L1_CHAIN_ID))?;
            }
            Corn => {
                if is_well_known(id) {
                    self.defaults
                        .get_mut(id)
                        .unwrap()
                        .entry(keys::EVM_RPC_URL)
                        .or_insert(Endpoint::from_ucs04(id, Protocol::RPC).to_string());
                }

                if let Entry::Vacant(vacant_entry) =
                    self.defaults.get_mut(id).unwrap().entry(keys::L1_CHAIN_ID)
                {
                    vacant_entry.insert(
                        if id == CORN_21000000 {
                            ETHEREUM_1
                        } else if id == CORN_21000001 {
                            ETHEREUM_11155111
                        } else {
                            input(keys::L1_CHAIN_ID)
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
                    .entry(keys::L1_CONTRACT_ADDRESS)
                {
                    vacant_entry.insert(if id == CORN_21000000 {
                        "0x828C71bc1D7A34F32FfA624240633b6B7272C3D6".to_owned()
                    } else if id == CORN_21000001 {
                        "0xD318638594A5B17b50a1389B0c0580576226C0AE".to_owned()
                    } else {
                        input(keys::L1_CONTRACT_ADDRESS)
                            .interact::<H160>()?
                            .to_string()
                    });
                }

                self.with_chain(&self.get_required_default(id, keys::L1_CHAIN_ID))?;
            }
            Ethereum => {
                if is_well_known(id) {
                    self.defaults
                        .get_mut(id)
                        .unwrap()
                        .entry(keys::EVM_RPC_URL)
                        .or_insert(Endpoint::from_ucs04(id, Protocol::RPC).to_string());
                    self.defaults
                        .get_mut(id)
                        .unwrap()
                        .entry(keys::BEACON_RPC_URL)
                        .or_insert(Endpoint::from_ucs04(id, Protocol::BEACON).to_string());
                }

                if let Entry::Vacant(vacant_entry) =
                    self.defaults.get_mut(id).unwrap().entry(keys::CHAIN_SPEC)
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
                            select(keys::CHAIN_SPEC)
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
                    .entry(keys::COSMOS_CHAIN_ID)
                    .or_insert(cosmos.to_string());
                chain.entry(keys::EVM_CHAIN_ID).or_insert(evm.to_string());

                if is_well_known(id) {
                    chain
                        .entry(keys::COSMOS_RPC_URL)
                        .or_insert(Endpoint::from_ucs04(id, Protocol::RPC).to_string());
                    chain
                        .entry(keys::EVM_RPC_URL)
                        .or_insert(Endpoint::from_ucs04(id, Protocol::RPC).to_string());
                }
            }
            Union => {
                self.defaults
                    .get_mut(id)
                    .unwrap()
                    .entry(keys::COSMOS_RPC_URL)
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
        title: &str,
        id: &UniversalChainId<'a>,
        key: &'static str,
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
        key: &'static str,
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
                        &format!("{id} finality rpc url"),
                        id,
                        keys::COSMOS_RPC_URL
                    )?,
                },
            }),
            Bob => {
                let l1_chain_id =
                    self.get_required_default::<UniversalChainId>(id, keys::L1_CHAIN_ID);

                module_config!(self {
                    info: FinalityModuleInfo {
                        chain_id: ChainId::new(id.id().to_string()),
                        consensus_type: ConsensusType::new(ConsensusType::ARBITRUM),
                    },
                    config: voyager_finality_module_bob::Config {
                        l1_chain_id: ChainId::new(l1_chain_id.id().to_string()),
                        l2_oracle_address: self.get_required_default(id, keys::L2_ORACLE_ADDRESS),
                        l1_rpc_url: self.read_value(
                            &format!("{l1_chain_id} finality rpc url"),
                            &l1_chain_id,
                            keys::EVM_RPC_URL
                        )?,
                        l2_rpc_url: self.read_value(
                            &format!("{id} finality rpc url"),
                            id,
                            keys::EVM_RPC_URL
                        )?,
                        max_cache_size: 1000,
                    },
                })
            }
            Arbitrum | Corn => {
                let l1_chain_id =
                    self.get_required_default::<UniversalChainId>(id, keys::L1_CHAIN_ID);

                module_config!(self {
                    info: FinalityModuleInfo {
                        chain_id: ChainId::new(id.id().to_string()),
                        consensus_type: ConsensusType::new(ConsensusType::ARBITRUM),
                    },
                    config: voyager_finality_module_arbitrum::Config {
                        l1_chain_id: ChainId::new(l1_chain_id.id().to_string()),
                        l1_contract_address: self
                            .get_required_default(id, keys::L1_CONTRACT_ADDRESS),
                        l1_rpc_url: self.read_value(
                            &format!("{l1_chain_id} finality rpc url"),
                            &l1_chain_id,
                            keys::EVM_RPC_URL
                        )?,
                        l2_rpc_url: self.read_value(
                            &format!("{id} finality rpc url"),
                            id,
                            keys::EVM_RPC_URL
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
                        &format!("{id} finality rpc url"),
                        id,
                        keys::EVM_RPC_URL
                    )?,
                    beacon_rpc_url: self.read_value(
                        &format!("{id} finality beacon rpc url"),
                        id,
                        keys::BEACON_RPC_URL
                    )?,
                    chain_spec: self.get_required_default(id, keys::CHAIN_SPEC),
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
                        &format!("{id} finality rpc url"),
                        id,
                        keys::COSMOS_RPC_URL
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
    ) -> Result<ModuleConfig<FinalityModuleInfo>> {
        Ok(match (id.family(), ibc_spec_id.as_str()) {
            (Babylon | Osmosis | Stargaze | Stride | Xion, IbcSpecId::UNION) => {
                module_config!(self {
                    info: FinalityModuleInfo {
                        chain_id: ChainId::new(id.id().to_string()),
                        consensus_type: ConsensusType::new(ConsensusType::TENDERMINT),
                    },
                    config: voyager_state_module_cosmos_sdk_union::Config {
                        rpc_url: self.read_value(
                            &format!("{id} finality rpc url"),
                            id,
                            keys::COSMOS_RPC_URL
                        )?,
                        ibc_host_contract_address: self
                            .get_required_default(id, keys::IBC_HOST_CONTRACT_ADDRESS)?,
                    },
                })
            }
            (Bob, IbcSpecId::UNION) => {
                let l1_chain_id =
                    self.get_required_default::<UniversalChainId>(id, keys::L1_CHAIN_ID);

                module_config!(self {
                    info: FinalityModuleInfo {
                        chain_id: ChainId::new(id.id().to_string()),
                        consensus_type: ConsensusType::new(ConsensusType::ARBITRUM),
                    },
                    config: voyager_finality_module_bob::Config {
                        l1_chain_id: ChainId::new(l1_chain_id.id().to_string()),
                        l2_oracle_address: self.get_required_default(id, keys::L2_ORACLE_ADDRESS),
                        l1_rpc_url: self.read_value(
                            &format!("{l1_chain_id} finality rpc url"),
                            &l1_chain_id,
                            keys::EVM_RPC_URL
                        )?,
                        l2_rpc_url: self.read_value(
                            &format!("{id} finality rpc url"),
                            id,
                            keys::EVM_RPC_URL
                        )?,
                        max_cache_size: 1000,
                    },
                })
            }
            Arbitrum | Corn => {
                let l1_chain_id =
                    self.get_required_default::<UniversalChainId>(id, keys::L1_CHAIN_ID);

                module_config!(self {
                    info: FinalityModuleInfo {
                        chain_id: ChainId::new(id.id().to_string()),
                        consensus_type: ConsensusType::new(ConsensusType::ARBITRUM),
                    },
                    config: voyager_finality_module_arbitrum::Config {
                        l1_chain_id: ChainId::new(l1_chain_id.id().to_string()),
                        l1_contract_address: self
                            .get_required_default(id, keys::L1_CONTRACT_ADDRESS),
                        l1_rpc_url: self.read_value(
                            &format!("{l1_chain_id} finality rpc url"),
                            &l1_chain_id,
                            keys::EVM_RPC_URL
                        )?,
                        l2_rpc_url: self.read_value(
                            &format!("{id} finality rpc url"),
                            id,
                            keys::EVM_RPC_URL
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
                        &format!("{id} finality rpc url"),
                        id,
                        keys::EVM_RPC_URL
                    )?,
                    beacon_rpc_url: self.read_value(
                        &format!("{id} finality beacon rpc url"),
                        id,
                        keys::BEACON_RPC_URL
                    )?,
                    chain_spec: self.get_required_default(id, keys::CHAIN_SPEC),
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
                        &format!("{id} finality rpc url"),
                        id,
                        keys::COSMOS_RPC_URL
                    )?,
                },
            }),
            family => bail!("{family} is not currently supported"),
        })
    }

    fn get_required_default<T: FromStr>(&self, id: &UniversalChainId<'a>, key: &'static str) -> T {
        self.defaults[id][key].parse().ok().unwrap()
    }

    fn build_chain_pair(
        &mut self,
        a: &UniversalChainId<'a>,
        b: &UniversalChainId<'a>,
    ) -> Result<ModulesConfig> {
        self.with_chain(a)?;
        self.with_chain(b)?;

        let mut config = ModulesConfig::default();

        config.consensus.push(self.finality_module(a)?);
        config.consensus.push(self.finality_module(b)?);

        config.state.push(self.finality_module(a)?);
        config.state.push(self.finality_module(b)?);

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
            serde_json::to_string_pretty(
                &self
                    .defaults
                    .iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect::<HashMap<_, _>>(),
            )
            .unwrap(),
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
