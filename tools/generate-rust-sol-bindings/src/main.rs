use std::{error::Error, path::PathBuf};

use clap::Parser;
use ethers_contract_abigen::Abigen;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    input: PathBuf,
    #[arg(long)]
    outfile: PathBuf,
    #[arg(long)]
    overwrite: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.outfile.exists() && !args.overwrite {
        return Err("out file exists, not overwriting".into());
    }

    Abigen::from_file(args.input)?
        .generate()?
        .write_to_file(args.outfile)?;

    Ok(())
}
