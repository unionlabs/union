#![warn(clippy::pedantic)]

use anyhow::{bail, Result};
use clap::Parser;
use cliclack::{input, intro, log, outro, select};
use ucs04::{Family, Id, UniversalChainId};
use voyager_core::context::ModuleConfig;
use voyager_rpc::types::FinalityModuleInfo;

const SUPPORTED_FAMILIES: &[Family] = &[
    Family::Babylon,
    Family::Bob,
    Family::Corn,
    Family::Ethereum,
    Family::Osmosis,
    Family::Sei,
    Family::Union,
    Family::Xion,
];

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
        (_, (Aptos, _)) | (Aptos, _) => bail!("aptos is not currently supported"),
        (_, (Movement, _)) | (Movement, _) => bail!("movement is not currently supported"),
        (_, (Scroll, _)) | (Scroll, _) => bail!("scroll is not currently supported"),
        (_, (Arbitrum, _)) | (Arbitrum, _) => bail!("arbitrum is not currently supported"),
        (_, (Berachain, _)) | (Berachain, _) => bail!("berachain is not currently supported"),
        (_, (Stargaze, _)) | (Stargaze, _) => bail!("stargaze is not currently supported"),
        ((Babylon, a_id), (Sepolia, b_id)) | ((Sepolia, b_id), (Babylon, a_id)) => {
            finality_module(Babylon);
        }
        _ => todo!(),
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
    }
}

fn finality_module(family: Family) -> Result<ModuleConfig<FinalityModuleInfo>> {
    match family {
        Family::Babylon => voyagerfinality,
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
