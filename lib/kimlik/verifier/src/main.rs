#![warn(clippy::pedantic, clippy::unwrap_used)]

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use kimlik_verifier::extract;

#[derive(Parser)]
enum App {
    /// Extract the kimlik id embedded in the provided wasm blob.
    Extract { path: PathBuf },
}

fn main() -> Result<()> {
    match App::parse() {
        App::Extract { path } => {
            let file = std::fs::read(path).context("reading wasm blob")?;

            let id = extract(&file)?;

            println!("{id}");
        }
    }

    Ok(())
}
