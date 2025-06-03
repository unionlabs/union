#![warn(clippy::pedantic)]

use std::{collections::HashMap, fmt::Display, path::PathBuf, str::FromStr};

use anyhow::{bail, Result};
use beacon_api_types::chain_spec::PresetBaseKind;
use chain_kitchen::{Endpoint, Protocol};
use clap::Parser;
use cliclack::{input, intro, log, outro, select};
use heck::ToKebabCase;
use ucs04::{is_well_known, well_known, Family, Id, UniversalChainId};
use voyager_core::context::ModuleConfig;
use voyager_primitives::{ChainId, ConsensusType};
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
    pub const RPC_URL: &str = "rpc_url";
    pub const BEACON_RPC_URL: &str = "beacon_rpc_url";
    pub const CHAIN_SPEC: &str = "chain_spec";
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

        build_chain_pair(&a, &b)?;
    }

    outro("create-voyager-config")?;

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn build_chain_pair(a: &UniversalChainId<'_>, b: &UniversalChainId<'_>) -> Result<()> {
    #[allow(clippy::enum_glob_use)]
    use Family::*;

    match (a.parts(), b.parts()) {
        ((Babylon, babylon), (Ethereum, ethereum)) | ((Ethereum, ethereum), (Babylon, babylon)) => {
            let babylon = UniversalChainId::new(Babylon, babylon);
            let ethereum = UniversalChainId::new(Ethereum, ethereum);

            let mut context = Context::new(
                input("plugins base path").interact()?,
                [
                    is_well_known(&babylon).then_some((
                        (keys::RPC_URL, babylon.clone()),
                        Endpoint::from_ucs04(&babylon, Protocol::RPC),
                    )),
                    is_well_known(&ethereum).then_some((
                        (keys::RPC_URL, ethereum.clone()),
                        Endpoint::from_ucs04(&ethereum, Protocol::RPC),
                    )),
                    is_well_known(&ethereum).then_some((
                        (keys::BEACON_RPC_URL, ethereum.clone()),
                        Endpoint::from_ucs04(&ethereum, Protocol::BEACON),
                    )),
                ]
                .into_iter()
                .flatten(),
            );

            let chain_spec_key = (keys::CHAIN_SPEC, ethereum.clone());
            if [
                well_known::ETHEREUM_1,
                well_known::ETHEREUM_11155111,
                well_known::ETHEREUM_17000,
            ]
            .iter()
            .any(|wk| &ethereum == wk)
            {
                context
                    .defaults
                    .insert(chain_spec_key, PresetBaseKind::Mainnet.to_string());
            } else {
                context.read_value_select(
                    "chain spec",
                    &[
                        (PresetBaseKind::Mainnet, "mainnet", "mainnet"),
                        (PresetBaseKind::Minimal, "minimal", "minimal"),
                    ],
                    chain_spec_key,
                )?;
            }

            let fm = context.finality_module(&babylon)?;

            dbg!(fm);

            let fm = context.finality_module(&ethereum)?;

            dbg!(fm);

            Ok(())
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
    defaults: HashMap<(&'static str, UniversalChainId<'a>), String>,
    base_path: String,
}

impl<'a> Context<'a> {
    fn new(
        base_path: String,
        defaults: impl IntoIterator<Item = ((&'static str, UniversalChainId<'a>), impl Display)>,
    ) -> Self {
        Self {
            defaults: defaults
                .into_iter()
                .map(|(k, v)| (k, v.to_string()))
                .collect(),
            base_path,
        }
    }

    fn with_chain(&mut self) {
        self.defaults.extend([
            is_well_known(&babylon).then_some((
                (keys::RPC_URL, babylon.clone()),
                Endpoint::from_ucs04(&babylon, Protocol::RPC),
            )),
            is_well_known(&ethereum).then_some((
                (keys::RPC_URL, ethereum.clone()),
                Endpoint::from_ucs04(&ethereum, Protocol::RPC),
            )),
            is_well_known(&ethereum).then_some((
                (keys::BEACON_RPC_URL, ethereum.clone()),
                Endpoint::from_ucs04(&ethereum, Protocol::BEACON),
            )),
        ]);
    }

    fn make_path(&self, path: impl Display) -> PathBuf {
        format!("{}/{path}", self.base_path).into()
    }

    #[track_caller]
    fn read_value<T: Display + FromStr>(
        &mut self,
        title: &str,
        key: (&'static str, UniversalChainId<'a>),
    ) -> Result<T> {
        let mut i = input(title);
        if let Some(default) = self.defaults.get(&key) {
            i = i.default_input(default);
        }
        let res = i.interact::<T>()?;

        self.defaults.insert(key, res.to_string());

        Ok(res)
    }

    #[track_caller]
    fn read_value_select<T: Clone + Eq + Display + FromStr>(
        &mut self,
        title: &str,
        items: &[(T, impl Display, impl Display)],
        key: (&'static str, UniversalChainId<'a>),
    ) -> Result<T> {
        let mut s = select::<T>(title).items(items).filter_mode();
        if let Some(default) = self.defaults.get(&key) {
            s = s.initial_value(default.parse().ok().unwrap());
        }
        let res = s.interact()?;

        self.defaults.insert(key, res.to_string());

        Ok(res)
    }

    fn finality_module(
        &mut self,
        id: &UniversalChainId<'a>,
    ) -> Result<ModuleConfig<FinalityModuleInfo>> {
        Ok(match id.family() {
            Family::Babylon | Family::Osmosis | Family::Sei => module_config!(self {
                info: FinalityModuleInfo {
                    chain_id: ChainId::new(id.id().to_string()),
                    consensus_type: ConsensusType::new(ConsensusType::TENDERMINT),
                },
                config: voyager_finality_module_tendermint::Config {
                    rpc_url: self.read_value(
                        &format!("{id} finality rpc url"),
                        (keys::RPC_URL, id.clone())
                    )?,
                },
            }),
            Family::Arbitrum => todo!(),
            Family::Berachain => todo!(),
            Family::Bob => todo!(),
            Family::Corn => todo!(),
            Family::Ethereum => module_config!(self {
                info: FinalityModuleInfo {
                    chain_id: ChainId::new(id.id().to_string()),
                    consensus_type: ConsensusType::new(ConsensusType::ETHEREUM),
                },
                config: voyager_finality_module_ethereum::Config {
                    rpc_url: self.read_value(
                        &format!("{id} finality rpc url"),
                        (keys::RPC_URL, id.clone())
                    )?,
                    beacon_rpc_url: self.read_value(
                        &format!("{id} finality beacon rpc url"),
                        (keys::BEACON_RPC_URL, id.clone())
                    )?,
                    chain_spec: self.defaults[&(keys::CHAIN_SPEC, id.clone())]
                        .parse()
                        .unwrap(),
                    max_cache_size: 1000
                },
            }),
            Family::Movement => todo!(),
            Family::Scroll => todo!(),
            Family::Stargaze => todo!(),
            Family::Stride => todo!(),
            Family::Union => todo!(),
            Family::Xion => todo!(),
            family => bail!("{family} is not currently supported"),
        })
    }
}

fn read_chain_family() -> Result<Family> {
    Ok(select("family")
        .filter_mode()
        .items(
            &SUPPORTED_FAMILIES
                .iter()
                .map(|f| (*f, f, f))
                .collect::<Vec<_>>(),
        )
        .interact()?)
}

fn read_chain_id() -> Result<Box<Id>> {
    Ok(Id::new_owned(input("chain id").interact::<String>()?).expect("valid"))
}
