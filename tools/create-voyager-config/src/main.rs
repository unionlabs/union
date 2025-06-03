#![warn(clippy::pedantic)]

use std::{borrow::Cow, io};

use anyhow::{bail, Result};
use clap::Parser;
use cliclack::{input, intro, log, outro, select};
use ucs04::{Family, Id, UniversalChainId};

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

        build_chain_pair(a, b)?;
    }

    outro("create-voyager-config")?;

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn build_chain_pair(a: UniversalChainId<'_>, b: UniversalChainId<'_>) -> Result<()> {
    use Family::*;

    match (a.family(), b.family()) {
        (_, Aptos) | (Aptos, _) => bail!("aptos is not currently supported"),
        (_, Movement) | (Movement, _) => bail!("movement is not currently supported"),
        (_, Scroll) | (Scroll, _) => bail!("scroll is not currently supported"),
        (_, Arbitrum) | (Arbitrum, _) => bail!("arbitrum is not currently supported"),
        (Babylon, Babylon) => todo!(),
        (Babylon, Berachain) => todo!(),
        (Babylon, Bob) => todo!(),
        (Babylon, Corn) => todo!(),
        (Babylon, Ethereum) => todo!(),
        (Babylon, Osmosis) => todo!(),
        (Babylon, Sei) => todo!(),
        (Babylon, Stargaze) => todo!(),
        (Babylon, Stride) => todo!(),
        (Babylon, Union) => todo!(),
        (Babylon, Xion) => todo!(),
        (Berachain, Babylon) => todo!(),
        (Berachain, Berachain) => todo!(),
        (Berachain, Bob) => todo!(),
        (Berachain, Corn) => todo!(),
        (Berachain, Ethereum) => todo!(),
        (Berachain, Osmosis) => todo!(),
        (Berachain, Sei) => todo!(),
        (Berachain, Stargaze) => todo!(),
        (Berachain, Stride) => todo!(),
        (Berachain, Union) => todo!(),
        (Berachain, Xion) => todo!(),
        (Bob, Babylon) => todo!(),
        (Bob, Berachain) => todo!(),
        (Bob, Bob) => todo!(),
        (Bob, Corn) => todo!(),
        (Bob, Ethereum) => todo!(),
        (Bob, Osmosis) => todo!(),
        (Bob, Sei) => todo!(),
        (Bob, Stargaze) => todo!(),
        (Bob, Stride) => todo!(),
        (Bob, Union) => todo!(),
        (Bob, Xion) => todo!(),
        (Corn, Babylon) => todo!(),
        (Corn, Berachain) => todo!(),
        (Corn, Bob) => todo!(),
        (Corn, Corn) => todo!(),
        (Corn, Ethereum) => todo!(),
        (Corn, Osmosis) => todo!(),
        (Corn, Sei) => todo!(),
        (Corn, Stargaze) => todo!(),
        (Corn, Stride) => todo!(),
        (Corn, Union) => todo!(),
        (Corn, Xion) => todo!(),
        (Ethereum, Babylon) => todo!(),
        (Ethereum, Berachain) => todo!(),
        (Ethereum, Bob) => todo!(),
        (Ethereum, Corn) => todo!(),
        (Ethereum, Ethereum) => todo!(),
        (Ethereum, Osmosis) => todo!(),
        (Ethereum, Sei) => todo!(),
        (Ethereum, Stargaze) => todo!(),
        (Ethereum, Stride) => todo!(),
        (Ethereum, Union) => todo!(),
        (Ethereum, Xion) => todo!(),
        (Osmosis, Babylon) => todo!(),
        (Osmosis, Berachain) => todo!(),
        (Osmosis, Bob) => todo!(),
        (Osmosis, Corn) => todo!(),
        (Osmosis, Ethereum) => todo!(),
        (Osmosis, Osmosis) => todo!(),
        (Osmosis, Sei) => todo!(),
        (Osmosis, Stargaze) => todo!(),
        (Osmosis, Stride) => todo!(),
        (Osmosis, Union) => todo!(),
        (Osmosis, Xion) => todo!(),
        (Sei, Babylon) => todo!(),
        (Sei, Berachain) => todo!(),
        (Sei, Bob) => todo!(),
        (Sei, Corn) => todo!(),
        (Sei, Ethereum) => todo!(),
        (Sei, Osmosis) => todo!(),
        (Sei, Sei) => todo!(),
        (Sei, Stargaze) => todo!(),
        (Sei, Stride) => todo!(),
        (Sei, Union) => todo!(),
        (Sei, Xion) => todo!(),
        (Stargaze, Babylon) => todo!(),
        (Stargaze, Berachain) => todo!(),
        (Stargaze, Bob) => todo!(),
        (Stargaze, Corn) => todo!(),
        (Stargaze, Ethereum) => todo!(),
        (Stargaze, Osmosis) => todo!(),
        (Stargaze, Sei) => todo!(),
        (Stargaze, Stargaze) => todo!(),
        (Stargaze, Stride) => todo!(),
        (Stargaze, Union) => todo!(),
        (Stargaze, Xion) => todo!(),
        (Stride, Babylon) => todo!(),
        (Stride, Berachain) => todo!(),
        (Stride, Bob) => todo!(),
        (Stride, Corn) => todo!(),
        (Stride, Ethereum) => todo!(),
        (Stride, Osmosis) => todo!(),
        (Stride, Sei) => todo!(),
        (Stride, Stargaze) => todo!(),
        (Stride, Stride) => todo!(),
        (Stride, Union) => todo!(),
        (Stride, Xion) => todo!(),
        (Union, Babylon) => todo!(),
        (Union, Berachain) => todo!(),
        (Union, Bob) => todo!(),
        (Union, Corn) => todo!(),
        (Union, Ethereum) => todo!(),
        (Union, Osmosis) => todo!(),
        (Union, Sei) => todo!(),
        (Union, Stargaze) => todo!(),
        (Union, Stride) => todo!(),
        (Union, Union) => todo!(),
        (Union, Xion) => todo!(),
        (Xion, Babylon) => todo!(),
        (Xion, Berachain) => todo!(),
        (Xion, Bob) => todo!(),
        (Xion, Corn) => todo!(),
        (Xion, Ethereum) => todo!(),
        (Xion, Osmosis) => todo!(),
        (Xion, Sei) => todo!(),
        (Xion, Stargaze) => todo!(),
        (Xion, Stride) => todo!(),
        (Xion, Union) => todo!(),
        (Xion, Xion) => todo!(),
    }
}

fn read_chain_family() -> Result<Family> {
    Ok(select("family")
        .filter_mode()
        .items(&Family::ALL.iter().map(|f| (*f, f, f)).collect::<Vec<_>>())
        .interact()?)
}

fn read_chain_id() -> Result<Box<Id>> {
    Ok(Id::new_owned(input("chain id").interact::<String>()?).expect("valid"))
}
