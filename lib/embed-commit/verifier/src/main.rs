#![warn(clippy::pedantic, clippy::unwrap_used)]

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Parser;
use embed_commit_verifier::{extract_elf, extract_wasm};

#[derive(Parser)]
enum App {
    /// Extract the commit information embedded in the artifact.
    Extract { path: PathBuf },
}

fn main() -> Result<()> {
    match App::parse() {
        App::Extract { path } => {
            let file = std::fs::read(path).context("reading input artifact")?;

            let rev = match &file.get(0..4) {
                Some(b"\0asm") => extract_wasm(&file)?,
                Some(b"\0elf") => extract_elf(&file)?,
                Some(b"\x7FELF") => extract_elf(&file)?,
                Some(magic) => bail!("unknown file magic {magic:?}"),
                None => bail!("file is < 4 bytes"),
            };

            match rev {
                Some(rev) => println!("{rev}"),
                None => println!("none"),
            }
        }
    }

    Ok(())
}
