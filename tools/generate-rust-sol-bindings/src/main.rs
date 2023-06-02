use std::{error::Error, path::PathBuf};

use clap::Parser;
use ethers_contract_abigen::{Abigen, MultiAbigen};

#[derive(Parser)]
struct Args {
    input: Vec<PathBuf>,
    #[arg(long)]
    cratedir: PathBuf,
    #[arg(long)]
    overwrite: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.cratedir.exists() && !args.overwrite {
        return Err("out file exists, not overwriting".into());
    }

    dbg!(&args.input);

    MultiAbigen::from_abigens(
        args.input
            .into_iter()
            .map(|s| Abigen::from_file(s).unwrap()),
    )
    .build()?
    .write_to_crate("contracts", "0.0.0", args.cratedir, false)?;

    Ok(())
}
